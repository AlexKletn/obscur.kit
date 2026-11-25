use obscur_data_transfer::drag::DragEffect;
use obscur_identity::NodeId;

use crate::transaction::TransactionCommit;

#[doc = include_str!("../../docs/api/orchestration/drag_coordinator.doc/DragOutcome.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DragOutcome {
    /// The drop landed on `target`, and `commit` is the work it caused —
    /// already validated, ready for
    /// [`apply_transaction`](crate::runtime::UiRuntime::apply_transaction).
    Dropped {
        /// The node that accepted the drop.
        target: NodeId,
        /// The negotiated effect (`Copy` / `Move` / …).
        effect: DragEffect,
        /// The invalidation the drop caused, as one atomic transaction.
        commit: TransactionCommit,
    },
    /// The drag ended without a valid drop: released over nothing, over a target
    /// that refused the payload, or abandoned. **Nothing was applied.**
    Cancelled,
}
