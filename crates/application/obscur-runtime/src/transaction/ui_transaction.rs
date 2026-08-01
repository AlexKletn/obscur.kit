#![doc = include_str!("ui_transaction.doc/_module.md")]

use core::slice::Iter;

use super::ui_operation::UiOperation;

#[doc = include_str!("ui_transaction.doc/UiTransaction.md")]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UiTransaction {
    operations: Vec<UiOperation>,
}

impl UiTransaction {
    #[doc = include_str!("ui_transaction.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    #[doc = include_str!("ui_transaction.doc/push.md")]
    pub fn push(&mut self, op: UiOperation) -> &mut Self {
        self.operations.push(op);
        self
    }

    #[doc = include_str!("ui_transaction.doc/operations.md")]
    #[must_use]
    pub fn operations(&self) -> &[UiOperation] {
        &self.operations
    }

    #[doc = include_str!("ui_transaction.doc/iter.md")]
    pub fn iter(&self) -> Iter<'_, UiOperation> {
        self.operations.iter()
    }

    #[doc = include_str!("ui_transaction.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.operations.len()
    }

    #[doc = include_str!("ui_transaction.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    #[doc = include_str!("ui_transaction.doc/is_root_replacement.md")]
    #[must_use]
    pub fn is_root_replacement(&self) -> bool {
        self.operations.iter().any(UiOperation::is_root_replacement)
    }

    #[doc = include_str!("ui_transaction.doc/into_operations.md")]
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
