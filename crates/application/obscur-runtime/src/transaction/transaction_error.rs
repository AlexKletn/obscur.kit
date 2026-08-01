#![doc = include_str!("transaction_error.doc/_module.md")]

use core::error::Error;
use core::fmt;

use obscur_identity::NodeId;

#[doc = include_str!("transaction_error.doc/TransactionError.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TransactionError {
    #[doc = include_str!("transaction_error.doc/ConflictingRootReplacement.md")]
    ConflictingRootReplacement,
    #[doc = include_str!("transaction_error.doc/DuplicateNodeOperation.md")]
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
        assert!(
            !TransactionError::ConflictingRootReplacement
                .to_string()
                .is_empty()
        );
        let err = TransactionError::DuplicateNodeOperation(NodeId::from_raw(9));
        assert!(err.to_string().contains('9'));
    }

    #[test]
    fn implements_std_error() {
        fn assert_error<E: Error>(_: &E) {}
        assert_error(&TransactionError::ConflictingRootReplacement);
    }
}
