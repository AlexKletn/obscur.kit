//! [`ProvisionalTransaction`]: commit/cancel semantics for provisional systems (spec §3.2).

use obscur_identity::VersionStamp;

use super::transaction_commit::TransactionCommit;
use super::transaction_error::TransactionError;
use super::transaction_validator::TransactionValidator;
use super::ui_operation::UiOperation;
use super::ui_transaction::UiTransaction;

#[doc = include_str!("../../docs/api/transaction/provisional_transaction.doc/ProvisionalTransaction.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvisionalTransaction {
    pending: UiTransaction,
}

impl ProvisionalTransaction {
    /// Begins a provisional transaction from an already-staged [`UiTransaction`].
    #[must_use]
    pub fn begin(tx: UiTransaction) -> Self {
        Self { pending: tx }
    }

    /// Creates an empty provisional transaction with no pending operations.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            pending: UiTransaction::new(),
        }
    }

    /// Stages one more operation onto the pending transaction and returns
    /// `&mut self` for chaining. Nothing is applied — the op only accumulates.
    pub fn stage(&mut self, op: UiOperation) -> &mut Self {
        self.pending.push(op);
        self
    }

    /// Returns the pending, not-yet-applied transaction.
    #[must_use]
    pub const fn transaction(&self) -> &UiTransaction {
        &self.pending
    }

    #[doc = include_str!("../../docs/api/transaction/provisional_transaction.doc/commit.md")]
    pub fn commit(
        self,
        validator: &TransactionValidator,
        version: VersionStamp,
    ) -> Result<TransactionCommit, TransactionError> {
        validator.validate(&self.pending)?;
        Ok(TransactionCommit::new(
            self.pending.into_operations(),
            version,
        ))
    }

    #[doc = include_str!("../../docs/api/transaction/provisional_transaction.doc/cancel.md")]
    pub fn cancel(self) {
        // Dropping `self` discards the pending transaction; nothing to undo.
    }
}

impl Default for ProvisionalTransaction {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::NodeId;

    #[test]
    fn cancel_discards_without_a_commit() {
        let node = NodeId::from_raw(1);
        let mut provisional = ProvisionalTransaction::new();
        provisional
            .stage(UiOperation::Rerender { node })
            .stage(UiOperation::InvalidateStyle { node });
        assert_eq!(provisional.transaction().len(), 2);
        // `cancel` consumes the transaction and yields nothing.
        provisional.cancel();
    }

    #[test]
    fn commit_validates_then_yields_commit_with_ops_and_version() {
        let node = NodeId::from_raw(2);
        let mut provisional = ProvisionalTransaction::new();
        provisional
            .stage(UiOperation::ReplaceRoot)
            .stage(UiOperation::Rerender { node });
        let version = VersionStamp::from_raw(7);

        let commit = provisional
            .commit(&TransactionValidator::new(), version)
            .expect("valid batch should commit");

        assert_eq!(
            commit.operations(),
            &[UiOperation::ReplaceRoot, UiOperation::Rerender { node }]
        );
        assert_eq!(commit.version(), version);
    }

    #[test]
    fn commit_of_invalid_batch_returns_error() {
        let mut provisional = ProvisionalTransaction::new();
        provisional
            .stage(UiOperation::ReplaceRoot)
            .stage(UiOperation::ReplaceRoot);

        let result = provisional.commit(&TransactionValidator::new(), VersionStamp::from_raw(1));
        assert_eq!(result, Err(TransactionError::ConflictingRootReplacement));
    }

    #[test]
    fn begin_wraps_an_existing_transaction() {
        let mut tx = UiTransaction::new();
        tx.push(UiOperation::ReplaceRoot);
        let provisional = ProvisionalTransaction::begin(tx);
        assert!(provisional.transaction().is_root_replacement());
    }
}
