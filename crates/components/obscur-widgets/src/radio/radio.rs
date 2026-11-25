//! `Radio`: a headless two-state radio-button control (spec §2.6).

use obscur_accessibility::semantic::SemanticRole;
use obscur_identity::{ComponentId, InternedSymbol};
use obscur_primitives::Text;
use obscur_view::view::{
    CHECKED as ATTR_CHECKED, CLASS, DISABLED as ATTR_DISABLED, EVENT_CHANGE, LOCAL_NAME, View,
    ViewEventBinding, ViewFragment, ViewNode, ViewProperty, ViewValue,
};

use super::RadioProps;

#[doc = include_str!("../../docs/api/radio/radio.doc/Radio.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Radio {
    label: Box<str>,
    command: InternedSymbol,
    selected: bool,
    disabled: bool,
}

impl Radio {
    #[doc = include_str!("../../docs/api/radio/radio.doc/COMPONENT_ID.md")]
    pub const COMPONENT_ID: ComponentId = ComponentId::from_raw(0x00D0_000C);

    /// Component id of the circular indicator sub-part (`.radio-dot`), in the
    /// widget sub-part id space (`..1C`).
    const DOT_ID: ComponentId = ComponentId::from_raw(0x00D0_001C);
    /// Component id of the filled centre dot sub-part (`.radio-fill`, `..2C`).
    const FILL_ID: ComponentId = ComponentId::from_raw(0x00D0_002C);

    /// Interned name of the semantic-role property (`role`); shares the
    /// `0x00D1_NNXX` scheme with `NN = 0C` (radio): role=`00`, disabled=`01`,
    /// selected=`02`.
    const PROP_ROLE: InternedSymbol = InternedSymbol::from_raw(0x00D1_0C00);
    /// Interned name of the `disabled` state property.
    const PROP_DISABLED: InternedSymbol = InternedSymbol::from_raw(0x00D1_0C01);
    /// Interned name of the `selected` state property.
    pub(crate) const PROP_SELECTED: InternedSymbol = InternedSymbol::from_raw(0x00D1_0C02);

    /// Creates an enabled, *unselected* radio showing `label` that fires
    /// `command` on change.
    #[must_use]
    pub fn new(label: impl Into<Box<str>>, command: InternedSymbol) -> Self {
        Self {
            label: label.into(),
            command,
            selected: false,
            disabled: false,
        }
    }

    /// Sets the selected state, returning the updated radio (fluent builder).
    #[must_use]
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Sets the disabled state, returning the updated radio (fluent builder).
    #[must_use]
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Selects this radio (a no-op while disabled). Deselection is the group's
    /// concern — a radio does not deselect itself on re-activation.
    pub fn select(&mut self) {
        if !self.disabled {
            self.selected = true;
        }
    }

    /// Toggles the selected state (a no-op while disabled). Mirrors
    /// [`Checkbox::toggle`](crate::checkbox::Checkbox::toggle) for symmetry;
    /// most groups will call [`select`](Self::select) instead.
    pub fn toggle(&mut self) {
        if !self.disabled {
            self.selected = !self.selected;
        }
    }

    /// Returns whether this radio is selected.
    #[must_use]
    pub const fn is_selected(&self) -> bool {
        self.selected
    }

    /// Sets the selected state in place.
    pub fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    /// Returns the command fired when the radio changes.
    #[must_use]
    pub const fn command(&self) -> InternedSymbol {
        self.command
    }

    /// Returns the radio label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns whether the radio is disabled.
    #[must_use]
    pub const fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Sets the disabled state in place.
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    /// Returns the accessible role of this control.
    #[must_use]
    pub const fn semantic_role(&self) -> SemanticRole {
        SemanticRole::RadioButton
    }

    #[doc = include_str!("../../docs/api/radio/radio.doc/view.md")]
    #[must_use]
    pub fn view(&self) -> View {
        let mut node = ViewNode::element(Self::COMPONENT_ID)
            // Stable CSS type / local name so a UA (or author) `radio { … }`
            // type selector matches this control (spec §4.13). Additive.
            .with_prop(ViewProperty::new(LOCAL_NAME, ViewValue::from("radio")))
            .with_prop(ViewProperty::new(Self::PROP_ROLE, ViewValue::from("radio")))
            .with_prop(ViewProperty::new(
                Self::PROP_DISABLED,
                ViewValue::Bool(self.disabled),
            ))
            .with_prop(ViewProperty::new(
                Self::PROP_SELECTED,
                ViewValue::Bool(self.selected),
            ))
            // The authored `label` attribute, emitted by the Rust builder too, so
            // this node and an authored `<radio label="…"/>` carry the SAME props.
            // That is what makes mounting `RadioComponent` on a node this method
            // built idempotent: the component decodes these props and regenerates
            // exactly the children installed below.
            .with_prop(ViewProperty::new(
                RadioProps::LABEL,
                ViewValue::from(&*self.label),
            ))
            .with_event(ViewEventBinding::new(EVENT_CHANGE, self.command));
        // Reserved structural-state attributes — emitted only when set, feeding
        // the matcher's `:checked` / `:disabled` pseudo-classes and surviving
        // reconcile.
        if self.selected {
            node = node.with_prop(ViewProperty::new(ATTR_CHECKED, ViewValue::Bool(true)));
        }
        if self.disabled {
            node = node.with_prop(ViewProperty::new(ATTR_DISABLED, ViewValue::Bool(true)));
        }
        let node = node.with_children(Self::contents(self.selected, &self.label));
        View::single(node)
    }

    #[doc = include_str!("../../docs/api/radio/radio.doc/contents.md")]
    #[must_use]
    pub(crate) fn contents(selected: bool, label: &str) -> ViewFragment {
        // The circular indicator sub-part: a `.radio-dot`, with the filled
        // `.radio-fill` centre present only when selected.
        let dot_child: Vec<ViewNode> = if selected {
            vec![
                ViewNode::element(Self::FILL_ID)
                    .with_prop(ViewProperty::new(CLASS, ViewValue::from("radio-fill"))),
            ]
        } else {
            Vec::new()
        };
        let indicator = ViewNode::element(Self::DOT_ID)
            .with_prop(ViewProperty::new(CLASS, ViewValue::from("radio-dot")))
            .with_children(ViewFragment::from_nodes(dot_child));
        ViewFragment::from_nodes(vec![indicator, Text::new(label).into_view_node()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cmd() -> InternedSymbol {
        InternedSymbol::from_raw(0x5678)
    }

    #[test]
    fn select_sets_selected_and_respects_disabled() {
        let mut r = Radio::new("A", cmd());
        assert!(!r.is_selected());
        r.select();
        assert!(r.is_selected());

        let mut d = Radio::new("B", cmd()).disabled(true);
        d.select();
        assert!(!d.is_selected());
    }

    #[test]
    fn toggle_flips_and_is_noop_when_disabled() {
        let mut r = Radio::new("A", cmd());
        r.toggle();
        assert!(r.is_selected());
        r.toggle();
        assert!(!r.is_selected());

        let mut d = Radio::new("B", cmd()).selected(true).disabled(true);
        d.toggle();
        assert!(d.is_selected());
    }

    #[test]
    fn semantic_role_is_radio_button() {
        assert_eq!(
            Radio::new("A", cmd()).semantic_role(),
            SemanticRole::RadioButton
        );
    }

    #[test]
    fn view_emits_reserved_state_attrs_only_when_set() {
        let attrs = |view: &View| -> (bool, bool) {
            match &view.root().nodes()[0] {
                ViewNode::Element { props, .. } => (
                    props.iter().any(|p| p.name() == ATTR_CHECKED),
                    props.iter().any(|p| p.name() == ATTR_DISABLED),
                ),
                ViewNode::Text { .. } => panic!("expected element root"),
            }
        };
        assert_eq!(attrs(&Radio::new("A", cmd()).view()), (false, false));
        assert_eq!(
            attrs(&Radio::new("A", cmd()).selected(true).view()),
            (true, false)
        );
        assert_eq!(
            attrs(&Radio::new("A", cmd()).selected(true).disabled(true).view()),
            (true, true)
        );
    }

    #[test]
    fn view_emits_dot_with_fill_only_when_selected() {
        // First child is the `.radio-dot`; its inner `.radio-fill` appears only
        // when selected.
        let dot_kids = |view: &View| -> usize {
            match &view.root().nodes()[0] {
                ViewNode::Element { children, .. } => match &children.nodes()[0] {
                    ViewNode::Element {
                        props, children, ..
                    } => {
                        assert!(props.iter().any(|p| p.name() == CLASS
                            && matches!(p.value(), ViewValue::Text(t) if &**t == "radio-dot")));
                        children.nodes().len()
                    }
                    ViewNode::Text { .. } => panic!("expected dot element as first child"),
                },
                ViewNode::Text { .. } => panic!("expected element root"),
            }
        };
        assert_eq!(dot_kids(&Radio::new("A", cmd()).view()), 0);
        assert_eq!(dot_kids(&Radio::new("A", cmd()).selected(true).view()), 1);
    }

    #[test]
    fn view_has_indicator_and_label_children() {
        let view = Radio::new("Opt", cmd()).view();
        match &view.root().nodes()[0] {
            ViewNode::Element {
                component,
                events,
                children,
                ..
            } => {
                assert_eq!(*component, Radio::COMPONENT_ID);
                assert_eq!(events[0].command(), cmd());
                assert_eq!(children.nodes().len(), 2);
            }
            ViewNode::Text { .. } => panic!("expected element root"),
        }
    }
}
