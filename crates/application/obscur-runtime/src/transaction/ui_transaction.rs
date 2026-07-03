//! [`UiTransaction`]: a staged, ordered batch of normalized UI operations (spec §3.2).

use core::slice::Iter;

use super::ui_operation::UiOperation;

/// A staged, ordered batch of [`UiOperation`]s accumulated between frames.
///
/// Per spec §3.2, mutations are normalized into validated `UiOps` and committed
/// atomically between frames. A `UiTransaction` is the *staging* container for
/// those normalized ops: operations are appended in order, but **nothing is
/// applied** while they sit in the transaction. Application happens only after
/// the transaction is validated (see [`TransactionValidator`]) and committed at
/// a frame boundary (see [`TransactionCommit`]).
///
/// Operation order is preserved; the frame pipeline may rely on it when
/// interpreting the batch.
///
/// [`TransactionValidator`]: crate::transaction::TransactionValidator
/// [`TransactionCommit`]: crate::transaction::TransactionCommit
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UiTransaction {
    operations: Vec<UiOperation>,
}

impl UiTransaction {
    /// Creates a new, empty transaction with no staged operations.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    /// Appends a normalized operation to the end of the batch and returns `&mut self`
    /// for chaining. The operation is staged only; it is not applied here.
    pub fn push(&mut self, op: UiOperation) -> &mut Self {
        self.operations.push(op);
        self
    }

    /// Returns the staged operations in insertion order.
    #[must_use]
    pub fn operations(&self) -> &[UiOperation] {
        &self.operations
    }

    /// Returns an iterator over the staged operations in insertion order.
    pub fn iter(&self) -> Iter<'_, UiOperation> {
        self.operations.iter()
    }

    /// Returns the number of staged operations.
    #[must_use]
    pub fn len(&self) -> usize {
        self.operations.len()
    }

    /// Returns `true` if no operations are staged.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    /// Returns `true` if any staged operation is a root replacement
    /// (a whole-tree reconcile).
    #[must_use]
    pub fn is_root_replacement(&self) -> bool {
        self.operations.iter().any(UiOperation::is_root_replacement)
    }

    /// Consumes the transaction and returns the owned batch of staged operations.
    #[must_use]
    pub fn into_operations(self) -> Vec<UiOperation> {
        self.operations
    }
}

impl<'a> IntoIterator for &'a UiTransaction {
    type Item = &'a UiOperation;
    type IntoIter = Iter<'a, UiOperation>;

    fn into_iter(self) -> Self::IntoIter {
        self.operations.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::NodeId;

    #[test]
    fn new_transaction_is_empty() {
        let tx = UiTransaction::new();
        assert!(tx.is_empty());
        assert_eq!(tx.len(), 0);
        assert!(!tx.is_root_replacement());
    }

    #[test]
    fn push_preserves_order_and_reports_len() {
        let node = NodeId::from_raw(3);
        let mut tx = UiTransaction::new();
        tx.push(UiOperation::Rerender { node })
            .push(UiOperation::InvalidateStyle { node });
        assert_eq!(tx.len(), 2);
        assert!(!tx.is_empty());
        assert_eq!(
            tx.operations(),
            &[
                UiOperation::Rerender { node },
                UiOperation::InvalidateStyle { node },
            ]
        );
    }

    #[test]
    fn detects_root_replacement_and_yields_owned_ops() {
        let mut tx = UiTransaction::new();
        tx.push(UiOperation::ReplaceRoot);
        assert!(tx.is_root_replacement());
        assert_eq!(tx.iter().count(), 1);
        assert_eq!(tx.into_operations(), vec![UiOperation::ReplaceRoot]);
    }
}
