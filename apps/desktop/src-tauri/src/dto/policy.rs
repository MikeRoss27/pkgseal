use pkgseal_policy::{CandidateEvidence, Confidence, PolicyCandidate, Recommendation};
use serde::{Deserialize, Serialize};

/// Input: policy preset string ("balanced", "native-first", "sandbox-first", "maximum-review")
/// + candidates with evidence. Frontend builds this from resolver details (heuristic mapping).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluatePolicyRequest {
    pub preset: String,
    pub candidates: Vec<PolicyCandidateDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyCandidateDto {
    pub source: String,
    pub package_name: String,
    pub version: String,
    #[serde(default)]
    pub evidence: CandidateEvidenceDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CandidateEvidenceDto {
    pub is_official_repository: bool,
    pub is_community_maintained: bool,
    pub publisher_verified: bool,
    pub publisher_supported: bool,
    pub signature_present: bool,
    pub checksum_present: bool,
    pub checksum_validated: bool,
    pub sandboxed: bool,
    pub permission_level: String,
    pub filesystem_access: String,
    pub dbus_access: String,
    pub network_access: bool,
    pub device_access: bool,
    pub findings: Vec<String>,
    pub install_script_present: bool,
    pub build_logic_changed: bool,
}

impl Default for CandidateEvidenceDto {
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
            permission_level: "moderate".to_string(),
            filesystem_access: "limited".to_string(),
            dbus_access: "none".to_string(),
            network_access: false,
            device_access: false,
            findings: Vec::new(),
            install_script_present: false,
            build_logic_changed: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluatePolicyResponse {
    pub recommendation: RecommendationDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationDto {
    pub recommended: Option<PolicyCandidateDto>,
    pub confidence: String,
    pub reasons: Vec<ReasonDto>,
    pub warnings: Vec<WarningDto>,
    pub alternatives: Vec<AlternativeDto>,
    pub score: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReasonDto {
    pub kind: String,
    pub detail: String,
    pub contribution: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WarningDto {
    pub kind: String,
    pub detail: String,
    pub severity: String,
    pub penalty: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlternativeDto {
    pub candidate: PolicyCandidateDto,
    pub score: i32,
    pub reasons: Vec<ReasonDto>,
    pub warnings: Vec<WarningDto>,
}

// ── mapping helpers ─────────────────────────────────────────────────────────

pub fn map_policy_preset(s: &str) -> pkgseal_policy::PolicyPreset {
    match s.to_ascii_lowercase().as_str() {
        "native-first" | "native_first" => pkgseal_policy::PolicyPreset::NativeFirst,
        "sandbox-first" | "sandbox_first" => pkgseal_policy::PolicyPreset::SandboxFirst,
        "maximum-review" | "maximum_review" => pkgseal_policy::PolicyPreset::MaximumReview,
        _ => pkgseal_policy::PolicyPreset::Balanced,
    }
}

pub fn map_permission_level(s: &str) -> pkgseal_policy::PermissionLevel {
    match s.to_ascii_lowercase().as_str() {
        "narrow" => pkgseal_policy::PermissionLevel::Narrow,
        "broad" => pkgseal_policy::PermissionLevel::Broad,
        "excessive" => pkgseal_policy::PermissionLevel::Excessive,
        _ => pkgseal_policy::PermissionLevel::Moderate,
    }
}

pub fn map_filesystem(s: &str) -> pkgseal_policy::FilesystemAccess {
    match s.to_ascii_lowercase().as_str() {
        "none" => pkgseal_policy::FilesystemAccess::None,
        "home-ro" | "home_ro" => pkgseal_policy::FilesystemAccess::HomeRo,
        "home-rw" | "home_rw" => pkgseal_policy::FilesystemAccess::HomeRw,
        "host" => pkgseal_policy::FilesystemAccess::Host,
        _ => pkgseal_policy::FilesystemAccess::Limited,
    }
}

pub fn map_dbus(s: &str) -> pkgseal_policy::DbusAccess {
    match s.to_ascii_lowercase().as_str() {
        "session-limited" | "session_limited" => pkgseal_policy::DbusAccess::SessionLimited,
        "session-full" | "session_full" => pkgseal_policy::DbusAccess::SessionFull,
        "system" => pkgseal_policy::DbusAccess::System,
        "host" => pkgseal_policy::DbusAccess::Host,
        _ => pkgseal_policy::DbusAccess::None,
    }
}

pub fn map_finding(s: &str) -> Option<pkgseal_policy::FindingKind> {
    match s.to_ascii_lowercase().as_str() {
        "network-execution" | "network_execution" => {
            Some(pkgseal_policy::FindingKind::NetworkExecution)
        }
        "eval-usage" | "eval_usage" => Some(pkgseal_policy::FindingKind::EvalUsage),
        "sudo-usage" | "sudo_usage" => Some(pkgseal_policy::FindingKind::SudoUsage),
        "setuid" => Some(pkgseal_policy::FindingKind::Setuid),
        "root-chown" | "root_chown" => Some(pkgseal_policy::FindingKind::RootChown),
        "root-write" | "root_write" => Some(pkgseal_policy::FindingKind::RootWrite),
        "base64-obfuscation" | "base64_obfuscation" => {
            Some(pkgseal_policy::FindingKind::Base64Obfuscation)
        }
        "downloaded-code-execution" | "downloaded_code_execution" => {
            Some(pkgseal_policy::FindingKind::DownloadedCodeExecution)
        }
        "install-script" | "install_script" => Some(pkgseal_policy::FindingKind::InstallScript),
        _ => None,
    }
}

pub fn dto_to_evidence(dto: &CandidateEvidenceDto) -> CandidateEvidence {
    CandidateEvidence {
        is_official_repository: dto.is_official_repository,
        is_community_maintained: dto.is_community_maintained,
        publisher_verified: dto.publisher_verified,
        publisher_supported: dto.publisher_supported,
        signature_present: dto.signature_present,
        checksum_present: dto.checksum_present,
        checksum_validated: dto.checksum_validated,
        sandboxed: dto.sandboxed,
        permission_level: map_permission_level(&dto.permission_level),
        filesystem_access: map_filesystem(&dto.filesystem_access),
        dbus_access: map_dbus(&dto.dbus_access),
        network_access: dto.network_access,
        device_access: dto.device_access,
        findings: dto.findings.iter().filter_map(|f| map_finding(f)).collect(),
        install_script_present: dto.install_script_present,
        build_logic_changed: dto.build_logic_changed,
        last_update_days_ago: None,
    }
}

pub fn parse_source(s: &str) -> pkgseal_domain::PackageSource {
    match s.to_ascii_lowercase().as_str() {
        "arch" | "arch-official" | "arch_official" => pkgseal_domain::PackageSource::ArchOfficial,
        "flatpak" => pkgseal_domain::PackageSource::Flatpak,
        _ => pkgseal_domain::PackageSource::Aur,
    }
}

pub fn dto_to_candidate(dto: &PolicyCandidateDto) -> Result<PolicyCandidate, String> {
    let source = parse_source(&dto.source);
    let name = pkgseal_domain::PackageName::new(&dto.package_name)
        .map_err(|e| format!("invalid package_name: {e}"))?;
    let evidence = dto_to_evidence(&dto.evidence);
    Ok(PolicyCandidate::new(
        source,
        name,
        dto.version.clone(),
        evidence,
    ))
}

pub fn candidate_to_dto(c: &PolicyCandidate) -> PolicyCandidateDto {
    PolicyCandidateDto {
        source: c.source.to_string(),
        package_name: c.package_name.as_str().to_string(),
        version: c.version.clone(),
        evidence: evidence_to_dto(&c.evidence),
    }
}

pub fn evidence_to_dto(e: &CandidateEvidence) -> CandidateEvidenceDto {
    CandidateEvidenceDto {
        is_official_repository: e.is_official_repository,
        is_community_maintained: e.is_community_maintained,
        publisher_verified: e.publisher_verified,
        publisher_supported: e.publisher_supported,
        signature_present: e.signature_present,
        checksum_present: e.checksum_present,
        checksum_validated: e.checksum_validated,
        sandboxed: e.sandboxed,
        permission_level: format!("{:?}", e.permission_level).to_ascii_lowercase(),
        filesystem_access: format!("{:?}", e.filesystem_access).to_ascii_lowercase(),
        dbus_access: format!("{:?}", e.dbus_access).to_ascii_lowercase(),
        network_access: e.network_access,
        device_access: e.device_access,
        findings: e
            .findings
            .iter()
            .map(|f| format!("{f:?}").to_ascii_lowercase())
            .collect(),
        install_script_present: e.install_script_present,
        build_logic_changed: e.build_logic_changed,
    }
}

pub fn recommendation_to_dto(r: Recommendation) -> RecommendationDto {
    RecommendationDto {
        recommended: r.recommended.as_ref().map(candidate_to_dto),
        confidence: format!("{:?}", r.confidence).to_ascii_lowercase(),
        reasons: r
            .reasons
            .into_iter()
            .map(|re| ReasonDto {
                kind: format!("{:?}", re.kind).to_ascii_lowercase(),
                detail: re.detail,
                contribution: re.contribution,
            })
            .collect(),
        warnings: r
            .warnings
            .into_iter()
            .map(|w| WarningDto {
                kind: format!("{:?}", w.kind).to_ascii_lowercase(),
                detail: w.detail,
                severity: format!("{:?}", w.severity).to_ascii_lowercase(),
                penalty: w.penalty,
            })
            .collect(),
        alternatives: r
            .alternatives
            .into_iter()
            .map(|a| AlternativeDto {
                candidate: candidate_to_dto(&a.candidate),
                score: a.score,
                reasons: a
                    .reasons
                    .into_iter()
                    .map(|re| ReasonDto {
                        kind: format!("{:?}", re.kind).to_ascii_lowercase(),
                        detail: re.detail,
                        contribution: re.contribution,
                    })
                    .collect(),
                warnings: a
                    .warnings
                    .into_iter()
                    .map(|w| WarningDto {
                        kind: format!("{:?}", w.kind).to_ascii_lowercase(),
                        detail: w.detail,
                        severity: format!("{:?}", w.severity).to_ascii_lowercase(),
                        penalty: w.penalty,
                    })
                    .collect(),
            })
            .collect(),
        score: r.score,
    }
}

/// Heuristic helper: build evidence from a generic package summary/details when
/// the frontend doesn't provide explicit evidence. Keeps policy usable even
/// before full inspector is wired.
#[allow(dead_code)]
pub fn heuristic_evidence_for_source(source: &pkgseal_domain::PackageSource) -> CandidateEvidence {
    match source {
        pkgseal_domain::PackageSource::ArchOfficial => CandidateEvidence::official_repository(),
        pkgseal_domain::PackageSource::Flatpak => CandidateEvidence::flatpak_verified_narrow(),
        pkgseal_domain::PackageSource::Aur => CandidateEvidence::aur_community(),
    }
}

// Unused import helper used for confidence string mapping tests
#[allow(dead_code)]
pub fn parse_confidence(s: &str) -> Confidence {
    match s.to_ascii_lowercase().as_str() {
        "high" => Confidence::High,
        "medium" => Confidence::Medium,
        "low" => Confidence::Low,
        "uncertain" => Confidence::Uncertain,
        _ => Confidence::None,
    }
}
