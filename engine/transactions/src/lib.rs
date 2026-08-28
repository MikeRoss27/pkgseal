//! `pkgseal-transactions` — inspectable transaction planning for PkgSeal.
//!
//! This crate produces **read-only** [`TransactionPlan`]s that preview what
//! would happen before any privileged operation is attempted. It never spawns
//! processes, touches the filesystem, or holds a generic root handle.
//!
//! ```text
//! Plan  -> Inspect -> Confirm -> Authorize (Polkit) -> Execute (privileged helper)
//! ```
//!
//! In the current milestone only planning and stub execution exist; real
//! `pacman` / `flatpak` execution lives in `platform/linux` behind narrow,
//! typed operations (see [`TransactionOperation`]).
//!
//! # Architecture invariants
//! - Depends only on `pkgseal-domain`, `thiserror`, `serde`, `uuid`, `time` —
//!   no Tauri, no HTTP clients, no filesystem IO.
//! - `engine/policy` is deterministic and IO-free; `engine/transactions`
//!   mirrors that discipline for planning.
//! - All privileged operations are narrow and allow-listed; there is no
//!   `run_as_root(command)` API.
//! - Plans are serializable and journalisable for audit trails.
//!
//! Modules mirror `docs/architecture/overview.md §30` and
//! `docs/adr/001-core-architecture.md §18`:
//! - `plan` — [`TransactionPlan`] (id, source, package, sizes, privileges, ops, metadata)
//! - `operation` — [`TransactionOperation`] (narrow, typed)
//! - `state` — [`TransactionState`] with validated transitions
//! - `executor` — [`TransactionExecutor`] trait + [`executor::StubExecutor`] (MVP)
//! - `error` — [`TransactionError`]

pub mod error;
pub mod executor;
pub mod operation;
pub mod plan;
pub mod state;

pub use error::TransactionError;
pub use executor::{
    FailingExecutor, StubExecutor, TransactionExecutionPreview, TransactionExecutor,
    TransactionResult,
};
pub use operation::TransactionOperation;
pub use plan::{TransactionId, TransactionMetadata, TransactionPlan};
pub use state::TransactionState;

#[cfg(test)]
mod tests {
    use super::*;
    use pkgseal_domain::{PackageName, PackageSource};
    use time::OffsetDateTime;

    fn fixed_time() -> OffsetDateTime {
        OffsetDateTime::from_unix_timestamp(1_767_225_600).unwrap()
    }

    #[test]
    fn plan_is_serializable_and_journalisable() {
        let plan = TransactionPlan::new_with_time(
            PackageSource::ArchOfficial,
            PackageName::new("brave").unwrap(),
            "1.70.0-1",
            vec![TransactionOperation::InstallPackage {
                name: PackageName::new("brave").unwrap(),
                version: "1.70.0-1".to_string(),
            }],
            true,
            fixed_time(),
            TransactionMetadata::new()
                .with_reason("user requested install")
                .with_extra("policy", "balanced"),
        )
        .unwrap()
        .with_expected_sizes(Some(150_000_000), Some(600_000_000));

        // Serializable
        let json = serde_json::to_string(&plan).unwrap();
        assert!(json.contains("\"arch-official\""));
        assert!(json.contains("\"planned\""));
        assert!(json.contains("\"brave\""));

        // Journalisable as JSON value
        let value = serde_json::to_value(&plan).unwrap();
        assert_eq!(value["source"], "arch-official");
        assert_eq!(value["state"], "planned");
        assert_eq!(value["privileges_required"], true);

        // Round-trip
        let parsed: TransactionPlan = serde_json::from_str(&json).unwrap();
        assert_eq!(plan, parsed);

        // Inspectable preview — needed for confirmation UI
        let preview = plan.preview();
        assert!(preview.contains("brave"));
        assert!(preview.contains("privileges required: true"));
        assert!(preview.contains("install brave"));
    }

    #[test]
    fn operation_is_narrow_and_validated() {
        // InstallPackage — requires valid name + non-empty version
        let install = TransactionOperation::InstallPackage {
            name: PackageName::new("visual-studio-code-bin").unwrap(),
            version: "1.99.0-1".to_string(),
        };
        assert!(install.validate().is_ok());
        assert!(install.requires_privileges());

        // Flatpak — reverse-DNS validation, not privileged in this layer
        let flatpak = TransactionOperation::InstallFlatpak {
            app_id: "com.brave.Browser".to_string(),
            version: None,
        };
        assert!(flatpak.validate().is_ok());
        assert!(!flatpak.requires_privileges());

        // Invalid flatpak app_id must be rejected
        let bad = TransactionOperation::InstallFlatpak {
            app_id: "brave".to_string(),
            version: None,
        };
        assert!(bad.validate().is_err());

        // No generic shell operation exists — compile-time guarantee.
        let json = serde_json::to_string(&install).unwrap();
        assert!(json.contains("install-package"));
        assert!(!json.contains("run_as_root"));
        assert!(!json.contains("sh -c"));
    }

    #[test]
    fn state_transitions_are_validated_and_read_only_flow() {
        let mut plan = TransactionPlan::new_with_time(
            PackageSource::Flatpak,
            PackageName::new("brave").unwrap(),
            "1.0",
            vec![TransactionOperation::InstallFlatpak {
                app_id: "com.brave.Browser".to_string(),
                version: None,
            }],
            false,
            fixed_time(),
            TransactionMetadata::default(),
        )
        .unwrap();

        // Read-only flow: Planned -> AwaitingConfirmation -> Authorizing -> Running -> Succeeded
        assert_eq!(plan.state, TransactionState::Planned);
        plan.transition_to(TransactionState::AwaitingConfirmation)
            .unwrap();
        assert!(plan.can_transition_to(TransactionState::Authorizing));
        plan.transition_to(TransactionState::Authorizing).unwrap();
        plan.transition_to(TransactionState::Running).unwrap();

        // Invalid transition must be rejected
        let err = plan.transition_to(TransactionState::Planned).unwrap_err();
        assert!(err.to_string().contains("invalid transition"));

        // Terminal state behaviour
        plan.transition_to(TransactionState::Succeeded).unwrap();
        assert!(plan.state.is_terminal());
        assert!(!plan.can_transition_to(TransactionState::Failed));

        // Executor in read-only MVP never mutates the system — stub only validates
        let executor = StubExecutor;
        let preview = executor.dry_run(&plan).unwrap();
        assert_eq!(preview.would_execute.len(), 1);
        assert!(!preview.privileges_required);
        assert!(preview.summary.contains("com.brave.Browser"));

        // Executing a succeeded plan is idempotent in the stub
        let result = executor.execute(&plan).unwrap();
        assert_eq!(result.state, TransactionState::Succeeded);
    }
}
