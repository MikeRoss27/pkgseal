use pkgseal_domain::{CandidateId, PackageName, PackageSource};
use serde::{Deserialize, Serialize};

/// Deterministic confidence for a recommendation.
///
/// Not a security guarantee — reflects how decisively the policy could separate
/// candidates given the available evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum Confidence {
    None,
    Uncertain,
    Low,
    Medium,
    High,
}

impl Confidence {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
            Self::Uncertain => "uncertain",
            Self::None => "none",
        }
    }
}

impl std::fmt::Display for Confidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Why the recommended candidate was preferred — a positive fact derived from evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReasonKind {
    OfficialRepository,
    PublisherSupported,
    PublisherVerified,
    SignaturePresent,
    ChecksumPresent,
    Sandboxed,
    NarrowPermissions,
    NativeIntegration,
    NoKnownFindings,
    MaintainedProvenance,
}

impl ReasonKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OfficialRepository => "official-repository",
            Self::PublisherSupported => "publisher-supported",
            Self::PublisherVerified => "publisher-verified",
            Self::SignaturePresent => "signature-present",
            Self::ChecksumPresent => "checksum-present",
            Self::Sandboxed => "sandboxed",
            Self::NarrowPermissions => "narrow-permissions",
            Self::NativeIntegration => "native-integration",
            Self::NoKnownFindings => "no-known-findings",
            Self::MaintainedProvenance => "maintained-provenance",
        }
    }
}

/// A trade-off or risk attached to a candidate — factual, not alarmist.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WarningKind {
    CommunityMaintained,
    BroadPermissions,
    UnverifiedPublisher,
    MissingSignature,
    MissingChecksum,
    InstallScriptPresent,
    BuildLogicChanged,
    FindingsDetected,
    HostFilesystemAccess,
    HostDbusAccess,
    NetworkAccess,
    ExcessivePermissions,
    OutdatedEvidence,
}

impl WarningKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CommunityMaintained => "community-maintained",
            Self::BroadPermissions => "broad-permissions",
            Self::UnverifiedPublisher => "unverified-publisher",
            Self::MissingSignature => "missing-signature",
            Self::MissingChecksum => "missing-checksum",
            Self::InstallScriptPresent => "install-script-present",
            Self::BuildLogicChanged => "build-logic-changed",
            Self::FindingsDetected => "findings-detected",
            Self::HostFilesystemAccess => "host-filesystem-access",
            Self::HostDbusAccess => "host-dbus-access",
            Self::NetworkAccess => "network-access",
            Self::ExcessivePermissions => "excessive-permissions",
            Self::OutdatedEvidence => "outdated-evidence",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reason {
    pub kind: ReasonKind,
    pub detail: String,
    /// Contribution to the final score (positive). Kept for explainability and deterministic ordering.
    pub contribution: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Warning {
    pub kind: WarningKind,
    pub detail: String,
    pub severity: Severity,
    /// Penalty applied to the final score (negative in scoring, stored as positive for display).
    pub penalty: i32,
}

/// Narrowness of Flatpak/sandbox permissions. Deterministic ordering Narrow < Moderate < Broad < Excessive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum PermissionLevel {
    Narrow,
    Moderate,
    Broad,
    Excessive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum FilesystemAccess {
    None,
    Limited,
    HomeRo,
    HomeRw,
    Host,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum DbusAccess {
    None,
    SessionLimited,
    SessionFull,
    System,
    Host,
}

/// Known finding categories from static AUR inspection (simplified, deterministic).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum FindingKind {
    NetworkExecution, // curl|sh / wget|sh
    EvalUsage,
    SudoUsage,
    Setuid,
    RootChown,
    RootWrite,
    Base64Obfuscation,
    DownloadedCodeExecution,
    InstallScript,
}

/// Evidence collected for a single candidate. Pure data, no IO. Missing fields are
/// represented as `Option` or defaults; absent evidence is treated conservatively.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateEvidence {
    /// Whether the package originates from an official distribution repository with distribution signatures.
    pub is_official_repository: bool,
    /// Whether the package recipe is community-maintained (e.g. AUR).
    pub is_community_maintained: bool,
    /// Flatpak publisher verification (e.g. Flathub verified).
    pub publisher_verified: bool,
    /// Whether the upstream publisher documents this install method as supported.
    pub publisher_supported: bool,
    /// Distribution package signature present (Arch `Validpgp` / db signature).
    pub signature_present: bool,
    /// Checksum present in recipe (AUR sha256sums etc.).
    pub checksum_present: bool,
    /// Checksum validated against downloaded sources (if inspection performed).
    pub checksum_validated: bool,
    /// Whether the application runs sandboxed (Flatpak).
    pub sandboxed: bool,
    /// Aggregate permission narrowness.
    pub permission_level: PermissionLevel,
    pub filesystem_access: FilesystemAccess,
    pub dbus_access: DbusAccess,
    pub network_access: bool,
    pub device_access: bool,
    /// Static findings from PKGBUILD/.install inspection.
    pub findings: Vec<FindingKind>,
    pub install_script_present: bool,
    pub build_logic_changed: bool,
    /// Days since last upstream update, if known. Used as a conservative stale signal only.
    pub last_update_days_ago: Option<u32>,
}

impl Default for CandidateEvidence {
    fn default() -> Self {
        Self {
            is_official_repository: false,
            is_community_maintained: false,
            publisher_verified: false,
            publisher_supported: false,
            signature_present: false,
            checksum_present: false,
            checksum_validated: false,
            sandboxed: false,
            permission_level: PermissionLevel::Moderate,
            filesystem_access: FilesystemAccess::Limited,
            dbus_access: DbusAccess::None,
            network_access: false,
            device_access: false,
            findings: Vec::new(),
            install_script_present: false,
            build_logic_changed: false,
            last_update_days_ago: None,
        }
    }
}

impl CandidateEvidence {
    pub fn official_repository() -> Self {
        Self {
            is_official_repository: true,
            is_community_maintained: false,
            publisher_supported: false,
            signature_present: true,
            checksum_present: true,
            checksum_validated: true,
            sandboxed: false,
            permission_level: PermissionLevel::Narrow,
            filesystem_access: FilesystemAccess::None,
            ..Default::default()
        }
    }

    pub fn aur_community() -> Self {
        Self {
            is_official_repository: false,
            is_community_maintained: true,
            publisher_supported: false,
            signature_present: false,
            checksum_present: false,
            sandboxed: false,
            permission_level: PermissionLevel::Moderate,
            ..Default::default()
        }
    }

    pub fn flatpak_verified_narrow() -> Self {
        Self {
            is_official_repository: false,
            is_community_maintained: false,
            publisher_verified: true,
            publisher_supported: true,
            signature_present: true,
            sandboxed: true,
            permission_level: PermissionLevel::Narrow,
            filesystem_access: FilesystemAccess::Limited,
            dbus_access: DbusAccess::SessionLimited,
            network_access: true,
            ..Default::default()
        }
    }
}

/// Input to the policy engine — a single installable variant with its evidence.
/// Deterministic `Ord` is provided for stable tie-breaking without implying a universal source ranking.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyCandidate {
    pub id: CandidateId,
    pub source: PackageSource,
    pub package_name: PackageName,
    pub version: String,
    pub evidence: CandidateEvidence,
}

impl PolicyCandidate {
    pub fn new(
        source: PackageSource,
        package_name: PackageName,
        version: impl Into<String>,
        evidence: CandidateEvidence,
    ) -> Self {
        Self {
            id: CandidateId::new(),
            source,
            package_name,
            version: version.into(),
            evidence,
        }
    }

    pub fn with_id(mut self, id: CandidateId) -> Self {
        self.id = id;
        self
    }
}

impl PartialOrd for PolicyCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PolicyCandidate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Deterministic lexical ordering for tie-breaking — package name, then id string.
        // This intentionally does NOT encode a universal source priority.
        self.package_name
            .as_str()
            .cmp(other.package_name.as_str())
            .then_with(|| self.id.0.to_string().cmp(&other.id.0.to_string()))
    }
}

/// Alternative candidate ranked after the recommendation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Alternative {
    pub candidate: PolicyCandidate,
    pub score: i32,
    pub reasons: Vec<Reason>,
    pub warnings: Vec<Warning>,
}

/// Pure recommendation output: Evidence -> Policy -> Recommendation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recommendation {
    /// The top-ranked candidate, if any were provided.
    pub recommended: Option<PolicyCandidate>,
    /// How decisively the policy separated the winner from alternatives.
    pub confidence: Confidence,
    /// Positive facts explaining the winner.
    pub reasons: Vec<Reason>,
    /// Trade-offs / risks attached to the winner (factual, not alarmist).
    pub warnings: Vec<Warning>,
    /// All other candidates sorted descending by score (most attractive first).
    pub alternatives: Vec<Alternative>,
    /// Internal score of the winner, exposed for explainability and testing (not a security score).
    pub score: i32,
}

impl Recommendation {
    pub fn none(confidence: Confidence) -> Self {
        Self {
            recommended: None,
            confidence,
            reasons: Vec::new(),
            warnings: Vec::new(),
            alternatives: Vec::new(),
            score: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.recommended.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn candidate_id(n: u128) -> CandidateId {
        CandidateId(Uuid::from_u128(n))
    }

    #[test]
    fn confidence_ordering() {
        assert!(Confidence::High > Confidence::Medium);
        assert!(Confidence::Medium > Confidence::Low);
        assert!(Confidence::Low > Confidence::Uncertain);
    }

    #[test]
    fn candidate_evidence_defaults_are_conservative() {
        let e = CandidateEvidence::default();
        assert!(!e.is_official_repository);
        assert!(!e.publisher_verified);
        assert!(!e.signature_present);
        assert_eq!(e.findings.len(), 0);
    }

    #[test]
    fn policy_candidate_ord_is_deterministic() {
        let a = PolicyCandidate::new(
            PackageSource::Flatpak,
            PackageName::new("brave-bin").unwrap(),
            "1.0",
            CandidateEvidence::default(),
        )
        .with_id(candidate_id(2));
        let b = PolicyCandidate::new(
            PackageSource::ArchOfficial,
            PackageName::new("brave-bin").unwrap(),
            "1.0",
            CandidateEvidence::default(),
        )
        .with_id(candidate_id(1));
        // Same package name -> order by id lexical, not by source.
        assert!(b < a);
        assert!(a > b);
    }

    #[test]
    fn recommendation_none_has_no_candidate() {
        let r = Recommendation::none(Confidence::None);
        assert!(r.is_empty());
        assert_eq!(r.confidence, Confidence::None);
        assert!(r.reasons.is_empty());
    }

    #[test]
    fn reasoning_serializes_deterministically() {
        let r = Reason {
            kind: ReasonKind::OfficialRepository,
            detail: "from Arch official repository".to_string(),
            contribution: 40,
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("official-repository"));
    }
}
