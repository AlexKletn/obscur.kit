//! `Input`: a headless single-line text-input control (spec §2.6, §5.11).

use obscur_accessibility::semantic::SemanticRole;
use obscur_identity::{ComponentId, InternedSymbol};
use obscur_text::editing::{EditCommand, TextBuffer};
use obscur_view::view::{
    EVENT_CHANGE, EVENT_INPUT, View, ViewEventBinding, ViewNode, ViewProperty, ViewValue,
};

/// A headless text-input control (spec §2.6).
///
/// `Input` owns a [`TextBuffer`] — the pure editing engine of spec §5.11 — as
/// its text state, plus a `placeholder`, the `command` fired on change, and a
/// disabled flag. Text edits flow in as [`EditCommand`]s through
/// [`apply`](Self::apply), which delegates to the buffer; this keeps the input
/// layout-free and lets a platform keymap / IME drive it with the same command
/// vocabulary the engine already understands.
///
/// It bakes in **no** visual theme: [`view`](Self::view) derives a
/// [`SemanticRole::TextInput`] element carrying the current value and
/// placeholder as text properties and a change event bound to `command`,
/// rendered by the style layer keyed on [`COMPONENT_ID`](Self::COMPONENT_ID).
///
/// Because [`TextBuffer`] holds a `String` and undo history it is neither
/// `Copy` nor `Eq`, so `Input` derives only `Debug`, `Clone`, and `PartialEq`.
#[derive(Debug, Clone, PartialEq)]
pub struct Input {
    buffer: TextBuffer,
    placeholder: Box<str>,
    command: InternedSymbol,
    disabled: bool,
}

impl Input {
    /// Stable component id for the input widget (see [`Button::COMPONENT_ID`]
    /// for the shared `0x00D0_00NN` scheme).
    ///
    /// [`Button::COMPONENT_ID`]: crate::button::Button::COMPONENT_ID
    pub const COMPONENT_ID: ComponentId = ComponentId::from_raw(0x00D0_0002);

    /// Interned name of the semantic-role property (`role`); shares the
    /// `0x00D1_NNXX` scheme with `NN = 02` (input): role=`00`, disabled=`01`,
    /// value=`02`, placeholder=`03`.
    const PROP_ROLE: InternedSymbol = InternedSymbol::from_raw(0x00D1_0200);
    /// Interned name of the `disabled` state property.
    const PROP_DISABLED: InternedSymbol = InternedSymbol::from_raw(0x00D1_0201);
    /// Interned name of the current `value` property.
    const PROP_VALUE: InternedSymbol = InternedSymbol::from_raw(0x00D1_0202);
    /// Interned name of the `placeholder` property.
    const PROP_PLACEHOLDER: InternedSymbol = InternedSymbol::from_raw(0x00D1_0203);


    /// Creates an empty, enabled input with no placeholder that fires `command`
    /// on change.
    #[must_use]
    pub fn new(command: InternedSymbol) -> Self {
        Self {
            buffer: TextBuffer::new(),
            placeholder: Box::<str>::from(""),
            command,
            disabled: false,
        }
    }

    /// Creates an input seeded with `value` that fires `command` on change.
    #[must_use]
    pub fn with_value(command: InternedSymbol, value: impl Into<String>) -> Self {
        Self {
            buffer: TextBuffer::from_text(value),
            placeholder: Box::<str>::from(""),
            command,
            disabled: false,
        }
    }

    /// Sets the placeholder text, returning the updated input (fluent builder).
    #[must_use]
    pub fn placeholder(mut self, placeholder: impl Into<Box<str>>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Sets the disabled state, returning the updated input (fluent builder).
    #[must_use]
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Returns the current text value.
    #[must_use]
    pub fn value(&self) -> &str {
        self.buffer.text()
    }

    /// Returns the placeholder text.
    #[must_use]
    pub fn placeholder_text(&self) -> &str {
        &self.placeholder
    }

    /// Applies an [`EditCommand`] to the underlying buffer — the way all text
    /// edits (typing, deletion, navigation, IME/programmatic edits) flow in. A
    /// no-op while [`disabled`](Self::is_disabled).
    pub fn apply(&mut self, command: EditCommand) {
        if !self.disabled {
            self.buffer.apply(command);
        }
    }

    /// Returns a shared reference to the underlying editing buffer.
    #[must_use]
    pub const fn buffer(&self) -> &TextBuffer {
        &self.buffer
    }

    /// Returns the command fired when the value changes.
    #[must_use]
    pub const fn command(&self) -> InternedSymbol {
        self.command
    }

    /// Returns whether the input is disabled.
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
        SemanticRole::TextInput
    }

    /// Derives the semantic view: a [`COMPONENT_ID`] element carrying the role,
    /// disabled state, current value, and placeholder as properties, plus two
    /// event bindings to `command`: a per-keystroke [`EVENT_INPUT`] (each typed
    /// character / `Backspace` delivers a
    /// [`TextEdit`](obscur_interaction::command::TextEdit) to the app) and a
    /// commit-on-blur/Enter [`EVENT_CHANGE`]. An input has no child structure —
    /// its text is carried as the value property, not a text node.
    ///
    /// [`COMPONENT_ID`]: Self::COMPONENT_ID
    #[must_use]
    pub fn view(&self) -> View {
        let node = ViewNode::element(Self::COMPONENT_ID)
            .with_prop(ViewProperty::new(
                Self::PROP_ROLE,
                ViewValue::from("textbox"),
            ))
            .with_prop(ViewProperty::new(
                Self::PROP_DISABLED,
                ViewValue::Bool(self.disabled),
            ))
            .with_prop(ViewProperty::new(
                Self::PROP_VALUE,
                ViewValue::from(self.buffer.text()),
            ))
            .with_prop(ViewProperty::new(
                Self::PROP_PLACEHOLDER,
                ViewValue::from(&*self.placeholder),
            ))
            .with_event(ViewEventBinding::new(EVENT_INPUT, self.command))
            .with_event(ViewEventBinding::new(EVENT_CHANGE, self.command));
        View::single(node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cmd() -> InternedSymbol {
        InternedSymbol::from_raw(0x5678)
    }

    #[test]
    fn with_value_exposes_value() {
        let i = Input::with_value(cmd(), "hello");
        assert_eq!(i.value(), "hello");
    }

    #[test]
    fn apply_insert_changes_value() {
        let mut i = Input::new(cmd());
        i.apply(EditCommand::insert("hi"));
        assert_eq!(i.value(), "hi");
    }

    #[test]
    fn disabled_apply_is_noop() {
        let mut i = Input::with_value(cmd(), "seed").disabled(true);
        i.apply(EditCommand::insert("more"));
        assert_eq!(i.value(), "seed");
    }

    #[test]
    fn semantic_role_is_text_input() {
        assert_eq!(Input::new(cmd()).semantic_role(), SemanticRole::TextInput);
    }

    #[test]
    fn view_carries_value_prop() {
        let i = Input::with_value(cmd(), "abc").placeholder("type here");
        let view = i.view();
        match &view.root().nodes()[0] {
            ViewNode::Element {
                component,
                props,
                events,
                children,
                ..
            } => {
                assert_eq!(*component, Input::COMPONENT_ID);
                let value = props
                    .iter()
                    .find(|p| p.name() == Input::PROP_VALUE)
                    .expect("value prop present");
                assert_eq!(*value.value(), ViewValue::from("abc"));
                let placeholder = props
                    .iter()
                    .find(|p| p.name() == Input::PROP_PLACEHOLDER)
                    .expect("placeholder prop present");
                assert_eq!(*placeholder.value(), ViewValue::from("type here"));
                assert_eq!(events[0].command(), cmd());
                assert!(children.is_empty());
            }
            ViewNode::Text { .. } => panic!("expected element root"),
        }
    }

    #[test]
    fn view_binds_both_input_and_change_to_the_command() {
        // The input fires per-keystroke EVENT_INPUT *and* commit EVENT_CHANGE,
        // both to its `command`, so the runtime's text-input dispatch and its
        // activation/change dispatch reach the same app handler.
        let i = Input::new(cmd());
        let view = i.view();
        match &view.root().nodes()[0] {
            ViewNode::Element { events, .. } => {
                let input = events
                    .iter()
                    .find(|b| b.event() == EVENT_INPUT)
                    .expect("EVENT_INPUT binding present");
                assert_eq!(input.command(), cmd());
                let change = events
                    .iter()
                    .find(|b| b.event() == EVENT_CHANGE)
                    .expect("EVENT_CHANGE binding present");
                assert_eq!(change.command(), cmd());
            }
            ViewNode::Text { .. } => panic!("expected element root"),
        }
    }
}
