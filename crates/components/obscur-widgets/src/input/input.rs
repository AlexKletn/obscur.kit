//! `Input`: a headless single-line text-input control (spec §2.6, §5.11).

use obscur_accessibility::semantic::SemanticRole;
use obscur_identity::{ComponentId, InternedSymbol};
use obscur_text::editing::{EditCommand, TextBuffer};
use obscur_view::view::{
    CLASS, DISABLED as ATTR_DISABLED, EVENT_CHANGE, EVENT_INPUT, TYPE, View, ViewEventBinding,
    ViewFragment, ViewNode, ViewProperty, ViewValue,
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
    /// Whether this is a **multi-line** field (a `<textarea>`, spec §2.6). A
    /// multi-line field emits the reserved CSS type `textarea` (so the UA
    /// `textarea { … }` rules bind and it wraps + preserves segment breaks, CSS
    /// Text 3 §4.1.1) and marks its editable MULTILINE for the runtime, so `Enter`
    /// inserts a newline and `ArrowUp`/`ArrowDown` move between laid-out lines. The
    /// default (`false`) is a single-line `input` — byte-identical to before.
    multiline: bool,
}

impl Input {
    /// Stable component id for the input widget (see [`Button::COMPONENT_ID`]
    /// for the shared `0x00D0_00NN` scheme).
    ///
    /// [`Button::COMPONENT_ID`]: crate::button::Button::COMPONENT_ID
    pub const COMPONENT_ID: ComponentId = ComponentId::from_raw(0x00D0_0002);

    /// Component id of the placeholder sub-part (the `.placeholder` overlay shown
    /// while the field is empty). Shares the widget id space with the high nibble
    /// of the suffix flipped (`..12`), mirroring the `.checkbox-box` sub-part
    /// scheme.
    const PLACEHOLDER_PART_ID: ComponentId = ComponentId::from_raw(0x00D0_0012);

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
            multiline: false,
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
            multiline: false,
        }
    }

    /// Makes this field **multi-line** (a `<textarea>`), returning the updated
    /// input (fluent builder). A multi-line field emits the reserved CSS type
    /// `textarea` and marks its editable MULTILINE (see [`multiline`](Self::multiline)
    /// field): `Enter` inserts a newline and `ArrowUp`/`ArrowDown` move between
    /// wrapped lines. The default is a single-line `input`.
    #[must_use]
    pub const fn multiline(mut self, multiline: bool) -> Self {
        self.multiline = multiline;
        self
    }

    /// Returns whether this is a multi-line (`<textarea>`) field.
    #[must_use]
    pub const fn is_multiline(&self) -> bool {
        self.multiline
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
    /// event bindings to `command` — the HTML `input`/`change` event model
    /// (WHATWG HTML §4.10.5.5 *Common input element APIs*): a per-keystroke
    /// [`EVENT_INPUT`] (each typed character / `Backspace` drives the runtime's
    /// editing session, which delivers the resulting field text to the app) and a
    /// commit-only [`EVENT_CHANGE`]. The widget only *declares* both bindings; the
    /// runtime does the routing — for this editable it fires `change` on **commit**
    /// (blur, or `Enter` for a single-line field) carrying the committed value, and
    /// deliberately *not* on a focus click (unlike a checkbox/radio/select, whose
    /// toggling click **is** the change).
    ///
    /// The current text is carried *both* as the [`PROP_VALUE`] property (the
    /// semantic / accessibility surface) *and*, when non-empty, as a single
    /// [`Text`] child node, so the field renders visibly and the runtime's
    /// editing-session auto-seeding — which reads the focused editable's
    /// descendant [`ViewNode::Text`] content — picks up the value.
    ///
    /// **Placeholder as a distinct overlay.** When the value is *empty*, the field
    /// shows its `placeholder` inside a marked `.placeholder` sub-part element
    /// instead of an editable value node — the toolkit's analogue of the CSS
    /// **`::placeholder`** pseudo-element, shown only while **`:placeholder-shown`**
    /// holds (an empty value): CSS Pseudo-Elements Level 4 §4.5 (`::placeholder`)
    /// and CSS Selectors Level 4 §11.2 (`:placeholder-shown`). Per that model the
    /// placeholder is a separate, non-editable rendering layer — *not* the value:
    /// the runtime skips the `.placeholder` subtree both when seeding the editing
    /// buffer (an empty field focuses to an empty buffer, caret at 0) and when the
    /// compose pass overrides the field's editable text (so typing replaces the
    /// placeholder with the typed value, never `placeholder + typed`). Styling the
    /// placeholder muted is a UA-sheet concern (`.placeholder { color: … }`), not
    /// baked in here.
    ///
    /// [`COMPONENT_ID`]: Self::COMPONENT_ID
    /// [`PROP_VALUE`]: Self::PROP_VALUE
    /// [`Text`]: obscur_view::view::ViewNode::text
    #[must_use]
    pub fn view(&self) -> View {
        let value = self.buffer.text();
        // Stable CSS type / local name so a UA (or author) type selector matches
        // this control (spec §4.13): `textarea` for a multi-line field (so the UA
        // `textarea { … }` rules bind — wrapping + `white-space: pre-wrap`), else
        // `input` for a single-line field (byte-identical to before). Additive.
        let type_name = if self.multiline { "textarea" } else { "input" };
        let node = ViewNode::element(Self::COMPONENT_ID)
            .with_prop(ViewProperty::new(TYPE, ViewValue::from(type_name)))
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
                ViewValue::from(value),
            ))
            .with_prop(ViewProperty::new(
                Self::PROP_PLACEHOLDER,
                ViewValue::from(&*self.placeholder),
            ))
            .with_event(ViewEventBinding::new(EVENT_INPUT, self.command))
            .with_event(ViewEventBinding::new(EVENT_CHANGE, self.command));
        // Reserved `:disabled` structural-state attribute — emitted only when
        // disabled; survives reconcile and feeds the matcher's `:disabled`.
        let node = if self.disabled {
            node.with_prop(ViewProperty::new(ATTR_DISABLED, ViewValue::Bool(true)))
        } else {
            node
        };
        // Children (CSS `::placeholder` / `:placeholder-shown` model):
        //   • non-empty value → a single editable `Text` child carrying the value
        //     (byte-identical to the pre-placeholder-fix rendering);
        //   • empty value → the `placeholder` inside a `.placeholder` sub-part
        //     element. The reserved [`CLASS`] survives reconcile (like DISABLED /
        //     CHECKED), so the UA sheet's `.placeholder { color: … }` mutes it and
        //     the runtime can identify — and skip — it as a non-editable overlay.
        let children = if value.is_empty() {
            vec![
                ViewNode::element(Self::PLACEHOLDER_PART_ID)
                    .with_prop(ViewProperty::new(CLASS, ViewValue::from("placeholder")))
                    .with_children(ViewFragment::from_nodes(vec![ViewNode::text(
                        &*self.placeholder,
                    )])),
            ]
        } else {
            vec![ViewNode::text(value)]
        };
        let node = node.with_children(ViewFragment::from_nodes(children));
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
                // A non-empty value is rendered as a single Text child (in
                // addition to the PROP_VALUE prop): visible + runtime-seedable.
                assert_eq!(children.len(), 1);
                match &children.nodes()[0] {
                    ViewNode::Text { content, .. } => assert_eq!(&**content, "abc"),
                    ViewNode::Element { .. } => panic!("expected text child"),
                }
            }
            ViewNode::Text { .. } => panic!("expected element root"),
        }
    }

    #[test]
    fn view_renders_value_as_text_child() {
        // The buffer text is carried both as PROP_VALUE and as a Text child so
        // the field displays and the runtime's editing-session auto-seeding
        // (which reads descendant Text content) picks it up.
        let i = Input::with_value(cmd(), "hi");
        let view = i.view();
        match &view.root().nodes()[0] {
            ViewNode::Element {
                props, children, ..
            } => {
                let value = props
                    .iter()
                    .find(|p| p.name() == Input::PROP_VALUE)
                    .expect("value prop present");
                assert_eq!(*value.value(), ViewValue::from("hi"));
                assert_eq!(children.len(), 1);
                match &children.nodes()[0] {
                    ViewNode::Text { content, .. } => assert_eq!(&**content, "hi"),
                    ViewNode::Element { .. } => panic!("expected text child"),
                }
            }
            ViewNode::Text { .. } => panic!("expected element root"),
        }
    }

    #[test]
    fn empty_input_renders_placeholder_in_a_marked_overlay_not_as_value() {
        // Previously an empty field emitted the placeholder AS its single editable
        // `Text` child (indistinguishable from a real value, and seeded into the
        // editing buffer on focus). Per the CSS `::placeholder` / `:placeholder-shown`
        // model the placeholder is now a SEPARATE, non-editable overlay: a
        // `.placeholder` sub-part element (reserved CLASS, survives reconcile), NOT
        // the value. The value prop is still the empty value.
        let i = Input::new(cmd()).placeholder("type here");
        let view = i.view();
        match &view.root().nodes()[0] {
            ViewNode::Element {
                props, children, ..
            } => {
                let value = props
                    .iter()
                    .find(|p| p.name() == Input::PROP_VALUE)
                    .expect("value prop present");
                assert_eq!(*value.value(), ViewValue::from(""));
                // The single child is the `.placeholder` overlay element, not a bare
                // Text value node.
                assert_eq!(children.len(), 1);
                match &children.nodes()[0] {
                    ViewNode::Element {
                        component,
                        props,
                        children,
                        ..
                    } => {
                        assert_eq!(*component, Input::PLACEHOLDER_PART_ID);
                        let class = props
                            .iter()
                            .find(|p| p.name() == CLASS)
                            .expect("placeholder overlay carries the reserved CLASS marker");
                        assert_eq!(*class.value(), ViewValue::from("placeholder"));
                        // The placeholder text lives inside the marked overlay.
                        assert_eq!(children.len(), 1);
                        match &children.nodes()[0] {
                            ViewNode::Text { content, .. } => assert_eq!(&**content, "type here"),
                            ViewNode::Element { .. } => panic!("expected placeholder text"),
                        }
                    }
                    ViewNode::Text { .. } => {
                        panic!("empty field must emit a `.placeholder` overlay, not a Text value")
                    }
                }
            }
            ViewNode::Text { .. } => panic!("expected element root"),
        }
    }

    #[test]
    fn nonempty_value_renders_the_value_directly_with_no_placeholder_overlay() {
        // Byte-identical-when-valued guard: a non-empty field emits exactly a single
        // editable `Text` child carrying the value — no `.placeholder` overlay — so
        // the render/edit path is unchanged from before the placeholder fix, even
        // when a placeholder string is configured.
        let i = Input::with_value(cmd(), "abc").placeholder("type here");
        let view = i.view();
        match &view.root().nodes()[0] {
            ViewNode::Element { children, .. } => {
                assert_eq!(children.len(), 1);
                match &children.nodes()[0] {
                    ViewNode::Text { content, .. } => assert_eq!(&**content, "abc"),
                    ViewNode::Element { .. } => {
                        panic!("a non-empty value must render as a bare Text child, no overlay")
                    }
                }
            }
            ViewNode::Text { .. } => panic!("expected element root"),
        }
    }

    #[test]
    fn single_line_input_emits_type_input() {
        // A plain (single-line) input carries the reserved CSS type `input` and is
        // not multiline — byte-identical to before the multiline mode existed.
        let i = Input::new(cmd());
        assert!(!i.is_multiline());
        let view = i.view();
        match &view.root().nodes()[0] {
            ViewNode::Element { props, .. } => {
                let ty = props.iter().find(|p| p.name() == TYPE).expect("type prop");
                assert_eq!(*ty.value(), ViewValue::from("input"));
            }
            ViewNode::Text { .. } => panic!("expected element root"),
        }
    }

    #[test]
    fn multiline_input_emits_type_textarea() {
        // A `.multiline(true)` field carries the reserved CSS type `textarea`, so
        // the UA `textarea { … }` rules bind and the runtime treats it as MULTILINE.
        let i = Input::with_value(cmd(), "line one").multiline(true);
        assert!(i.is_multiline());
        let view = i.view();
        match &view.root().nodes()[0] {
            ViewNode::Element { props, .. } => {
                let ty = props.iter().find(|p| p.name() == TYPE).expect("type prop");
                assert_eq!(*ty.value(), ViewValue::from("textarea"));
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
