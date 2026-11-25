//! `Tabs`: a headless tab strip selecting one panel (spec §2.6).

use super::{TabSelectionState, tab_entry::TabEntry};
use obscur_accessibility::semantic::SemanticRole;
use obscur_identity::{ComponentId, InternedSymbol};
use obscur_primitives::Text;
use obscur_view::view::{View, ViewFragment, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("../../docs/api/tabs/tabs.doc/Tabs.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tabs {
    tabs: Vec<TabEntry>,
    active: usize,
}

impl Tabs {
    #[doc = include_str!("../../docs/api/tabs/tabs.doc/COMPONENT_ID.md")]
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
        self.tabs.push(TabEntry {
            id: None,
            label: label.into(),
        });
        self
    }

    /// Adds a tab with an application-defined ID for selection persistence. Use a nonempty ID that is unique within the strip.
    #[must_use]
    pub fn tab_with_id(mut self, id: impl Into<Box<str>>, label: impl Into<Box<str>>) -> Self {
        self.tabs.push(TabEntry {
            id: Some(id.into()),
            label: label.into(),
        });
        self
    }

    /// Captures the selected tab when it has a unique, nonempty ID.
    #[must_use]
    pub fn capture_selection_state(&self) -> Option<TabSelectionState> {
        let id = self.tabs.get(self.active_index())?.id.as_deref()?;
        self.unique_tab_index(id)
            .map(|_| TabSelectionState::capture(id))
    }

    /// Restores the selection by ID. Unknown versions and missing or ambiguous IDs leave the current selection intact.
    pub fn restore_selection_state(&mut self, state: &TabSelectionState) -> bool {
        let Some(index) = state
            .validated_id()
            .and_then(|id| self.unique_tab_index(id))
        else {
            return false;
        };
        self.activate(index)
    }

    fn unique_tab_index(&self, id: &str) -> Option<usize> {
        if id.trim().is_empty() {
            return None;
        }
        let mut matches = self
            .tabs
            .iter()
            .enumerate()
            .filter(|(_, tab)| tab.id.as_deref() == Some(id));
        let (index, _) = matches.next()?;
        matches.next().is_none().then_some(index)
    }

    /// Appends every label from `labels`, returning the updated strip.
    #[must_use]
    pub fn with_tabs<I, S>(mut self, labels: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<Box<str>>,
    {
        self.tabs.extend(labels.into_iter().map(|label| TabEntry {
            id: None,
            label: label.into(),
        }));
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
        self.tabs
            .get(self.active_index())
            .map(|tab| tab.label.as_ref())
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

    #[doc = include_str!("../../docs/api/tabs/tabs.doc/view.md")]
    #[must_use]
    pub fn view(&self) -> View {
        let active = self.active_index();
        let active_prop = i64::try_from(active).unwrap_or(0);
        let children: Vec<ViewNode> = self
            .tabs
            .iter()
            .enumerate()
            .map(|(index, tab)| {
                ViewNode::element(Self::TAB_ID)
                    .with_prop(ViewProperty::new(Self::PROP_ROLE, ViewValue::from("tab")))
                    .with_prop(ViewProperty::new(
                        Self::PROP_SELECTED,
                        ViewValue::Bool(index == active),
                    ))
                    .with_children(ViewFragment::from_nodes(vec![
                        Text::new(tab.label.clone()).into_view_node(),
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
