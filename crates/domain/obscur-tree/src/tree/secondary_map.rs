//! A projection map keyed by `NodeId` that augments — never replaces — identity.

use std::collections::HashMap;

use obscur_identity::NodeId;

/// A map from [`NodeId`] to a projection value `V`.
///
/// Heavy per-node projections — layout boxes, paint data, accessibility nodes,
/// backend handles — live in secondary maps keyed by the node's stable
/// [`NodeId`] (spec §1.5). A secondary map **augments** identity: the `NodeId`
/// remains owned and minted by the
/// [`RetainedTree`](super::RetainedTree), and this map merely attaches derived
/// data to it. It never becomes an alternate identity of its own.
///
/// This Phase-1 implementation is backed by a
/// [`HashMap`]. A dense generational-vector variant (indexing directly by slot,
/// like the arena) is a later optimization behind this same API.
#[derive(Debug, Clone)]
pub struct SecondaryMap<V> {
    entries: HashMap<NodeId, V>,
}

impl<V> SecondaryMap<V> {
    /// Creates an empty secondary map.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Inserts `value` for `id`, returning the previous value if one was present.
    pub fn insert(&mut self, id: NodeId, value: V) -> Option<V> {
        self.entries.insert(id, value)
    }

    /// Returns a shared reference to the value attached to `id`, if any.
    #[must_use]
    pub fn get(&self, id: NodeId) -> Option<&V> {
        self.entries.get(&id)
    }

    /// Returns a mutable reference to the value attached to `id`, if any.
    #[must_use]
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut V> {
        self.entries.get_mut(&id)
    }

    /// Removes and returns the value attached to `id`, if any.
    pub fn remove(&mut self, id: NodeId) -> Option<V> {
        self.entries.remove(&id)
    }

    /// Returns `true` if a value is attached to `id`.
    #[must_use]
    pub fn contains(&self, id: NodeId) -> bool {
        self.entries.contains_key(&id)
    }

    /// Returns the number of attached values.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if no values are attached.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterates over every `(NodeId, &V)` pair, in unspecified order.
    #[must_use]
    pub fn iter(&self) -> impl Iterator<Item = (NodeId, &V)> {
        self.entries.iter().map(|(id, value)| (*id, value))
    }
}

impl<V> Default for SecondaryMap<V> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(raw: u64) -> NodeId {
        NodeId::from_raw(raw)
    }

    #[test]
    fn insert_get_remove() {
        let mut map: SecondaryMap<u32> = SecondaryMap::new();
        assert!(map.is_empty());
        assert_eq!(map.insert(id(1), 10), None);
        assert_eq!(map.insert(id(1), 11), Some(10));
        assert_eq!(map.get(id(1)), Some(&11));
        assert!(map.contains(id(1)));
        assert_eq!(map.len(), 1);
        assert_eq!(map.remove(id(1)), Some(11));
        assert!(!map.contains(id(1)));
        assert!(map.is_empty());
    }

    #[test]
    fn get_mut_updates_in_place() {
        let mut map: SecondaryMap<u32> = SecondaryMap::default();
        map.insert(id(2), 5);
        if let Some(v) = map.get_mut(id(2)) {
            *v += 1;
        }
        assert_eq!(map.get(id(2)), Some(&6));
    }

    #[test]
    fn iter_visits_all_entries() {
        let mut map: SecondaryMap<u32> = SecondaryMap::new();
        map.insert(id(1), 1);
        map.insert(id(2), 2);
        let mut collected: Vec<(NodeId, u32)> = map.iter().map(|(k, v)| (k, *v)).collect();
        collected.sort_by_key(|(k, _)| k.raw());
        assert_eq!(collected, vec![(id(1), 1), (id(2), 2)]);
    }
}
