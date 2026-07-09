//! `Select`: a headless single-choice dropdown control (spec §2.6).

use obscur_accessibility::semantic::SemanticRole;
use obscur_identity::{ComponentId, InternedSymbol};
use obscur_primitives::Text;
use obscur_view::view::{
    CLASS, DISABLED as ATTR_DISABLED, EVENT_CLICK, TYPE, View, ViewEventBinding, ViewFragment,
    ViewNode, ViewProperty, ViewValue,
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

    /// Component id of the disclosure-arrow indicator sub-part
    /// (`.select-arrow`), in the widget sub-part id space (`..14`).
    const ARROW_ID: ComponentId = ComponentId::from_raw(0x00D0_0014);

    /// Component id of the always-present **header** row sub-part
    /// (`.select-header`) — the current label + arrow, carrying the box-interior
    /// padding. The outer control is a vertical column of this header and (when
    /// open) the option list. Sub-part id space (`..24`).
    const HEADER_ID: ComponentId = ComponentId::from_raw(0x00D0_0024);

    /// Component id of the open-popup option-list container sub-part
    /// (`.select-list`), rendered in normal flow directly below the header.
    /// Sub-part id space (`..44`).
    const LIST_ID: ComponentId = ComponentId::from_raw(0x00D0_0044);

    /// Base component id for the option rows of the open popup
    /// (`.select-option`). Option `i` gets `OPTION_ID_BASE + i` so each row is a
    /// DISTINCT box, matched positionally across reconciles. Sub-part id space
    /// (`..34`).
    const OPTION_ID_BASE: u64 = 0x00D0_0034;

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

    /// Derives the semantic view: a [`COMPONENT_ID`] outer element — the box and
    /// the click/`:disabled` target — laid out as a **vertical column** of a
    /// `.select-header` row and, while open, a `.select-list` popup.
    ///
    /// The **header** (`.select-header`) is a row of the **current label** (the
    /// selected option, else the first option, else empty — the browser-faithful
    /// closed display) followed by a trailing `.select-arrow` indicator sub-part.
    /// The arrow carries an `open` state class (so a `select-arrow.open { … }` or
    /// `select:disabled .select-arrow { … }` rule can key on it) and shows the
    /// disclosure glyph as its text child — "▼" closed, "▲" open.
    ///
    /// When [`is_open`](Self::is_open), a `.select-list` of `.select-option` rows
    /// (one per option, the selected one also carrying a `selected` class) is
    /// appended after the header, so the popup renders as a vertical stack directly
    /// below it. The list is emitted **in normal flow** (not absolutely
    /// positioned): the toolkit's layout/render of a positioned popup subtree is
    /// unreliable here, so an in-flow list is the faithful, deterministic choice —
    /// it grows the control's box downward rather than overlaying following content.
    /// The click event and reserved `:disabled` attr stay on the OUTER element, so a
    /// click anywhere in the control (header or a row) bubbles to the one handler.
    ///
    /// [`COMPONENT_ID`]: Self::COMPONENT_ID
    #[must_use]
    pub fn view(&self) -> View {
        let selected = self.selected.map_or(-1, |i| i64::try_from(i).unwrap_or(-1));
        // Header label: the selected option, else the first option (a bare
        // `<select>` shows its first option when nothing is picked), else empty.
        let header_label: Box<str> = self
            .selected_label()
            .or_else(|| self.options.first().map(|s| &**s))
            .unwrap_or("")
            .into();
        // The disclosure-arrow indicator sub-part.
        let (arrow_class, arrow_glyph) = if self.open {
            ("select-arrow open", "▲")
        } else {
            ("select-arrow", "▼")
        };
        let arrow = ViewNode::element(Self::ARROW_ID)
            .with_prop(ViewProperty::new(CLASS, ViewValue::from(arrow_class)))
            .with_children(ViewFragment::from_nodes(vec![
                Text::new(arrow_glyph).into_view_node(),
            ]));
        // The header row: current label + arrow. Carries the box-interior padding.
        let header = ViewNode::element(Self::HEADER_ID)
            .with_prop(ViewProperty::new(CLASS, ViewValue::from("select-header")))
            .with_children(ViewFragment::from_nodes(vec![
                Text::new(header_label).into_view_node(),
                arrow,
            ]));
        let mut children: Vec<ViewNode> = vec![header];
        // The open dropdown popup: a `.select-list` of stacked `.select-option`
        // rows, the selected one carrying a `selected` class. Absent when closed.
        if self.open {
            let options: Vec<ViewNode> = self
                .options
                .iter()
                .enumerate()
                .map(|(i, label)| {
                    let class = if self.selected == Some(i) {
                        "select-option selected"
                    } else {
                        "select-option"
                    };
                    let id = ComponentId::from_raw(Self::OPTION_ID_BASE + i as u64);
                    ViewNode::element(id)
                        .with_prop(ViewProperty::new(CLASS, ViewValue::from(class)))
                        .with_children(ViewFragment::from_nodes(vec![
                            Text::new(label.clone()).into_view_node(),
                        ]))
                })
                .collect();
            children.push(
                ViewNode::element(Self::LIST_ID)
                    .with_prop(ViewProperty::new(CLASS, ViewValue::from("select-list")))
                    .with_children(ViewFragment::from_nodes(options)),
            );
        }
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

    /// The `.select-header` child of the outer select node (the label + arrow row).
    fn header_of(children: &ViewFragment) -> &ViewFragment {
        match &children.nodes()[0] {
            ViewNode::Element { component, props, children, .. } => {
                assert_eq!(*component, Select::HEADER_ID);
                assert!(props.iter().any(|p| p.name() == CLASS
                    && matches!(p.value(), ViewValue::Text(t) if &**t == "select-header")));
                children
            }
            ViewNode::Text { .. } => panic!("expected the .select-header element"),
        }
    }

    #[test]
    fn closed_view_is_header_row_with_label_and_arrow() {
        // CLOSED: the outer has one child — the `.select-header` row of the
        // current-label Text (the first option when nothing is picked) + the
        // `.select-arrow` indicator. No option list is emitted while closed.
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
                assert_eq!(children.len(), 1, "closed: header only, no list");
                let head = header_of(children);
                assert_eq!(head.nodes().len(), 2, "header = label + arrow");
                match &head.nodes()[0] {
                    ViewNode::Text { content, .. } => assert_eq!(&**content, "One"),
                    ViewNode::Element { .. } => panic!("expected header label text"),
                }
                assert!(matches!(
                    &head.nodes()[1],
                    ViewNode::Element { props, .. }
                        if props.iter().any(|p| p.name() == CLASS
                            && matches!(p.value(), ViewValue::Text(t) if t.starts_with("select-arrow")))
                ));
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

    #[test]
    fn open_view_appends_a_select_list_of_options() {
        // OPEN: the header plus a trailing `.select-list` popup whose children are
        // one `.select-option` per option, the selected one carrying `selected`.
        let mut s = sample();
        s.select(1); // picks "Two", but also closes — reopen for the popup.
        s.open();
        let view = s.view();
        let list = match &view.root().nodes()[0] {
            ViewNode::Element { children, .. } => {
                assert_eq!(children.len(), 2, "header + list");
                header_of(children);
                match &children.nodes()[1] {
                    ViewNode::Element { component, props, children, .. } => {
                        assert_eq!(*component, Select::LIST_ID);
                        assert!(props.iter().any(|p| p.name() == CLASS
                            && matches!(p.value(), ViewValue::Text(t) if &**t == "select-list")));
                        children.clone()
                    }
                    ViewNode::Text { .. } => panic!("expected the .select-list element"),
                }
            }
            ViewNode::Text { .. } => panic!("expected element root"),
        };
        assert_eq!(list.nodes().len(), 3, "one option row per option");
        // The selected row (index 1) carries the `selected` class; the others do not.
        for (i, opt) in list.nodes().iter().enumerate() {
            match opt {
                ViewNode::Element { component, props, .. } => {
                    assert_eq!(*component, ComponentId::from_raw(Select::OPTION_ID_BASE + i as u64));
                    let class = props
                        .iter()
                        .find(|p| p.name() == CLASS)
                        .and_then(|p| match p.value() {
                            ViewValue::Text(t) => Some(&**t),
                            _ => None,
                        })
                        .expect("option class");
                    if i == 1 {
                        assert_eq!(class, "select-option selected");
                    } else {
                        assert_eq!(class, "select-option");
                    }
                }
                ViewNode::Text { .. } => panic!("expected option element"),
            }
        }
    }
}
