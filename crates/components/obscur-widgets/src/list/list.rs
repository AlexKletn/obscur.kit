#![doc = include_str!("list.doc/_module.md")]

use std::collections::BTreeSet;

use obscur_accessibility::semantic::SemanticRole;
use obscur_identity::{ComponentId, InternedSymbol};
use obscur_primitives::Text;
use obscur_view::view::{View, ViewFragment, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("list.doc/List.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct List {
    items: Vec<Box<str>>,
    selected: BTreeSet<usize>,
    multi: bool,
}

impl List {
    #[doc = include_str!("list.doc/COMPONENT_ID.md")]
    pub const COMPONENT_ID: ComponentId = ComponentId::from_raw(0x00D0_0008);

    #[doc = include_str!("list.doc/LIST_ITEM_ID.md")]
    const LIST_ITEM_ID: ComponentId = ComponentId::from_raw(0x00D0_0018);

    #[doc = include_str!("list.doc/PROP_ROLE.md")]
    const PROP_ROLE: InternedSymbol = InternedSymbol::from_raw(0x00D1_0800);
    #[doc = include_str!("list.doc/PROP_MULTI.md")]
    const PROP_MULTI: InternedSymbol = InternedSymbol::from_raw(0x00D1_0801);
    #[doc = include_str!("list.doc/PROP_SELECTED.md")]
    const PROP_SELECTED: InternedSymbol = InternedSymbol::from_raw(0x00D1_0802);

    #[doc = include_str!("list.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected: BTreeSet::new(),
            multi: false,
        }
    }

    #[doc = include_str!("list.doc/item.md")]
    #[must_use]
    pub fn item(mut self, label: impl Into<Box<str>>) -> Self {
        self.items.push(label.into());
        self
    }

    #[doc = include_str!("list.doc/with_items.md")]
    #[must_use]
    pub fn with_items<I, S>(mut self, labels: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<Box<str>>,
    {
        self.items.extend(labels.into_iter().map(Into::into));
        self
    }

    #[doc = include_str!("list.doc/multi.md")]
    #[must_use]
    pub fn multi(mut self, multi: bool) -> Self {
        self.multi = multi;
        self
    }

    #[doc = include_str!("list.doc/select.md")]
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

    #[doc = include_str!("list.doc/toggle.md")]
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

    #[doc = include_str!("list.doc/deselect.md")]
    pub fn deselect(&mut self, index: usize) -> bool {
        self.selected.remove(&index)
    }

    #[doc = include_str!("list.doc/clear_selection.md")]
    pub fn clear_selection(&mut self) {
        self.selected.clear();
    }

    #[doc = include_str!("list.doc/is_selected.md")]
    #[must_use]
    pub fn is_selected(&self, index: usize) -> bool {
        self.selected.contains(&index)
    }

    #[doc = include_str!("list.doc/selected_indices.md")]
    pub fn selected_indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.selected.iter().copied()
    }

    #[doc = include_str!("list.doc/selection_count.md")]
    #[must_use]
    pub fn selection_count(&self) -> usize {
        self.selected.len()
    }

    #[doc = include_str!("list.doc/is_multi.md")]
    #[must_use]
    pub const fn is_multi(&self) -> bool {
        self.multi
    }

    #[doc = include_str!("list.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    #[doc = include_str!("list.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    #[doc = include_str!("list.doc/semantic_role.md")]
    #[must_use]
    pub const fn semantic_role(&self) -> SemanticRole {
        SemanticRole::List
    }

    #[doc = include_str!("list.doc/view.md")]
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
            .with_prop(ViewProperty::new(
                Self::PROP_MULTI,
                ViewValue::Bool(self.multi),
            ))
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
