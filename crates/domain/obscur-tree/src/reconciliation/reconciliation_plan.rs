//! `ReconciliationPlan`: the ordered op log produced by a reconcile pass.

use super::ReconciliationOp;

/// An ordered record of the [`ReconciliationOp`]s produced by a reconcile pass.
///
/// The plan is the op log a [`Reconciler`](super::Reconciler) emits while
/// transforming the retained tree from a desired tree: a flat, ordered sequence
/// of the inserts, removes, moves, and in-place updates it performed. It exists
/// for diagnostics, tests, and — later — observers.
///
/// The transaction sub-step will wrap a reconcile pass in atomic `UiOps` commit
/// boundaries; here the plan is simply the produced op log, with no transactional
/// guarantees of its own.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReconciliationPlan {
    ops: Vec<ReconciliationOp>,
}

impl ReconciliationPlan {
    /// Creates an empty plan.
    #[must_use]
    pub const fn new() -> Self {
        Self { ops: Vec::new() }
    }

    /// Appends an op to the end of the log, preserving order.
    pub fn push(&mut self, op: ReconciliationOp) {
        self.ops.push(op);
    }

    /// Returns the recorded ops in order.
    #[must_use]
    pub fn ops(&self) -> &[ReconciliationOp] {
        &self.ops
    }

    /// Returns an iterator over the recorded ops in order.
    pub fn iter(&self) -> core::slice::Iter<'_, ReconciliationOp> {
        self.ops.iter()
    }

    /// Returns the number of recorded ops.
    #[must_use]
    pub fn len(&self) -> usize {
        self.ops.len()
    }

    /// Returns `true` if no ops were recorded (a no-op reconcile pass).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }
}

impl<'a> IntoIterator for &'a ReconciliationPlan {
    type Item = &'a ReconciliationOp;
    type IntoIter = core::slice::Iter<'a, ReconciliationOp>;

    fn into_iter(self) -> Self::IntoIter {
        self.ops.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::NodeId;

    #[test]
    fn new_plan_is_empty() {
        let plan = ReconciliationPlan::new();
        assert!(plan.is_empty());
        assert_eq!(plan.len(), 0);
        assert!(plan.ops().is_empty());
    }

    #[test]
    fn push_records_in_order() {
        let mut plan = ReconciliationPlan::new();
        let a = ReconciliationOp::Remove {
            id: NodeId::from_raw(1),
        };
        let b = ReconciliationOp::Update {
            id: NodeId::from_raw(2),
        };
        plan.push(a);
        plan.push(b);
        assert_eq!(plan.len(), 2);
        assert_eq!(plan.ops(), &[a, b]);
        let collected: Vec<_> = plan.iter().copied().collect();
        assert_eq!(collected, vec![a, b]);
    }
}
