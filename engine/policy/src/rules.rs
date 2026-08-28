use crate::decision::{
    CandidateEvidence, DbusAccess, FilesystemAccess, PermissionLevel, PolicyCandidate, Reason,
    ReasonKind, Severity, Warning, WarningKind,
};
use crate::presets::PolicyWeights;

/// Aggregated contribution of a single atomic rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleOutcome {
    pub score: i32,
    pub reasons: Vec<Reason>,
    pub warnings: Vec<Warning>,
}

impl RuleOutcome {
    pub fn empty() -> Self {
        Self {
            score: 0,
            reasons: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn with_score(mut self, delta: i32) -> Self {
        self.score += delta;
        self
    }
}

fn reason(kind: ReasonKind, detail: impl Into<String>, contribution: i32) -> Reason {
    Reason {
        kind,
        detail: detail.into(),
        contribution,
    }
}

fn warning(
    kind: WarningKind,
    detail: impl Into<String>,
    severity: Severity,
    penalty: i32,
) -> Warning {
    Warning {
        kind,
        detail: detail.into(),
        severity,
        penalty,
    }
}

// ---------------------------------------------------------------------------
// Atomic rules — each is pure, deterministic, and depends only on evidence + weights.
// No IO, no global state.
// ---------------------------------------------------------------------------

/// Provenance: official repository vs community.
pub fn rule_provenance(candidate: &PolicyCandidate, w: &PolicyWeights) -> RuleOutcome {
    let e = &candidate.evidence;
    let mut out = RuleOutcome::empty();

    if e.is_official_repository {
        out.score += w.official_repository;
        out.reasons.push(reason(
            ReasonKind::OfficialRepository,
            format!(
                "{} is from the official Arch repository",
                candidate.package_name
            ),
            w.official_repository,
        ));
        // Native integration is a separate reason but only meaningful for official native packages.
        if !e.sandboxed {
            out.score += w.native_integration_bonus;
            out.reasons.push(reason(
                ReasonKind::NativeIntegration,
                "native system integration".to_string(),
                w.native_integration_bonus,
            ));
        }
    }

    if e.is_community_maintained {
        out.score += w.community_penalty;
        // Penalty magnitude stored as positive for display; score delta is negative.
        let penalty = w.community_penalty.abs();
        out.warnings.push(warning(
            WarningKind::CommunityMaintained,
            format!(
                "{} is community-maintained (AUR) — recipe is unaudited at runtime",
                candidate.package_name
            ),
            Severity::Medium,
            penalty,
        ));
    }

    out
}

/// Publisher verification and support.
pub fn rule_publisher(candidate: &PolicyCandidate, w: &PolicyWeights) -> RuleOutcome {
    let e = &candidate.evidence;
    let mut out = RuleOutcome::empty();

    if e.publisher_verified {
        out.score += w.publisher_verified;
        out.reasons.push(reason(
            ReasonKind::PublisherVerified,
            "publisher is verified".to_string(),
            w.publisher_verified,
        ));
    } else if e.sandboxed {
        // Flatpak sandboxed but unverified is a factual risk to surface.
        out.warnings.push(warning(
            WarningKind::UnverifiedPublisher,
            "Flatpak publisher is not verified on Flathub".to_string(),
            Severity::Low,
            0,
        ));
    }

    if e.publisher_supported {
        out.score += w.publisher_supported;
        out.reasons.push(reason(
            ReasonKind::PublisherSupported,
            "publisher documents this install method as supported".to_string(),
            w.publisher_supported,
        ));
    }

    out
}

/// Signatures, checksums, and integrity.
pub fn rule_integrity(candidate: &PolicyCandidate, w: &PolicyWeights) -> RuleOutcome {
    let e = &candidate.evidence;
    let mut out = RuleOutcome::empty();

    if e.signature_present {
        out.score += w.signature_present;
        out.reasons.push(reason(
            ReasonKind::SignaturePresent,
            "package signature is present".to_string(),
            w.signature_present,
        ));
    } else {
        // Missing signature is more severe for official repos than for AUR (where it's often absent).
        let severity = if e.is_official_repository {
            Severity::High
        } else {
            Severity::Low
        };
        out.warnings.push(warning(
            WarningKind::MissingSignature,
            "no package signature available".to_string(),
            severity,
            0,
        ));
    }

    if e.checksum_present {
        out.score += w.checksum_present;
        out.reasons.push(reason(
            ReasonKind::ChecksumPresent,
            "checksum is present".to_string(),
            w.checksum_present,
        ));
        if e.checksum_validated {
            out.score += w.checksum_validated_bonus;
            out.reasons.push(reason(
                ReasonKind::ChecksumPresent,
                "checksum validated".to_string(),
                w.checksum_validated_bonus,
            ));
        }
    } else if e.is_community_maintained {
        // AUR without checksums is a concrete risk: downloaded upstream artifacts are not pinned.
        out.warnings.push(warning(
            WarningKind::MissingChecksum,
            "AUR recipe has no checksums (skipping integrity pinning)".to_string(),
            Severity::Medium,
            0,
        ));
    }

    out
}

/// Sandbox presence — bonus for narrow, penalty for broad.
pub fn rule_sandbox(candidate: &PolicyCandidate, w: &PolicyWeights) -> RuleOutcome {
    let e = &candidate.evidence;
    let mut out = RuleOutcome::empty();

    if !e.sandboxed {
        return out;
    }

    match e.permission_level {
        PermissionLevel::Narrow | PermissionLevel::Moderate => {
            // Moderate still gets the narrow bonus in this model — narrowness is measured
            // more precisely by the permissions rule. SandboxFirst strengthens this.
            out.score += w.sandboxed_narrow_bonus;
            out.reasons.push(reason(
                ReasonKind::Sandboxed,
                "application is sandboxed with contained permissions".to_string(),
                w.sandboxed_narrow_bonus,
            ));
        }
        PermissionLevel::Broad => {
            out.score += w.sandboxed_broad_penalty;
            let penalty = w.sandboxed_broad_penalty.abs();
            out.warnings.push(warning(
                WarningKind::BroadPermissions,
                "sandboxed but with broad permissions — sandbox value is reduced".to_string(),
                Severity::Medium,
                penalty,
            ));
        }
        PermissionLevel::Excessive => {
            out.score += w.sandboxed_broad_penalty;
            out.score += w.excessive_permissions_penalty;
            let penalty =
                (w.sandboxed_broad_penalty.abs() + w.excessive_permissions_penalty.abs()) / 2;
            out.warnings.push(warning(
                WarningKind::ExcessivePermissions,
                "sandboxed but permissions are excessive — sandbox provides little isolation"
                    .to_string(),
                Severity::High,
                penalty,
            ));
        }
    }

    out
}

/// Permission analysis — filesystem, D-Bus, network, devices.
pub fn rule_permissions(candidate: &PolicyCandidate, w: &PolicyWeights) -> RuleOutcome {
    let e: &CandidateEvidence = &candidate.evidence;
    let mut out = RuleOutcome::empty();

    // Filesystem
    match e.filesystem_access {
        FilesystemAccess::None | FilesystemAccess::Limited => {}
        FilesystemAccess::HomeRo | FilesystemAccess::HomeRw => {
            if e.permission_level == PermissionLevel::Narrow {
                // Home access with Narrow overall is considered intentional (e.g. file manager).
                // No penalty here — let narrow reason below apply.
            } else {
                out.score += w.broad_permissions_penalty;
                out.warnings.push(warning(
                    WarningKind::BroadPermissions,
                    "home directory access".to_string(),
                    Severity::Medium,
                    w.broad_permissions_penalty.abs(),
                ));
            }
        }
        FilesystemAccess::Host => {
            out.score += w.host_filesystem_penalty;
            out.warnings.push(warning(
                WarningKind::HostFilesystemAccess,
                "host filesystem access (filesystem=host)".to_string(),
                Severity::High,
                w.host_filesystem_penalty.abs(),
            ));
        }
    }

    // D-Bus
    match e.dbus_access {
        DbusAccess::None | DbusAccess::SessionLimited => {}
        DbusAccess::SessionFull => {
            out.score += w.broad_permissions_penalty;
            out.warnings.push(warning(
                WarningKind::BroadPermissions,
                "broad session D-Bus access".to_string(),
                Severity::Medium,
                w.broad_permissions_penalty.abs(),
            ));
        }
        DbusAccess::System | DbusAccess::Host => {
            out.score += w.host_dbus_penalty;
            out.warnings.push(warning(
                WarningKind::HostDbusAccess,
                "host or system D-Bus access".to_string(),
                Severity::High,
                w.host_dbus_penalty.abs(),
            ));
        }
    }

    if e.device_access && e.permission_level != PermissionLevel::Narrow {
        out.score += w.broad_permissions_penalty;
        out.warnings.push(warning(
            WarningKind::BroadPermissions,
            "device access (camera, input, etc.)".to_string(),
            Severity::Medium,
            w.broad_permissions_penalty.abs(),
        ));
    }

    if e.network_access && w.network_penalty < 0 && e.sandboxed {
        // Network is expected for most apps; only penalize mildly when the preset says so
        // (MaximumReview and SandboxFirst care more).
        out.score += w.network_penalty;
        if w.network_penalty.abs() >= 4 {
            out.warnings.push(warning(
                WarningKind::NetworkAccess,
                "network access".to_string(),
                Severity::Low,
                w.network_penalty.abs(),
            ));
        }
    }

    // PermissionLevel aggregate
    match e.permission_level {
        PermissionLevel::Narrow => {
            if !e.sandboxed {
                // Narrow for non-sandboxed is still a positive signal (minimal filesystem/dbus).
                out.score += w.narrow_permissions_bonus;
                out.reasons.push(reason(
                    ReasonKind::NarrowPermissions,
                    "minimal permissions".to_string(),
                    w.narrow_permissions_bonus,
                ));
            } else {
                // Sandboxed + narrow already rewarded via rule_sandbox; add a small stacking bonus.
                out.score += w.narrow_permissions_bonus / 2;
                if w.narrow_permissions_bonus / 2 != 0 {
                    out.reasons.push(reason(
                        ReasonKind::NarrowPermissions,
                        "narrow sandbox permissions".to_string(),
                        w.narrow_permissions_bonus / 2,
                    ));
                }
            }
        }
        PermissionLevel::Moderate => {}
        PermissionLevel::Broad => {
            // Already counted via filesystem/dbus above; add residual if not already warned via those.
            if e.filesystem_access == FilesystemAccess::Limited && e.dbus_access == DbusAccess::None
            {
                out.score += w.broad_permissions_penalty;
                out.warnings.push(warning(
                    WarningKind::BroadPermissions,
                    "broad permissions".to_string(),
                    Severity::Medium,
                    w.broad_permissions_penalty.abs(),
                ));
            }
        }
        PermissionLevel::Excessive => {
            out.score += w.excessive_permissions_penalty;
            out.warnings.push(warning(
                WarningKind::ExcessivePermissions,
                "excessive permissions".to_string(),
                Severity::High,
                w.excessive_permissions_penalty.abs(),
            ));
        }
    }

    out
}

/// Static findings and maintenance signals.
pub fn rule_findings(candidate: &PolicyCandidate, w: &PolicyWeights) -> RuleOutcome {
    let e = &candidate.evidence;
    let mut out = RuleOutcome::empty();

    if e.findings.is_empty() {
        if e.is_community_maintained {
            // Absence of findings for AUR is a mild positive — inspection was clean.
            out.reasons.push(reason(
                ReasonKind::NoKnownFindings,
                "no suspicious patterns detected in PKGBUILD".to_string(),
                0,
            ));
        }
        return out;
    }

    let penalty = w.findings_penalty_per_finding * e.findings.len() as i32;
    out.score += penalty;
    let detail = format!(
        "{} finding(s) in PKGBUILD/.install: {}",
        e.findings.len(),
        e.findings
            .iter()
            .map(|f| format!("{f:?}"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    out.warnings.push(warning(
        WarningKind::FindingsDetected,
        detail,
        Severity::High,
        penalty.abs(),
    ));

    if e.install_script_present {
        out.score += w.install_script_penalty;
        out.warnings.push(warning(
            WarningKind::InstallScriptPresent,
            ".install script is present".to_string(),
            Severity::Medium,
            w.install_script_penalty.abs(),
        ));
    }

    if e.build_logic_changed {
        out.score += w.build_changed_penalty;
        out.warnings.push(warning(
            WarningKind::BuildLogicChanged,
            "build logic changed since previous snapshot".to_string(),
            Severity::Medium,
            w.build_changed_penalty.abs(),
        ));
    }

    // Even when findings exist, surface stale evidence as an additional warning if old.
    if let Some(days) = e.last_update_days_ago
        && days > 365
    {
        out.warnings.push(warning(
            WarningKind::OutdatedEvidence,
            format!("package last updated {days} days ago — evidence may be stale"),
            Severity::Low,
            0,
        ));
    }

    out
}

/// Score a candidate by applying all atomic rules and summing contributions.
/// Deterministic: same candidate + same weights -> same score/reasons/warnings.
pub fn score_candidate(candidate: &PolicyCandidate, weights: &PolicyWeights) -> RuleOutcome {
    let mut aggregate = RuleOutcome::empty();
    for rule in [
        rule_provenance,
        rule_publisher,
        rule_integrity,
        rule_sandbox,
        rule_permissions,
        rule_findings,
    ] {
        let r = rule(candidate, weights);
        aggregate.score += r.score;
        aggregate.reasons.extend(r.reasons);
        aggregate.warnings.extend(r.warnings);
    }
    // Stable ordering for explainability: reasons/warnings sorted by kind string.
    aggregate.reasons.sort_by(|a, b| {
        a.kind
            .as_str()
            .cmp(b.kind.as_str())
            .then_with(|| a.detail.cmp(&b.detail))
    });
    aggregate.warnings.sort_by(|a, b| {
        a.kind
            .as_str()
            .cmp(b.kind.as_str())
            .then_with(|| a.detail.cmp(&b.detail))
    });
    aggregate
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decision::{CandidateEvidence, DbusAccess, FilesystemAccess, PermissionLevel};
    use crate::presets::PolicyWeights;
    use pkgseal_domain::{PackageName, PackageSource};

    fn candidate(evidence: CandidateEvidence) -> PolicyCandidate {
        PolicyCandidate::new(
            PackageSource::Flatpak,
            PackageName::new("test-pkg").unwrap(),
            "1.0",
            evidence,
        )
    }

    #[test]
    fn provenance_official_adds_bonus_not_penalty() {
        let c = candidate(CandidateEvidence {
            is_official_repository: true,
            signature_present: true,
            ..Default::default()
        });
        let r = rule_provenance(&c, &PolicyWeights::balanced());
        assert!(r.score > 0);
        assert!(
            r.reasons
                .iter()
                .any(|x| x.kind == ReasonKind::OfficialRepository)
        );
        assert!(r.warnings.is_empty());
    }

    #[test]
    fn community_penalizes_and_warns() {
        let c = candidate(CandidateEvidence {
            is_community_maintained: true,
            ..Default::default()
        });
        let r = rule_provenance(&c, &PolicyWeights::balanced());
        assert!(r.score < 0);
        assert!(
            r.warnings
                .iter()
                .any(|w| w.kind == WarningKind::CommunityMaintained)
        );
    }

    #[test]
    fn sandboxed_narrow_scores_higher_than_broad() {
        let narrow = candidate(CandidateEvidence {
            sandboxed: true,
            permission_level: PermissionLevel::Narrow,
            filesystem_access: FilesystemAccess::Limited,
            dbus_access: DbusAccess::SessionLimited,
            ..Default::default()
        });
        let broad = candidate(CandidateEvidence {
            sandboxed: true,
            permission_level: PermissionLevel::Broad,
            filesystem_access: FilesystemAccess::Host,
            dbus_access: DbusAccess::Host,
            ..Default::default()
        });
        let w = PolicyWeights::balanced();
        let rn = rule_sandbox(&narrow, &w);
        let rb = rule_sandbox(&broad, &w);
        // Also include permissions rule contribution for the contrast.
        let pn = rule_permissions(&narrow, &w);
        let pb = rule_permissions(&broad, &w);
        assert!(rn.score + pn.score > rb.score + pb.score);
    }

    #[test]
    fn no_hardcoded_source_ranking_in_rules() {
        // Same neutral evidence across sources should not produce a universal ranking
        // via provenance alone — only evidence should differentiate.
        let base = CandidateEvidence::default();
        let arch = PolicyCandidate::new(
            PackageSource::ArchOfficial,
            PackageName::new("brave").unwrap(),
            "1.0",
            base.clone(),
        );
        let aur = PolicyCandidate::new(
            PackageSource::Aur,
            PackageName::new("brave").unwrap(),
            "1.0",
            base.clone(),
        );
        let flatpak = PolicyCandidate::new(
            PackageSource::Flatpak,
            PackageName::new("brave").unwrap(),
            "1.0",
            base,
        );
        let w = PolicyWeights::balanced();
        // With empty/default evidence, official vs community distinction still applies,
        // but flatpak vs aur with same neutral evidence: flatpak should not automatically beat aur
        // by virtue of being flatpak — score them purely via evidence.
        // Here aur has is_community_maintained false (default), so scores should be equal.
        let s_arch = score_candidate(&arch, &w).score;
        let s_aur = score_candidate(&aur, &w).score;
        let s_flatpak = score_candidate(&flatpak, &w).score;
        assert_eq!(s_aur, s_flatpak);
        // arch with is_official false is same as above — no implicit arch bonus.
        assert_eq!(s_arch, s_aur);
    }

    #[test]
    fn scoring_is_deterministic() {
        let c = candidate(CandidateEvidence {
            sandboxed: true,
            permission_level: PermissionLevel::Narrow,
            publisher_verified: true,
            publisher_supported: true,
            ..Default::default()
        });
        let w = PolicyWeights::balanced();
        let a = score_candidate(&c, &w);
        let b = score_candidate(&c, &w);
        assert_eq!(a, b);
    }

    #[test]
    fn findings_increase_penalty_linearly() {
        use crate::decision::FindingKind;
        let one = candidate(CandidateEvidence {
            is_community_maintained: true,
            findings: vec![FindingKind::SudoUsage],
            ..Default::default()
        });
        let two = candidate(CandidateEvidence {
            is_community_maintained: true,
            findings: vec![FindingKind::SudoUsage, FindingKind::EvalUsage],
            ..Default::default()
        });
        let w = PolicyWeights::balanced();
        let s1 = score_candidate(&one, &w).score;
        let s2 = score_candidate(&two, &w).score;
        assert!(s2 < s1);
    }

    #[test]
    fn publisher_verified_reason_vs_unverified_warning() {
        let verified = candidate(CandidateEvidence {
            sandboxed: true,
            publisher_verified: true,
            ..Default::default()
        });
        let unverified = candidate(CandidateEvidence {
            sandboxed: true,
            publisher_verified: false,
            ..Default::default()
        });
        let w = PolicyWeights::balanced();
        let rv = rule_publisher(&verified, &w);
        let ru = rule_publisher(&unverified, &w);
        assert!(
            rv.reasons
                .iter()
                .any(|r| r.kind == ReasonKind::PublisherVerified)
        );
        assert!(
            ru.warnings
                .iter()
                .any(|x| x.kind == WarningKind::UnverifiedPublisher)
        );
    }
}
