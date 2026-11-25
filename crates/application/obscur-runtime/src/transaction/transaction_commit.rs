//! [`TransactionCommit`]: the immutable record of a committed UI transaction (spec §3.2).

use obscur_identity::VersionStamp;

use super::ui_operation::UiOperation;

#[doc = include_str!("../../docs/api/transaction/transaction_commit.doc/TransactionCommit.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionCommit {
    operations: Vec<UiOperation>,
    version: VersionStamp,
}

impl TransactionCommit {
    /// Creates a commit record from the applied `operations` and the `version`
    /// stamped at the frame boundary.
    #[must_use]
    pub fn new(operations: Vec<UiOperation>, version: VersionStamp) -> Self {
        Self {
            operations,
            version,
        }
    }

    /// Returns the committed normalized operations in application order.
    #[must_use]
    pub fn operations(&self) -> &[UiOperation] {
        &self.operations
    }

    /// Returns the commit version stamped at the frame boundary.
    #[must_use]
    pub const fn version(&self) -> VersionStamp {
        self.version
    }

    /// Returns `true` if the commit carries no operations.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    /// Returns the number of committed operations.
    #[must_use]
    pub fn len(&self) -> usize {
        self.operations.len()
    }

    /// Returns `true` if any committed operation is a root replacement.
    #[must_use]
    pub fn is_root_replacement(&self) -> bool {
        self.operations.iter().any(UiOperation::is_root_replacement)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::NodeId;

    #[test]
    fn accessors_expose_operations_and_version() {
        let node = NodeId::from_raw(4);
        let ops = vec![UiOperation::ReplaceRoot, UiOperation::Rerender { node }];
        let version = VersionStamp::from_raw(42);
        let commit = TransactionCommit::new(ops.clone(), version);

        assert_eq!(commit.operations(), ops.as_slice());
        assert_eq!(commit.version(), version);
        assert_eq!(commit.len(), 2);
        assert!(!commit.is_empty());
        assert!(commit.is_root_replacement());
    }

    #[test]
    fn empty_commit_reports_empty() {
        let commit = TransactionCommit::new(Vec::new(), VersionStamp::default());
        assert!(commit.is_empty());
        assert_eq!(commit.len(), 0);
        assert!(!commit.is_root_replacement());
    }
}
