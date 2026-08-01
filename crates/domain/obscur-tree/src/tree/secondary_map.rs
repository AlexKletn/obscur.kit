#![doc = include_str!("secondary_map.doc/_module.md")]

use std::collections::HashMap;

use obscur_identity::NodeId;

#[doc = include_str!("secondary_map.doc/SecondaryMap.md")]
#[derive(Debug, Clone)]
pub struct SecondaryMap<V> {
    entries: HashMap<NodeId, V>,
}

impl<V> SecondaryMap<V> {
    #[doc = include_str!("secondary_map.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    #[doc = include_str!("secondary_map.doc/insert.md")]
    pub fn insert(&mut self, id: NodeId, value: V) -> Option<V> {
        self.entries.insert(id, value)
    }

    #[doc = include_str!("secondary_map.doc/get.md")]
    #[must_use]
    pub fn get(&self, id: NodeId) -> Option<&V> {
        self.entries.get(&id)
    }

    #[doc = include_str!("secondary_map.doc/get_mut.md")]
    #[must_use]
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut V> {
        self.entries.get_mut(&id)
    }

    #[doc = include_str!("secondary_map.doc/remove.md")]
    pub fn remove(&mut self, id: NodeId) -> Option<V> {
        self.entries.remove(&id)
    }

    #[doc = include_str!("secondary_map.doc/contains.md")]
    #[must_use]
    pub fn contains(&self, id: NodeId) -> bool {
        self.entries.contains_key(&id)
    }

    #[doc = include_str!("secondary_map.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[doc = include_str!("secondary_map.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[doc = include_str!("secondary_map.doc/iter.md")]
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
