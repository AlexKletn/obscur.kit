//! `Select`: a headless single-choice dropdown control (spec §2.6).

use obscur_accessibility::semantic::SemanticRole;
use obscur_identity::{ComponentId, InternedSymbol};
use obscur_primitives::Text;
use obscur_view::view::{
    DISABLED as ATTR_DISABLED, EVENT_CLICK, TYPE, View, ViewEventBinding, ViewFragment, ViewNode,
    ViewProperty, ViewValue,
};

/// A headless single-choice dropdown control (spec §2.6).
///
/// `Select` owns its list of `options`, the index of the currently
/// [`selected`](Self::selected_index) option (if any), whether the popup is
/// [`open`](Self::is_open), and a disabled flag. It bakes in **no** visual
/// theme — [`view`](Self::view) derives a semantic element carrying the open
/// and selected state as properties, a click event bound to `command`, and
/// each option as a [`Text`] child, rendered by the style layer keyed on
/// [`COMPONENT_ID`](Self::COMPONENT_ID).
///
/// The accessible role is [`SemanticRole::Menu`]: [`SemanticRole`] has no
/// dedicated list-box role, and a menu is the closest choice-list classification
/// in the common cross-platform set.
///
/// The state machine covers popup open/close ([`open`](Self::open),
/// [`close`](Self::close), [`toggle_open`](Self::toggle_open)) and committing a
/// choice ([`select`](Self::select), which validates the index and closes the
/// popup).
#[derive(Debug, Clone, PartialEq)]
pub struct Select {
    options: Vec<Box<str>>,
    command: InternedSymbol,
    selected: Option<usize>,
    open: bool,
    disabled: bool,
}

impl Select {
    /// Stable component id for the select widget (see [`Button::COMPONENT_ID`]
    /// for the shared `0x00D0_00NN` scheme).
    ///
    /// [`Button::COMPONENT_ID`]: crate::button::Button::COMPONENT_ID
    pub const COMPONENT_ID: ComponentId = ComponentId::from_raw(0x00D0_0004);

    /// Interned name of the semantic-role property (`role`); shares the
    /// `0x00D1_NNXX` scheme with `NN = 04` (select): role=`00`, disabled=`01`,
    /// open=`02`, selected=`03`.
    const PROP_ROLE: InternedSymbol = InternedSymbol::from_raw(0x00D1_0400);
    /// Interned name of the `disabled` state property.
    const PROP_DISABLED: InternedSymbol = InternedSymbol::from_raw(0x00D1_0401);
    /// Interned name of the `open` state property.
    const PROP_OPEN: InternedSymbol = InternedSymbol::from_raw(0x00D1_0402);
    /// Interned name of the `selected` index property (`-1` = no selection).
    const PROP_SELECTED: InternedSymbol = InternedSymbol::from_raw(0x00D1_0403);


    /// Creates an enabled, closed, option-less select that fires `command` when
    /// a choice is committed.
    #[must_use]
    pub fn new(command: InternedSymbol) -> Self {
        Self {
            options: Vec::new(),
            command,
            selected: None,
            open: false,
            disabled: false,
        }
    }

    /// Appends a single option, returning the updated select (fluent builder).
    #[must_use]
    pub fn option(mut self, label: impl Into<Box<str>>) -> Self {
        self.options.push(label.into());
        self
    }

    /// Appends every option from `labels`, returning the updated select.
    #[must_use]
    pub fn with_options<I, S>(mut self, labels: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<Box<str>>,
    {
        self.options.extend(labels.into_iter().map(Into::into));
        self
    }

    /// Sets the disabled state, returning the updated select (fluent builder).
    #[must_use]
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Opens the popup. A no-op while [`disabled`](Self::is_disabled).
    pub fn open(&mut self) {
        if !self.disabled {
            self.open = true;
        }
    }

    /// Closes the popup.
    pub fn close(&mut self) {
        self.open = false;
    }

    /// Toggles the popup open/closed. A no-op while disabled and closed (a
    /// disabled select can still be closed).
    pub fn toggle_open(&mut self) {
        if self.open {
            self.open = false;
        } else if !self.disabled {
            self.open = true;
        }
    }

    /// Selects the option at `index`, closing the popup, and returns `true`.
    /// Returns `false` (leaving state untouched) when `index` is out of range.
    pub fn select(&mut self, index: usize) -> bool {
        if index < self.options.len() {
            self.selected = Some(index);
            self.open = false;
            true
        } else {
            false
        }
    }

    /// Returns the index of the selected option, if any.
    #[must_use]
    pub const fn selected_index(&self) -> Option<usize> {
        self.selected
    }

    /// Returns the label of the selected option, if any.
    #[must_use]
    pub fn selected_label(&self) -> Option<&str> {
        self.selected
            .and_then(|i| self.options.get(i))
            .map(|s| &**s)
    }

    /// Returns whether the popup is open.
    #[must_use]
    pub const fn is_open(&self) -> bool {
        self.open
    }

    /// Returns whether the select is disabled.
    #[must_use]
    pub const fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Sets the disabled state in place.
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    /// Returns the command fired when a choice is committed.
    #[must_use]
    pub const fn command(&self) -> InternedSymbol {
        self.command
    }

    /// Returns the number of options.
    #[must_use]
    pub fn len(&self) -> usize {
        self.options.len()
    }

    /// Returns whether the select has no options.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.options.is_empty()
    }

    /// Returns the accessible role of this control ([`SemanticRole::Menu`], the
    /// closest choice-list role — see the type docs).
    #[must_use]
    pub const fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Menu
    }

    /// Derives the semantic view: a [`COMPONENT_ID`] element carrying the role,
    /// disabled and open state, the selected index as an integer property
    /// (`-1` when nothing is selected), a click event bound to `command`,
    /// and each option as a [`Text`] child in order.
    ///
    /// [`COMPONENT_ID`]: Self::COMPONENT_ID
    #[must_use]
    pub fn view(&self) -> View {
        let selected = self.selected.map_or(-1, |i| i64::try_from(i).unwrap_or(-1));
        let children: Vec<ViewNode> = self
            .options
            .iter()
            .map(|label| Text::new(label.clone()).into_view_node())
            .collect();
        let node = ViewNode::element(Self::COMPONENT_ID)
            // Stable CSS type / local name so a UA (or author) `select { … }` type
            // selector matches this control (spec §4.13). Additive.
            .with_prop(ViewProperty::new(TYPE, ViewValue::from("select")))
            .with_prop(ViewProperty::new(Self::PROP_ROLE, ViewValue::from("menu")))
            .with_prop(ViewProperty::new(
                Self::PROP_DISABLED,
                ViewValue::Bool(self.disabled),
            ))
            .with_prop(ViewProperty::new(
                Self::PROP_OPEN,
                ViewValue::Bool(self.open),
            ))
            .with_prop(ViewProperty::new(
                Self::PROP_SELECTED,
                ViewValue::Int(selected),
            ))
            .with_event(ViewEventBinding::new(EVENT_CLICK, self.command));
        // Reserved `:disabled` structural-state attribute — emitted only when
        // disabled; survives reconcile and feeds the matcher's `:disabled`.
        let node = if self.disabled {
            node.with_prop(ViewProperty::new(ATTR_DISABLED, ViewValue::Bool(true)))
        } else {
            node
        }
        .with_children(ViewFragment::from_nodes(children));
        View::single(node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cmd() -> InternedSymbol {
        InternedSymbol::from_raw(0x9ABC)
    }

    fn sample() -> Select {
        Select::new(cmd()).with_options(["One", "Two", "Three"])
    }

    #[test]
    fn option_builder_and_valid_select() {
        let mut s = Select::new(cmd()).option("Only");
        assert_eq!(s.len(), 1);
        assert!(s.select(0));
        assert_eq!(s.selected_index(), Some(0));
        assert_eq!(s.selected_label(), Some("Only"));
    }

    #[test]
    fn out_of_range_select_returns_false() {
        let mut s = sample();
        assert!(!s.select(3));
        assert_eq!(s.selected_index(), None);
        assert_eq!(s.selected_label(), None);
    }

    #[test]
    fn open_close_toggle_cycle() {
        let mut s = sample();
        assert!(!s.is_open());
        s.open();
        assert!(s.is_open());
        s.close();
        assert!(!s.is_open());
        s.toggle_open();
        assert!(s.is_open());
        s.toggle_open();
        assert!(!s.is_open());
    }

    #[test]
    fn select_closes_popup() {
        let mut s = sample();
        s.open();
        assert!(s.select(1));
        assert!(!s.is_open());
    }

    #[test]
    fn disabled_open_is_noop() {
        let mut s = sample().disabled(true);
        s.open();
        assert!(!s.is_open());
    }

    #[test]
    fn semantic_role_is_menu() {
        assert_eq!(sample().semantic_role(), SemanticRole::Menu);
    }

    #[test]
    fn view_children_match_option_count() {
        let s = sample();
        let view = s.view();
        match &view.root().nodes()[0] {
            ViewNode::Element {
                component,
                props,
                events,
                children,
                ..
            } => {
                assert_eq!(*component, Select::COMPONENT_ID);
                assert_eq!(children.len(), 3);
                let selected = props
                    .iter()
                    .find(|p| p.name() == Select::PROP_SELECTED)
                    .expect("selected prop present");
                assert_eq!(*selected.value(), ViewValue::Int(-1));
                assert_eq!(events[0].command(), cmd());
            }
            ViewNode::Text { .. } => panic!("expected element root"),
        }
    }
}
