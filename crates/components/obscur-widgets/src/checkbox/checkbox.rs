#![doc = include_str!("checkbox.doc/_module.md")]

use obscur_accessibility::semantic::SemanticRole;
use obscur_identity::{ComponentId, InternedSymbol};
use obscur_primitives::Text;
use obscur_view::view::{
    CHECKED as ATTR_CHECKED, CLASS, DISABLED as ATTR_DISABLED, EVENT_CHANGE, LOCAL_NAME, View,
    ViewEventBinding, ViewFragment, ViewNode, ViewProperty, ViewValue,
};

use super::CheckboxProps;

#[doc = include_str!("checkbox.doc/Checkbox.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checkbox {
    label: Box<str>,
    command: InternedSymbol,
    checked: Option<bool>,
    disabled: bool,
}

impl Checkbox {
    #[doc = include_str!("checkbox.doc/COMPONENT_ID.md")]
    pub const COMPONENT_ID: ComponentId = ComponentId::from_raw(0x00D0_0003);

    #[doc = include_str!("checkbox.doc/BOX_ID.md")]
    const BOX_ID: ComponentId = ComponentId::from_raw(0x00D0_0013);
    #[doc = include_str!("checkbox.doc/BAR_ID.md")]
    const BAR_ID: ComponentId = ComponentId::from_raw(0x00D0_0023);

    #[doc = include_str!("checkbox.doc/PROP_ROLE.md")]
    const PROP_ROLE: InternedSymbol = InternedSymbol::from_raw(0x00D1_0300);
    #[doc = include_str!("checkbox.doc/PROP_DISABLED.md")]
    const PROP_DISABLED: InternedSymbol = InternedSymbol::from_raw(0x00D1_0301);
    #[doc = include_str!("checkbox.doc/PROP_CHECKED.md")]
    pub(crate) const PROP_CHECKED: InternedSymbol = InternedSymbol::from_raw(0x00D1_0302);

    #[doc = include_str!("checkbox.doc/new.md")]
    #[must_use]
    pub fn new(label: impl Into<Box<str>>, command: InternedSymbol) -> Self {
        Self {
            label: label.into(),
            command,
            checked: Some(false),
            disabled: false,
        }
    }

    #[doc = include_str!("checkbox.doc/indeterminate.md")]
    #[must_use]
    pub fn indeterminate(label: impl Into<Box<str>>, command: InternedSymbol) -> Self {
        Self {
            label: label.into(),
            command,
            checked: None,
            disabled: false,
        }
    }

    #[doc = include_str!("checkbox.doc/checked.md")]
    #[must_use]
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    #[doc = include_str!("checkbox.doc/disabled.md")]
    #[must_use]
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    #[doc = include_str!("checkbox.doc/toggle.md")]
    pub fn toggle(&mut self) {
        if self.disabled {
            return;
        }
        self.checked = match self.checked {
            None => Some(true),
            Some(value) => Some(!value),
        };
    }

    #[doc = include_str!("checkbox.doc/is_checked.md")]
    #[must_use]
    pub const fn is_checked(&self) -> bool {
        matches!(self.checked, Some(true))
    }

    #[doc = include_str!("checkbox.doc/is_indeterminate.md")]
    #[must_use]
    pub const fn is_indeterminate(&self) -> bool {
        self.checked.is_none()
    }

    #[doc = include_str!("checkbox.doc/checked_state.md")]
    #[must_use]
    pub const fn checked_state(&self) -> Option<bool> {
        self.checked
    }

    #[doc = include_str!("checkbox.doc/set_checked.md")]
    pub fn set_checked(&mut self, checked: bool) {
        self.checked = Some(checked);
    }

    #[doc = include_str!("checkbox.doc/command.md")]
    #[must_use]
    pub const fn command(&self) -> InternedSymbol {
        self.command
    }

    #[doc = include_str!("checkbox.doc/label.md")]
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    #[doc = include_str!("checkbox.doc/is_disabled.md")]
    #[must_use]
    pub const fn is_disabled(&self) -> bool {
        self.disabled
    }

    #[doc = include_str!("checkbox.doc/set_disabled.md")]
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    #[doc = include_str!("checkbox.doc/semantic_role.md")]
    #[must_use]
    pub const fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Checkbox
    }

    #[doc = include_str!("checkbox.doc/view.md")]
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
            .with_prop(ViewProperty::new(LOCAL_NAME, ViewValue::from("checkbox")))
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
            // The authored `label` attribute, emitted by the Rust builder too, so
            // this node and an authored `<checkbox label="…"/>` carry the SAME
            // props. That is what makes mounting `CheckboxComponent` on a node
            // this method built idempotent: the component decodes these props and
            // regenerates exactly the children installed below.
            .with_prop(ViewProperty::new(
                CheckboxProps::LABEL,
                ViewValue::from(&*self.label),
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
        let node = node.with_children(Self::contents(self.checked, &self.label));
        View::single(node)
    }

    #[doc = include_str!("checkbox.doc/contents.md")]
    #[must_use]
    pub(crate) fn contents(checked: Option<bool>, label: &str) -> ViewFragment {
        // The boxed indicator sub-part: a `.checkbox-box` element carrying a
        // state class, with the tick / bar as its child.
        let mut box_class = String::from("checkbox-box");
        let box_child: Vec<ViewNode> = match checked {
            Some(true) => {
                box_class.push_str(" checked");
                vec![Text::new("✓").into_view_node()]
            }
            None => {
                box_class.push_str(" indeterminate");
                vec![
                    ViewNode::element(Self::BAR_ID)
                        .with_prop(ViewProperty::new(CLASS, ViewValue::from("checkbox-bar"))),
                ]
            }
            Some(false) => Vec::new(),
        };
        let indicator = ViewNode::element(Self::BOX_ID)
            .with_prop(ViewProperty::new(CLASS, ViewValue::from(box_class)))
            .with_children(ViewFragment::from_nodes(box_child));
        ViewFragment::from_nodes(vec![indicator, Text::new(label).into_view_node()])
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
