#![doc = include_str!("button.doc/_module.md")]

use obscur_accessibility::semantic::SemanticRole;
use obscur_identity::{ComponentId, InternedSymbol};
use obscur_primitives::Text;
use obscur_view::view::{
    DISABLED as ATTR_DISABLED, EVENT_CLICK, LOCAL_NAME, View, ViewEventBinding, ViewFragment,
    ViewNode, ViewProperty, ViewValue,
};

#[doc = include_str!("button.doc/Button.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Button {
    label: Box<str>,
    command: InternedSymbol,
    pressed: bool,
    disabled: bool,
}

impl Button {
    #[doc = include_str!("button.doc/COMPONENT_ID.md")]
    pub const COMPONENT_ID: ComponentId = ComponentId::from_raw(0x00D0_0001);

    #[doc = include_str!("button.doc/PROP_ROLE.md")]
    const PROP_ROLE: InternedSymbol = InternedSymbol::from_raw(0x00D1_0100);
    #[doc = include_str!("button.doc/PROP_DISABLED.md")]
    const PROP_DISABLED: InternedSymbol = InternedSymbol::from_raw(0x00D1_0101);
    #[doc = include_str!("button.doc/PROP_PRESSED.md")]
    const PROP_PRESSED: InternedSymbol = InternedSymbol::from_raw(0x00D1_0102);

    #[doc = include_str!("button.doc/new.md")]
    #[must_use]
    pub fn new(label: impl Into<Box<str>>, command: InternedSymbol) -> Self {
        Self {
            label: label.into(),
            command,
            pressed: false,
            disabled: false,
        }
    }

    #[doc = include_str!("button.doc/disabled.md")]
    #[must_use]
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    #[doc = include_str!("button.doc/press.md")]
    pub fn press(&mut self) {
        if !self.disabled {
            self.pressed = true;
        }
    }

    #[doc = include_str!("button.doc/release.md")]
    pub fn release(&mut self) -> bool {
        let activated = self.pressed && !self.disabled;
        self.pressed = false;
        activated
    }

    #[doc = include_str!("button.doc/command.md")]
    #[must_use]
    pub const fn command(&self) -> InternedSymbol {
        self.command
    }

    #[doc = include_str!("button.doc/label.md")]
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    #[doc = include_str!("button.doc/is_pressed.md")]
    #[must_use]
    pub const fn is_pressed(&self) -> bool {
        self.pressed
    }

    #[doc = include_str!("button.doc/is_disabled.md")]
    #[must_use]
    pub const fn is_disabled(&self) -> bool {
        self.disabled
    }

    #[doc = include_str!("button.doc/set_disabled.md")]
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    #[doc = include_str!("button.doc/semantic_role.md")]
    #[must_use]
    pub const fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Button
    }

    #[doc = include_str!("button.doc/view.md")]
    #[must_use]
    pub fn view(&self) -> View {
        let mut node = ViewNode::element(Self::COMPONENT_ID)
            // Stable CSS type / local name so a UA (or author) `button { … }` type
            // selector matches this control (spec §4.13). Additive — it sits
            // alongside the semantic-role/state props, never replacing them.
            .with_prop(ViewProperty::new(LOCAL_NAME, ViewValue::from("button")))
            .with_prop(ViewProperty::new(
                Self::PROP_ROLE,
                ViewValue::from("button"),
            ))
            .with_prop(ViewProperty::new(
                Self::PROP_DISABLED,
                ViewValue::Bool(self.disabled),
            ))
            .with_prop(ViewProperty::new(
                Self::PROP_PRESSED,
                ViewValue::Bool(self.pressed),
            ))
            .with_event(ViewEventBinding::new(EVENT_CLICK, self.command));
        // Reserved `:disabled` structural-state attribute — emitted only when
        // disabled, so toggling it off drops the attribute and the node restyles.
        // Survives reconcile (unlike the private `PROP_DISABLED` above), feeding
        // the matcher's `:disabled` pseudo-class.
        if self.disabled {
            node = node.with_prop(ViewProperty::new(ATTR_DISABLED, ViewValue::Bool(true)));
        }
        let node = node.with_children(Self::contents(&self.label));
        View::single(node)
    }

    #[doc = include_str!("button.doc/contents.md")]
    #[must_use]
    pub(crate) fn contents(label: &str) -> ViewFragment {
        if label.is_empty() {
            return ViewFragment::new();
        }
        ViewFragment::from_nodes(vec![Text::new(label).into_view_node()])
    }

    #[doc = include_str!("button.doc/into_view_node.md")]
    #[must_use]
    pub fn into_view_node(&self) -> ViewNode {
        self.view()
            .into_single_node()
            .expect("a button always renders exactly one root node")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cmd() -> InternedSymbol {
        InternedSymbol::from_raw(0xABCD)
    }

    #[test]
    fn press_sets_pressed() {
        let mut b = Button::new("Go", cmd());
        assert!(!b.is_pressed());
        b.press();
        assert!(b.is_pressed());
    }

    #[test]
    fn release_after_press_when_enabled_activates_and_clears() {
        let mut b = Button::new("Go", cmd());
        b.press();
        assert!(b.release());
        assert!(!b.is_pressed());
    }

    #[test]
    fn release_when_disabled_returns_false() {
        let mut b = Button::new("Go", cmd()).disabled(true);
        b.press(); // no-op while disabled
        assert!(!b.is_pressed());
        assert!(!b.release());
    }

    #[test]
    fn disabled_button_can_be_re_enabled() {
        let mut b = Button::new("Go", cmd()).disabled(true);
        b.set_disabled(false);
        b.press();
        assert!(b.release());
    }

    #[test]
    fn view_emits_reserved_disabled_attr_only_when_disabled() {
        // Enabled: no reserved DISABLED attribute (so it never restyles as :disabled).
        let enabled = Button::new("Go", cmd()).view();
        let has_attr = |view: &View| match &view.root().nodes()[0] {
            ViewNode::Element { props, .. } => props.iter().any(|p| p.name() == ATTR_DISABLED),
            ViewNode::Text { .. } => panic!("expected element root"),
        };
        assert!(!has_attr(&enabled));

        // Disabled: the reserved attribute is present and truthy.
        let disabled = Button::new("Go", cmd()).disabled(true).view();
        assert!(has_attr(&disabled));
    }

    #[test]
    fn semantic_role_is_button() {
        assert_eq!(
            Button::new("Go", cmd()).semantic_role(),
            SemanticRole::Button
        );
    }

    #[test]
    fn view_root_binds_click_to_command_with_label_child() {
        let b = Button::new("Save", cmd());
        let view = b.view();
        let root = view.root();
        assert_eq!(root.len(), 1);
        match &root.nodes()[0] {
            ViewNode::Element {
                component,
                events,
                children,
                props,
                ..
            } => {
                assert_eq!(*component, Button::COMPONENT_ID);
                assert!(
                    props.iter().any(|p| p.name() == Button::PROP_PRESSED
                        && *p.value() == ViewValue::Bool(false))
                );
                assert_eq!(events.len(), 1);
                // The binding uses the RESERVED click event symbol the runtime's
                // click resolver looks up (`obscur_view::EVENT_CLICK`), not a
                // widget-local one, so a runtime click on the button dispatches
                // its command.
                assert_eq!(events[0].event(), EVENT_CLICK);
                assert_eq!(events[0].command(), cmd());
                assert_eq!(children.len(), 1);
                match &children.nodes()[0] {
                    ViewNode::Text { content, .. } => assert_eq!(&**content, "Save"),
                    ViewNode::Element { .. } => panic!("expected text label child"),
                }
            }
            ViewNode::Text { .. } => panic!("expected element root"),
        }
    }

    #[test]
    fn into_view_node_is_the_single_root_of_view() {
        // The infallible convenience is exactly `view()`'s sole node — same
        // component, same click binding — with none of the caller-side unwrapping.
        let b = Button::new("Save", cmd());
        let expected = b.view().into_single_node().expect("single root");
        assert_eq!(b.into_view_node(), expected);
    }
}
