#![doc = include_str!("transaction_validator.doc/_module.md")]

use super::transaction_error::TransactionError;
use super::ui_operation::UiOperation;
use super::ui_transaction::UiTransaction;

#[doc = include_str!("transaction_validator.doc/TransactionValidator.md")]
#[derive(Debug, Clone, Copy, Default)]
pub struct TransactionValidator;

impl TransactionValidator {
    #[doc = include_str!("transaction_validator.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[doc = include_str!("transaction_validator.doc/validate.md")]
    pub fn validate(&self, tx: &UiTransaction) -> Result<(), TransactionError> {
        let mut root_replacements = 0usize;
        // (discriminant tag, node) pairs already seen, used to detect a node
        // targeted twice by the same kind of operation. A small linear scan is
        // used to stay allocation-light and dependency-free.
        let mut seen: Vec<(u8, u64)> = Vec::new();

        for op in tx.operations() {
            if op.is_root_replacement() {
                root_replacements += 1;
                if root_replacements > 1 {
                    return Err(TransactionError::ConflictingRootReplacement);
                }
                continue;
            }

            if let Some(node) = op.target() {
                let tag = Self::kind_tag(op);
                let key = (tag, node.raw());
                if seen.contains(&key) {
                    return Err(TransactionError::DuplicateNodeOperation(node));
                }
                seen.push(key);
            }
        }

        Ok(())
    }

    #[doc = include_str!("transaction_validator.doc/kind_tag.md")]
    const fn kind_tag(op: &UiOperation) -> u8 {
        match op {
            UiOperation::ReplaceRoot => 0,
            UiOperation::Rerender { .. } => 1,
            UiOperation::InvalidateStyle { .. } => 2,
            UiOperation::InvalidateLayout { .. } => 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::NodeId;

    #[test]
    fn rejects_two_root_replacements() {
        let mut tx = UiTransaction::new();
        tx.push(UiOperation::ReplaceRoot)
            .push(UiOperation::ReplaceRoot);
        assert_eq!(
            TransactionValidator::new().validate(&tx),
            Err(TransactionError::ConflictingRootReplacement)
        );
    }

    #[test]
    fn accepts_single_root_and_mixed_node_ops() {
        let a = NodeId::from_raw(1);
        let b = NodeId::from_raw(2);
        let mut tx = UiTransaction::new();
        tx.push(UiOperation::ReplaceRoot)
            .push(UiOperation::Rerender { node: a })
            .push(UiOperation::InvalidateStyle { node: a })
            .push(UiOperation::InvalidateLayout { node: b });
        assert_eq!(TransactionValidator.validate(&tx), Ok(()));
    }

    #[test]
    fn rejects_duplicate_same_kind_node_op() {
        let node = NodeId::from_raw(5);
        let mut tx = UiTransaction::new();
        tx.push(UiOperation::Rerender { node })
            .push(UiOperation::Rerender { node });
        assert_eq!(
            TransactionValidator.validate(&tx),
            Err(TransactionError::DuplicateNodeOperation(node))
        );
    }

    #[test]
    fn allows_same_node_with_different_kinds() {
        let node = NodeId::from_raw(5);
        let mut tx = UiTransaction::new();
        tx.push(UiOperation::InvalidateStyle { node })
            .push(UiOperation::InvalidateLayout { node });
        assert_eq!(TransactionValidator.validate(&tx), Ok(()));
    }

    #[test]
    fn empty_transaction_is_valid() {
        assert_eq!(TransactionValidator.validate(&UiTransaction::new()), Ok(()));
    }
}
