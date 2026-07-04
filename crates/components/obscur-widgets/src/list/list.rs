//! `List`: a headless selectable list (spec §2.6, §4.9).

use std::collections::BTreeSet;

use obscur_accessibility::semantic::SemanticRole;
use obscur_identity::{ComponentId, InternedSymbol};
use obscur_primitives::Text;
use obscur_view::view::{View, ViewFragment, ViewNode, ViewProperty, ViewValue};

/// A headless selectable list (spec §2.6, §4.9).
///
/// `List` owns its `items` (display labels), the set of currently
/// [`selected`](Self::selected_indices) item indices, and a
/// [`multi`](Self::is_multi) flag choosing between single- and multi-selection.
/// It bakes in **no** visual theme: [`view`](Self::view) derives a
/// [`SemanticRole::List`] element carrying the multi-select flag, with one
/// [`SemanticRole::ListItem`]-shaped child element per item (each carrying its
/// selected flag and a [`Text`] label child), rendered by the style layer keyed
/// on [`COMPONENT_ID`](Self::COMPONENT_ID).
///
/// The selection state machine covers [`select`](Self::select) (in single-select
/// mode this replaces the selection; in multi-select mode it adds to it),
/// [`toggle`](Self::toggle), [`deselect`](Self::deselect), and
/// [`clear_selection`](Self::clear_selection). Selection is stored by item index,
/// so it is stable and deterministic independent of any node identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct List {
    items: Vec<Box<str>>,
    selected: BTreeSet<usize>,
    multi: bool,
}

impl List {
    /// Stable component id for the list widget (see [`Button::COMPONENT_ID`] for
    /// the shared `0x00D0_00NN` scheme).
    ///
    /// [`Button::COMPONENT_ID`]: crate::button::Button::COMPONENT_ID
    pub const COMPONENT_ID: ComponentId = ComponentId::from_raw(0x00D0_0008);

    /// Stable component id for the per-item child element.
    const LIST_ITEM_ID: ComponentId = ComponentId::from_raw(0x00D0_0018);

    /// Interned name of the semantic-role property (`role`); shares the
    /// `0x00D1_NNXX` scheme with `NN = 08` (list): role=`00`, multi=`01`,
    /// selected=`02` (on item children).
    const PROP_ROLE: InternedSymbol = InternedSymbol::from_raw(0x00D1_0800);
    /// Interned name of the `multi` (multi-select) state property.
    const PROP_MULTI: InternedSymbol = InternedSymbol::from_raw(0x00D1_0801);
    /// Interned name of the per-item `selected` state property.
    const PROP_SELECTED: InternedSymbol = InternedSymbol::from_raw(0x00D1_0802);

    /// Creates an empty, single-select list with nothing selected.
    #[must_use]
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected: BTreeSet::new(),
            multi: false,
        }
    }

    /// Appends a single item, returning the updated list (fluent builder).
    #[must_use]
    pub fn item(mut self, label: impl Into<Box<str>>) -> Self {
        self.items.push(label.into());
        self
    }

    /// Appends every item from `labels`, returning the updated list.
    #[must_use]
    pub fn with_items<I, S>(mut self, labels: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<Box<str>>,
    {
        self.items.extend(labels.into_iter().map(Into::into));
        self
    }

    /// Sets the multi-select flag, returning the updated list (fluent builder).
    #[must_use]
    pub fn multi(mut self, multi: bool) -> Self {
        self.multi = multi;
        self
    }

    /// Selects the item at `index`, returning `true` on success. In
    /// single-select mode this replaces the selection with just `index`; in
    /// multi-select mode it adds `index` to the selection. Returns `false`
    /// (leaving state untouched) when `index` is out of range.
    pub fn select(&mut self, index: usize) -> bool {
        if index < self.items.len() {
            if !self.multi {
                self.selected.clear();
            }
            self.selected.insert(index);
            true
        } else {
            false
        }
    }

    /// Toggles the selected membership of the item at `index`, returning the new
    /// membership (`true` if now selected). In single-select mode toggling an
    /// item on still replaces any prior selection. Out-of-range indices leave
    /// state untouched and return `false`.
    pub fn toggle(&mut self, index: usize) -> bool {
        if index >= self.items.len() {
            return false;
        }
        if self.selected.contains(&index) {
            self.selected.remove(&index);
            false
        } else {
            if !self.multi {
                self.selected.clear();
            }
            self.selected.insert(index);
            true
        }
    }

    /// Deselects the item at `index`, returning `true` when it was previously
    /// selected. Out-of-range (or unselected) indices return `false`.
    pub fn deselect(&mut self, index: usize) -> bool {
        self.selected.remove(&index)
    }

    /// Clears the entire selection.
    pub fn clear_selection(&mut self) {
        self.selected.clear();
    }

    /// Returns whether the item at `index` is selected.
    #[must_use]
    pub fn is_selected(&self, index: usize) -> bool {
        self.selected.contains(&index)
    }

    /// Returns the selected item indices in ascending, deterministic order.
    pub fn selected_indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.selected.iter().copied()
    }

    /// Returns the number of selected items.
    #[must_use]
    pub fn selection_count(&self) -> usize {
        self.selected.len()
    }

    /// Returns whether the list is in multi-select mode.
    #[must_use]
    pub const fn is_multi(&self) -> bool {
        self.multi
    }

    /// Returns the number of items.
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns whether the list has no items.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the accessible role of this control ([`SemanticRole::List`]).
    #[must_use]
    pub const fn semantic_role(&self) -> SemanticRole {
        SemanticRole::List
    }

    /// Derives the semantic view: a [`COMPONENT_ID`] element carrying the role
    /// and multi-select flag, with one listitem-role child element per item
    /// (each carrying its selected flag and a [`Text`] label child) in order.
    ///
    /// [`COMPONENT_ID`]: Self::COMPONENT_ID
    #[must_use]
    pub fn view(&self) -> View {
        let children: Vec<ViewNode> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, label)| {
                ViewNode::element(Self::LIST_ITEM_ID)
                    .with_prop(ViewProperty::new(
                        Self::PROP_ROLE,
                        ViewValue::from("listitem"),
                    ))
                    .with_prop(ViewProperty::new(
                        Self::PROP_SELECTED,
                        ViewValue::Bool(self.is_selected(i)),
                    ))
                    .with_children(ViewFragment::from_nodes(vec![
                        Text::new(label.clone()).into_view_node(),
                    ]))
            })
            .collect();
        let node = ViewNode::element(Self::COMPONENT_ID)
            .with_prop(ViewProperty::new(Self::PROP_ROLE, ViewValue::from("list")))
            .with_prop(ViewProperty::new(Self::PROP_MULTI, ViewValue::Bool(self.multi)))
            .with_children(ViewFragment::from_nodes(children));
        View::single(node)
    }
}

impl Default for List {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> List {
        List::new().with_items(["Alpha", "Beta", "Gamma"])
    }

    #[test]
    fn item_builder_and_len() {
        let l = List::new().item("Only");
        assert_eq!(l.len(), 1);
        assert!(!l.is_empty());
    }

    #[test]
    fn single_select_replaces() {
        let mut l = sample();
        assert!(l.select(0));
        assert!(l.select(2));
        assert!(!l.is_selected(0));
        assert!(l.is_selected(2));
        assert_eq!(l.selection_count(), 1);
    }

    #[test]
    fn multi_select_adds() {
        let mut l = sample().multi(true);
        assert!(l.select(0));
        assert!(l.select(2));
        assert!(l.is_selected(0));
        assert!(l.is_selected(2));
        assert_eq!(l.selection_count(), 2);
        let indices: Vec<usize> = l.selected_indices().collect();
        assert_eq!(indices, vec![0, 2]);
    }

    #[test]
    fn toggle_membership() {
        let mut l = sample().multi(true);
        assert!(l.toggle(1)); // now selected
        assert!(l.is_selected(1));
        assert!(!l.toggle(1)); // now deselected
        assert!(!l.is_selected(1));
    }

    #[test]
    fn deselect_and_clear() {
        let mut l = sample().multi(true);
        l.select(0);
        l.select(1);
        assert!(l.deselect(0));
        assert!(!l.deselect(0));
        assert!(l.is_selected(1));
        l.clear_selection();
        assert_eq!(l.selection_count(), 0);
    }

    #[test]
    fn out_of_range_select_returns_false() {
        let mut l = sample();
        assert!(!l.select(3));
        assert!(!l.toggle(9));
        assert_eq!(l.selection_count(), 0);
    }

    #[test]
    fn semantic_role_is_list() {
        assert_eq!(sample().semantic_role(), SemanticRole::List);
    }

    #[test]
    fn view_children_match_item_count_and_selection() {
        let mut l = sample();
        l.select(1);
        let view = l.view();
        match &view.root().nodes()[0] {
            ViewNode::Element {
                component,
                children,
                ..
            } => {
                assert_eq!(*component, List::COMPONENT_ID);
                assert_eq!(children.len(), 3);
                match &children.nodes()[1] {
                    ViewNode::Element { props, .. } => {
                        let sel = props
                            .iter()
                            .find(|p| p.name() == List::PROP_SELECTED)
                            .expect("selected prop present");
                        assert_eq!(*sel.value(), ViewValue::Bool(true));
                    }
                    ViewNode::Text { .. } => panic!("expected element child"),
                }
            }
            ViewNode::Text { .. } => panic!("expected element root"),
        }
    }
}
