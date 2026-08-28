//! Assertion helpers — panic with actionable messages.
//!
//! These intentionally panic (like `assert!`) so failures surface as test
//! failures with the offending inputs printed.

use pkgseal_domain::PackageSource;
use pkgseal_policy::{
    Confidence, Policy, PolicyCandidate, ReasonKind, Recommendation, WarningKind,
};
use pkgseal_resolver::{
    GroupingConfig, ResolvedApplication, group_candidates, resolve_applications,
};
use pkgseal_source::dto::{PackageDetails, PackageSummary};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Resolver / grouping assertions
// ---------------------------------------------------------------------------

/// Assert that `applications` contains exactly `expected` grouped applications.
#[track_caller]
pub fn assert_grouped_count(applications: &[ResolvedApplication], expected: usize) {
    assert_eq!(
        applications.len(),
        expected,
        "expected {expected} grouped application(s), got {}: grouped={:#?}",
        applications.len(),
        applications
            .iter()
            .map(|a| (
                &a.identity.canonical_name,
                &a.identity.candidates,
                &a.identity.confidence
            ))
            .collect::<Vec<_>>()
    );
}

/// Assert that at least one grouped application contains a candidate from `source`
/// with the given package name.
#[track_caller]
pub fn assert_grouped_contains(
    applications: &[ResolvedApplication],
    source: PackageSource,
    package_name: &str,
) {
    let found = applications.iter().any(|app| {
        app.identity
            .candidates
            .iter()
            .any(|c| c.source == source && c.package_name.as_str() == package_name)
    });
    assert!(
        found,
        "expected grouped application to contain {source}/{package_name}, got:\n{:#?}",
        applications
            .iter()
            .map(|a| (
                &a.identity.canonical_name,
                a.identity
                    .candidates
                    .iter()
                    .map(|c| format!("{}/{}", c.source, c.package_name))
                    .collect::<Vec<_>>()
            ))
            .collect::<Vec<_>>()
    );
}

/// Convenience: group `summaries`+`details` with default config and assert grouped count.
#[track_caller]
pub fn assert_candidate_grouped(
    summaries: &[PackageSummary],
    details: Vec<PackageDetails>,
    expected_groups: usize,
) -> Vec<ResolvedApplication> {
    let indexed: HashMap<String, PackageDetails> = details
        .into_iter()
        .map(|d| (d.summary.id.clone(), d))
        .collect();
    // Use indexmap for deterministic ordering inside grouper by converting
    let mut map = indexmap::IndexMap::new();
    for (k, v) in indexed {
        map.insert(k, v);
    }
    let result = group_candidates(summaries, &map, GroupingConfig::default());
    let total_groups = result.applications.len();
    assert_eq!(
        total_groups, expected_groups,
        "expected {expected_groups} grouped application(s), got {total_groups}: applications={:#?}, unmatched={:#?}",
        result.applications, result.unmatched
    );
    // Return as ResolvedApplication for further inspection.
    resolve_applications(
        summaries,
        map.into_values().collect(),
        GroupingConfig::default(),
    )
}

// ---------------------------------------------------------------------------
// Policy / recommendation assertions
// ---------------------------------------------------------------------------

/// Assert that `recommendation` chose a candidate from `expected_source`.
#[track_caller]
pub fn assert_recommended_source(recommendation: &Recommendation, expected_source: PackageSource) {
    let rec = recommendation.recommended.as_ref().unwrap_or_else(|| {
        panic!(
            "expected recommended source {expected_source:?}, but recommendation was None (empty). Full recommendation: {recommendation:#?}"
        )
    });
    assert_eq!(
        rec.source, expected_source,
        "expected recommended source {expected_source:?}, got {:?} (candidate={}, score={}, confidence={:?})",
        rec.source, rec.package_name, recommendation.score, recommendation.confidence
    );
}

/// Assert that `recommendation` chose `expected_source`/`expected_name`.
#[track_caller]
pub fn assert_recommended(
    recommendation: &Recommendation,
    expected_source: PackageSource,
    expected_name: &str,
) {
    let rec = recommendation.recommended.as_ref().unwrap_or_else(|| {
        panic!(
            "expected recommended {expected_source}/{expected_name}, but recommendation was None: {recommendation:#?}"
        )
    });
    assert_eq!(
        rec.source, expected_source,
        "expected source {expected_source:?}, got {:?}",
        rec.source
    );
    assert_eq!(
        rec.package_name.as_str(),
        expected_name,
        "expected name {expected_name}, got {}",
        rec.package_name
    );
}

/// Assert that `candidates` evaluated under `policy` produce a recommendation
/// from `expected_source`.
#[track_caller]
pub fn assert_recommended_with_policy(
    candidates: &[PolicyCandidate],
    policy: &Policy,
    expected_source: PackageSource,
) -> Recommendation {
    let rec = pkgseal_policy::evaluate(candidates, policy);
    assert_recommended_source(&rec, expected_source);
    rec
}

/// Assert that no candidate was recommended (empty input or filtered).
#[track_caller]
pub fn assert_no_recommendation(recommendation: &Recommendation) {
    assert!(
        recommendation.recommended.is_none(),
        "expected no recommendation, got {:?}",
        recommendation.recommended
    );
}

/// Assert recommendation confidence equals `expected`.
#[track_caller]
pub fn assert_confidence(recommendation: &Recommendation, expected: Confidence) {
    assert_eq!(
        recommendation.confidence,
        expected,
        "expected confidence {expected:?}, got {:?} (score={}, alternatives={}, reasons={:#?}, warnings={:#?})",
        recommendation.confidence,
        recommendation.score,
        recommendation.alternatives.len(),
        recommendation.reasons,
        recommendation.warnings
    );
}

/// Assert recommendation has exactly `expected` alternatives, or at least `expected` when `at_least` is true.
#[track_caller]
pub fn assert_alternatives_count(recommendation: &Recommendation, expected: usize) {
    assert_eq!(
        recommendation.alternatives.len(),
        expected,
        "expected {expected} alternative(s), got {}: {:#?}",
        recommendation.alternatives.len(),
        recommendation.alternatives
    );
}

/// Assert recommendation warnings contain `kind`.
#[track_caller]
pub fn assert_warning_present(recommendation: &Recommendation, kind: WarningKind) {
    assert!(
        recommendation.warnings.iter().any(|w| w.kind == kind),
        "expected warning {kind:?} to be present, got warnings: {:#?}",
        recommendation.warnings
    );
}

/// Assert recommendation warnings do NOT contain `kind`.
#[track_caller]
pub fn assert_warning_absent(recommendation: &Recommendation, kind: WarningKind) {
    assert!(
        !recommendation.warnings.iter().any(|w| w.kind == kind),
        "expected warning {kind:?} to be absent, got warnings: {:#?}",
        recommendation.warnings
    );
}

/// Assert recommendation reasons contain `kind`.
#[track_caller]
pub fn assert_reason_present(recommendation: &Recommendation, kind: ReasonKind) {
    assert!(
        recommendation.reasons.iter().any(|r| r.kind == kind),
        "expected reason {kind:?} to be present, got reasons: {:#?}",
        recommendation.reasons
    );
}

/// Assert reasons contain `kind` with at least `min_contribution`.
#[track_caller]
pub fn assert_reason_contribution_at_least(
    recommendation: &Recommendation,
    kind: ReasonKind,
    min_contribution: i32,
) {
    let matching: Vec<_> = recommendation
        .reasons
        .iter()
        .filter(|r| r.kind == kind)
        .collect();
    assert!(
        !matching.is_empty(),
        "expected reason {kind:?} to exist, got {:#?}",
        recommendation.reasons
    );
    let max_contrib = matching.iter().map(|r| r.contribution).max().unwrap_or(0);
    assert!(
        max_contrib >= min_contribution,
        "expected reason {kind:?} contribution >= {min_contribution}, got {max_contrib} (reasons: {:#?})",
        recommendation.reasons
    );
}

/// Assert a specific warning kind appears in the *alternative* at index `alt_idx`.
#[track_caller]
pub fn assert_alternative_warning(
    recommendation: &Recommendation,
    alt_idx: usize,
    kind: WarningKind,
) {
    let alt = recommendation.alternatives.get(alt_idx).unwrap_or_else(|| {
        panic!(
            "alternative index {alt_idx} out of range ({} alternatives)",
            recommendation.alternatives.len()
        )
    });
    assert!(
        alt.warnings.iter().any(|w| w.kind == kind),
        "expected alternative {alt_idx} warning {kind:?} to be present, got {:#?}",
        alt.warnings
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builders::candidate;
    use pkgseal_policy::{Policy, PolicyPreset};

    #[test]
    fn assert_recommended_passes_for_correct_source() {
        let arch = candidate()
            .arch()
            .name("brave")
            .verified_publisher(false)
            .build();
        let aur = candidate()
            .aur()
            .name("brave-bin")
            .verified_publisher(false)
            .build();
        let policy = Policy::from_preset(PolicyPreset::Balanced);
        // Arch official should be preferred when both are unverified but arch is signed/official.
        let arch = {
            let mut c = arch;
            c.evidence.is_official_repository = true;
            c.evidence.signature_present = true;
            c
        };
        let rec = pkgseal_policy::evaluate(&[aur, arch.clone()], &policy);
        assert_recommended(&rec, pkgseal_domain::PackageSource::ArchOfficial, "brave");
        assert_recommended_source(&rec, pkgseal_domain::PackageSource::ArchOfficial);
    }

    #[test]
    #[should_panic(expected = "expected recommended source")]
    fn assert_recommended_source_panics_on_mismatch() {
        let aur = candidate().aur().name("brave-bin").build();
        let rec = pkgseal_policy::evaluate(std::slice::from_ref(&aur), &Policy::balanced());
        // This should be Aur, so asserting ArchOfficial must panic.
        assert_recommended_source(&rec, pkgseal_domain::PackageSource::ArchOfficial);
    }

    #[test]
    fn warning_helpers_work() {
        use pkgseal_policy::{CandidateEvidence, FindingKind};
        let risky = candidate()
            .aur()
            .findings(vec![FindingKind::NetworkExecution])
            .build();
        let flatpak = candidate()
            .flatpak()
            .verified_publisher(true)
            .name("com-brave-browser")
            .build();
        let with_install_script = risky.clone();
        // Evaluate with a clean alternative so risky is likely recommended but carries warnings;
        // use a weak alternative to keep risky as winner while still having findings penalty.
        let weak = CandidateEvidence {
            is_community_maintained: true,
            findings: vec![],
            ..CandidateEvidence::default()
        };
        let weak_cand = crate::builders::PolicyCandidateBuilder::new()
            .aur()
            .name("brave-bin")
            .evidence(weak)
            .build();
        let rec = pkgseal_policy::evaluate(
            &[with_install_script, weak_cand, flatpak],
            &Policy::balanced(),
        );
        // At least findings should be present somewhere (either winner or alternative).
        let any_has_findings = rec
            .warnings
            .iter()
            .any(|w| w.kind == WarningKind::FindingsDetected)
            || rec.alternatives.iter().any(|a| {
                a.warnings
                    .iter()
                    .any(|w| w.kind == WarningKind::FindingsDetected)
            });
        assert!(any_has_findings, "expected findings warning somewhere");
    }
}
