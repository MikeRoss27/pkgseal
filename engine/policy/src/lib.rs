//! PkgSeal policy engine — pure, deterministic, no IO.
//!
//! ```text
//! Candidates + Evidence + UserPolicy -> Recommendation + Reasons + Warnings
//! ```
//!
//! Architecture invariants:
//! - No `tokio`, `reqwest`, filesystem, or network access.
//! - Deterministic: same input order-independent ranking (stable tie-break), no randomness.
//! - No hard-coded universal `Arch > Flatpak > Aur` ordering — ranking is derived from
//!   per-candidate evidence interpreted through the selected `Policy` preset.
//! - Explainable: every score delta maps to a `Reason` or `Warning` that can be rendered as
//!   `Evidence -> Policy -> Recommendation`.
//!
//! Modules mirror `docs/architecture/overview.md §29`:
//! - `decision` — core types (`PolicyCandidate`, `CandidateEvidence`, `Recommendation`, `Reason`, `Warning`, `Confidence`)
//! - `presets` — four policy presets (`Balanced`, `NativeFirst`, `SandboxFirst`, `MaximumReview`) with distinct weight tables
//! - `rules` — atomic, testable rules (`provenance`, `publisher`, `integrity`, `sandbox`, `permissions`, `findings`)
//! - `engine` — deterministic `evaluate` / `rank` entry points
//! - `explanation` — `Evidence -> Policy -> Recommendation` human-readable rendering

pub mod decision;
pub mod engine;
pub mod explanation;
pub mod presets;
pub mod rules;

pub use decision::{
    Alternative, CandidateEvidence, Confidence, DbusAccess, FilesystemAccess, FindingKind,
    PermissionLevel, PolicyCandidate, Reason, ReasonKind, Recommendation, Severity, Warning,
    WarningKind,
};
pub use engine::{ScoredCandidate, evaluate, rank};
pub use explanation::{EvidenceLine, Explanation};
pub use presets::{Policy, PolicyPreset, PolicyWeights};
pub use rules::{RuleOutcome, score_candidate};

#[cfg(test)]
mod tests {
    use super::*;
    use pkgseal_domain::{PackageName, PackageSource};

    #[test]
    fn crate_reexports_are_accessible() {
        let policy = Policy::balanced();
        assert_eq!(policy.preset, PolicyPreset::Balanced);

        let candidate = PolicyCandidate::new(
            PackageSource::ArchOfficial,
            PackageName::new("brave").unwrap(),
            "1.0",
            CandidateEvidence::official_repository(),
        );
        let rec = evaluate(std::slice::from_ref(&candidate), &policy);
        assert!(rec.recommended.is_some());
    }

    #[test]
    fn all_presets_produce_deterministic_rankings() {
        for preset in PolicyPreset::all() {
            let policy = Policy::from_preset(preset);
            let candidates = vec![
                PolicyCandidate::new(
                    PackageSource::Flatpak,
                    PackageName::new("brave").unwrap(),
                    "1.0",
                    CandidateEvidence::flatpak_verified_narrow(),
                ),
                PolicyCandidate::new(
                    PackageSource::Aur,
                    PackageName::new("brave-bin").unwrap(),
                    "1.0",
                    CandidateEvidence::aur_community(),
                ),
            ];
            let first = evaluate(&candidates, &policy);
            let mut reversed = candidates.clone();
            reversed.reverse();
            let second = evaluate(&reversed, &policy);
            assert_eq!(
                first.recommended.unwrap().id.0,
                second.recommended.unwrap().id.0,
                "preset {preset:?} must be order-independent"
            );
        }
    }
}
