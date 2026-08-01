#![doc = include_str!("reconciliation_plan.doc/_module.md")]

use obscur_identity::{NodeId, StableKey};

use super::ReconciliationOp;

#[doc = include_str!("reconciliation_plan.doc/ReconciliationPlan.md")]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReconciliationPlan {
    #[doc = include_str!("reconciliation_plan.doc/duplicate_keys.md")]
    duplicate_keys: Vec<DuplicateKey>,
    ops: Vec<ReconciliationOp>,
}

impl ReconciliationPlan {
    #[doc = include_str!("reconciliation_plan.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            ops: Vec::new(),
            duplicate_keys: Vec::new(),
        }
    }

    #[doc = include_str!("reconciliation_plan.doc/push.md")]
    pub fn push(&mut self, op: ReconciliationOp) {
        self.ops.push(op);
    }

    #[doc = include_str!("reconciliation_plan.doc/push_duplicate_key.md")]
    #[must_use]
    #[doc = include_str!("reconciliation_plan.doc/push_duplicate_key_2.md")]
    pub fn push_duplicate_key(&mut self, parent: NodeId, key: StableKey, count: usize) {
        self.duplicate_keys
            .push(DuplicateKey { parent, key, count });
    }

    #[doc = include_str!("reconciliation_plan.doc/duplicate_keys_2.md")]
    #[must_use]
    pub fn duplicate_keys(&self) -> &[DuplicateKey] {
        &self.duplicate_keys
    }

    #[doc = include_str!("reconciliation_plan.doc/ops.md")]
    #[must_use]
    pub fn ops(&self) -> &[ReconciliationOp] {
        &self.ops
    }

    #[doc = include_str!("reconciliation_plan.doc/iter.md")]
    pub fn iter(&self) -> core::slice::Iter<'_, ReconciliationOp> {
        self.ops.iter()
    }

    #[doc = include_str!("reconciliation_plan.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.ops.len()
    }

    #[doc = include_str!("reconciliation_plan.doc/is_empty.md")]
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
