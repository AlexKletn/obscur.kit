//! `RecoveryResult`: the outcome of a single recovery decision (spec §1.7).

use obscur_diagnostics::RecoveryAction;

#[doc = include_str!("../../docs/api/recovery/recovery_result.doc/RecoveryResult.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RecoveryResult {
    /// The boundary is enabled and has retry budget left; the failed unit will
    /// be retried.
    Retried,
    /// The boundary is enabled but out of retries; the fallback UI is kept in
    /// place of the failed subtree.
    FellBack,
    /// The boundary is disabled; the failure escapes to the nearest enclosing
    /// boundary.
    Propagated,
}

impl RecoveryResult {
    /// Whether the decision was to retry the failed unit ([`Retried`](Self::Retried)).
    #[must_use]
    pub const fn is_retried(self) -> bool {
        matches!(self, Self::Retried)
    }

    /// Whether the decision was to keep the fallback UI ([`FellBack`](Self::FellBack)).
    #[must_use]
    pub const fn is_fallback(self) -> bool {
        matches!(self, Self::FellBack)
    }

    /// Whether the failure escaped to an enclosing boundary
    /// ([`Propagated`](Self::Propagated)).
    #[must_use]
    pub const fn is_propagated(self) -> bool {
        matches!(self, Self::Propagated)
    }

    #[doc = include_str!("../../docs/api/recovery/recovery_result.doc/recovery_action.md")]
    #[must_use]
    pub const fn recovery_action(self) -> RecoveryAction {
        match self {
            Self::Retried => RecoveryAction::Retried,
            Self::FellBack => RecoveryAction::BoundaryReplaced,
            Self::Propagated => RecoveryAction::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RecoveryResult;
    use obscur_diagnostics::RecoveryAction;

    #[test]
    fn predicates_are_mutually_exclusive() {
        assert!(RecoveryResult::Retried.is_retried());
        assert!(!RecoveryResult::Retried.is_fallback());
        assert!(!RecoveryResult::Retried.is_propagated());

        assert!(RecoveryResult::FellBack.is_fallback());
        assert!(!RecoveryResult::FellBack.is_retried());
        assert!(!RecoveryResult::FellBack.is_propagated());

        assert!(RecoveryResult::Propagated.is_propagated());
        assert!(!RecoveryResult::Propagated.is_retried());
        assert!(!RecoveryResult::Propagated.is_fallback());
    }

    #[test]
    fn recovery_action_mapping() {
        assert_eq!(
            RecoveryResult::Retried.recovery_action(),
            RecoveryAction::Retried
        );
        assert_eq!(
            RecoveryResult::FellBack.recovery_action(),
            RecoveryAction::BoundaryReplaced
        );
        assert_eq!(
            RecoveryResult::Propagated.recovery_action(),
            RecoveryAction::None
        );
    }
}
