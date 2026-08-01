#![doc = include_str!("tx_error.doc/_module.md")]

use core::fmt;

use obscur_identity::NodeId;

#[doc = include_str!("tx_error.doc/TxError.md")]
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TxError {
    #[doc = include_str!("tx_error.doc/StaleNode.md")]
    StaleNode(NodeId),
    #[doc = include_str!("tx_error.doc/UndefinedNewRef.md")]
    UndefinedNewRef(u32),
    #[doc = include_str!("tx_error.doc/NotTextNode.md")]
    NotTextNode(NodeId),
    #[doc = include_str!("tx_error.doc/IndexOutOfRange.md")]
    IndexOutOfRange {
        #[doc = include_str!("tx_error.doc/index.md")]
        index: usize,
        #[doc = include_str!("tx_error.doc/child_count.md")]
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
