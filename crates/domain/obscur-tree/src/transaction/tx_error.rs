//! `TxError`: typed validation errors returned by a transaction commit.

use core::fmt;

use obscur_identity::NodeId;

#[doc = include_str!("../../docs/api/transaction/tx_error.doc/TxError.md")]
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TxError {
    /// A referenced existing node is not (or no longer) live in the tree.
    StaleNode(NodeId),
    /// A [`StagedRef::New`](super::StagedRef::New) index refers to a creation
    /// that has not been staged before its use in this transaction.
    UndefinedNewRef(u32),
    /// A [`SetText`](super::TreeChange::SetText) targeted a node that is not a
    /// text node.
    NotTextNode(NodeId),
    /// A child insertion or move used an index outside the valid range for the
    /// parent's (projected) child count.
    IndexOutOfRange {
        /// The offending index that was requested.
        index: usize,
        /// The parent's child count against which the index was validated.
        child_count: usize,
    },
}

impl fmt::Display for TxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StaleNode(id) => {
                write!(f, "referenced node {id:?} is not live in the tree")
            }
            Self::UndefinedNewRef(i) => {
                write!(
                    f,
                    "staged new-node reference {i} has no matching creation in this transaction"
                )
            }
            Self::NotTextNode(id) => {
                write!(f, "set-text targeted node {id:?}, which is not a text node")
            }
            Self::IndexOutOfRange { index, child_count } => {
                write!(
                    f,
                    "child index {index} is out of range for a parent with {child_count} child(ren)"
                )
            }
        }
    }
}

impl core::error::Error for TxError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_is_non_empty() {
        let errs = [
            TxError::StaleNode(NodeId::from_raw(1)),
            TxError::UndefinedNewRef(2),
            TxError::NotTextNode(NodeId::from_raw(3)),
            TxError::IndexOutOfRange {
                index: 5,
                child_count: 2,
            },
        ];
        for e in errs {
            assert!(!format!("{e}").is_empty());
        }
    }

    #[test]
    fn is_std_error() {
        fn assert_error<E: core::error::Error>(_: &E) {}
        assert_error(&TxError::StaleNode(NodeId::from_raw(1)));
    }
}
