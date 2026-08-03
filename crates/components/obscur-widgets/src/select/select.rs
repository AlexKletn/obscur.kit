#![doc = include_str!("select.doc/_module.md")]

use obscur_accessibility::semantic::SemanticRole;
use obscur_identity::{ComponentId, InternedSymbol};
use obscur_primitives::Text;
use obscur_view::view::{
    CLASS, DISABLED as ATTR_DISABLED, EVENT_CLICK, LOCAL_NAME, View, ViewEventBinding,
    ViewFragment, ViewNode, ViewProperty, ViewValue,
};

#[doc = include_str!("select.doc/Select.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct Select {
    options: Vec<Box<str>>,
    command: InternedSymbol,
    selected: Option<usize>,
    open: bool,
    disabled: bool,
}

impl Select {
    #[doc = include_str!("select.doc/COMPONENT_ID.md")]
    pub const COMPONENT_ID: ComponentId = ComponentId::from_raw(0x00D0_0004);

    #[doc = include_str!("select.doc/ARROW_ID.md")]
    const ARROW_ID: ComponentId = ComponentId::from_raw(0x00D0_0014);

    #[doc = include_str!("select.doc/HEADER_ID.md")]
    const HEADER_ID: ComponentId = ComponentId::from_raw(0x00D0_0024);

    #[doc = include_str!("select.doc/LIST_ID.md")]
    const LIST_ID: ComponentId = ComponentId::from_raw(0x00D0_0044);

    #[doc = include_str!("select.doc/OPTION_ID_BASE.md")]
    const OPTION_ID_BASE: u64 = 0x00D0_0034;

    #[doc = include_str!("select.doc/PROP_ROLE.md")]
    const PROP_ROLE: InternedSymbol = InternedSymbol::from_raw(0x00D1_0400);
    #[doc = include_str!("select.doc/PROP_DISABLED.md")]
    const PROP_DISABLED: InternedSymbol = InternedSymbol::from_raw(0x00D1_0401);
    #[doc = include_str!("select.doc/PROP_OPEN.md")]
    pub(crate) const PROP_OPEN: InternedSymbol = InternedSymbol::from_raw(0x00D1_0402);
    #[doc = include_str!("select.doc/PROP_SELECTED.md")]
    pub(crate) const PROP_SELECTED: InternedSymbol = InternedSymbol::from_raw(0x00D1_0403);
    #[doc = include_str!("select.doc/PROP_OPTIONS.md")]
    pub(crate) const PROP_OPTIONS: InternedSymbol = InternedSymbol::from_raw(0x00D1_0404);

    #[doc = include_str!("select.doc/OPTION_SEPARATOR.md")]
    pub(crate) const OPTION_SEPARATOR: char = '\u{1f}';

    #[doc = include_str!("select.doc/new.md")]
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

    #[doc = include_str!("select.doc/option.md")]
    #[must_use]
    pub fn option(mut self, label: impl Into<Box<str>>) -> Self {
        self.options.push(label.into());
        self
    }

    #[doc = include_str!("select.doc/with_options.md")]
    #[must_use]
    pub fn with_options<I, S>(mut self, labels: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<Box<str>>,
    {
        self.options.extend(labels.into_iter().map(Into::into));
        self
    }

    #[doc = include_str!("select.doc/disabled.md")]
    #[must_use]
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    #[doc = include_str!("select.doc/open.md")]
    pub fn open(&mut self) {
        if !self.disabled {
            self.open = true;
        }
    }

    #[doc = include_str!("select.doc/close.md")]
    pub fn close(&mut self) {
        self.open = false;
    }

    #[doc = include_str!("select.doc/toggle_open.md")]
    pub fn toggle_open(&mut self) {
        if self.open {
            self.open = false;
        } else if !self.disabled {
            self.open = true;
        }
    }

    #[doc = include_str!("select.doc/select.md")]
    pub fn select(&mut self, index: usize) -> bool {
        if index < self.options.len() {
            self.selected = Some(index);
            self.open = false;
            true
        } else {
            false
        }
    }

    #[doc = include_str!("select.doc/selected_index.md")]
    #[must_use]
    pub const fn selected_index(&self) -> Option<usize> {
        self.selected
    }

    #[doc = include_str!("select.doc/selected_label.md")]
    #[must_use]
    pub fn selected_label(&self) -> Option<&str> {
        self.selected
            .and_then(|i| self.options.get(i))
            .map(|s| &**s)
    }

    #[doc = include_str!("select.doc/is_open.md")]
    #[must_use]
    pub const fn is_open(&self) -> bool {
        self.open
    }

    #[doc = include_str!("select.doc/is_disabled.md")]
    #[must_use]
    pub const fn is_disabled(&self) -> bool {
        self.disabled
    }

    #[doc = include_str!("select.doc/set_disabled.md")]
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    #[doc = include_str!("select.doc/command.md")]
    #[must_use]
    pub const fn command(&self) -> InternedSymbol {
        self.command
    }

    #[doc = include_str!("select.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.options.len()
    }

    #[doc = include_str!("select.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.options.is_empty()
    }

    #[doc = include_str!("select.doc/semantic_role.md")]
    #[must_use]
    pub const fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Menu
    }

    #[doc = include_str!("select.doc/view.md")]
    #[must_use]
    pub fn view(&self) -> View {
        let selected = self.selected.map_or(-1, |i| i64::try_from(i).unwrap_or(-1));
        let node = ViewNode::element(Self::COMPONENT_ID)
            // Stable CSS type / local name so a UA (or author) `select { … }` type
            // selector matches this control (spec §4.13). Additive.
            .with_prop(ViewProperty::new(LOCAL_NAME, ViewValue::from("select")))
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
        // The option labels, so this node and an authored `<select options="…"/>`
        // carry the SAME facts. That is what makes mounting `SelectComponent` on a
        // node this method built idempotent: the component decodes these props and
        // regenerates exactly the children installed below. Emitted only when there
        // are options, so "no options" and "one empty-labelled option" stay
        // distinguishable — see `PROP_OPTIONS`.
        let node = if self.options.is_empty() {
            node
        } else {
            node.with_prop(ViewProperty::new(
                Self::PROP_OPTIONS,
                ViewValue::from(self.options.join(&Self::OPTION_SEPARATOR.to_string())),
            ))
        };
        // Reserved `:disabled` structural-state attribute — emitted only when
        // disabled; survives reconcile and feeds the matcher's `:disabled`.
        let node = if self.disabled {
            node.with_prop(ViewProperty::new(ATTR_DISABLED, ViewValue::Bool(true)))
        } else {
            node
        }
        .with_children(Self::contents(&self.options, self.selected, self.open));
        View::single(node)
    }

    #[doc = include_str!("select.doc/contents.md")]
    #[must_use]
    pub(crate) fn contents(
        options: &[Box<str>],
        selected: Option<usize>,
        open: bool,
    ) -> ViewFragment {
        // Header label: the selected option, else the first option (a bare
        // `<select>` shows its first option when nothing is picked), else empty.
        let header_label: Box<str> = selected
            .and_then(|i| options.get(i))
            .or_else(|| options.first())
            .map_or("", |label| &**label)
            .into();
        let (arrow_class, arrow_glyph) = if open {
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
        if open {
            children.push(Self::open_option_list(options, selected));
        }
        ViewFragment::from_nodes(children)
    }

    fn open_option_list(options: &[Box<str>], selected: Option<usize>) -> ViewNode {
        let rows: Vec<ViewNode> = options
            .iter()
            .enumerate()
            .map(|(i, label)| {
                let class = if selected == Some(i) {
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
        ViewNode::element(Self::LIST_ID)
            .with_prop(ViewProperty::new(CLASS, ViewValue::from("select-list")))
            .with_children(ViewFragment::from_nodes(rows))
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
            ViewNode::Element {
                component,
                props,
                children,
                ..
            } => {
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
                    ViewNode::Element {
                        component,
                        props,
                        children,
                        ..
                    } => {
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
                ViewNode::Element {
                    component, props, ..
                } => {
                    assert_eq!(
                        *component,
                        ComponentId::from_raw(Select::OPTION_ID_BASE + i as u64)
                    );
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
