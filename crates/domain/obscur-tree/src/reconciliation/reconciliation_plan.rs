//! `ReconciliationPlan`: the ordered op log produced by a reconcile pass.

use obscur_identity::{NodeId, StableKey};

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
    /// Sibling groups whose desired children reused one key (spec §2.4 keyed
    /// identity). Recorded rather than reported: this crate is domain code and
    /// performs no I/O, so the runtime raises the diagnostic.
    duplicate_keys: Vec<DuplicateKey>,
    ops: Vec<ReconciliationOp>,
}

impl ReconciliationPlan {
    /// Creates an empty plan.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            ops: Vec::new(),
            duplicate_keys: Vec::new(),
        }
    }

    /// Appends an op to the end of the log, preserving order.
    pub fn push(&mut self, op: ReconciliationOp) {
        self.ops.push(op);
    }

    /// Returns the recorded ops in order.
    #[must_use]
    /// Records a sibling group that reused `key` `count` times under `parent`.
    pub fn push_duplicate_key(&mut self, parent: NodeId, key: StableKey, count: usize) {
        self.duplicate_keys
            .push(DuplicateKey { parent, key, count });
    }

    /// The duplicate-key collisions this reconcile saw (spec §2.4).
    #[must_use]
    pub fn duplicate_keys(&self) -> &[DuplicateKey] {
        &self.duplicate_keys
    }

    /// The recorded operations, in the order the reconcile produced them.
    ///
    /// Order is part of the contract: an op may refer to a node an earlier op
    /// created, so applying them out of sequence is not the same plan.
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
            parent: None,
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

/// One sibling group that gave the same key to more than one desired child.
///
/// Keyed identity assumes a key names **one** child (spec §2.4). When several share
/// it, the matcher's key index keeps the first and the rest find no partner: they are
/// inserted as new while the previous ones are removed, every frame, which makes the
/// whole group churn — a full restyle and a full layout each time. It is silent
/// otherwise, so the runtime turns this into a diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateKey {
    /// The parent whose children collided.
    pub parent: NodeId,
    /// The key more than one child claimed.
    pub key: StableKey,
    /// How many children claimed it.
    pub count: usize,
}
