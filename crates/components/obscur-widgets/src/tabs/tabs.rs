//! `Tabs`: a headless tab strip selecting one panel (spec §2.6).

use obscur_accessibility::semantic::SemanticRole;
use obscur_identity::{ComponentId, InternedSymbol};
use obscur_primitives::Text;
use obscur_view::view::{View, ViewFragment, ViewNode, ViewProperty, ViewValue};

/// A headless tab strip selecting one of several panels (spec §2.6).
///
/// `Tabs` owns its list of tab `labels` and the index of the currently
/// [`active`](Self::active_index) tab. It bakes in **no** visual theme:
/// [`view`](Self::view) derives a `tablist`-role element carrying the active
/// index as a property, with one `tab`-role child element per label (each
/// carrying a selected flag and a [`Text`] label child), rendered by the style
/// layer keyed on [`COMPONENT_ID`](Self::COMPONENT_ID).
///
/// The [`semantic_role`](Self::semantic_role) reported for the strip is
/// [`SemanticRole::Tab`] (the accessible model has no dedicated `tablist` role
/// in the common cross-platform set). Conceptually each label child is a
/// [`SemanticRole::Tab`] and the content it reveals would be a
/// [`SemanticRole::TabPanel`]; `Tabs` models only the strip, leaving the panel
/// to the host.
///
/// The state machine is index selection ([`activate`](Self::activate), which
/// validates the index) plus wrapping movement ([`next`](Self::next),
/// [`previous`](Self::previous)).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tabs {
    tabs: Vec<Box<str>>,
    active: usize,
}

impl Tabs {
    /// Stable component id for the tabs widget (see [`Button::COMPONENT_ID`]
    /// for the shared `0x00D0_00NN` scheme).
    ///
    /// [`Button::COMPONENT_ID`]: crate::button::Button::COMPONENT_ID
    pub const COMPONENT_ID: ComponentId = ComponentId::from_raw(0x00D0_0007);

    /// Stable component id for the per-tab child element.
    const TAB_ID: ComponentId = ComponentId::from_raw(0x00D0_0017);

    /// Interned name of the semantic-role property (`role`); shares the
    /// `0x00D1_NNXX` scheme with `NN = 07` (tabs): role=`00`, active=`01`,
    /// selected=`02` (on tab children).
    const PROP_ROLE: InternedSymbol = InternedSymbol::from_raw(0x00D1_0700);
    /// Interned name of the `active` index property.
    const PROP_ACTIVE: InternedSymbol = InternedSymbol::from_raw(0x00D1_0701);
    /// Interned name of the per-tab `selected` state property.
    const PROP_SELECTED: InternedSymbol = InternedSymbol::from_raw(0x00D1_0702);

    /// Creates an empty tab strip with the active index at `0`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active: 0,
        }
    }

    /// Appends a single tab label, returning the updated strip (fluent
    /// builder).
    #[must_use]
    pub fn tab(mut self, label: impl Into<Box<str>>) -> Self {
        self.tabs.push(label.into());
        self
    }

    /// Appends every label from `labels`, returning the updated strip.
    #[must_use]
    pub fn with_tabs<I, S>(mut self, labels: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<Box<str>>,
    {
        self.tabs.extend(labels.into_iter().map(Into::into));
        self
    }

    /// Activates the tab at `index`, returning `true` when `index` is in range;
    /// otherwise leaves the active tab untouched and returns `false`.
    pub fn activate(&mut self, index: usize) -> bool {
        if index < self.tabs.len() {
            self.active = index;
            true
        } else {
            false
        }
    }

    /// Returns the active tab index. Clamped to a valid index, or `0` when the
    /// strip is empty.
    #[must_use]
    pub fn active_index(&self) -> usize {
        if self.tabs.is_empty() {
            0
        } else {
            self.active.min(self.tabs.len() - 1)
        }
    }

    /// Returns the active tab label, or `None` when the strip is empty.
    #[must_use]
    pub fn active_label(&self) -> Option<&str> {
        self.tabs.get(self.active_index()).map(|s| &**s)
    }

    /// Moves selection to the next tab, wrapping around. A no-op when empty.
    pub fn next(&mut self) {
        if !self.tabs.is_empty() {
            self.active = (self.active_index() + 1) % self.tabs.len();
        }
    }

    /// Moves selection to the previous tab, wrapping around. A no-op when
    /// empty.
    pub fn previous(&mut self) {
        let len = self.tabs.len();
        if len != 0 {
            self.active = (self.active_index() + len - 1) % len;
        }
    }

    /// Returns the number of tabs.
    #[must_use]
    pub fn len(&self) -> usize {
        self.tabs.len()
    }

    /// Returns whether the strip has no tabs.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    /// Returns the accessible role of this control ([`SemanticRole::Tab`]; see
    /// the type docs on the strip vs. individual tab/panel distinction).
    #[must_use]
    pub const fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Tab
    }

    /// Derives the semantic view: a `tablist`-role [`COMPONENT_ID`] element
    /// carrying the active index as an integer property, with one `tab`-role
    /// child element per label (each carrying a selected flag set on the active
    /// tab and a [`Text`] label child).
    ///
    /// [`COMPONENT_ID`]: Self::COMPONENT_ID
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
