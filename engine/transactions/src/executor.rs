use time::OffsetDateTime;

use crate::error::TransactionError;
use crate::operation::TransactionOperation;
use crate::plan::{TransactionId, TransactionPlan};
use crate::state::TransactionState;

/// Result of executing (or dry-running) a transaction plan.
///
/// **Read-only first**: in the current milestone the executor never performs
/// real privileged mutations. It only validates the plan, simulates state
/// transitions, and produces an inspectable result. Future milestones will
/// delegate Arch/Flatpak operations to `platform/linux` via Polkit.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TransactionResult {
    pub plan_id: TransactionId,
    pub state: TransactionState,
    pub message: Option<String>,
    pub executed_operations: Vec<TransactionOperation>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub started_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub finished_at: Option<OffsetDateTime>,
}

impl TransactionResult {
    pub fn success(plan: &TransactionPlan, message: impl Into<String>) -> Self {
        Self {
            plan_id: plan.id.clone(),
            state: TransactionState::Succeeded,
            message: Some(message.into()),
            executed_operations: plan.operations.clone(),
            started_at: Some(OffsetDateTime::now_utc()),
            finished_at: Some(OffsetDateTime::now_utc()),
        }
    }

    pub fn failure(plan: &TransactionPlan, message: impl Into<String>) -> Self {
        Self {
            plan_id: plan.id.clone(),
            state: TransactionState::Failed,
            message: Some(message.into()),
            executed_operations: Vec::new(),
            started_at: Some(OffsetDateTime::now_utc()),
            finished_at: Some(OffsetDateTime::now_utc()),
        }
    }
}

/// Preview of what `execute` would do — no side effects.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TransactionExecutionPreview {
    pub plan_id: TransactionId,
    pub would_execute: Vec<TransactionOperation>,
    pub privileges_required: bool,
    pub summary: String,
}

/// Abstraction over transaction execution. Implementors decide how to
/// materialize a `TransactionPlan`.
///
/// The trait is deliberately narrow and synchronous — no generic
/// `run_as_root(command)` — so every privileged operation remains
/// auditable and Polkit-scoped.
pub trait TransactionExecutor: Send + Sync {
    /// Validate and (in future) execute the plan, producing a result.
    ///
    /// In the read-only MVP this performs **no** system mutation.
    fn execute(&self, plan: &TransactionPlan) -> Result<TransactionResult, TransactionError>;

    /// Dry-run preview — always side-effect free.
    fn dry_run(
        &self,
        plan: &TransactionPlan,
    ) -> Result<TransactionExecutionPreview, TransactionError>;
}

/// Stub executor for the read-only milestone.
///
/// - Validates the plan.
/// - Returns `Succeeded` if the plan is well-formed and in a runnable state.
/// - Returns `Failed`/error otherwise.
/// - Never spawns processes, touches the filesystem, or asks for privileges.
#[derive(Debug, Default, Clone, Copy)]
pub struct StubExecutor;

impl TransactionExecutor for StubExecutor {
    fn execute(&self, plan: &TransactionPlan) -> Result<TransactionResult, TransactionError> {
        plan.validate()?;

        // Only allow execution from states that would be reachable in a real flow.
        // For the stub we are permissive: Planned and AwaitingConfirmation are
        // considered runnable via an implicit confirmation in tests.
        match plan.state {
            TransactionState::Planned
            | TransactionState::AwaitingConfirmation
            | TransactionState::Authorizing
            | TransactionState::Running => {}
            TransactionState::Succeeded => {
                return Ok(TransactionResult::success(plan, "already succeeded (stub)"));
            }
            TransactionState::Failed | TransactionState::Cancelled => {
                return Err(TransactionError::execution_failed(format!(
                    "cannot execute plan in state {}",
                    plan.state
                )));
            }
        }

        // Simulate successful execution without side effects.
        Ok(TransactionResult {
            plan_id: plan.id.clone(),
            state: TransactionState::Succeeded,
            message: Some(format!(
                "stub executed {} operation(s) — no system mutation",
                plan.operations.len()
            )),
            executed_operations: plan.operations.clone(),
            started_at: Some(OffsetDateTime::now_utc()),
            finished_at: Some(OffsetDateTime::now_utc()),
        })
    }

    fn dry_run(
        &self,
        plan: &TransactionPlan,
    ) -> Result<TransactionExecutionPreview, TransactionError> {
        plan.validate()?;
        Ok(TransactionExecutionPreview {
            plan_id: plan.id.clone(),
            would_execute: plan.operations.clone(),
            privileges_required: plan.privileges_required,
            summary: plan.preview(),
        })
    }
}

/// Executor that always fails — useful for testing error paths.
#[derive(Debug, Default, Clone, Copy)]
pub struct FailingExecutor {
    pub reason: Option<&'static str>,
}

impl TransactionExecutor for FailingExecutor {
    fn execute(&self, plan: &TransactionPlan) -> Result<TransactionResult, TransactionError> {
        plan.validate()?;
        Ok(TransactionResult::failure(
            plan,
            self.reason.unwrap_or("injected failure (test)"),
        ))
    }

    fn dry_run(
        &self,
        plan: &TransactionPlan,
    ) -> Result<TransactionExecutionPreview, TransactionError> {
        plan.validate()?;
        Err(TransactionError::execution_failed(
            self.reason.unwrap_or("dry-run injected failure"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pkgseal_domain::{PackageName, PackageSource};
    use time::OffsetDateTime;

    use crate::plan::{TransactionMetadata, TransactionPlan};
    use crate::state::TransactionState;

    fn fixed_time() -> OffsetDateTime {
        OffsetDateTime::from_unix_timestamp(1_767_225_600).unwrap()
    }

    fn plan() -> TransactionPlan {
        TransactionPlan::new_with_time(
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
        .unwrap()
    }

    #[test]
    fn stub_executor_succeeds_on_valid_plan() {
        let executor = StubExecutor;
        let p = plan();
        let result = executor.execute(&p).unwrap();
        assert_eq!(result.state, TransactionState::Succeeded);
        assert_eq!(result.executed_operations, p.operations);
        assert!(result.message.unwrap().contains("no system mutation"));
    }

    #[test]
    fn stub_executor_rejects_cancelled_plan() {
        let executor = StubExecutor;
        let mut p = plan();
        // Walk to Cancelled via valid transitions
        p.transition_to(TransactionState::AwaitingConfirmation)
            .unwrap();
        p.transition_to(TransactionState::Authorizing).unwrap();
        // Authorizing -> Cancelled is allowed
        p.transition_to(TransactionState::Cancelled).unwrap();
        let err = executor.execute(&p).unwrap_err();
        assert!(err.to_string().contains("cannot execute"));
    }

    #[test]
    fn dry_run_is_side_effect_free_and_inspectable() {
        let executor = StubExecutor;
        let p = plan().with_expected_sizes(Some(10_000), Some(50_000));
        let preview = executor.dry_run(&p).unwrap();
        assert_eq!(preview.would_execute, p.operations);
        assert_eq!(preview.privileges_required, p.privileges_required);
        assert!(preview.summary.contains("com.brave.Browser"));
    }

    #[test]
    fn failing_executor_returns_failed_result() {
        let executor = FailingExecutor { reason: None };
        let p = plan();
        let result = executor.execute(&p).unwrap();
        assert_eq!(result.state, TransactionState::Failed);
    }

    #[test]
    fn executor_validates_plan_before_execution() {
        let executor = StubExecutor;
        // Plan with empty operations is rejected at construction; construct a
        // plan then corrupt it to simulate a validation failure path.
        let mut p = plan();
        p.operations.clear();
        let err = executor.execute(&p).unwrap_err();
        assert!(err.to_string().contains("at least one operation"));
    }
}
