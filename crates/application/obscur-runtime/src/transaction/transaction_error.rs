//! [`TransactionError`]: typed validation failure for UI transactions (spec §3.2).

use core::error::Error;
use core::fmt;

use obscur_identity::NodeId;

/// A typed error produced when a [`UiTransaction`] fails validation.
///
/// Per spec §3.2, normal UI commits are *validated before application*: a batch
/// that is not well-formed is rejected and nothing is applied. These errors are
/// returned (never panicked) so the caller can decide how to recover.
///
/// This enum is `#[non_exhaustive]`: later phases may add validation rules and
/// corresponding variants without a breaking change.
///
/// [`UiTransaction`]: crate::transaction::UiTransaction
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TransactionError {
    /// The batch contains more than one root replacement, which is ambiguous:
    /// a single frame can reconcile at most one new root.
    ConflictingRootReplacement,
    /// The same node is targeted by two conflicting operations of the same kind
    /// within one batch. Carries the offending node.
    DuplicateNodeOperation(NodeId),
}

impl fmt::Display for TransactionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConflictingRootReplacement => {
                f.write_str("transaction contains more than one root replacement")
            }
            Self::DuplicateNodeOperation(node) => {
                write!(
                    f,
                    "transaction contains a duplicate operation for node {}",
                    node.raw()
                )
            }
        }
    }
}

impl Error for TransactionError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_is_non_empty_for_each_variant() {
        assert!(!TransactionError::ConflictingRootReplacement.to_string().is_empty());
        let err = TransactionError::DuplicateNodeOperation(NodeId::from_raw(9));
        assert!(err.to_string().contains('9'));
    }

    #[test]
    fn implements_std_error() {
        fn assert_error<E: Error>(_: &E) {}
        assert_error(&TransactionError::ConflictingRootReplacement);
    }
}
