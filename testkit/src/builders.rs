//! Builders for domain, source and policy types.
//!
//! The API is intentionally fluent so downstream crates can write:
//! ```rust
//! use pkgseal_testkit::builders::candidate;
//! let c = candidate().aur().verified_publisher(false).build();
//! ```
//! without touching the filesystem or the network.

use pkgseal_domain::{CandidateId, PackageName, PackageSource};
use pkgseal_policy::{
    CandidateEvidence, DbusAccess, FilesystemAccess, FindingKind, PermissionLevel, PolicyCandidate,
};
use pkgseal_resolver::{ApplicationIdentity, ResolvedApplication};
use pkgseal_source::dto::{PackageDetails, PackageSummary};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// CandidateEvidence builder
// ---------------------------------------------------------------------------

/// Fluent builder for [`CandidateEvidence`].
#[derive(Debug, Clone, Default)]
pub struct CandidateEvidenceBuilder {
    inner: CandidateEvidence,
}

impl CandidateEvidenceBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn official(mut self, v: bool) -> Self {
        self.inner.is_official_repository = v;
        if v {
            self.inner.is_community_maintained = false;
        }
        self
    }

    pub fn is_official_repository(mut self, v: bool) -> Self {
        self.inner.is_official_repository = v;
        self
    }

    pub fn community(mut self, v: bool) -> Self {
        self.inner.is_community_maintained = v;
        if v {
            self.inner.is_official_repository = false;
        }
        self
    }

    pub fn is_community_maintained(mut self, v: bool) -> Self {
        self.inner.is_community_maintained = v;
        self
    }

    pub fn verified_publisher(mut self, v: bool) -> Self {
        self.inner.publisher_verified = v;
        self
    }

    pub fn publisher_verified(self, v: bool) -> Self {
        self.verified_publisher(v)
    }

    pub fn publisher_supported(mut self, v: bool) -> Self {
        self.inner.publisher_supported = v;
        self
    }

    pub fn signature_present(mut self, v: bool) -> Self {
        self.inner.signature_present = v;
        self
    }

    pub fn checksum_present(mut self, v: bool) -> Self {
        self.inner.checksum_present = v;
        self
    }

    pub fn checksum_validated(mut self, v: bool) -> Self {
        self.inner.checksum_validated = v;
        self
    }

    pub fn sandboxed(mut self, v: bool) -> Self {
        self.inner.sandboxed = v;
        self
    }

    pub fn permission_level(mut self, lvl: PermissionLevel) -> Self {
        self.inner.permission_level = lvl;
        self
    }

    pub fn filesystem(mut self, access: FilesystemAccess) -> Self {
        self.inner.filesystem_access = access;
        self
    }

    pub fn filesystem_access(self, access: FilesystemAccess) -> Self {
        self.filesystem(access)
    }

    pub fn dbus(mut self, access: DbusAccess) -> Self {
        self.inner.dbus_access = access;
        self
    }

    pub fn dbus_access(self, access: DbusAccess) -> Self {
        self.dbus(access)
    }

    pub fn network(mut self, v: bool) -> Self {
        self.inner.network_access = v;
        self
    }

    pub fn network_access(self, v: bool) -> Self {
        self.network(v)
    }

    pub fn device_access(mut self, v: bool) -> Self {
        self.inner.device_access = v;
        self
    }

    pub fn findings(mut self, findings: Vec<FindingKind>) -> Self {
        self.inner.findings = findings;
        self
    }

    pub fn with_finding(mut self, kind: FindingKind) -> Self {
        self.inner.findings.push(kind);
        self
    }

    pub fn install_script(mut self, v: bool) -> Self {
        self.inner.install_script_present = v;
        self
    }

    pub fn install_script_present(self, v: bool) -> Self {
        self.install_script(v)
    }

    pub fn build_logic_changed(mut self, v: bool) -> Self {
        self.inner.build_logic_changed = v;
        self
    }

    pub fn last_update_days(mut self, days: Option<u32>) -> Self {
        self.inner.last_update_days_ago = days;
        self
    }

    pub fn build(self) -> CandidateEvidence {
        self.inner
    }
}

/// Convenience entry point.
pub fn candidate_evidence() -> CandidateEvidenceBuilder {
    CandidateEvidenceBuilder::new()
}

// ---------------------------------------------------------------------------
// PolicyCandidate builder — the primary `candidate()` entry point required by
// `docs/architecture/overview.md §39`.
// ---------------------------------------------------------------------------

/// Fluent builder for [`PolicyCandidate`].
///
/// Example:
/// ```rust
/// use pkgseal_testkit::builders::candidate;
/// let c = candidate().aur().verified_publisher(false).build();
/// assert_eq!(c.source, pkgseal_domain::PackageSource::Aur);
/// ```
#[derive(Debug, Clone)]
pub struct PolicyCandidateBuilder {
    source: PackageSource,
    package_name: String,
    version: String,
    evidence: CandidateEvidence,
    id: Option<CandidateId>,
}

impl Default for PolicyCandidateBuilder {
    fn default() -> Self {
        Self {
            source: PackageSource::Aur,
            package_name: "brave-bin".to_string(),
            version: "1.70.0-1".to_string(),
            evidence: CandidateEvidence {
                is_community_maintained: true,
                ..CandidateEvidence::default()
            },
            id: None,
        }
    }
}

impl PolicyCandidateBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    // --- source shortcuts ---

    pub fn source(mut self, source: PackageSource) -> Self {
        self.source = source;
        self
    }

    pub fn aur(mut self) -> Self {
        self.source = PackageSource::Aur;
        // Keep existing evidence but align provenance flags to Aur defaults.
        self.evidence.is_community_maintained = true;
        self.evidence.is_official_repository = false;
        self
    }

    pub fn arch(mut self) -> Self {
        self.source = PackageSource::ArchOfficial;
        self.evidence.is_official_repository = true;
        self.evidence.is_community_maintained = false;
        self
    }

    pub fn arch_official(self) -> Self {
        self.arch()
    }

    pub fn flatpak(mut self) -> Self {
        self.source = PackageSource::Flatpak;
        self.evidence.sandboxed = true;
        self.evidence.is_official_repository = false;
        // is_community_maintained is orthogonal for Flatpak; keep as-is.
        self
    }

    // --- identity ---

    pub fn name(mut self, name: impl AsRef<str>) -> Self {
        self.package_name = name.as_ref().to_string();
        self
    }

    pub fn package_name(self, name: impl AsRef<str>) -> Self {
        self.name(name)
    }

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    pub fn id(mut self, id: CandidateId) -> Self {
        self.id = Some(id);
        self
    }

    // --- evidence passthroughs ---

    pub fn evidence(mut self, evidence: CandidateEvidence) -> Self {
        self.evidence = evidence;
        self
    }

    pub fn with_evidence(mut self, f: impl FnOnce(CandidateEvidence) -> CandidateEvidence) -> Self {
        self.evidence = f(self.evidence);
        self
    }

    pub fn verified_publisher(mut self, v: bool) -> Self {
        self.evidence.publisher_verified = v;
        self
    }

    pub fn publisher_verified(self, v: bool) -> Self {
        self.verified_publisher(v)
    }

    pub fn publisher_supported(mut self, v: bool) -> Self {
        self.evidence.publisher_supported = v;
        self
    }

    pub fn official(mut self, v: bool) -> Self {
        self.evidence.is_official_repository = v;
        if v {
            self.evidence.is_community_maintained = false;
        }
        self
    }

    pub fn community(mut self, v: bool) -> Self {
        self.evidence.is_community_maintained = v;
        if v {
            self.evidence.is_official_repository = false;
        }
        self
    }

    pub fn sandboxed(mut self, v: bool) -> Self {
        self.evidence.sandboxed = v;
        self
    }

    pub fn permission_level(mut self, lvl: PermissionLevel) -> Self {
        self.evidence.permission_level = lvl;
        self
    }

    pub fn filesystem(mut self, access: FilesystemAccess) -> Self {
        self.evidence.filesystem_access = access;
        self
    }

    pub fn dbus(mut self, access: DbusAccess) -> Self {
        self.evidence.dbus_access = access;
        self
    }

    pub fn network(mut self, v: bool) -> Self {
        self.evidence.network_access = v;
        self
    }

    pub fn findings(mut self, findings: Vec<FindingKind>) -> Self {
        self.evidence.findings = findings;
        self
    }

    pub fn with_finding(mut self, kind: FindingKind) -> Self {
        self.evidence.findings.push(kind);
        self
    }

    pub fn install_script(mut self, present: bool) -> Self {
        self.evidence.install_script_present = present;
        self
    }

    pub fn signature(mut self, present: bool) -> Self {
        self.evidence.signature_present = present;
        self
    }

    pub fn signature_present(self, present: bool) -> Self {
        self.signature(present)
    }

    pub fn checksum(mut self, present: bool) -> Self {
        self.evidence.checksum_present = present;
        self
    }

    pub fn checksum_present(self, present: bool) -> Self {
        self.checksum(present)
    }

    // --- build ---

    pub fn build(self) -> PolicyCandidate {
        let pkg_name = PackageName::new(&self.package_name).unwrap_or_else(|e| {
            panic!(
                "PolicyCandidateBuilder: invalid package name '{}': {e}",
                self.package_name
            )
        });
        let mut candidate =
            PolicyCandidate::new(self.source, pkg_name, self.version, self.evidence);
        if let Some(id) = self.id {
            candidate = candidate.with_id(id);
        }
        candidate
    }
}

/// Top-level entry point expected by `docs/architecture/overview.md §39`.
pub fn candidate() -> PolicyCandidateBuilder {
    PolicyCandidateBuilder::new()
}

// ---------------------------------------------------------------------------
// PackageSummary builder
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct PackageSummaryBuilder {
    id: Option<String>,
    name: String,
    version: String,
    description: Option<String>,
    source: PackageSource,
    repository: Option<String>,
    installed: bool,
    download_size: Option<u64>,
    installed_size: Option<u64>,
}

impl Default for PackageSummaryBuilder {
    fn default() -> Self {
        Self {
            id: None,
            name: "brave-bin".to_string(),
            version: "1.70.0-1".to_string(),
            description: Some("Brave web browser".to_string()),
            source: PackageSource::Aur,
            repository: Some("aur".to_string()),
            installed: false,
            download_size: None,
            installed_size: None,
        }
    }
}

impl PackageSummaryBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn aur(mut self) -> Self {
        self.source = PackageSource::Aur;
        self.repository = Some("aur".to_string());
        self
    }

    pub fn arch(mut self) -> Self {
        self.source = PackageSource::ArchOfficial;
        self.repository = Some("extra".to_string());
        self
    }

    pub fn arch_official(self) -> Self {
        self.arch()
    }

    pub fn flatpak(mut self) -> Self {
        self.source = PackageSource::Flatpak;
        self.repository = Some("flathub".to_string());
        self
    }

    pub fn source(mut self, source: PackageSource) -> Self {
        self.source = source;
        self
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn name(mut self, name: impl AsRef<str>) -> Self {
        self.name = name.as_ref().to_string();
        self
    }

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    pub fn description(mut self, desc: Option<String>) -> Self {
        self.description = desc;
        self
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn repository(mut self, repo: Option<String>) -> Self {
        self.repository = repo;
        self
    }

    pub fn with_repository(mut self, repo: impl Into<String>) -> Self {
        self.repository = Some(repo.into());
        self
    }

    pub fn installed(mut self, v: bool) -> Self {
        self.installed = v;
        self
    }

    pub fn download_size(mut self, size: Option<u64>) -> Self {
        self.download_size = size;
        self
    }

    pub fn installed_size(mut self, size: Option<u64>) -> Self {
        self.installed_size = size;
        self
    }

    #[must_use]
    pub fn build(self) -> PackageSummary {
        let pkg_name = PackageName::new(&self.name).unwrap_or_else(|e| {
            panic!(
                "PackageSummaryBuilder: invalid package name '{}': {e}",
                self.name
            )
        });
        let id = self
            .id
            .unwrap_or_else(|| format!("{}/{}", self.source.as_str(), pkg_name.as_str()));
        PackageSummary {
            id,
            name: pkg_name,
            version: self.version,
            description: self.description,
            source: self.source,
            repository: self.repository,
            installed: self.installed,
            download_size: self.download_size,
            installed_size: self.installed_size,
        }
    }
}

pub fn package_summary() -> PackageSummaryBuilder {
    PackageSummaryBuilder::new()
}

// ---------------------------------------------------------------------------
// PackageDetails builder
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct PackageDetailsBuilder {
    summary: PackageSummary,
    architecture: Option<String>,
    maintainer: Option<String>,
    url: Option<String>,
    license: Option<String>,
    dependencies: Vec<String>,
    optional_dependencies: Vec<String>,
    provides: Vec<String>,
    conflicts: Vec<String>,
    replaces: Vec<String>,
    groups: Vec<String>,
    build_date: Option<String>,
    install_date: Option<String>,
    validation: Option<String>,
    raw_metadata: HashMap<String, serde_json::Value>,
}

impl Default for PackageDetailsBuilder {
    fn default() -> Self {
        Self {
            summary: PackageSummaryBuilder::default().build(),
            architecture: Some("x86_64".to_string()),
            maintainer: None,
            url: Some("https://brave.com".to_string()),
            license: Some("MPL-2.0".to_string()),
            dependencies: Vec::new(),
            optional_dependencies: Vec::new(),
            provides: Vec::new(),
            conflicts: Vec::new(),
            replaces: Vec::new(),
            groups: Vec::new(),
            build_date: None,
            install_date: None,
            validation: None,
            raw_metadata: HashMap::new(),
        }
    }
}

impl PackageDetailsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_summary(mut self, summary: PackageSummary) -> Self {
        self.summary = summary;
        self
    }

    pub fn summary(mut self, summary: PackageSummary) -> Self {
        self.summary = summary;
        self
    }

    pub fn arch(mut self) -> Self {
        self.summary.source = PackageSource::ArchOfficial;
        self.summary.repository = Some("extra".to_string());
        self
    }

    pub fn aur(mut self) -> Self {
        self.summary.source = PackageSource::Aur;
        self.summary.repository = Some("aur".to_string());
        self
    }

    pub fn flatpak(mut self) -> Self {
        self.summary.source = PackageSource::Flatpak;
        self.summary.repository = Some("flathub".to_string());
        self
    }

    pub fn name(mut self, name: impl AsRef<str>) -> Self {
        let n = name.as_ref().to_string();
        let pkg_name = PackageName::new(&n)
            .unwrap_or_else(|e| panic!("PackageDetailsBuilder: invalid package name '{n}': {e}"));
        self.summary.name = pkg_name;
        // Keep id consistent unless explicitly overridden.
        self.summary.id = format!("{}/{}", self.summary.source.as_str(), n);
        self
    }

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.summary.version = version.into();
        self
    }

    pub fn description(mut self, desc: Option<String>) -> Self {
        self.summary.description = desc;
        self
    }

    pub fn repository(mut self, repo: Option<String>) -> Self {
        self.summary.repository = repo;
        self
    }

    pub fn maintainer(mut self, maintainer: Option<String>) -> Self {
        self.maintainer = maintainer;
        self
    }

    pub fn with_maintainer(mut self, maintainer: impl Into<String>) -> Self {
        self.maintainer = Some(maintainer.into());
        self
    }

    pub fn url(mut self, url: Option<String>) -> Self {
        self.url = url;
        self
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    pub fn license(mut self, license: Option<String>) -> Self {
        self.license = license;
        self
    }

    pub fn architecture(mut self, arch: Option<String>) -> Self {
        self.architecture = arch;
        self
    }

    pub fn dependencies(mut self, deps: Vec<String>) -> Self {
        self.dependencies = deps;
        self
    }

    pub fn with_dependency(mut self, dep: impl Into<String>) -> Self {
        self.dependencies.push(dep.into());
        self
    }

    pub fn provides(mut self, provides: Vec<String>) -> Self {
        self.provides = provides;
        self
    }

    pub fn raw_metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.raw_metadata = metadata;
        self
    }

    pub fn with_raw(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.raw_metadata.insert(key.into(), value);
        self
    }

    pub fn with_application_id(mut self, app_id: impl Into<String>) -> Self {
        self.raw_metadata.insert(
            "application_id".to_string(),
            serde_json::Value::String(app_id.into()),
        );
        self
    }

    pub fn build(self) -> PackageDetails {
        PackageDetails {
            summary: self.summary,
            architecture: self.architecture,
            maintainer: self.maintainer,
            url: self.url,
            license: self.license,
            dependencies: self.dependencies,
            optional_dependencies: self.optional_dependencies,
            provides: self.provides,
            conflicts: self.conflicts,
            replaces: self.replaces,
            groups: self.groups,
            build_date: self.build_date,
            install_date: self.install_date,
            validation: self.validation,
            raw_metadata: self.raw_metadata,
        }
    }
}

pub fn package_details() -> PackageDetailsBuilder {
    PackageDetailsBuilder::new()
}

// ---------------------------------------------------------------------------
// ResolvedApplication builder
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ResolvedApplicationBuilder {
    canonical_name: String,
    display_name: String,
    candidates: Vec<pkgseal_domain::CandidateRef>,
    details: Vec<PackageDetails>,
    primary_source: Option<PackageSource>,
}

impl Default for ResolvedApplicationBuilder {
    fn default() -> Self {
        Self {
            canonical_name: "brave".to_string(),
            display_name: "Brave Browser".to_string(),
            candidates: Vec::new(),
            details: Vec::new(),
            primary_source: None,
        }
    }
}

impl ResolvedApplicationBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn canonical_name(mut self, name: impl Into<String>) -> Self {
        self.canonical_name = name.into();
        self
    }

    pub fn display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = name.into();
        self
    }

    pub fn name(self, name: impl Into<String>) -> Self {
        let n = name.into();
        Self {
            canonical_name: n.clone(),
            display_name: n,
            ..self
        }
    }

    pub fn with_candidate(mut self, candidate: pkgseal_domain::CandidateRef) -> Self {
        self.candidates.push(candidate);
        self
    }

    pub fn with_candidates(
        mut self,
        candidates: impl IntoIterator<Item = pkgseal_domain::CandidateRef>,
    ) -> Self {
        self.candidates.extend(candidates);
        self
    }

    pub fn with_detail(mut self, detail: PackageDetails) -> Self {
        self.details.push(detail);
        self
    }

    pub fn with_details(mut self, details: impl IntoIterator<Item = PackageDetails>) -> Self {
        self.details.extend(details);
        self
    }

    pub fn primary_source(mut self, source: PackageSource) -> Self {
        self.primary_source = Some(source);
        self
    }

    pub fn build(self) -> ResolvedApplication {
        let mut identity = ApplicationIdentity::new(self.canonical_name, self.display_name);
        identity.candidates = self.candidates;
        identity.primary_source = self.primary_source;
        let mut app = ResolvedApplication::new(identity);
        app.candidate_details = self.details;
        app
    }

    /// Build with a convenience: generate `CandidateRef`s from summaries if none were
    /// explicitly added, using the provided details' summaries.
    pub fn build_with_generated_refs(self) -> ResolvedApplication {
        if !self.candidates.is_empty() {
            return self.build();
        }
        let mut with_refs = self;
        for d in &with_refs.details {
            let cref = pkgseal_domain::CandidateRef::new(
                d.summary.source,
                d.summary.name.clone(),
                d.summary.id.clone(),
            );
            with_refs.candidates.push(cref);
        }
        with_refs.build()
    }
}

pub fn resolved_application() -> ResolvedApplicationBuilder {
    ResolvedApplicationBuilder::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pkgseal_domain::PackageSource;
    use pkgseal_policy::FindingKind;

    #[test]
    fn candidate_builder_example_from_spec() {
        let c = candidate().aur().verified_publisher(false).build();
        assert_eq!(c.source, PackageSource::Aur);
        assert!(!c.evidence.publisher_verified);
        assert_eq!(c.package_name.as_str(), "brave-bin");
    }

    #[test]
    fn candidate_builder_flatpak_narrow() {
        let c = candidate()
            .flatpak()
            .name("com-brave-browser")
            .verified_publisher(true)
            .sandboxed(true)
            .permission_level(PermissionLevel::Narrow)
            .build();
        assert_eq!(c.source, PackageSource::Flatpak);
        assert!(c.evidence.publisher_verified);
        assert!(c.evidence.sandboxed);
    }

    #[test]
    fn package_summary_builder_defaults() {
        let s = package_summary().aur().name("yay").build();
        assert_eq!(s.name.as_str(), "yay");
        assert_eq!(s.source, PackageSource::Aur);
    }

    #[test]
    fn package_details_builder_with_raw() {
        let d = package_details()
            .flatpak()
            .name("com-brave-browser")
            .with_application_id("com.brave.Browser")
            .build();
        assert_eq!(d.summary.source, PackageSource::Flatpak);
        assert_eq!(
            d.raw_metadata
                .get("application_id")
                .and_then(|v| v.as_str()),
            Some("com.brave.Browser")
        );
    }

    #[test]
    fn evidence_builder_is_independent() {
        let e = candidate_evidence()
            .official(true)
            .verified_publisher(true)
            .with_finding(FindingKind::SudoUsage)
            .build();
        assert!(e.is_official_repository);
        assert!(e.publisher_verified);
        assert_eq!(e.findings.len(), 1);
    }

    #[test]
    fn resolved_application_builder_generates_refs() {
        let details = package_details().arch().name("brave").build();
        let app = resolved_application()
            .name("brave")
            .with_detail(details)
            .build_with_generated_refs();
        assert_eq!(app.identity.candidates.len(), 1);
        assert_eq!(app.candidate_details.len(), 1);
    }
}
