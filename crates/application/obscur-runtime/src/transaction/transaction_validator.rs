//! [`TransactionValidator`]: stateless structural validation of UI transactions (spec §3.2).

use super::transaction_error::TransactionError;
use super::ui_operation::UiOperation;
use super::ui_transaction::UiTransaction;

/// A stateless validator for [`UiTransaction`]s.
///
/// Per spec §3.2, normal UI commits are *validated before application*. This
/// validator performs the **structural** checks that require only the batch
/// itself — it never mutates anything and holds no state:
///
/// - at most one [`UiOperation::ReplaceRoot`] per batch, otherwise
///   [`TransactionError::ConflictingRootReplacement`] (a frame can reconcile at
///   most one new root);
/// - a node is not targeted twice by the same kind of operation within the
///   batch, otherwise [`TransactionError::DuplicateNodeOperation`].
///
/// **Liveness** validation — checking that targeted nodes actually exist in the
/// tree — is deliberately *not* done here; it is applied later in the frame
/// pipeline, where the live tree is available. Keeping this validator
/// tree-agnostic is what lets it stay stateless and reusable for provisional
/// (commit/cancel) systems.
///
/// [`UiTransaction`]: crate::transaction::UiTransaction
#[derive(Debug, Clone, Copy, Default)]
pub struct TransactionValidator;

impl TransactionValidator {
    /// Creates a new validator. The validator is zero-sized and stateless.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Validates that `tx` is structurally well-formed without mutating it.
    ///
    /// Returns `Ok(())` if the batch may be committed, or the first
    /// [`TransactionError`] encountered otherwise. Because validation is
    /// side-effect free, a rejected transaction leaves every domain untouched.
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

    /// Returns a stable discriminant tag for the node-targeting operation kinds,
    /// used only to compare "same kind" during duplicate detection.
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
        assert_eq!(
            TransactionValidator.validate(&UiTransaction::new()),
            Ok(())
        );
    }
}
