#![doc = include_str!("tabs.doc/_module.md")]

use obscur_accessibility::semantic::SemanticRole;
use obscur_identity::{ComponentId, InternedSymbol};
use obscur_primitives::Text;
use obscur_view::view::{View, ViewFragment, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("tabs.doc/Tabs.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tabs {
    tabs: Vec<Box<str>>,
    active: usize,
}

impl Tabs {
    #[doc = include_str!("tabs.doc/COMPONENT_ID.md")]
    pub const COMPONENT_ID: ComponentId = ComponentId::from_raw(0x00D0_0007);

    #[doc = include_str!("tabs.doc/TAB_ID.md")]
    const TAB_ID: ComponentId = ComponentId::from_raw(0x00D0_0017);

    #[doc = include_str!("tabs.doc/PROP_ROLE.md")]
    const PROP_ROLE: InternedSymbol = InternedSymbol::from_raw(0x00D1_0700);
    #[doc = include_str!("tabs.doc/PROP_ACTIVE.md")]
    const PROP_ACTIVE: InternedSymbol = InternedSymbol::from_raw(0x00D1_0701);
    #[doc = include_str!("tabs.doc/PROP_SELECTED.md")]
    const PROP_SELECTED: InternedSymbol = InternedSymbol::from_raw(0x00D1_0702);

    #[doc = include_str!("tabs.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active: 0,
        }
    }

    #[doc = include_str!("tabs.doc/tab.md")]
    #[must_use]
    pub fn tab(mut self, label: impl Into<Box<str>>) -> Self {
        self.tabs.push(label.into());
        self
    }

    #[doc = include_str!("tabs.doc/with_tabs.md")]
    #[must_use]
    pub fn with_tabs<I, S>(mut self, labels: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<Box<str>>,
    {
        self.tabs.extend(labels.into_iter().map(Into::into));
        self
    }

    #[doc = include_str!("tabs.doc/activate.md")]
    pub fn activate(&mut self, index: usize) -> bool {
        if index < self.tabs.len() {
            self.active = index;
            true
        } else {
            false
        }
    }

    #[doc = include_str!("tabs.doc/active_index.md")]
    #[must_use]
    pub fn active_index(&self) -> usize {
        if self.tabs.is_empty() {
            0
        } else {
            self.active.min(self.tabs.len() - 1)
        }
    }

    #[doc = include_str!("tabs.doc/active_label.md")]
    #[must_use]
    pub fn active_label(&self) -> Option<&str> {
        self.tabs.get(self.active_index()).map(|s| &**s)
    }

    #[doc = include_str!("tabs.doc/next.md")]
    pub fn next(&mut self) {
        if !self.tabs.is_empty() {
            self.active = (self.active_index() + 1) % self.tabs.len();
        }
    }

    #[doc = include_str!("tabs.doc/previous.md")]
    pub fn previous(&mut self) {
        let len = self.tabs.len();
        if len != 0 {
            self.active = (self.active_index() + len - 1) % len;
        }
    }

    #[doc = include_str!("tabs.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.tabs.len()
    }

    #[doc = include_str!("tabs.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    #[doc = include_str!("tabs.doc/semantic_role.md")]
    #[must_use]
    pub const fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Tab
    }

    #[doc = include_str!("tabs.doc/view.md")]
    #[must_use]
    pub fn view(&self) -> View {
        let active = self.active_index();
        let active_prop = i64::try_from(active).unwrap_or(0);
        let children: Vec<ViewNode> = self
            .tabs
            .iter()
            .enumerate()
            .map(|(index, label)| {
                ViewNode::element(Self::TAB_ID)
                    .with_prop(ViewProperty::new(Self::PROP_ROLE, ViewValue::from("tab")))
                    .with_prop(ViewProperty::new(
                        Self::PROP_SELECTED,
                        ViewValue::Bool(index == active),
                    ))
                    .with_children(ViewFragment::from_nodes(vec![
                        Text::new(label.clone()).into_view_node(),
                    ]))
            })
            .collect();
        let node = ViewNode::element(Self::COMPONENT_ID)
            .with_prop(ViewProperty::new(
                Self::PROP_ROLE,
                ViewValue::from("tablist"),
            ))
            .with_prop(ViewProperty::new(
                Self::PROP_ACTIVE,
                ViewValue::Int(active_prop),
            ))
            .with_children(ViewFragment::from_nodes(children));
        View::single(node)
    }
}

impl Default for Tabs {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Tabs {
        Tabs::new().with_tabs(["One", "Two", "Three"])
    }

    #[test]
    fn tab_builder_and_valid_activate() {
        let mut t = sample();
        assert_eq!(t.len(), 3);
        assert!(t.activate(2));
        assert_eq!(t.active_index(), 2);
        assert_eq!(t.active_label(), Some("Three"));
    }

    #[test]
    fn out_of_range_activate_returns_false() {
        let mut t = sample();
        assert!(!t.activate(3));
        assert_eq!(t.active_index(), 0);
    }

    #[test]
    fn next_wraps() {
        let mut t = sample();
        t.next();
        assert_eq!(t.active_index(), 1);
        t.next();
        t.next();
        assert_eq!(t.active_index(), 0); // wrapped past 2
    }

    #[test]
    fn previous_wraps() {
        let mut t = sample();
        t.previous();
        assert_eq!(t.active_index(), 2); // wrapped below 0
        t.previous();
        assert_eq!(t.active_index(), 1);
    }

    #[test]
    fn empty_strip_is_stable() {
        let mut t = Tabs::new();
        t.next();
        t.previous();
        assert_eq!(t.active_index(), 0);
        assert_eq!(t.active_label(), None);
        assert!(t.is_empty());
    }

    #[test]
    fn semantic_role_is_tab() {
        assert_eq!(sample().semantic_role(), SemanticRole::Tab);
    }

    #[test]
    fn view_children_mark_active_selected() {
        let mut t = sample();
        assert!(t.activate(1));
        let view = t.view();
        match &view.root().nodes()[0] {
            ViewNode::Element {
                component,
                children,
                ..
            } => {
                assert_eq!(*component, Tabs::COMPONENT_ID);
                assert_eq!(children.len(), 3);
                for (index, child) in children.nodes().iter().enumerate() {
                    match child {
                        ViewNode::Element { props, .. } => {
                            let selected = props
                                .iter()
                                .find(|p| p.name() == Tabs::PROP_SELECTED)
                                .expect("selected prop present");
                            assert_eq!(*selected.value(), ViewValue::Bool(index == 1));
                        }
                        ViewNode::Text { .. } => panic!("expected element child"),
                    }
                }
            }
            ViewNode::Text { .. } => panic!("expected element root"),
        }
    }
}
