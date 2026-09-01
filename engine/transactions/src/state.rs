use serde::{Deserialize, Serialize};

use crate::error::TransactionError;

/// Lifecycle of a transaction plan.
///
/// ```text
/// Planned -> AwaitingConfirmation -> Authorizing -> Running -> Succeeded
///                                                       \-> Failed
/// Any of Planned/AwaitingConfirmation/Authorizing/Running may become Cancelled.
/// Succeeded, Failed, Cancelled are terminal.
/// ```
///
/// All transitions are validated via [`TransactionState::can_transition_to`] and
/// [`TransactionState::transition`]; no direct field mutation should bypass them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum TransactionState {
    Planned,
    AwaitingConfirmation,
    Authorizing,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

impl TransactionState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::AwaitingConfirmation => "awaiting-confirmation",
            Self::Authorizing => "authorizing",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    /// Whether this state is terminal — no outgoing transitions allowed.
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Cancelled)
    }

    /// Whether `next` is a valid successor of `self`.
    pub fn can_transition_to(self, next: Self) -> bool {
        if self == next {
            return false;
        }
        if self.is_terminal() {
            return false;
        }
        match self {
            Self::Planned => matches!(
                next,
                Self::AwaitingConfirmation | Self::Cancelled | Self::Failed
            ),
            Self::AwaitingConfirmation => {
                matches!(next, Self::Authorizing | Self::Cancelled | Self::Failed)
            }
            Self::Authorizing => matches!(next, Self::Running | Self::Cancelled | Self::Failed),
            Self::Running => matches!(next, Self::Succeeded | Self::Failed | Self::Cancelled),
            Self::Succeeded | Self::Failed | Self::Cancelled => false,
        }
    }

    /// Validated transition. Returns `next` on success or a
    /// [`TransactionError::InvalidTransition`] on failure.
    pub fn transition(self, next: Self) -> Result<Self, TransactionError> {
        if self.can_transition_to(next) {
            Ok(next)
        } else {
            Err(TransactionError::invalid_transition(
                self.as_str(),
                next.as_str(),
            ))
        }
    }
}

impl std::fmt::Display for TransactionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for TransactionState {
    type Err = TransactionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "planned" => Ok(Self::Planned),
            "awaiting-confirmation" => Ok(Self::AwaitingConfirmation),
            "authorizing" => Ok(Self::Authorizing),
            "running" => Ok(Self::Running),
            "succeeded" => Ok(Self::Succeeded),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            other => Err(TransactionError::validation(format!(
                "unknown TransactionState '{other}'"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn planned_to_awaiting_confirmation_valid() {
        assert!(
            TransactionState::Planned.can_transition_to(TransactionState::AwaitingConfirmation)
        );
        assert!(
            TransactionState::Planned
                .transition(TransactionState::AwaitingConfirmation)
                .is_ok()
        );
    }

    #[test]
    fn planned_cannot_jump_to_running() {
        assert!(!TransactionState::Planned.can_transition_to(TransactionState::Running));
        assert!(
            TransactionState::Planned
                .transition(TransactionState::Running)
                .is_err()
        );
    }

    #[test]
    fn full_happy_path() {
        let mut s = TransactionState::Planned;
        s = s
            .transition(TransactionState::AwaitingConfirmation)
            .unwrap();
        s = s.transition(TransactionState::Authorizing).unwrap();
        s = s.transition(TransactionState::Running).unwrap();
        s = s.transition(TransactionState::Succeeded).unwrap();
        assert_eq!(s, TransactionState::Succeeded);
        assert!(s.is_terminal());
    }

    #[test]
    fn terminal_states_have_no_outgoing() {
        for terminal in [
            TransactionState::Succeeded,
            TransactionState::Failed,
            TransactionState::Cancelled,
        ] {
            for candidate in [
                TransactionState::Planned,
                TransactionState::AwaitingConfirmation,
                TransactionState::Authorizing,
                TransactionState::Running,
                TransactionState::Succeeded,
                TransactionState::Failed,
                TransactionState::Cancelled,
            ] {
                assert!(
                    !terminal.can_transition_to(candidate),
                    "{terminal} should not transition to {candidate}"
                );
            }
        }
    }

    #[test]
    fn cancel_allowed_from_early_states() {
        assert!(TransactionState::Planned.can_transition_to(TransactionState::Cancelled));
        assert!(
            TransactionState::AwaitingConfirmation.can_transition_to(TransactionState::Cancelled)
        );
        assert!(TransactionState::Authorizing.can_transition_to(TransactionState::Cancelled));
        assert!(TransactionState::Running.can_transition_to(TransactionState::Cancelled));
    }

    #[test]
    fn display_and_fromstr_roundtrip() {
        for state in [
            TransactionState::Planned,
            TransactionState::AwaitingConfirmation,
            TransactionState::Authorizing,
            TransactionState::Running,
            TransactionState::Succeeded,
            TransactionState::Failed,
            TransactionState::Cancelled,
        ] {
            let s = state.as_str();
            let parsed = TransactionState::from_str(s).unwrap();
            assert_eq!(state, parsed);
            assert_eq!(format!("{state}"), s);
        }
    }

    #[test]
    fn serde_roundtrip() {
        let state = TransactionState::AwaitingConfirmation;
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, "\"awaiting-confirmation\"");
        let parsed: TransactionState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, parsed);
    }

    #[test]
    fn self_transition_forbidden() {
        assert!(!TransactionState::Planned.can_transition_to(TransactionState::Planned));
    }
}
