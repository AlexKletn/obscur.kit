//! `Input`: a headless single-line text-input control (spec §2.6, §5.11).

use obscur_accessibility::semantic::SemanticRole;
use obscur_identity::{ComponentId, InternedSymbol};
use obscur_text::editing::{EditCommand, TextBuffer};
use obscur_view::view::{
    DISABLED as ATTR_DISABLED, EVENT_CHANGE, EVENT_INPUT, PLACEHOLDER as ATTR_PLACEHOLDER,
    READONLY as ATTR_READONLY, TYPE, VALUE as ATTR_VALUE, View, ViewEventBinding, ViewFragment,
    ViewNode, ViewProperty, ViewValue,
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
    /// Whether this field is **readonly** (the HTML `readonly` attribute, WHATWG
    /// HTML §attr-input-readonly). **Distinct from — and independent of —
    /// [`disabled`](Self::disabled):** a readonly field can still receive focus, a
    /// caret, and a selection (it is selectable / copyable), but its text cannot be
    /// *edited* (no insert/delete, no IME commit, no paste — [`apply`](Self::apply)
    /// drops mutating edits, and the runtime gates only the mutation path on the
    /// reserved `READONLY` attribute this emits). A field may be neither, one, or
    /// both flags; the default (`false`) is a normal editable field —
    /// byte-identical to before.
    readonly: bool,
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
    /// Interned name of the `readonly` state property (`NN = 02`, suffix `04`).
    const PROP_READONLY: InternedSymbol = InternedSymbol::from_raw(0x00D1_0204);

    /// Creates an empty, enabled input with no placeholder that fires `command`
    /// on change.
    #[must_use]
    pub fn new(command: InternedSymbol) -> Self {
        Self {
            buffer: TextBuffer::new(),
            placeholder: Box::<str>::from(""),
            command,
            disabled: false,
            readonly: false,
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
            readonly: false,
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

    /// Sets the **readonly** state, returning the updated input (fluent builder).
    ///
    /// A readonly field (the HTML `readonly` attribute, WHATWG HTML
    /// §attr-input-readonly) is still focusable and its text still selectable /
    /// copyable, but it cannot be edited (no insert/delete, no IME commit, no
    /// paste). It is **independent of** [`disabled`](Self::disabled): a field can be
    /// neither, one, or both.
    #[must_use]
    pub const fn readonly(mut self, readonly: bool) -> Self {
        self.readonly = readonly;
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
    /// edits (typing, deletion, navigation, IME/programmatic edits) flow in.
    ///
    /// A no-op while [`disabled`](Self::is_disabled) (the field is fully inert).
    /// While [`readonly`](Self::is_readonly) (but not disabled) only the
    /// **text-mutating** commands (insert / delete / replace) are dropped — caret
    /// and selection navigation still apply, matching the HTML `readonly` model
    /// (WHATWG HTML §attr-input-readonly: focusable + selectable, not editable). A
    /// normal field applies every command (byte-identical to before).
    pub fn apply(&mut self, command: EditCommand) {
        if self.disabled {
            return;
        }
        if self.readonly && command.is_mutating() {
            return;
        }
        self.buffer.apply(command);
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

    /// Returns whether the input is readonly (HTML `readonly`).
    #[must_use]
    pub const fn is_readonly(&self) -> bool {
        self.readonly
    }

    /// Sets the readonly state in place.
    pub const fn set_readonly(&mut self, readonly: bool) {
        self.readonly = readonly;
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
    /// **Placeholder as a §4.16 synthetic part.** The `placeholder` hint is carried
    /// as the reserved [`PLACEHOLDER`](obscur_view::view::PLACEHOLDER) structural
    /// attribute — *not* materialised into the tree as a child element. The runtime
    /// paints it as a synthetic part (paint-only, DevTools-visible, hit-transparent,
    /// and **not** a component-tree child) while the field is empty — the toolkit's
    /// analogue of the CSS **`::placeholder`** pseudo-element, shown only while
    /// **`:placeholder-shown`** holds (an empty value): CSS Pseudo-Elements Level 4
    /// §4.5 (`::placeholder`) and CSS Selectors Level 4 §11.2 (`:placeholder-shown`).
    /// Per that model the placeholder is a separate, non-editable rendering layer —
    /// never seeded into the editing buffer and never a hit target. An author can
    /// restyle it through the exported-part selector `Input::part(placeholder)`.
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
                Self::PROP_READONLY,
                ViewValue::Bool(self.readonly),
            ))
            .with_prop(ViewProperty::new(Self::PROP_VALUE, ViewValue::from(value)))
            // The reserved `VALUE` structural attribute (WHATWG HTML §4.10.5.5
            // value reflection; §3.1 "values flow down") declares this field's
            // authoritative CONTROLLED value to the runtime: a focused buffer that
            // drifts from it (a keystroke the app held constant) is snapped back, so
            // the field never drifts from the application's value. Distinct from the
            // semantic `PROP_VALUE` (the a11y surface): this one the runtime reads
            // via `StyleAttributes::control_value`. Survives reconcile.
            .with_prop(ViewProperty::new(ATTR_VALUE, ViewValue::from(value)))
            .with_prop(ViewProperty::new(
                Self::PROP_PLACEHOLDER,
                ViewValue::from(&*self.placeholder),
            ))
            // The reserved `PLACEHOLDER` structural attribute (distinct from the
            // semantic `PROP_PLACEHOLDER` a11y surface): the runtime reads it (via
            // `StyleAttributes::placeholder`) to paint the placeholder as a §4.16
            // synthetic part while the field is empty, rather than materialising a
            // `.placeholder` child. Survives reconcile.
            .with_prop(ViewProperty::new(
                ATTR_PLACEHOLDER,
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
        // Reserved `readonly` structural-state attribute (WHATWG HTML
        // §attr-input-readonly) — emitted only when readonly; survives reconcile so
        // the runtime reads it (via `StyleAttributes::is_readonly`) to gate ONLY the
        // mutation path, leaving focus / caret / selection / copy intact. Distinct
        // from and independent of DISABLED (both may be present at once).
        let node = if self.readonly {
            node.with_prop(ViewProperty::new(ATTR_READONLY, ViewValue::Bool(true)))
        } else {
            node
        };
        // The field's editable text is always a single `Text` child carrying the
        // current value (empty when the field is empty). It is the caret/selection
        // and IME-preedit paint target and what the editing session auto-seeds from;
        // being a `Text` node it is content, not a component-tree child. The
        // placeholder is NOT a child — it is a §4.16 synthetic part the runtime
        // paints from the reserved `PLACEHOLDER` attribute while the value is empty.
        let node = node.with_children(ViewFragment::from_nodes(vec![ViewNode::text(value)]));
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
    fn readonly_apply_drops_mutations_but_keeps_navigation() {
        // A readonly field (HTML `readonly`) is not editable: a mutating edit is a
        // no-op. It is still selectable, so a non-mutating navigation/selection edit
        // still applies. Independent of `disabled` (this field is not disabled).
        let mut i = Input::with_value(cmd(), "seed").readonly(true);
        assert!(i.is_readonly() && !i.is_disabled());
        i.apply(EditCommand::insert("more"));
        assert_eq!(i.value(), "seed", "readonly drops an insert");
        i.apply(EditCommand::SelectAll);
        // SelectAll is non-mutating (selectable), so the buffer applied it: the whole
        // value is now selected while the text is unchanged.
        assert_eq!(i.value(), "seed");
        assert!(
            !i.buffer().selection().is_collapsed(),
            "select-all selected the text"
        );
    }

    #[test]
    fn readonly_field_emits_reserved_readonly_attr_not_disabled() {
        // A readonly field carries the reserved READONLY structural-state attribute
        // (so the runtime gates its mutation path) but NOT DISABLED (it stays
        // focusable). The two flags are independent.
        let view = Input::with_value(cmd(), "ro").readonly(true).view();
        match &view.root().nodes()[0] {
            ViewNode::Element { props, .. } => {
                assert!(
                    props
                        .iter()
                        .any(|p| p.name() == ATTR_READONLY && *p.value() == ViewValue::Bool(true)),
                    "readonly field emits the reserved READONLY attr"
                );
                assert!(
                    !props.iter().any(|p| p.name() == ATTR_DISABLED),
                    "a readonly (not disabled) field must NOT emit DISABLED"
                );
            }
            ViewNode::Text { .. } => panic!("expected element root"),
        }
    }

    #[test]
    fn plain_field_emits_no_readonly_attr() {
        // Byte-identical-when-unset guard: a normal field emits no reserved READONLY
        // attribute (only the semantic PROP_READONLY = false property).
        let view = Input::with_value(cmd(), "x").view();
        match &view.root().nodes()[0] {
            ViewNode::Element { props, .. } => assert!(
                !props.iter().any(|p| p.name() == ATTR_READONLY),
                "a normal field emits no reserved READONLY attr"
            ),
            ViewNode::Text { .. } => panic!("expected element root"),
        }
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
    fn empty_input_carries_placeholder_as_a_reserved_attr_not_a_child_overlay() {
        // Per §4.16 the placeholder is a SYNTHETIC part, not a tree node: an empty
        // field emits its (empty) value as the single editable `Text` child — never
        // a `.placeholder` overlay element — and carries the placeholder hint as the
        // reserved `PLACEHOLDER` structural attribute for the runtime to paint from.
        // The value prop is still the empty value.
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
                // The placeholder rides on the reserved PLACEHOLDER attribute.
                let placeholder = props
                    .iter()
                    .find(|p| p.name() == ATTR_PLACEHOLDER)
                    .expect("reserved PLACEHOLDER attr present");
                assert_eq!(*placeholder.value(), ViewValue::from("type here"));
                // The single child is the (empty) editable value Text node, NOT a
                // `.placeholder` overlay element.
                assert_eq!(children.len(), 1);
                match &children.nodes()[0] {
                    ViewNode::Text { content, .. } => assert_eq!(&**content, ""),
                    ViewNode::Element { .. } => {
                        panic!("empty field must emit its value Text child, not an overlay element")
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
