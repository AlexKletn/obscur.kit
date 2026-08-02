#![doc = include_str!("input.doc/_module.md")]

use obscur_accessibility::semantic::SemanticRole;
use obscur_identity::{ComponentId, InternedSymbol};
use obscur_text::editing::{EditCommand, TextBuffer};
use obscur_view::view::{
    DISABLED as ATTR_DISABLED, EVENT_CHANGE, EVENT_INPUT, LOCAL_NAME,
    PLACEHOLDER as ATTR_PLACEHOLDER, READONLY as ATTR_READONLY, VALUE as ATTR_VALUE, View,
    ViewEventBinding, ViewFragment, ViewNode, ViewProperty, ViewValue,
};

#[doc = include_str!("input.doc/Input.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct Input {
    buffer: TextBuffer,
    placeholder: Box<str>,
    command: InternedSymbol,
    disabled: bool,
    #[doc = include_str!("input.doc/readonly.md")]
    readonly: bool,
    #[doc = include_str!("input.doc/multiline.md")]
    multiline: bool,
}

impl Input {
    #[doc = include_str!("input.doc/COMPONENT_ID.md")]
    pub const COMPONENT_ID: ComponentId = ComponentId::from_raw(0x00D0_0002);

    #[doc = include_str!("input.doc/PROP_ROLE.md")]
    const PROP_ROLE: InternedSymbol = InternedSymbol::from_raw(0x00D1_0200);
    #[doc = include_str!("input.doc/PROP_DISABLED.md")]
    const PROP_DISABLED: InternedSymbol = InternedSymbol::from_raw(0x00D1_0201);
    #[doc = include_str!("input.doc/PROP_VALUE.md")]
    pub(crate) const PROP_VALUE: InternedSymbol = InternedSymbol::from_raw(0x00D1_0202);
    #[doc = include_str!("input.doc/PROP_PLACEHOLDER.md")]
    const PROP_PLACEHOLDER: InternedSymbol = InternedSymbol::from_raw(0x00D1_0203);
    #[doc = include_str!("input.doc/PROP_READONLY.md")]
    const PROP_READONLY: InternedSymbol = InternedSymbol::from_raw(0x00D1_0204);

    #[doc = include_str!("input.doc/new.md")]
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

    #[doc = include_str!("input.doc/with_value.md")]
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

    #[doc = include_str!("input.doc/multiline_2.md")]
    #[must_use]
    pub const fn multiline(mut self, multiline: bool) -> Self {
        self.multiline = multiline;
        self
    }

    #[doc = include_str!("input.doc/is_multiline.md")]
    #[must_use]
    pub const fn is_multiline(&self) -> bool {
        self.multiline
    }

    #[doc = include_str!("input.doc/placeholder.md")]
    #[must_use]
    pub fn placeholder(mut self, placeholder: impl Into<Box<str>>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    #[doc = include_str!("input.doc/disabled.md")]
    #[must_use]
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    #[doc = include_str!("input.doc/readonly_2.md")]
    #[must_use]
    pub const fn readonly(mut self, readonly: bool) -> Self {
        self.readonly = readonly;
        self
    }

    #[doc = include_str!("input.doc/value.md")]
    #[must_use]
    pub fn value(&self) -> &str {
        self.buffer.text()
    }

    #[doc = include_str!("input.doc/placeholder_text.md")]
    #[must_use]
    pub fn placeholder_text(&self) -> &str {
        &self.placeholder
    }

    #[doc = include_str!("input.doc/apply.md")]
    pub fn apply(&mut self, command: EditCommand) {
        if self.disabled {
            return;
        }
        if self.readonly && command.is_mutating() {
            return;
        }
        self.buffer.apply(command);
    }

    #[doc = include_str!("input.doc/buffer.md")]
    #[must_use]
    pub const fn buffer(&self) -> &TextBuffer {
        &self.buffer
    }

    #[doc = include_str!("input.doc/command.md")]
    #[must_use]
    pub const fn command(&self) -> InternedSymbol {
        self.command
    }

    #[doc = include_str!("input.doc/is_disabled.md")]
    #[must_use]
    pub const fn is_disabled(&self) -> bool {
        self.disabled
    }

    #[doc = include_str!("input.doc/set_disabled.md")]
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    #[doc = include_str!("input.doc/is_readonly.md")]
    #[must_use]
    pub const fn is_readonly(&self) -> bool {
        self.readonly
    }

    #[doc = include_str!("input.doc/set_readonly.md")]
    pub const fn set_readonly(&mut self, readonly: bool) {
        self.readonly = readonly;
    }

    #[doc = include_str!("input.doc/semantic_role.md")]
    #[must_use]
    pub const fn semantic_role(&self) -> SemanticRole {
        SemanticRole::TextInput
    }

    #[doc = include_str!("input.doc/view.md")]
    #[must_use]
    pub fn view(&self) -> View {
        let value = self.buffer.text();
        // Stable CSS type / local name so a UA (or author) type selector matches
        // this control (spec §4.13): `textarea` for a multi-line field (so the UA
        // `textarea { … }` rules bind — wrapping + `white-space: pre-wrap`), else
        // `input` for a single-line field (byte-identical to before). Additive.
        let type_name = if self.multiline { "textarea" } else { "input" };
        let node = ViewNode::element(Self::COMPONENT_ID)
            .with_prop(ViewProperty::new(LOCAL_NAME, ViewValue::from(type_name)))
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
        let node = node.with_children(Self::contents(value));
        View::single(node)
    }

    #[doc = include_str!("input.doc/contents.md")]
    #[must_use]
    pub(crate) fn contents(value: &str) -> ViewFragment {
        // The field's editable text is always a single `Text` child carrying the
        // current value (empty when the field is empty). It is the caret/selection
        // and IME-preedit paint target and what the editing session auto-seeds from;
        // being a `Text` node it is content, not a component-tree child. The
        // placeholder is NOT a child — it is a §4.16 synthetic part the runtime
        // paints from the reserved `PLACEHOLDER` attribute while the value is empty.
        ViewFragment::from_nodes(vec![ViewNode::text(value)])
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
    fn single_line_input_emits_the_input_local_name() {
        // A plain (single-line) input carries the reserved CSS local name `input`
        // and is not multiline. The name is what the cascade's type selector and
        // `EditableField::is_multiline` both read; the authored `type` attribute is
        // a separate channel this control never writes.
        let i = Input::new(cmd());
        assert!(!i.is_multiline());
        let view = i.view();
        match &view.root().nodes()[0] {
            ViewNode::Element { props, .. } => {
                let ty = props
                    .iter()
                    .find(|p| p.name() == LOCAL_NAME)
                    .expect("local-name prop");
                assert_eq!(*ty.value(), ViewValue::from("input"));
            }
            ViewNode::Text { .. } => panic!("expected element root"),
        }
    }

    #[test]
    fn multiline_input_emits_the_textarea_local_name() {
        // A `.multiline(true)` field carries the reserved CSS local name `textarea`,
        // so the UA `textarea { … }` rules bind and the runtime treats it as
        // MULTILINE. This is the only place the two tags differ, and it rides on
        // the host node — which a mounted `InputComponent` never rewrites.
        let i = Input::with_value(cmd(), "line one").multiline(true);
        assert!(i.is_multiline());
        let view = i.view();
        match &view.root().nodes()[0] {
            ViewNode::Element { props, .. } => {
                let ty = props
                    .iter()
                    .find(|p| p.name() == LOCAL_NAME)
                    .expect("local-name prop");
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
