//! `Checkbox`: a headless tri-state checkbox control (spec §2.6).

use obscur_accessibility::semantic::SemanticRole;
use obscur_identity::{ComponentId, InternedSymbol};
use obscur_primitives::Text;
use obscur_view::view::{
    CHECKED as ATTR_CHECKED, CLASS, DISABLED as ATTR_DISABLED, EVENT_CHANGE, TYPE, View,
    ViewEventBinding, ViewFragment, ViewNode, ViewProperty, ViewValue,
};

/// A headless tri-state checkbox control (spec §2.6).
///
/// `Checkbox` owns its `checked` state as an [`Option<bool>`]: `Some(true)` is
/// checked, `Some(false)` is unchecked, and `None` is the *mixed* /
/// indeterminate state a parent can hold while its descendants disagree. It
/// bakes in **no** visual theme — [`view`](Self::view) derives a
/// [`SemanticRole::Checkbox`] element whose checked state is encoded as a
/// tri-state integer property (see [`view`](Self::view)) plus a change event
/// bound to `command` and a single [`Text`] label child, rendered by the style
/// layer keyed on [`COMPONENT_ID`](Self::COMPONENT_ID).
///
/// The state machine is [`toggle`](Self::toggle): disabled is a no-op, `None`
/// (mixed) advances to `Some(true)`, and a definite value flips.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checkbox {
    label: Box<str>,
    command: InternedSymbol,
    checked: Option<bool>,
    disabled: bool,
}

impl Checkbox {
    /// Stable component id for the checkbox widget (see [`Button::COMPONENT_ID`]
    /// for the shared `0x00D0_00NN` scheme).
    ///
    /// [`Button::COMPONENT_ID`]: crate::button::Button::COMPONENT_ID
    pub const COMPONENT_ID: ComponentId = ComponentId::from_raw(0x00D0_0003);

    /// Component id of the boxed indicator sub-part (the 20×20 `.checkbox-box`).
    /// Shares the widget id space with the high nibble of the suffix flipped
    /// (`..13`), mirroring the menu/table sub-part scheme.
    const BOX_ID: ComponentId = ComponentId::from_raw(0x00D0_0013);
    /// Component id of the indeterminate bar sub-part (`.checkbox-bar`).
    const BAR_ID: ComponentId = ComponentId::from_raw(0x00D0_0023);

    /// Interned name of the semantic-role property (`role`); shares the
    /// `0x00D1_NNXX` scheme with `NN = 03` (checkbox): role=`00`, disabled=`01`,
    /// checked=`02`.
    const PROP_ROLE: InternedSymbol = InternedSymbol::from_raw(0x00D1_0300);
    /// Interned name of the `disabled` state property.
    const PROP_DISABLED: InternedSymbol = InternedSymbol::from_raw(0x00D1_0301);
    /// Interned name of the tri-state `checked` property (see [`view`](Self::view)).
    const PROP_CHECKED: InternedSymbol = InternedSymbol::from_raw(0x00D1_0302);

    /// Creates an enabled, *unchecked* (`Some(false)`) checkbox showing `label`
    /// that fires `command` on change.
    #[must_use]
    pub fn new(label: impl Into<Box<str>>, command: InternedSymbol) -> Self {
        Self {
            label: label.into(),
            command,
            checked: Some(false),
            disabled: false,
        }
    }

    /// Creates an enabled *indeterminate* (`None`, mixed) checkbox showing
    /// `label` that fires `command` on change.
    #[must_use]
    pub fn indeterminate(label: impl Into<Box<str>>, command: InternedSymbol) -> Self {
        Self {
            label: label.into(),
            command,
            checked: None,
            disabled: false,
        }
    }

    /// Sets the definite checked state, returning the updated checkbox (fluent).
    #[must_use]
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    /// Sets the disabled state, returning the updated checkbox (fluent).
    #[must_use]
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Advances the tri-state: a no-op while disabled; `None` (mixed) resolves
    /// to `Some(true)`; a definite value flips.
    pub fn toggle(&mut self) {
        if self.disabled {
            return;
        }
        self.checked = match self.checked {
            None => Some(true),
            Some(value) => Some(!value),
        };
    }

    /// Returns whether the checkbox is definitely checked (`Some(true)`).
    #[must_use]
    pub const fn is_checked(&self) -> bool {
        matches!(self.checked, Some(true))
    }

    /// Returns whether the checkbox is in the mixed / indeterminate state.
    #[must_use]
    pub const fn is_indeterminate(&self) -> bool {
        self.checked.is_none()
    }

    /// Returns the raw tri-state checked value.
    #[must_use]
    pub const fn checked_state(&self) -> Option<bool> {
        self.checked
    }

    /// Sets the definite checked state in place.
    pub fn set_checked(&mut self, checked: bool) {
        self.checked = Some(checked);
    }

    /// Returns the command fired when the checkbox changes.
    #[must_use]
    pub const fn command(&self) -> InternedSymbol {
        self.command
    }

    /// Returns the checkbox label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns whether the checkbox is disabled.
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
        SemanticRole::Checkbox
    }

    /// Derives the semantic view: a [`COMPONENT_ID`] element carrying the role
    /// and disabled state, the tri-state checked value as an integer property
    /// (`-1` = mixed/`None`, `0` = unchecked, `1` = checked), a change event
    /// bound to `command`, and — as children — the boxed indicator sub-part
    /// followed by the label [`Text`].
    ///
    /// The indicator is a `.checkbox-box` element the UA/theme sheet fills; it
    /// carries a state class (`checked` when definitely on, `indeterminate` when
    /// mixed) so a `.checkbox-box.checked { … }` rule (or a
    /// `checkbox:hover .checkbox-box { … }` descendant rule) can key on it. Its
    /// child is the tick [`Text`] "✓" when checked, or a thin `.checkbox-bar`
    /// element when mixed; nothing when unchecked. The interactive state
    /// (`EVENT_CHANGE`, reserved `:checked`/`:disabled` attrs) stays on the OUTER
    /// control element, so a click on the box bubbles to it and pseudo-classes
    /// still drive the whole control.
    ///
    /// [`COMPONENT_ID`]: Self::COMPONENT_ID
    #[must_use]
    pub fn view(&self) -> View {
        let checked = match self.checked {
            None => -1,
            Some(false) => 0,
            Some(true) => 1,
        };
        let mut node = ViewNode::element(Self::COMPONENT_ID)
            // Stable CSS type / local name so a UA (or author) `checkbox { … }`
            // type selector matches this control (spec §4.13). Additive.
            .with_prop(ViewProperty::new(TYPE, ViewValue::from("checkbox")))
            .with_prop(ViewProperty::new(
                Self::PROP_ROLE,
                ViewValue::from("checkbox"),
            ))
            .with_prop(ViewProperty::new(
                Self::PROP_DISABLED,
                ViewValue::Bool(self.disabled),
            ))
            .with_prop(ViewProperty::new(
                Self::PROP_CHECKED,
                ViewValue::Int(checked),
            ))
            .with_event(ViewEventBinding::new(EVENT_CHANGE, self.command));
        // Reserved structural-state attributes — emitted only when set, so a
        // toggle drops them and the node restyles. These survive reconcile
        // (unlike the private props above) and feed the matcher's `:checked` /
        // `:disabled` pseudo-classes. Only a *definite* checked value is
        // `:checked`; the mixed (indeterminate) state is not.
        if self.is_checked() {
            node = node.with_prop(ViewProperty::new(ATTR_CHECKED, ViewValue::Bool(true)));
        }
        if self.disabled {
            node = node.with_prop(ViewProperty::new(ATTR_DISABLED, ViewValue::Bool(true)));
        }
        // The boxed indicator sub-part: a `.checkbox-box` element carrying a
        // state class, with the tick / bar as its child.
        let mut box_class = String::from("checkbox-box");
        let box_child: Vec<ViewNode> = if self.is_checked() {
            box_class.push_str(" checked");
            vec![Text::new("✓").into_view_node()]
        } else if self.is_indeterminate() {
            box_class.push_str(" indeterminate");
            vec![
                ViewNode::element(Self::BAR_ID)
                    .with_prop(ViewProperty::new(CLASS, ViewValue::from("checkbox-bar"))),
            ]
        } else {
            Vec::new()
        };
        let indicator = ViewNode::element(Self::BOX_ID)
            .with_prop(ViewProperty::new(CLASS, ViewValue::from(box_class)))
            .with_children(ViewFragment::from_nodes(box_child));
        let node = node.with_children(ViewFragment::from_nodes(vec![
            indicator,
            Text::new(self.label.clone()).into_view_node(),
        ]));
        View::single(node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cmd() -> InternedSymbol {
        InternedSymbol::from_raw(0x1234)
    }

    #[test]
    fn toggle_flips_definite_state() {
        let mut c = Checkbox::new("A", cmd());
        assert!(!c.is_checked());
        c.toggle();
        assert!(c.is_checked());
        c.toggle();
        assert!(!c.is_checked());
    }

    #[test]
    fn indeterminate_toggles_to_checked() {
        let mut c = Checkbox::indeterminate("A", cmd());
        assert!(c.is_indeterminate());
        c.toggle();
        assert!(c.is_checked());
        assert!(!c.is_indeterminate());
    }

    #[test]
    fn disabled_toggle_is_noop() {
        let mut c = Checkbox::new("A", cmd()).disabled(true);
        c.toggle();
        assert!(!c.is_checked());
        assert_eq!(c.checked_state(), Some(false));
    }

    #[test]
    fn is_checked_reflects_state() {
        assert!(Checkbox::new("A", cmd()).checked(true).is_checked());
        assert!(!Checkbox::new("A", cmd()).checked(false).is_checked());
        assert!(!Checkbox::indeterminate("A", cmd()).is_checked());
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

        // Unchecked + enabled: neither reserved attribute.
        assert_eq!(attrs(&Checkbox::new("A", cmd()).view()), (false, false));
        // Checked: the CHECKED attribute appears.
        assert_eq!(
            attrs(&Checkbox::new("A", cmd()).checked(true).view()),
            (true, false)
        );
        // Indeterminate is NOT :checked.
        assert_eq!(
            attrs(&Checkbox::indeterminate("A", cmd()).view()),
            (false, false)
        );
        // Checked AND disabled: both appear.
        assert_eq!(
            attrs(
                &Checkbox::new("A", cmd())
                    .checked(true)
                    .disabled(true)
                    .view()
            ),
            (true, true)
        );
    }

    #[test]
    fn semantic_role_is_checkbox() {
        assert_eq!(
            Checkbox::new("A", cmd()).semantic_role(),
            SemanticRole::Checkbox
        );
    }

    #[test]
    fn view_emits_indicator_box_with_state_class() {
        // The first child is the `.checkbox-box` indicator; its class reflects
        // the tri-state, and its own child is the tick / bar / nothing.
        let box_of = |view: &View| -> (Vec<String>, usize) {
            match &view.root().nodes()[0] {
                ViewNode::Element { children, .. } => match &children.nodes()[0] {
                    ViewNode::Element {
                        props, children, ..
                    } => {
                        let classes = props
                            .iter()
                            .find(|p| p.name() == CLASS)
                            .map(|p| match p.value() {
                                ViewValue::Text(t) => {
                                    t.split_ascii_whitespace().map(String::from).collect()
                                }
                                _ => Vec::new(),
                            })
                            .unwrap_or_default();
                        (classes, children.nodes().len())
                    }
                    ViewNode::Text { .. } => panic!("expected box element as first child"),
                },
                ViewNode::Text { .. } => panic!("expected element root"),
            }
        };

        // Unchecked: bare box, no inner tick.
        let (classes, kids) = box_of(&Checkbox::new("A", cmd()).view());
        assert_eq!(classes, ["checkbox-box"]);
        assert_eq!(kids, 0);
        // Checked: `checked` state class + a single tick child.
        let (classes, kids) = box_of(&Checkbox::new("A", cmd()).checked(true).view());
        assert_eq!(classes, ["checkbox-box", "checked"]);
        assert_eq!(kids, 1);
        // Indeterminate: `indeterminate` state class + a single bar child.
        let (classes, kids) = box_of(&Checkbox::indeterminate("A", cmd()).view());
        assert_eq!(classes, ["checkbox-box", "indeterminate"]);
        assert_eq!(kids, 1);
    }

    #[test]
    fn view_encodes_tri_state_checked_prop() {
        let mixed = Checkbox::indeterminate("A", cmd());
        let view = mixed.view();
        match &view.root().nodes()[0] {
            ViewNode::Element {
                component,
                props,
                events,
                children,
                ..
            } => {
                assert_eq!(*component, Checkbox::COMPONENT_ID);
                let checked = props
                    .iter()
                    .find(|p| p.name() == Checkbox::PROP_CHECKED)
                    .expect("checked prop present");
                assert_eq!(*checked.value(), ViewValue::Int(-1));
                assert_eq!(events[0].command(), cmd());
                // Two children now: the boxed indicator sub-part + the label
                // Text (was a single label child before the wireframe indicator
                // landed).
                assert_eq!(children.len(), 2);
            }
            ViewNode::Text { .. } => panic!("expected element root"),
        }

        // A definite checked value encodes to 1.
        let on = Checkbox::new("A", cmd()).checked(true).view();
        match &on.root().nodes()[0] {
            ViewNode::Element { props, .. } => {
                let checked = props
                    .iter()
                    .find(|p| p.name() == Checkbox::PROP_CHECKED)
                    .expect("checked prop present");
                assert_eq!(*checked.value(), ViewValue::Int(1));
            }
            ViewNode::Text { .. } => panic!("expected element root"),
        }
    }
}
