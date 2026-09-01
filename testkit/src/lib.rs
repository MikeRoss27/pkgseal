//! `pkgseal-testkit` — reusable test infrastructure.
//!
//! ```text
//! testkit/
//!   builders.rs      -> fluent builders for PackageSummary, PackageDetails,
//!                       CandidateEvidence, PolicyCandidate, ResolvedApplication
//!   fixtures.rs      -> offline fixture loading from `fixtures/` (arch/aur/flatpak)
//!   fake_sources.rs  -> in-memory PackageSourceAdapter (no network)
//!   assertions.rs    -> assert helpers for grouping / policy
//! ```
//!
//! The crate intentionally stays lightweight and never performs network or
//! privileged operations.

pub mod assertions;
pub mod builders;
pub mod fake_sources;
pub mod fixtures;

// Builders re-exports for ergonomic `candidate().aur()....` usage.
pub use builders::{
    CandidateEvidenceBuilder, PackageDetailsBuilder, PackageSummaryBuilder, PolicyCandidateBuilder,
    ResolvedApplicationBuilder, candidate, candidate_evidence, package_details, package_summary,
    resolved_application,
};

// Fixtures re-exports — most common offline helpers.
pub use fixtures::{
    FixtureError, fixture_path, fixtures_root, list_fixtures, load_all_details, load_all_fixtures,
    load_arch_details, load_arch_summary, load_aur_details, load_aur_summary, load_details,
    load_fixture, load_flatpak_details, load_flatpak_summary, load_json, load_summary,
};

// Fake source re-exports.
pub use fake_sources::{FakeSource, fake_source_from_details, fake_source_seeded};

// Assertion re-exports.
pub use assertions::{
    assert_alternative_warning, assert_alternatives_count, assert_candidate_grouped,
    assert_confidence, assert_grouped_contains, assert_grouped_count, assert_no_recommendation,
    assert_reason_contribution_at_least, assert_reason_present, assert_recommended,
    assert_recommended_source, assert_recommended_with_policy, assert_warning_absent,
    assert_warning_present,
};
