#![doc = include_str!("style_invalidation.doc/_module.md")]

use crate::computed::StyleDiff;
use crate::invalidation::{StyleDependency, StyleInvalidationReason};
use crate::selector::PseudoState;
use obscur_identity::NodeId;
use std::collections::HashMap;
use std::collections::hash_map::Iter;

#[doc = include_str!("style_invalidation.doc/StyleInvalidation.md")]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StyleInvalidation {
    pending: HashMap<NodeId, StyleInvalidationReason>,
}

impl StyleInvalidation {
    #[doc = include_str!("style_invalidation.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            pending: HashMap::new(),
        }
    }

    #[doc = include_str!("style_invalidation.doc/invalidate.md")]
    pub fn invalidate(&mut self, node: NodeId, reason: StyleInvalidationReason) {
        self.pending.entry(node).or_insert(reason);
    }

    #[doc = include_str!("style_invalidation.doc/is_invalidated.md")]
    #[must_use]
    pub fn is_invalidated(&self, node: NodeId) -> bool {
        self.pending.contains_key(&node)
    }

    #[doc = include_str!("style_invalidation.doc/reason.md")]
    #[must_use]
    pub fn reason(&self, node: NodeId) -> Option<StyleInvalidationReason> {
        self.pending.get(&node).copied()
    }

    #[doc = include_str!("style_invalidation.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.pending.len()
    }

    #[doc = include_str!("style_invalidation.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    #[doc = include_str!("style_invalidation.doc/iter.md")]
    #[must_use]
    pub fn iter(&self) -> Iter<'_, NodeId, StyleInvalidationReason> {
        self.pending.iter()
    }

    #[doc = include_str!("style_invalidation.doc/take.md")]
    #[must_use]
    pub fn take(&mut self) -> HashMap<NodeId, StyleInvalidationReason> {
        std::mem::take(&mut self.pending)
    }

    #[doc = include_str!("style_invalidation.doc/drain.md")]
    #[must_use]
    pub fn drain(&mut self) -> HashMap<NodeId, StyleInvalidationReason> {
        self.take()
    }

    #[doc = include_str!("style_invalidation.doc/note_parent_style_changed.md")]
    pub fn note_parent_style_changed(
        &mut self,
        child: NodeId,
        dep: &StyleDependency,
        parent_diff: &StyleDiff,
    ) -> bool {
        if let Some(reason) = dep.affected_by_parent_change(parent_diff) {
            self.invalidate(child, reason);
            true
        } else {
            false
        }
    }

    #[doc = include_str!("style_invalidation.doc/note_pseudo_state_changed.md")]
    pub fn note_pseudo_state_changed(
        &mut self,
        node: NodeId,
        dep: &StyleDependency,
        changed: PseudoState,
    ) -> bool {
        if dep.affected_by_pseudo_change(changed) {
            self.invalidate(node, StyleInvalidationReason::PseudoStateChanged);
            true
        } else {
            false
        }
    }
}

impl<'a> IntoIterator for &'a StyleInvalidation {
    type Item = (&'a NodeId, &'a StyleInvalidationReason);
    type IntoIter = Iter<'a, NodeId, StyleInvalidationReason>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::computed::{ComputedStyle, ComputedValue};
    use crate::property::PropertyId;

    fn node(raw: u64) -> NodeId {
        NodeId::from_raw(raw)
    }

    #[test]
    fn records_and_reports() {
        let mut inv = StyleInvalidation::new();
        assert!(inv.is_empty());
        inv.invalidate(node(1), StyleInvalidationReason::StylesheetChanged);
        assert!(inv.is_invalidated(node(1)));
        assert_eq!(
            inv.reason(node(1)),
            Some(StyleInvalidationReason::StylesheetChanged)
        );
        assert_eq!(inv.len(), 1);
        assert!(!inv.is_invalidated(node(2)));
        assert_eq!(inv.reason(node(2)), None);
    }

    #[test]
    fn merge_keeps_first_reason() {
        let mut inv = StyleInvalidation::new();
        inv.invalidate(node(1), StyleInvalidationReason::StylesheetChanged);
        inv.invalidate(node(1), StyleInvalidationReason::PseudoStateChanged);
        // Earliest reason wins, and the node is still recorded exactly once.
        assert_eq!(inv.len(), 1);
        assert_eq!(
            inv.reason(node(1)),
            Some(StyleInvalidationReason::StylesheetChanged)
        );
    }

    #[test]
    fn drain_empties() {
        let mut inv = StyleInvalidation::new();
        inv.invalidate(node(1), StyleInvalidationReason::DeclarationsChanged);
        inv.invalidate(node(2), StyleInvalidationReason::InlineStyleChanged);
        let drained = inv.drain();
        assert_eq!(drained.len(), 2);
        assert!(inv.is_empty());
        assert_eq!(inv.len(), 0);
    }

    #[test]
    fn note_parent_style_changed_gates_on_dependency() {
        // Parent changed the inherited `color` property.
        let mut old = ComputedStyle::new();
        old.set(PropertyId::Color, ComputedValue::number(0.0));
        let mut new = ComputedStyle::new();
        new.set(PropertyId::Color, ComputedValue::number(1.0));
        let diff = StyleDiff::between(&old, &new);

        let inheriting = StyleDependency::new().with_inherits_from_parent(true);
        let non_inheriting = StyleDependency::new().with_inherits_from_parent(false);

        let mut inv = StyleInvalidation::new();
        assert!(inv.note_parent_style_changed(node(1), &inheriting, &diff));
        assert!(!inv.note_parent_style_changed(node(2), &non_inheriting, &diff));

        assert_eq!(
            inv.reason(node(1)),
            Some(StyleInvalidationReason::ParentStyleChanged)
        );
        assert!(!inv.is_invalidated(node(2)));
    }

    #[test]
    fn note_pseudo_state_changed_gates_on_dependency() {
        let dep = StyleDependency::new().with_pseudo_states(PseudoState::HOVER);
        let mut inv = StyleInvalidation::new();
        assert!(inv.note_pseudo_state_changed(node(1), &dep, PseudoState::HOVER));
        assert!(!inv.note_pseudo_state_changed(node(2), &dep, PseudoState::ACTIVE));
        assert_eq!(
            inv.reason(node(1)),
            Some(StyleInvalidationReason::PseudoStateChanged)
        );
        assert!(!inv.is_invalidated(node(2)));
    }
}
