//! The [`Selection`] holding the set of selected nodes and the range anchor
//! (item-selection model).

use std::collections::BTreeSet;

use obscur_identity::NodeId;

use super::SelectionAnchor;

#[doc = include_str!("../../docs/api/selection/selection.doc/Selection.md")]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Selection {
    items: BTreeSet<NodeId>,
    anchor: Option<SelectionAnchor>,
}

impl Selection {
    /// Creates an empty selection with no anchor.
    #[must_use]
    pub fn new() -> Self {
        Self {
            items: BTreeSet::new(),
            anchor: None,
        }
    }

    #[doc = include_str!("../../docs/api/selection/selection.doc/select.md")]
    pub fn select(&mut self, node: NodeId) {
        self.items.clear();
        self.items.insert(node);
        self.anchor = Some(SelectionAnchor::collapsed(node));
    }

    #[doc = include_str!("../../docs/api/selection/selection.doc/add.md")]
    pub fn add(&mut self, node: NodeId) {
        self.items.insert(node);
        self.anchor = Some(match self.anchor {
            Some(existing) => SelectionAnchor::new(existing.anchor(), node),
            None => SelectionAnchor::collapsed(node),
        });
    }

    #[doc = include_str!("../../docs/api/selection/selection.doc/toggle.md")]
    pub fn toggle(&mut self, node: NodeId) -> bool {
        if self.items.contains(&node) {
            self.remove(node);
            false
        } else {
            self.add(node);
            true
        }
    }

    #[doc = include_str!("../../docs/api/selection/selection.doc/remove.md")]
    pub fn remove(&mut self, node: NodeId) -> bool {
        let removed = self.items.remove(&node);
        if self.items.is_empty() {
            self.anchor = None;
        }
        removed
    }

    /// Clears the selection and the anchor.
    pub fn clear(&mut self) {
        self.items.clear();
        self.anchor = None;
    }

    /// Returns whether `node` is currently selected.
    #[must_use]
    pub fn contains(&self, node: NodeId) -> bool {
        self.items.contains(&node)
    }

    /// Returns the selected items in ascending [`NodeId`] order.
    pub fn items(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.items.iter().copied()
    }

    /// Returns the number of selected items.
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns whether nothing is selected.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the current range anchor, if any.
    #[must_use]
    pub fn anchor(&self) -> Option<SelectionAnchor> {
        self.anchor
    }

    /// Returns the current lead item (the anchor's lead), if any.
    #[must_use]
    pub fn lead(&self) -> Option<NodeId> {
        self.anchor.map(|a| a.lead())
    }

    #[doc = include_str!("../../docs/api/selection/selection.doc/set_range.md")]
    pub fn set_range(&mut self, range: &[NodeId]) {
        self.items.clear();
        self.items.extend(range.iter().copied());
        self.anchor = match (range.first(), range.last()) {
            (Some(&first), Some(&last)) => Some(SelectionAnchor::new(first, last)),
            _ => None,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(raw: u64) -> NodeId {
        NodeId::from_raw(raw)
    }

    #[test]
    fn new_is_empty() {
        let s = Selection::new();
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
        assert_eq!(s.anchor(), None);
        assert_eq!(s.lead(), None);
    }

    #[test]
    fn select_replaces_and_collapses_anchor() {
        let mut s = Selection::new();
        s.add(node(1));
        s.add(node(2));
        s.select(node(5));
        assert_eq!(s.items().collect::<Vec<_>>(), vec![node(5)]);
        assert_eq!(s.anchor(), Some(SelectionAnchor::collapsed(node(5))));
        assert_eq!(s.lead(), Some(node(5)));
    }

    #[test]
    fn add_inserts_and_moves_lead_keeping_anchor() {
        let mut s = Selection::new();
        s.select(node(1));
        s.add(node(3));
        assert_eq!(s.items().collect::<Vec<_>>(), vec![node(1), node(3)]);
        assert_eq!(s.anchor(), Some(SelectionAnchor::new(node(1), node(3))));
        assert_eq!(s.lead(), Some(node(3)));
    }

    #[test]
    fn toggle_returns_new_membership() {
        let mut s = Selection::new();
        assert!(s.toggle(node(2))); // added
        assert!(s.contains(node(2)));
        assert!(!s.toggle(node(2))); // removed
        assert!(!s.contains(node(2)));
    }

    #[test]
    fn remove_reports_presence_and_clears_anchor_when_empty() {
        let mut s = Selection::new();
        s.select(node(1));
        assert!(s.remove(node(1)));
        assert!(!s.remove(node(1)));
        assert_eq!(s.anchor(), None);
    }

    #[test]
    fn set_range_replaces_and_spans_anchor() {
        let mut s = Selection::new();
        s.select(node(9));
        s.set_range(&[node(2), node(3), node(4)]);
        assert_eq!(
            s.items().collect::<Vec<_>>(),
            vec![node(2), node(3), node(4)]
        );
        assert_eq!(s.anchor(), Some(SelectionAnchor::new(node(2), node(4))));
    }

    #[test]
    fn set_range_empty_clears() {
        let mut s = Selection::new();
        s.select(node(1));
        s.set_range(&[]);
        assert!(s.is_empty());
        assert_eq!(s.anchor(), None);
    }

    #[test]
    fn items_iterate_in_ascending_order() {
        let mut s = Selection::new();
        s.add(node(3));
        s.add(node(1));
        s.add(node(2));
        assert_eq!(
            s.items().collect::<Vec<_>>(),
            vec![node(1), node(2), node(3)]
        );
    }
}
