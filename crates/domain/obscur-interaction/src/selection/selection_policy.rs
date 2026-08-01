#![doc = include_str!("selection_policy.doc/_module.md")]

use obscur_identity::NodeId;

use super::{Selection, SelectionMode};

#[doc = include_str!("selection_policy.doc/SelectionPolicy.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectionPolicy {
    mode: SelectionMode,
    allow_empty: bool,
    toggle_on_click: bool,
}

impl SelectionPolicy {
    #[doc = include_str!("selection_policy.doc/new.md")]
    #[must_use]
    pub const fn new(mode: SelectionMode) -> Self {
        Self {
            mode,
            allow_empty: true,
            toggle_on_click: false,
        }
    }

    #[doc = include_str!("selection_policy.doc/allow_empty.md")]
    #[must_use]
    pub const fn allow_empty(mut self, allow_empty: bool) -> Self {
        self.allow_empty = allow_empty;
        self
    }

    #[doc = include_str!("selection_policy.doc/toggle_on_click.md")]
    #[must_use]
    pub const fn toggle_on_click(mut self, toggle_on_click: bool) -> Self {
        self.toggle_on_click = toggle_on_click;
        self
    }

    #[doc = include_str!("selection_policy.doc/mode.md")]
    #[must_use]
    pub const fn mode(&self) -> SelectionMode {
        self.mode
    }

    #[doc = include_str!("selection_policy.doc/allows_empty.md")]
    #[must_use]
    pub const fn allows_empty(&self) -> bool {
        self.allow_empty
    }

    #[doc = include_str!("selection_policy.doc/toggles_on_click.md")]
    #[must_use]
    pub const fn toggles_on_click(&self) -> bool {
        self.toggle_on_click
    }

    #[doc = include_str!("selection_policy.doc/apply_click.md")]
    pub fn apply_click(&self, selection: &mut Selection, node: NodeId, toggle_modifier: bool) {
        match self.mode {
            SelectionMode::None => {}
            SelectionMode::Single => selection.select(node),
            SelectionMode::Multiple => {
                if toggle_modifier || self.toggle_on_click {
                    let would_empty =
                        selection.contains(node) && selection.len() == 1 && !self.allow_empty;
                    if !would_empty {
                        selection.toggle(node);
                    }
                } else {
                    selection.select(node);
                }
            }
        }
    }

    #[doc = include_str!("selection_policy.doc/apply_range.md")]
    pub fn apply_range(&self, selection: &mut Selection, range: &[NodeId]) {
        if self.mode.allows_multiple() {
            selection.set_range(range);
        } else if self.mode.is_enabled()
            && let Some(&last) = range.last()
        {
            selection.select(last);
        }
    }
}

impl Default for SelectionPolicy {
    #[doc = include_str!("selection_policy.doc/default.md")]
    fn default() -> Self {
        Self::new(SelectionMode::Single)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(raw: u64) -> NodeId {
        NodeId::from_raw(raw)
    }

    #[test]
    fn default_is_single_allow_empty() {
        let p = SelectionPolicy::default();
        assert_eq!(p.mode(), SelectionMode::Single);
        assert!(p.allows_empty());
        assert!(!p.toggles_on_click());
    }

    #[test]
    fn apply_click_none_is_no_op() {
        let p = SelectionPolicy::new(SelectionMode::None);
        let mut s = Selection::new();
        p.apply_click(&mut s, node(1), false);
        assert!(s.is_empty());
    }

    #[test]
    fn apply_click_single_replaces() {
        let p = SelectionPolicy::new(SelectionMode::Single);
        let mut s = Selection::new();
        p.apply_click(&mut s, node(1), false);
        p.apply_click(&mut s, node(2), false);
        assert_eq!(s.items().collect::<Vec<_>>(), vec![node(2)]);
    }

    #[test]
    fn apply_click_multiple_replaces_without_toggle() {
        let p = SelectionPolicy::new(SelectionMode::Multiple);
        let mut s = Selection::new();
        p.apply_click(&mut s, node(1), false);
        p.apply_click(&mut s, node(2), false);
        assert_eq!(s.items().collect::<Vec<_>>(), vec![node(2)]);
    }

    #[test]
    fn apply_click_multiple_toggles_with_modifier() {
        let p = SelectionPolicy::new(SelectionMode::Multiple);
        let mut s = Selection::new();
        p.apply_click(&mut s, node(1), true);
        p.apply_click(&mut s, node(2), true);
        assert_eq!(s.items().collect::<Vec<_>>(), vec![node(1), node(2)]);
        // Toggling an already-selected item removes it.
        p.apply_click(&mut s, node(1), true);
        assert_eq!(s.items().collect::<Vec<_>>(), vec![node(2)]);
    }

    #[test]
    fn apply_click_multiple_toggles_when_toggle_on_click() {
        let p = SelectionPolicy::new(SelectionMode::Multiple).toggle_on_click(true);
        let mut s = Selection::new();
        p.apply_click(&mut s, node(1), false);
        p.apply_click(&mut s, node(2), false);
        assert_eq!(s.items().collect::<Vec<_>>(), vec![node(1), node(2)]);
    }

    #[test]
    fn apply_click_allow_empty_false_keeps_last_item() {
        let p = SelectionPolicy::new(SelectionMode::Multiple)
            .toggle_on_click(true)
            .allow_empty(false);
        let mut s = Selection::new();
        p.apply_click(&mut s, node(1), false);
        // Toggling off the only item is suppressed.
        p.apply_click(&mut s, node(1), false);
        assert_eq!(s.items().collect::<Vec<_>>(), vec![node(1)]);
    }

    #[test]
    fn apply_range_multiple_replaces() {
        let p = SelectionPolicy::new(SelectionMode::Multiple);
        let mut s = Selection::new();
        p.apply_range(&mut s, &[node(1), node(2), node(3)]);
        assert_eq!(
            s.items().collect::<Vec<_>>(),
            vec![node(1), node(2), node(3)]
        );
    }

    #[test]
    fn apply_range_single_collapses_to_last() {
        let p = SelectionPolicy::new(SelectionMode::Single);
        let mut s = Selection::new();
        p.apply_range(&mut s, &[node(1), node(2), node(3)]);
        assert_eq!(s.items().collect::<Vec<_>>(), vec![node(3)]);
    }

    #[test]
    fn apply_range_none_is_no_op() {
        let p = SelectionPolicy::new(SelectionMode::None);
        let mut s = Selection::new();
        p.apply_range(&mut s, &[node(1), node(2)]);
        assert!(s.is_empty());
    }
}
