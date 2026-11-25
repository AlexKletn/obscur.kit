//! Internal graph of delegated grants.

use std::collections::{BTreeMap, BTreeSet};

use super::grant::Grant;

/// Tracks service grants and their delegation ancestry.
#[derive(Default)]
pub(crate) struct DelegationGraph(BTreeMap<u64, Grant>);

impl DelegationGraph {
    pub(super) fn insert(&mut self, id: u64, grant: Grant) {
        self.0.insert(id, grant);
    }

    pub(super) fn get(&self, id: u64) -> Option<&Grant> {
        self.0.get(&id)
    }

    pub(super) fn grants(&self) -> impl Iterator<Item = &Grant> {
        self.0.values()
    }

    pub(super) fn revoke_where(&mut self, mut matches: impl FnMut(u64, &Grant) -> bool) -> usize {
        let mut removed = BTreeSet::new();
        // A parent always has a smaller ID, so it is visited before its children.
        self.0.retain(|&id, grant| {
            if matches(id, grant) || grant.parent.is_some_and(|parent| removed.contains(&parent)) {
                removed.insert(id);
                false
            } else {
                true
            }
        });
        removed.len()
    }
}
