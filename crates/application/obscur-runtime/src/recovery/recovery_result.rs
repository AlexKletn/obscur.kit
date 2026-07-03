//! `RecoveryResult`: the outcome of a single recovery decision (spec §1.7).

use obscur_diagnostics::RecoveryAction;

/// The outcome of a recovery decision for a failed unit of work (spec §1.7).
///
/// A recovery decision follows the spec's *contain → (retry | fallback) |
/// propagate* rule. This value records which of those three branches was taken
/// so the runtime can act on it — rerun the unit, keep the fallback UI, or hand
/// the failure to an enclosing boundary — and report the choice to diagnostics
/// via [`recovery_action`](Self::recovery_action).
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

    /// Maps this outcome to the [`RecoveryAction`] reported to diagnostics.
    ///
    /// - [`Retried`](Self::Retried) → [`RecoveryAction::Retried`]
    /// - [`FellBack`](Self::FellBack) → [`RecoveryAction::BoundaryReplaced`]
    /// - [`Propagated`](Self::Propagated) → [`RecoveryAction::None`] (no
    ///   recovery happened at this boundary; the failure was handed outward)
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
