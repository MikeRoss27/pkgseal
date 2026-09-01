use crate::decision::{Confidence, PolicyCandidate, Recommendation};
use crate::presets::Policy;

/// Human-readable explanation of Evidence -> Policy -> Recommendation chain.
///
/// Pure data, no IO. Rendering is via `to_text()` / `to_markdown()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Explanation {
    pub title: String,
    pub policy_summary: String,
    pub evidence_summary: Vec<EvidenceLine>,
    pub recommendation_summary: String,
    pub reasons_text: Vec<String>,
    pub warnings_text: Vec<String>,
    pub alternatives_text: Vec<String>,
    pub confidence: Confidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceLine {
    pub candidate: String,
    pub evidence: Vec<String>,
}

impl Explanation {
    pub fn from_recommendation(
        recommendation: &Recommendation,
        policy: &Policy,
        all_candidates: &[PolicyCandidate],
    ) -> Self {
        let confidence = recommendation.confidence;

        let title = if let Some(c) = &recommendation.recommended {
            format!("Recommended: {} · {}", c.package_name, c.source)
        } else {
            "No recommendation — no candidates".to_string()
        };

        let policy_summary = format!(
            "Policy '{}' — {}",
            policy.preset.as_str(),
            policy.preset.description()
        );

        let evidence_summary = all_candidates
            .iter()
            .map(|c| EvidenceLine {
                candidate: format!("{} · {} {}", c.package_name, c.source, c.version),
                evidence: summarize_evidence(c),
            })
            .collect();

        let recommendation_summary = if let Some(winner) = &recommendation.recommended {
            let alt_count = recommendation.alternatives.len();
            format!(
                "PkgSeal recommends {} (score {}, confidence {}) over {} alternative(s).",
                winner.package_name, recommendation.score, confidence, alt_count
            )
        } else {
            "PkgSeal could not determine a recommendation from the available evidence.".to_string()
        };

        let reasons_text = recommendation
            .reasons
            .iter()
            .map(|r| format!("✓ {} — {} (+{})", r.kind.as_str(), r.detail, r.contribution))
            .collect();

        let warnings_text = recommendation
            .warnings
            .iter()
            .map(|w| {
                let pen = if w.penalty != 0 {
                    format!(" (-{})", w.penalty)
                } else {
                    String::new()
                };
                format!("⚠ {} — {}{}", w.kind.as_str(), w.detail, pen)
            })
            .collect();

        let alternatives_text = recommendation
            .alternatives
            .iter()
            .map(|alt| {
                format!(
                    "· {} · {} {} — score {}",
                    alt.candidate.package_name,
                    alt.candidate.source,
                    alt.candidate.version,
                    alt.score
                )
            })
            .collect();

        Self {
            title,
            policy_summary,
            evidence_summary,
            recommendation_summary,
            reasons_text,
            warnings_text,
            alternatives_text,
            confidence,
        }
    }

    /// Compact human-readable rendering, suitable for CLI or UI preview.
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        out.push_str(&self.title);
        out.push('\n');
        out.push_str(&self.policy_summary);
        out.push('\n');
        out.push_str(&format!("Confidence: {}\n", self.confidence));
        out.push('\n');
        out.push_str(&self.recommendation_summary);
        out.push('\n');

        if !self.reasons_text.is_empty() {
            out.push_str("\nWhy PkgSeal recommends this:\n");
            for r in &self.reasons_text {
                out.push_str(r);
                out.push('\n');
            }
        }
        if !self.warnings_text.is_empty() {
            out.push_str("\nTrade-offs / warnings:\n");
            for w in &self.warnings_text {
                out.push_str(w);
                out.push('\n');
            }
        }
        if !self.alternatives_text.is_empty() {
            out.push_str("\nAlternatives:\n");
            for a in &self.alternatives_text {
                out.push_str(a);
                out.push('\n');
            }
        }
        if !self.evidence_summary.is_empty() {
            out.push_str("\nEvidence per candidate:\n");
            for line in &self.evidence_summary {
                out.push_str(&format!("  {}:\n", line.candidate));
                for e in &line.evidence {
                    out.push_str(&format!("    - {e}\n"));
                }
            }
        }
        out.push_str("\nEvidence -> Policy -> Recommendation\n");
        out
    }

    /// Markdown rendering for UI or docs.
    pub fn to_markdown(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("## {}\n\n", self.title));
        out.push_str(&format!("**Policy:** {}\n\n", self.policy_summary));
        out.push_str(&format!("**Confidence:** `{}`\n\n", self.confidence));
        out.push_str(&format!("{}\n\n", self.recommendation_summary));

        if !self.reasons_text.is_empty() {
            out.push_str("### Why PkgSeal recommends this\n\n");
            for r in &self.reasons_text {
                out.push_str(&format!("- {r}\n"));
            }
            out.push('\n');
        }
        if !self.warnings_text.is_empty() {
            out.push_str("### Trade-offs / warnings\n\n");
            for w in &self.warnings_text {
                out.push_str(&format!("- {w}\n"));
            }
            out.push('\n');
        }
        if !self.alternatives_text.is_empty() {
            out.push_str("### Alternatives\n\n");
            for a in &self.alternatives_text {
                out.push_str(&format!("- {a}\n"));
            }
            out.push('\n');
        }
        out.push_str("> Evidence -> Policy -> Recommendation\n");
        out
    }
}

fn summarize_evidence(c: &PolicyCandidate) -> Vec<String> {
    let e = &c.evidence;
    let mut lines = Vec::new();
    if e.is_official_repository {
        lines.push("official repository".to_string());
    }
    if e.is_community_maintained {
        lines.push("community-maintained (AUR)".to_string());
    }
    if e.publisher_verified {
        lines.push("publisher verified".to_string());
    } else if e.sandboxed {
        lines.push("publisher not verified".to_string());
    }
    if e.publisher_supported {
        lines.push("publisher-supported install method".to_string());
    }
    if e.signature_present {
        lines.push("signature present".to_string());
    } else {
        lines.push("no signature".to_string());
    }
    if e.checksum_present {
        if e.checksum_validated {
            lines.push("checksum present and validated".to_string());
        } else {
            lines.push("checksum present".to_string());
        }
    } else if e.is_community_maintained {
        lines.push("no checksum".to_string());
    }
    if e.sandboxed {
        lines.push(format!(
            "sandboxed — permissions: {:?}, fs: {:?}, dbus: {:?}",
            e.permission_level, e.filesystem_access, e.dbus_access
        ));
        if e.network_access {
            lines.push("network access: yes".to_string());
        }
    } else {
        lines.push("not sandboxed (native)".to_string());
    }
    if !e.findings.is_empty() {
        let kinds = e
            .findings
            .iter()
            .map(|f| format!("{f:?}"))
            .collect::<Vec<_>>()
            .join(", ");
        lines.push(format!("findings: {kinds}"));
    } else if e.is_community_maintained {
        lines.push("no suspicious patterns in PKGBUILD".to_string());
    }
    if e.install_script_present {
        lines.push(".install script present".to_string());
    }
    if e.build_logic_changed {
        lines.push("build logic changed since previous snapshot".to_string());
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decision::{CandidateEvidence, PermissionLevel};
    use crate::engine::evaluate;
    use crate::presets::Policy;
    use pkgseal_domain::{PackageName, PackageSource};

    fn candidate(
        source: PackageSource,
        name: &str,
        evidence: CandidateEvidence,
    ) -> PolicyCandidate {
        PolicyCandidate::new(source, PackageName::new(name).unwrap(), "1.0", evidence)
    }

    #[test]
    fn explanation_contains_chain() {
        let arch = candidate(
            PackageSource::ArchOfficial,
            "brave",
            CandidateEvidence {
                is_official_repository: true,
                signature_present: true,
                publisher_supported: true,
                ..Default::default()
            },
        );
        let aur = candidate(
            PackageSource::Aur,
            "brave-bin",
            CandidateEvidence {
                is_community_maintained: true,
                checksum_present: true,
                ..Default::default()
            },
        );
        let policy = Policy::balanced();
        let rec = evaluate(&[aur.clone(), arch.clone()], &policy);
        let exp = Explanation::from_recommendation(&rec, &policy, &[aur, arch]);
        let text = exp.to_text();
        assert!(text.contains("Evidence -> Policy -> Recommendation"));
        assert!(text.contains("Policy"));
        assert!(text.contains("Confidence"));
        assert!(text.contains("brave"));
    }

    #[test]
    fn explanation_markdown_structure() {
        let flatpak = candidate(
            PackageSource::Flatpak,
            "brave",
            CandidateEvidence::flatpak_verified_narrow(),
        );
        let policy = Policy::sandbox_first();
        let rec = evaluate(std::slice::from_ref(&flatpak), &policy);
        let exp = Explanation::from_recommendation(&rec, &policy, &[flatpak]);
        let md = exp.to_markdown();
        assert!(md.contains("## Recommended"));
        assert!(md.contains("**Policy:**"));
        assert!(md.contains("**Confidence:**"));
        assert!(md.contains("Evidence -> Policy -> Recommendation"));
    }

    #[test]
    fn explanation_no_candidates() {
        let policy = Policy::balanced();
        let rec = evaluate(&[], &policy);
        let exp = Explanation::from_recommendation(&rec, &policy, &[]);
        assert!(exp.title.contains("No recommendation"));
        assert!(exp.reasons_text.is_empty());
    }

    #[test]
    fn evidence_summarised_per_candidate() {
        let c = candidate(
            PackageSource::Flatpak,
            "spotify",
            CandidateEvidence {
                sandboxed: true,
                permission_level: PermissionLevel::Broad,
                publisher_verified: true,
                ..Default::default()
            },
        );
        let policy = Policy::balanced();
        let rec = evaluate(std::slice::from_ref(&c), &policy);
        let exp = Explanation::from_recommendation(&rec, &policy, &[c]);
        assert!(!exp.evidence_summary.is_empty());
        let first = &exp.evidence_summary[0];
        assert!(first.evidence.iter().any(|s| s.contains("sandboxed")));
    }
}
