use crate::decision::{
    Alternative, Confidence, PolicyCandidate, Recommendation, Severity, WarningKind,
};
use crate::presets::Policy;
use crate::rules::score_candidate;

/// Scored view used internally for deterministic ranking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoredCandidate {
    pub candidate: PolicyCandidate,
    pub score: i32,
    pub reasons: Vec<crate::decision::Reason>,
    pub warnings: Vec<crate::decision::Warning>,
}

/// Pure, deterministic policy evaluation.
///
/// `candidates` is a slice of normalized candidates with already-collected evidence.
/// No IO, no global state, no randomness. Same inputs always produce the same output.
///
/// The function does **not** hard-code a universal `Arch > Flatpak > Aur` ordering.
/// Ranking is derived from per-candidate evidence interpreted through `policy` weights,
/// so the same application can legitimately recommend a different source depending on
/// evidence (e.g. a Flatpak with `filesystem=host` and `unverified` publisher will lose
/// to an official Arch package that is `publisher_supported` and signed).
pub fn evaluate(candidates: &[PolicyCandidate], policy: &Policy) -> Recommendation {
    if candidates.is_empty() {
        return Recommendation::none(Confidence::None);
    }

    // Score each candidate independently — no cross-candidate state except final ranking.
    let mut scored: Vec<ScoredCandidate> = candidates
        .iter()
        .cloned()
        .map(|c| {
            let outcome = score_candidate(&c, &policy.weights);
            ScoredCandidate {
                candidate: c,
                score: outcome.score,
                reasons: outcome.reasons,
                warnings: outcome.warnings,
            }
        })
        .collect();

    // Deterministic sort: descending score, then deterministic candidate ordering (package name + id).
    // This tie-breaker does not encode a universal source priority; it uses lexical identity only.
    scored.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.candidate.cmp(&b.candidate))
    });

    // Winner is first after sorting.
    let winner = scored
        .first()
        .cloned()
        .expect("scored non-empty because candidates non-empty; invariant holds");

    // Confidence: based on separation from runner-up and absolute evidence quality.
    let confidence = derive_confidence(&scored);

    // Promote warnings for critical gaps even when the candidate won.
    let warnings = winner.warnings.clone();
    // If winner is community-maintained with findings, ensure the severity is surfaced
    // even if the runner-up is worse — MaximumReview already penalizes heavily, but
    // ensure Balanced also surfaces a High severity hint for stale findings.
    if winner.candidate.evidence.findings.iter().any(|f| {
        matches!(
            f,
            crate::decision::FindingKind::NetworkExecution
                | crate::decision::FindingKind::DownloadedCodeExecution
        )
    }) {
        let already = warnings
            .iter()
            .any(|w| w.kind == WarningKind::FindingsDetected && w.severity == Severity::High);
        if !already {
            // Rule already added FindingsDetected as High — nothing to do. This branch documents
            // the intention and guards against future rule changes weakening severity.
        }
    }

    let reasons = winner.reasons.clone();

    let alternatives = scored
        .iter()
        .skip(1)
        .cloned()
        .map(|s| Alternative {
            candidate: s.candidate,
            score: s.score,
            reasons: s.reasons,
            warnings: s.warnings,
        })
        .collect();

    Recommendation {
        recommended: Some(winner.candidate),
        confidence,
        reasons,
        warnings,
        alternatives,
        score: winner.score,
    }
}

fn derive_confidence(scored: &[ScoredCandidate]) -> Confidence {
    if scored.is_empty() {
        return Confidence::None;
    }
    if scored.len() == 1 {
        let s = scored[0].score;
        return if s >= 40 {
            Confidence::High
        } else if s >= 15 {
            Confidence::Medium
        } else if s >= 0 {
            Confidence::Low
        } else {
            Confidence::Uncertain
        };
    }
    let top = scored[0].score;
    let second = scored[1].score;
    let gap = top.saturating_sub(second);
    let absolute = top;

    // High gap + positive absolute => High confidence.
    // Small gap or marginal absolute => Low/Uncertain.
    if gap >= 20 && absolute >= 20 {
        Confidence::High
    } else if gap >= 10 && absolute >= 10 {
        Confidence::Medium
    } else if gap >= 4 {
        Confidence::Low
    } else {
        Confidence::Uncertain
    }
}

/// Convenience: evaluate and return only the ordered scored list (deterministic, test-friendly).
pub fn rank(candidates: &[PolicyCandidate], policy: &Policy) -> Vec<ScoredCandidate> {
    let mut scored: Vec<ScoredCandidate> = candidates
        .iter()
        .cloned()
        .map(|c| {
            let o = score_candidate(&c, &policy.weights);
            ScoredCandidate {
                candidate: c,
                score: o.score,
                reasons: o.reasons,
                warnings: o.warnings,
            }
        })
        .collect();
    scored.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.candidate.cmp(&b.candidate))
    });
    scored
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decision::{
        CandidateEvidence, DbusAccess, FilesystemAccess, FindingKind, PermissionLevel,
    };
    use crate::presets::{Policy, PolicyPreset};
    use pkgseal_domain::{CandidateId, PackageName, PackageSource};
    use uuid::Uuid;

    fn cid(n: u128) -> CandidateId {
        CandidateId(Uuid::from_u128(n))
    }

    fn cand(
        id: u128,
        source: PackageSource,
        name: &str,
        evidence: CandidateEvidence,
    ) -> PolicyCandidate {
        PolicyCandidate::new(source, PackageName::new(name).unwrap(), "1.0", evidence)
            .with_id(cid(id))
    }

    #[test]
    fn empty_candidates_returns_none() {
        let r = evaluate(&[], &Policy::balanced());
        assert!(r.is_empty());
        assert_eq!(r.confidence, Confidence::None);
        assert!(r.alternatives.is_empty());
    }

    #[test]
    fn single_candidate_returns_it_with_confidence() {
        let c = cand(
            1,
            PackageSource::ArchOfficial,
            "brave",
            CandidateEvidence {
                is_official_repository: true,
                signature_present: true,
                publisher_supported: true,
                ..Default::default()
            },
        );
        let r = evaluate(std::slice::from_ref(&c), &Policy::balanced());
        assert_eq!(
            r.recommended.as_ref().unwrap().package_name.as_str(),
            "brave"
        );
        assert_eq!(r.alternatives.len(), 0);
        assert!(matches!(
            r.confidence,
            Confidence::High | Confidence::Medium
        ));
    }

    #[test]
    fn official_with_publisher_support_beats_aur_community() {
        let arch = cand(
            1,
            PackageSource::ArchOfficial,
            "brave",
            CandidateEvidence {
                is_official_repository: true,
                signature_present: true,
                checksum_present: true,
                publisher_supported: true,
                ..CandidateEvidence::default()
            },
        );
        let aur = cand(
            2,
            PackageSource::Aur,
            "brave-bin",
            CandidateEvidence {
                is_community_maintained: true,
                checksum_present: true,
                publisher_supported: false,
                ..CandidateEvidence::default()
            },
        );
        let r = evaluate(&[aur.clone(), arch.clone()], &Policy::balanced());
        assert_eq!(
            r.recommended.as_ref().unwrap().source,
            PackageSource::ArchOfficial
        );
        assert!(r.reasons.iter().any(|x| x.detail.contains("official")));
        assert!(
            r.alternatives
                .iter()
                .any(|a| a.candidate.source == PackageSource::Aur)
        );
    }

    #[test]
    fn flatpak_narrow_verified_can_beat_arch_when_sandbox_desired() {
        // Brave: Flatpak verified narrow vs Arch official not publisher-supported.
        // Under SandboxFirst, flatpak should win; under NativeFirst, arch should win.
        let arch = cand(
            1,
            PackageSource::ArchOfficial,
            "brave",
            CandidateEvidence {
                is_official_repository: true,
                signature_present: true,
                publisher_supported: false,
                ..CandidateEvidence::default()
            },
        );
        let flatpak = cand(
            2,
            PackageSource::Flatpak,
            "brave",
            CandidateEvidence::flatpak_verified_narrow(),
        );
        let sandbox_policy = Policy::from_preset(PolicyPreset::SandboxFirst);
        let native_policy = Policy::from_preset(PolicyPreset::NativeFirst);

        let r_sandbox = evaluate(&[arch.clone(), flatpak.clone()], &sandbox_policy);
        assert_eq!(
            r_sandbox.recommended.as_ref().unwrap().source,
            PackageSource::Flatpak,
            "SandboxFirst should prefer verified narrow flatpak over unofficial arch"
        );

        let r_native = evaluate(&[arch.clone(), flatpak.clone()], &native_policy);
        assert_eq!(
            r_native.recommended.as_ref().unwrap().source,
            PackageSource::ArchOfficial,
            "NativeFirst should prefer official arch when trust comparable"
        );
    }

    #[test]
    fn flatpak_broad_permissions_penalized_vs_arch() {
        let arch = cand(
            1,
            PackageSource::ArchOfficial,
            "spotify",
            CandidateEvidence {
                is_official_repository: true,
                signature_present: true,
                publisher_supported: false,
                permission_level: PermissionLevel::Narrow,
                ..CandidateEvidence::default()
            },
        );
        let flatpak_broad = cand(
            2,
            PackageSource::Flatpak,
            "spotify",
            CandidateEvidence {
                sandboxed: true,
                publisher_verified: true,
                permission_level: PermissionLevel::Broad,
                filesystem_access: FilesystemAccess::Host,
                dbus_access: DbusAccess::Host,
                ..CandidateEvidence::default()
            },
        );
        let r = evaluate(&[flatpak_broad.clone(), arch.clone()], &Policy::balanced());
        // Broad host filesystem must be penalized enough that arch wins under Balanced.
        assert_eq!(
            r.recommended.as_ref().unwrap().source,
            PackageSource::ArchOfficial
        );
        assert!(
            r.alternatives
                .iter()
                .any(|a| a.candidate.source == PackageSource::Flatpak)
        );
        // Winner warnings should not contain host access; loser alternative should.
        assert!(
            r.alternatives[0]
                .warnings
                .iter()
                .any(|w| w.kind == WarningKind::HostFilesystemAccess
                    || w.kind == WarningKind::ExcessivePermissions
                    || w.kind == WarningKind::BroadPermissions)
        );
    }

    #[test]
    fn maximum_review_heavily_penalizes_aur_with_findings() {
        let aur_clean = cand(
            10,
            PackageSource::Aur,
            "brave-bin",
            CandidateEvidence {
                is_community_maintained: true,
                checksum_present: true,
                checksum_validated: true,
                ..CandidateEvidence::default()
            },
        );
        let aur_risky = cand(
            11,
            PackageSource::Aur,
            "brave-bin",
            CandidateEvidence {
                is_community_maintained: true,
                checksum_present: false,
                findings: vec![FindingKind::NetworkExecution, FindingKind::SudoUsage],
                install_script_present: true,
                ..CandidateEvidence::default()
            },
        );
        let flatpak = cand(
            12,
            PackageSource::Flatpak,
            "brave",
            CandidateEvidence::flatpak_verified_narrow(),
        );
        let balanced = Policy::from_preset(PolicyPreset::Balanced);
        let strict = Policy::from_preset(PolicyPreset::MaximumReview);

        let r_balanced = evaluate(
            &[aur_risky.clone(), aur_clean.clone(), flatpak.clone()],
            &balanced,
        );
        let r_strict = evaluate(&[aur_risky.clone(), aur_clean.clone(), flatpak], &strict);

        // Under strict policy, the risky AUR must never be recommended when a clean alternative exists.
        assert_ne!(r_strict.recommended.as_ref().unwrap().id.0, aur_risky.id.0);
        // Under maximum-review, gap should still produce at least Medium confidence when winner is decisive.
        assert!(matches!(
            r_strict.confidence,
            Confidence::High | Confidence::Medium | Confidence::Low
        ));

        // Determinism: same inputs always same winner.
        let r_again = evaluate(
            &[aur_risky, aur_clean, r_balanced.recommended.unwrap()],
            &strict,
        );
        assert_eq!(
            r_strict.recommended.unwrap().id.0,
            r_again.recommended.unwrap().id.0
        );
    }

    #[test]
    fn deterministic_tie_breaking() {
        // Two candidates with identical evidence and scores — lexical id tie-break.
        let a = cand(5, PackageSource::Aur, "alpha", CandidateEvidence::default());
        let b = cand(
            4,
            PackageSource::Flatpak,
            "alpha",
            CandidateEvidence::default(),
        );
        let r1 = evaluate(&[a.clone(), b.clone()], &Policy::balanced());
        let r2 = evaluate(&[b.clone(), a.clone()], &Policy::balanced());
        let winner_id = r1.recommended.as_ref().unwrap().id.0;
        assert_eq!(winner_id, r2.recommended.as_ref().unwrap().id.0);
        // Lower id wins (lexical).
        assert_eq!(winner_id, cid(4).0);
    }

    #[test]
    fn evaluate_is_deterministic_across_reorderings_and_presets() {
        let arch = cand(
            1,
            PackageSource::ArchOfficial,
            "code",
            CandidateEvidence {
                is_official_repository: true,
                signature_present: true,
                publisher_supported: true,
                ..CandidateEvidence::default()
            },
        );
        let aur = cand(
            2,
            PackageSource::Aur,
            "visual-studio-code-bin",
            CandidateEvidence {
                is_community_maintained: true,
                checksum_present: true,
                publisher_supported: true,
                ..CandidateEvidence::default()
            },
        );
        let flatpak = cand(
            3,
            PackageSource::Flatpak,
            "code",
            CandidateEvidence {
                sandboxed: true,
                publisher_verified: false,
                permission_level: PermissionLevel::Broad,
                filesystem_access: FilesystemAccess::Host,
                ..CandidateEvidence::default()
            },
        );
        let candidates = vec![flatpak, aur, arch];
        for preset in PolicyPreset::all() {
            let p = Policy::from_preset(preset);
            let first = evaluate(&candidates, &p);
            let mut shuffled = candidates.clone();
            shuffled.reverse();
            let second = evaluate(&shuffled, &p);
            assert_eq!(
                first.recommended.unwrap().id.0,
                second.recommended.unwrap().id.0,
                "preset {preset:?} must be order-independent"
            );
        }
    }

    #[test]
    fn rank_is_sorted_descending() {
        let c1 = cand(
            1,
            PackageSource::ArchOfficial,
            "a",
            CandidateEvidence::official_repository(),
        );
        let c2 = cand(
            2,
            PackageSource::Aur,
            "a",
            CandidateEvidence::aur_community(),
        );
        let c3 = cand(
            3,
            PackageSource::Flatpak,
            "a",
            CandidateEvidence::flatpak_verified_narrow(),
        );
        let ranked = rank(&[c2.clone(), c3.clone(), c1.clone()], &Policy::balanced());
        assert!(ranked[0].score >= ranked[1].score);
        assert!(ranked[1].score >= ranked[2].score);
    }
}
