//! `Button`: a headless push-button control (spec §2.6).

use obscur_accessibility::semantic::SemanticRole;
use obscur_identity::{ComponentId, InternedSymbol};
use obscur_primitives::Text;
use obscur_view::view::{View, ViewEventBinding, ViewFragment, ViewNode, ViewProperty, ViewValue};

/// A headless push-button control (spec §2.6).
///
/// `Button` owns its interactive state — whether it is currently
/// [`pressed`](Self::is_pressed) and whether it is
/// [`disabled`](Self::is_disabled) — plus the `command` it fires when
/// activated. It bakes in **no** visual theme: [`view`](Self::view) derives a
/// semantic [`ViewNode`] tree (a [`SemanticRole::Button`] element carrying its
/// state as properties, a click event bound to `command`, and a single
/// [`Text`] label child) that the style layer renders keyed on
/// [`COMPONENT_ID`](Self::COMPONENT_ID).
///
/// The state machine is a plain press/release cycle: [`press`](Self::press)
/// arms the button and [`release`](Self::release) reports whether that cycle
/// *activated* it (pressed while enabled). A disabled button neither arms nor
/// activates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Button {
    label: Box<str>,
    command: InternedSymbol,
    pressed: bool,
    disabled: bool,
}

impl Button {
    /// Stable component id for the button widget.
    ///
    /// Widgets share the `0x00D0_00NN` id space, one fixed value per widget so
    /// the style/reconcile layer can map them without a name lookup:
    /// button=`..01`, input=`..02`, checkbox=`..03`, select=`..04`.
    pub const COMPONENT_ID: ComponentId = ComponentId::from_raw(0x00D0_0001);

    /// Interned name of the semantic-role property (`role`).
    ///
    /// Widget property names share the `0x00D1_NNXX` symbol space, where `NN`
    /// matches the widget's `COMPONENT_ID` suffix and `XX` numbers the property:
    /// role=`00`, disabled=`01`, pressed=`02`.
    const PROP_ROLE: InternedSymbol = InternedSymbol::from_raw(0x00D1_0100);
    /// Interned name of the `disabled` state property.
    const PROP_DISABLED: InternedSymbol = InternedSymbol::from_raw(0x00D1_0101);
    /// Interned name of the `pressed` state property.
    const PROP_PRESSED: InternedSymbol = InternedSymbol::from_raw(0x00D1_0102);

    /// Interned name of the click event a button reports when activated.
    const EVENT_CLICK: InternedSymbol = InternedSymbol::from_raw(0x00D2_0001);

    /// Creates an enabled, un-pressed button showing `label` that fires
    /// `command` on activation.
    #[must_use]
    pub fn new(label: impl Into<Box<str>>, command: InternedSymbol) -> Self {
        Self {
            label: label.into(),
            command,
            pressed: false,
            disabled: false,
        }
    }

    /// Sets the disabled state, returning the updated button (fluent builder).
    #[must_use]
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Arms the button. A no-op while [`disabled`](Self::is_disabled).
    pub fn press(&mut self) {
        if !self.disabled {
            self.pressed = true;
        }
    }

    /// Ends a press cycle, clearing the pressed state and returning whether the
    /// button was *activated* — i.e. it had been pressed and is enabled.
    pub fn release(&mut self) -> bool {
        let activated = self.pressed && !self.disabled;
        self.pressed = false;
        activated
    }

    /// Returns the command fired when the button is activated.
    #[must_use]
    pub const fn command(&self) -> InternedSymbol {
        self.command
    }

    /// Returns the button label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns whether the button is currently armed.
    #[must_use]
    pub const fn is_pressed(&self) -> bool {
        self.pressed
    }

    /// Returns whether the button is disabled.
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
        SemanticRole::Button
    }

    /// Derives the semantic view for this button: a [`COMPONENT_ID`] element
    /// carrying the role, disabled, and pressed state as properties, a click
    /// event bound to `command`, and the label as a single [`Text`] child.
    ///
    /// [`COMPONENT_ID`]: Self::COMPONENT_ID
    #[must_use]
    pub fn view(&self) -> View {
        let node = ViewNode::element(Self::COMPONENT_ID)
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
            .with_event(ViewEventBinding::new(Self::EVENT_CLICK, self.command))
            .with_children(ViewFragment::from_nodes(vec![
                Text::new(self.label.clone()).into_view_node(),
            ]));
        View::single(node)
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
}
