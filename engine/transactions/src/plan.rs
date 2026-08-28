use std::collections::HashMap;
use std::fmt;

use pkgseal_domain::{CandidateId, PackageName, PackageSource};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::TransactionError;
use crate::operation::TransactionOperation;
use crate::state::TransactionState;

/// Newtype for transaction identity. Distinct from `CandidateId` / `ApplicationId`
/// to prevent accidental mixing of identifiers.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransactionId(pub Uuid);

impl TransactionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn nil() -> Self {
        Self(Uuid::nil())
    }

    pub fn from_uuid(id: Uuid) -> Self {
        Self(id)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for TransactionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TransactionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Extra context attached to a plan for journaling and UI rendering.
///
/// Kept generic but typed: known fields are explicit, free-form data goes
/// into `extra`. No secrets should be stored here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TransactionMetadata {
    /// Candidate that triggered the plan, if any.
    pub candidate_id: Option<CandidateId>,
    /// Why this plan was created (e.g. "user requested install").
    pub reason: Option<String>,
    /// Free-form key/value for UI or journaling.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub extra: HashMap<String, String>,
}

impl TransactionMetadata {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_candidate(mut self, id: CandidateId) -> Self {
        self.candidate_id = Some(id);
        self
    }

    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    pub fn with_extra(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra.insert(key.into(), value.into());
        self
    }
}

/// An inspectable, serializable plan that previews what *would* happen
/// before any privileged operation is attempted.
///
/// **Read-only first**: constructing or serializing a plan never touches
/// the filesystem, spawns a process, or requires privileges. Execution is
/// delegated to a future privileged helper via narrow, typed operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionPlan {
    /// Unique plan identifier — stable for journaling.
    pub id: TransactionId,

    /// Source of the primary package (arch-official, aur, flatpak).
    pub source: PackageSource,

    /// Primary package name shown in the preview.
    pub package_name: PackageName,

    /// Primary package version shown in the preview.
    pub package_version: String,

    /// Expected network download (bytes), if known.
    pub expected_download_size: Option<u64>,

    /// Expected disk change (bytes): positive for installs, negative for removals.
    pub expected_disk_change: Option<i64>,

    /// Whether the plan will require elevated privileges when executed.
    /// Arch installs always `true`; Flatpak user installs may be `false`.
    pub privileges_required: bool,

    /// Ordered, narrow operations. The privileged helper will only accept these.
    pub operations: Vec<TransactionOperation>,

    /// Creation timestamp. Serialized as RFC 3339.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,

    /// Current lifecycle state — starts at `Planned`.
    pub state: TransactionState,

    /// Additional journaling context.
    #[serde(default)]
    pub metadata: TransactionMetadata,
}

impl TransactionPlan {
    /// Creates a new plan in `Planned` state with `created_at = now`.
    ///
    /// Validates that `operations` is non-empty and each operation validates.
    pub fn new(
        source: PackageSource,
        package_name: PackageName,
        package_version: impl Into<String>,
        operations: Vec<TransactionOperation>,
        privileges_required: bool,
    ) -> Result<Self, TransactionError> {
        Self::new_with_time(
            source,
            package_name,
            package_version,
            operations,
            privileges_required,
            OffsetDateTime::now_utc(),
            TransactionMetadata::default(),
        )
    }

    /// Deterministic constructor for tests — caller provides `created_at`.
    pub fn new_with_time(
        source: PackageSource,
        package_name: PackageName,
        package_version: impl Into<String>,
        operations: Vec<TransactionOperation>,
        privileges_required: bool,
        created_at: OffsetDateTime,
        metadata: TransactionMetadata,
    ) -> Result<Self, TransactionError> {
        let package_version = package_version.into();
        if package_version.trim().is_empty() {
            return Err(TransactionError::validation(
                "package_version cannot be empty",
            ));
        }
        if operations.is_empty() {
            return Err(TransactionError::validation(
                "TransactionPlan requires at least one operation",
            ));
        }
        for op in &operations {
            op.validate()?;
        }
        Ok(Self {
            id: TransactionId::new(),
            source,
            package_name,
            package_version,
            expected_download_size: None,
            expected_disk_change: None,
            privileges_required,
            operations,
            created_at,
            state: TransactionState::Planned,
            metadata,
        })
    }

    /// Builder-style helper to set expected sizes.
    pub fn with_expected_sizes(mut self, download: Option<u64>, disk_change: Option<i64>) -> Self {
        self.expected_download_size = download;
        self.expected_disk_change = disk_change;
        self
    }

    /// Builder-style helper to set metadata.
    pub fn with_metadata(mut self, metadata: TransactionMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Builder-style helper to set a deterministic id (tests).
    pub fn with_id(mut self, id: TransactionId) -> Self {
        self.id = id;
        self
    }

    /// Validated state transition — mutates `self.state` only via
    /// [`TransactionState::transition`].
    pub fn transition_to(&mut self, next: TransactionState) -> Result<(), TransactionError> {
        let next = self.state.transition(next)?;
        self.state = next;
        Ok(())
    }

    /// Non-mutating preview of whether a transition would succeed.
    pub fn can_transition_to(&self, next: TransactionState) -> bool {
        self.state.can_transition_to(next)
    }

    /// Human-readable one-line summary for logs and journal (no secrets).
    pub fn summary(&self) -> String {
        format!(
            "tx {} {} {} {} ops={} priv={} state={}",
            self.id,
            self.source,
            self.package_name,
            self.package_version,
            self.operations.len(),
            self.privileges_required,
            self.state
        )
    }

    /// Detailed inspectable preview for UI — lists operations, sizes, and privileges.
    pub fn preview(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!("Transaction {} [{}]", self.id, self.state));
        lines.push(format!(
            "  source: {}  package: {} {}",
            self.source, self.package_name, self.package_version
        ));
        if let Some(dl) = self.expected_download_size {
            lines.push(format!("  expected download: {} bytes", dl));
        }
        if let Some(disk) = self.expected_disk_change {
            let sign = if disk >= 0 { "+" } else { "" };
            lines.push(format!("  expected disk change: {sign}{disk} bytes"));
        }
        lines.push(format!(
            "  privileges required: {}",
            self.privileges_required
        ));
        lines.push(format!("  operations ({}):", self.operations.len()));
        for (idx, op) in self.operations.iter().enumerate() {
            lines.push(format!("    {}. {}", idx + 1, op.summary()));
        }
        if let Some(reason) = &self.metadata.reason {
            lines.push(format!("  reason: {reason}"));
        }
        lines.join("\n")
    }

    /// Validates the whole plan (operations + state coherence).
    pub fn validate(&self) -> Result<(), TransactionError> {
        if self.package_version.trim().is_empty() {
            return Err(TransactionError::validation(
                "package_version cannot be empty",
            ));
        }
        if self.operations.is_empty() {
            return Err(TransactionError::validation(
                "TransactionPlan requires at least one operation",
            ));
        }
        for op in &self.operations {
            op.validate()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pkgseal_domain::PackageName;
    use time::OffsetDateTime;

    fn fixed_time() -> OffsetDateTime {
        // 2026-01-01T00:00:00Z
        time::OffsetDateTime::from_unix_timestamp(1_767_225_600).unwrap()
    }

    fn arch_install_plan() -> TransactionPlan {
        TransactionPlan::new_with_time(
            PackageSource::ArchOfficial,
            PackageName::new("brave").unwrap(),
            "1.70.0-1",
            vec![TransactionOperation::InstallPackage {
                name: PackageName::new("brave").unwrap(),
                version: "1.70.0-1".to_string(),
            }],
            true,
            fixed_time(),
            TransactionMetadata::default(),
        )
        .unwrap()
    }

    #[test]
    fn new_plan_starts_planned() {
        let plan = arch_install_plan();
        assert_eq!(plan.state, TransactionState::Planned);
        assert!(plan.privileges_required);
        assert_eq!(plan.operations.len(), 1);
    }

    #[test]
    fn new_plan_rejects_empty_operations() {
        let err = TransactionPlan::new_with_time(
            PackageSource::ArchOfficial,
            PackageName::new("brave").unwrap(),
            "1.0",
            vec![],
            true,
            fixed_time(),
            TransactionMetadata::default(),
        )
        .unwrap_err();
        assert!(err.to_string().contains("at least one operation"));
    }

    #[test]
    fn plan_serializes_and_deserializes() {
        let plan = arch_install_plan()
            .with_expected_sizes(Some(120_000_000), Some(450_000_000))
            .with_metadata(
                TransactionMetadata::new()
                    .with_reason("user requested install")
                    .with_extra("policy", "balanced"),
            );
        let json = serde_json::to_string(&plan).unwrap();
        assert!(json.contains("brave"));
        assert!(json.contains("planned"));
        // RFC3339 timestamp present
        assert!(json.contains("2026-01-01"));
        let parsed: TransactionPlan = serde_json::from_str(&json).unwrap();
        assert_eq!(plan, parsed);
    }

    #[test]
    fn plan_summary_is_log_safe() {
        let plan = arch_install_plan();
        let summary = plan.summary();
        assert!(summary.contains("brave"));
        assert!(summary.contains("planned"));
        // Must not contain secrets like tokens — we only emit ids and package info.
        assert!(!summary.contains("password"));
    }

    #[test]
    fn plan_preview_lists_operations() {
        let plan = arch_install_plan();
        let preview = plan.preview();
        assert!(preview.contains("install brave"));
        assert!(preview.contains("privileges required: true"));
    }

    #[test]
    fn transition_via_plan_validates() {
        let mut plan = arch_install_plan();
        assert!(plan.can_transition_to(TransactionState::AwaitingConfirmation));
        plan.transition_to(TransactionState::AwaitingConfirmation)
            .unwrap();
        assert_eq!(plan.state, TransactionState::AwaitingConfirmation);
        // Invalid jump should fail and not mutate.
        let err = plan.transition_to(TransactionState::Succeeded).unwrap_err();
        assert!(err.to_string().contains("invalid transition"));
        assert_eq!(plan.state, TransactionState::AwaitingConfirmation);
    }

    #[test]
    fn journalisable_via_json_value() {
        let plan = arch_install_plan();
        let value = serde_json::to_value(&plan).unwrap();
        assert_eq!(value["source"], "arch-official");
        assert_eq!(value["state"], "planned");
        assert_eq!(value["package_name"], "brave");
    }
}
