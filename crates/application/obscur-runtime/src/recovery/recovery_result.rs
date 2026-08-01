#![doc = include_str!("recovery_result.doc/_module.md")]

use obscur_diagnostics::RecoveryAction;

#[doc = include_str!("recovery_result.doc/RecoveryResult.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RecoveryResult {
    #[doc = include_str!("recovery_result.doc/Retried.md")]
    Retried,
    #[doc = include_str!("recovery_result.doc/FellBack.md")]
    FellBack,
    #[doc = include_str!("recovery_result.doc/Propagated.md")]
    Propagated,
}

impl RecoveryResult {
    #[doc = include_str!("recovery_result.doc/is_retried.md")]
    #[must_use]
    pub const fn is_retried(self) -> bool {
        matches!(self, Self::Retried)
    }

    #[doc = include_str!("recovery_result.doc/is_fallback.md")]
    #[must_use]
    pub const fn is_fallback(self) -> bool {
        matches!(self, Self::FellBack)
    }

    #[doc = include_str!("recovery_result.doc/is_propagated.md")]
    #[must_use]
    pub const fn is_propagated(self) -> bool {
        matches!(self, Self::Propagated)
    }

    #[doc = include_str!("recovery_result.doc/recovery_action.md")]
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
