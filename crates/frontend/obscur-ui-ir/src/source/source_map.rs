//! Maps IR node ids to their originating source spans [`SourceMap`] (spec §2.5
//! source maps).

use std::collections::BTreeMap;

use crate::source::SourceReference;

/// A map from IR node ids to the [`SourceReference`]s they originated from
/// (spec §2.5).
///
/// Diagnostics raised against the IR carry a node id; the source map resolves
/// that id back to a span in the original source (for example the `.okui` file)
/// so errors can point at the code the author wrote.
///
/// Node ids are stored as raw `u64` keys. The document module is what assigns
/// and interprets these ids; keeping them as plain integers here lets the source
/// map stay independent of the document types (no dependency cycle) while still
/// keying on the same value. A [`BTreeMap`] gives deterministic iteration order,
/// which keeps serialized source maps and diagnostics stable across runs.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SourceMap {
    entries: BTreeMap<u64, SourceReference>,
}

impl SourceMap {
    /// Creates an empty source map.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    /// Inserts the source `reference` for `node_id`, returning any reference it
    /// replaced.
    pub fn insert(&mut self, node_id: u64, reference: SourceReference) -> Option<SourceReference> {
        self.entries.insert(node_id, reference)
    }

    /// Returns the source reference recorded for `node_id`, if any.
    #[must_use]
    pub fn get(&self, node_id: u64) -> Option<&SourceReference> {
        self.entries.get(&node_id)
    }

    /// Returns whether a reference is recorded for `node_id`.
    #[must_use]
    pub fn contains(&self, node_id: u64) -> bool {
        self.entries.contains_key(&node_id)
    }

    /// Removes the reference for `node_id`, returning whether one was present.
    pub fn remove(&mut self, node_id: u64) -> bool {
        self.entries.remove(&node_id).is_some()
    }

    /// Returns the number of recorded references.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether the map is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterates over the `(node_id, reference)` pairs in ascending node-id
    /// order (deterministic [`BTreeMap`] order).
    pub fn iter(&self) -> impl Iterator<Item = (u64, &SourceReference)> {
        self.entries.iter().map(|(&id, reference)| (id, reference))
    }

    /// Removes all recorded references.
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::SourceLocation;

    fn reference(name: &str) -> SourceReference {
        SourceReference::point(name, SourceLocation::start())
    }

    #[test]
    fn insert_returns_replaced() {
        let mut map = SourceMap::new();
        assert!(map.insert(1, reference("a")).is_none());
        let replaced = map.insert(1, reference("b"));
        assert_eq!(replaced, Some(reference("a")));
        assert_eq!(map.get(1), Some(&reference("b")));
    }

    #[test]
    fn get_and_contains() {
        let mut map = SourceMap::new();
        map.insert(7, reference("x"));
        assert!(map.contains(7));
        assert_eq!(map.get(7), Some(&reference("x")));
        assert!(!map.contains(8));
        assert_eq!(map.get(8), None);
    }

    #[test]
    fn remove_reports_presence() {
        let mut map = SourceMap::new();
        map.insert(3, reference("y"));
        assert!(map.remove(3));
        assert!(!map.remove(3));
        assert!(map.is_empty());
    }

    #[test]
    fn len_and_clear() {
        let mut map = SourceMap::new();
        assert!(map.is_empty());
        map.insert(1, reference("a"));
        map.insert(2, reference("b"));
        assert_eq!(map.len(), 2);
        map.clear();
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
    }

    #[test]
    fn iter_is_in_ascending_node_id_order() {
        let mut map = SourceMap::new();
        map.insert(30, reference("c"));
        map.insert(10, reference("a"));
        map.insert(20, reference("b"));
        let ids: Vec<u64> = map.iter().map(|(id, _)| id).collect();
        assert_eq!(ids, vec![10, 20, 30]);
    }

    #[test]
    fn default_is_empty() {
        assert_eq!(SourceMap::default(), SourceMap::new());
    }
}
