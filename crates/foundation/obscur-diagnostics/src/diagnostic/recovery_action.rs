//! Recovery action taken or recommended for a diagnostic.

/// The recovery action associated with a diagnostic (spec §1.6). Failures
/// prefer fallback over propagation where correctness holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryAction {
    /// No recovery was needed.
    None,
    /// A fallback value or UI was substituted.
    Fallback,
    /// The affected subtree was retried.
    Retried,
    /// The affected subtree was replaced with an error boundary.
    BoundaryReplaced,
}
