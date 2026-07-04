//! The [`SelectionPolicy`] rules that govern how input mutates a
//! [`Selection`](super::Selection) (item-selection model).

use obscur_identity::NodeId;

use super::{Selection, SelectionMode};

/// The rules that translate a click or range gesture into mutations of a
/// [`Selection`](super::Selection).
///
/// A policy pairs a [`SelectionMode`] with two flags:
///
/// - `allow_empty` — when `false`, at least one item must stay selected, so a
///   toggle that would empty the set is suppressed.
/// - `toggle_on_click` — when `true`, a plain click toggles (Ctrl-like) rather
///   than replacing the selection; a modifier can request the same behaviour
///   per-click.
///
/// The policy is pure: [`apply_click`](Self::apply_click) and
/// [`apply_range`](Self::apply_range) mutate the passed-in selection and read no
/// external state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectionPolicy {
    mode: SelectionMode,
    allow_empty: bool,
    toggle_on_click: bool,
}

impl SelectionPolicy {
    /// Creates a policy for `mode` with `allow_empty` on and `toggle_on_click`
    /// off.
    #[must_use]
    pub const fn new(mode: SelectionMode) -> Self {
        Self {
            mode,
            allow_empty: true,
            toggle_on_click: false,
        }
    }

    /// Returns this policy with `allow_empty` set (fluent).
    #[must_use]
    pub const fn allow_empty(mut self, allow_empty: bool) -> Self {
        self.allow_empty = allow_empty;
        self
    }

    /// Returns this policy with `toggle_on_click` set (fluent).
    #[must_use]
    pub const fn toggle_on_click(mut self, toggle_on_click: bool) -> Self {
        self.toggle_on_click = toggle_on_click;
        self
    }

    /// Returns the selection mode.
    #[must_use]
    pub const fn mode(&self) -> SelectionMode {
        self.mode
    }

    /// Returns whether an empty selection is permitted.
    #[must_use]
    pub const fn allows_empty(&self) -> bool {
        self.allow_empty
    }

    /// Returns whether a plain click toggles rather than replaces.
    #[must_use]
    pub const fn toggles_on_click(&self) -> bool {
        self.toggle_on_click
    }

    /// Applies a click on `node` to `selection`.
    ///
    /// `toggle_modifier` is `true` when the click carried a toggle modifier
    /// (Ctrl/Cmd). The behaviour depends on the mode:
    ///
    /// - [`None`](SelectionMode::None): selection is disabled, so this is a
    ///   no-op.
    /// - [`Single`](SelectionMode::Single): replaces the selection with `node`.
    /// - [`Multiple`](SelectionMode::Multiple): if `toggle_modifier` or
    ///   [`toggle_on_click`](Self::toggles_on_click) is set, toggles `node`;
    ///   otherwise replaces the selection with `node`. When `allow_empty` is
    ///   `false`, a toggle that would remove the only selected item is
    ///   suppressed so at least one item stays selected.
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

    /// Applies a range gesture over `range` to `selection`.
    ///
    /// - When the mode [`allows_multiple`](SelectionMode::allows_multiple),
    ///   replaces the selection with `range`.
    /// - For [`Single`](SelectionMode::Single), a range collapses to its last
    ///   item (selected on its own).
    /// - For [`None`](SelectionMode::None), selection is disabled, so this is a
    ///   no-op.
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
    /// Defaults to [`Single`](SelectionMode::Single) with `allow_empty` on and
    /// `toggle_on_click` off.
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
