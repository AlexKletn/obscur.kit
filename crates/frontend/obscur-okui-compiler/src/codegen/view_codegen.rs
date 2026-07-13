//! `ViewCodegen`: emits ordinary Rust that builds a runtime [`View`] from a
//! compiled [`UiIrDocument`] at *compile* time (spec §2.8, §2.15).
//!
//! This is the code-generation counterpart of the runtime
//! [`UiIrViewBridge`](crate::bridge::UiIrViewBridge): where the bridge
//! *interprets* the IR into a desired tree every time it runs, `ViewCodegen`
//! lowers the same IR once, ahead of time, into Rust source that constructs the
//! identical [`View`] with no `.okui` machinery present at runtime. A build step
//! writes the emitted function into a generated `.rs` file the application
//! `include!`s (or the same text is produced inline by the `okui!` macro).
//!
//! # What it emits
//!
//! [`document_to_view_fn`](ViewCodegen::document_to_view_fn) emits a free
//! function
//!
//! ```ignore
//! pub fn <name>() -> ::obscur_view::view::View { <expr> }
//! ```
//!
//! whose body is the fully-qualified [`View`] expression from
//! [`document_to_view_expr`](ViewCodegen::document_to_view_expr). Every path is
//! written with a leading `::` so the generated code needs no `use` items; it
//! only requires the consuming crate to have `obscur-view` and `obscur-identity`
//! as dependencies (the facade re-exports both, so its examples compile the
//! output directly).
//!
//! # Name binding
//!
//! Tags, attributes and events resolve through the shared
//! [`name_binding`] registry — the *same* policy the runtime bridge uses — so a
//! `<Button>` emits its real `COMPONENT_ID`, an `<img src=…>` emits the reserved
//! `SRC` symbol (reaching real decoded pixels), a `<div>` emits the generic
//! `Container`, and `click`/`input`/`change` emit the reserved `EVENT_*`
//! symbols. The emitted handles are raw values (`from_raw(0x…)`) resolved from
//! the registry at generation time.
//!
//! # Dynamic nodes
//!
//! Like the bridge, `<If>`/`<For>` are *statically materialised*: a conditional
//! emits its guarded body and a loop its template body, spliced into the
//! surrounding fragment. Evaluating a real condition or iterating a live
//! collection needs runtime data the IR does not carry, so that expansion
//! remains the runtime's job; a dynamic binding emits its path as placeholder
//! text.

use std::fmt::Write as _;

use obscur_ui_ir::document::{UiIrDocument, UiIrExpression, UiIrNode, UiIrNodeKind, UiIrValue};
use obscur_ui_ir::name_binding;

/// Emits ordinary Rust source building a [`View`](obscur_view::view::View) from
/// a compiled [`UiIrDocument`].
#[derive(Debug, Clone, Copy, Default)]
pub struct ViewCodegen;

impl ViewCodegen {
    /// Emits a `pub fn <fn_name>() -> View { … }` whose body constructs the
    /// [`View`](obscur_view::view::View) equivalent to `document`.
    ///
    /// The function is self-contained (fully-qualified paths, no `use` items);
    /// drop it into a generated module the consumer `include!`s.
    #[must_use]
    pub fn document_to_view_fn(document: &UiIrDocument, fn_name: &str) -> String {
        let expr = Self::document_to_view_expr(document);
        format!(
            "/// Generated from a `.okui` document by `obscur-okui-compiler`.\n\
             /// Do not edit by hand.\n\
             pub fn {fn_name}() -> ::obscur_view::view::View {{\n{expr}\n}}\n"
        )
    }

    /// Emits just the [`View`](obscur_view::view::View) expression (no enclosing
    /// function), indented for embedding.
    #[must_use]
    pub fn document_to_view_expr(document: &UiIrDocument) -> String {
        let mut out = String::new();
        out.push_str("    ::obscur_view::view::View::from_fragment(\n");
        out.push_str("        ::obscur_view::view::ViewFragment::from_nodes(vec![\n");
        emit_node(document.root(), &mut out, 3);
        out.push_str("        ]),\n");
        out.push_str("    )");
        out
    }
}

/// Appends the `ViewNode` expression(s) `node` lowers to, each as its own
/// `vec!` element terminated by a comma, at `indent` levels (4 spaces each).
///
/// Most kinds emit exactly one node; a transparent fragment and the statically
/// materialised conditional/loop splice their children, so several elements may
/// be appended.
fn emit_node(node: &UiIrNode, out: &mut String, indent: usize) {
    match node.kind() {
        UiIrNodeKind::Text => {
            let pad = indentation(indent);
            let content = string_literal(node.content());
            let _ = write!(out, "{pad}::obscur_view::view::ViewNode::text({content})");
            emit_key(node, out);
            out.push_str(",\n");
        }
        UiIrNodeKind::Element | UiIrNodeKind::Component => emit_element(node, out, indent),
        UiIrNodeKind::Fragment | UiIrNodeKind::Conditional | UiIrNodeKind::Loop => {
            for child in node.children() {
                emit_node(child, out, indent);
            }
        }
    }
}

/// Appends a single `ViewNode::Element` expression for `node`.
fn emit_element(node: &UiIrNode, out: &mut String, indent: usize) {
    let pad = indentation(indent);
    let inner = indentation(indent + 1);
    let component = name_binding::component_id_for(node.tag()).raw();
    let _ = writeln!(
        out,
        "{pad}::obscur_view::view::ViewNode::element(::obscur_identity::ComponentId::from_raw({component:#x}))"
    );

    for property in node.properties() {
        let symbol = name_binding::attr_symbol(property.name()).raw();
        let value = emit_value(property.value());
        let _ = writeln!(
            out,
            "{inner}.with_prop(::obscur_view::view::ViewProperty::new(::obscur_identity::InternedSymbol::from_raw({symbol:#x}), {value}))"
        );
    }

    for event in node.events() {
        let event_symbol = name_binding::event_symbol(event.event()).raw();
        let command = name_binding::fallback_symbol(event.handler()).raw();
        let _ = writeln!(
            out,
            "{inner}.with_event(::obscur_view::view::ViewEventBinding::new(::obscur_identity::InternedSymbol::from_raw({event_symbol:#x}), ::obscur_identity::InternedSymbol::from_raw({command:#x})))"
        );
    }

    let children: Vec<&UiIrNode> = node.children().iter().collect();
    if !children.is_empty() {
        let _ = writeln!(
            out,
            "{inner}.with_children(::obscur_view::view::ViewFragment::from_nodes(vec!["
        );
        for child in children {
            emit_node(child, out, indent + 2);
        }
        let _ = writeln!(out, "{inner}]))");
    }

    // The key modifier (if any) closes the builder chain before the comma.
    let _ = write!(out, "{inner}");
    emit_key(node, out);
    out.push_str(",\n");
}

/// Appends a `.with_key(…)` modifier for `node`'s key, if it has one; a no-op
/// otherwise. Mirrors the runtime bridge's key lowering.
fn emit_key(node: &UiIrNode, out: &mut String) {
    let Some(key) = node.key() else { return };
    let key_expr = match key {
        UiIrExpression::Literal(UiIrValue::Int(number)) => {
            format!("::obscur_view::view::ViewKey::from({}u64)", number.cast_unsigned())
        }
        UiIrExpression::Literal(UiIrValue::Text(text)) => {
            format!("::obscur_view::view::ViewKey::from({})", string_literal(text))
        }
        UiIrExpression::Binding(path) => {
            format!("::obscur_view::view::ViewKey::from({})", string_literal(path))
        }
        UiIrExpression::Literal(UiIrValue::Bool(flag)) => format!(
            "::obscur_view::view::ViewKey::from({:?})",
            if *flag { "true" } else { "false" }
        ),
        UiIrExpression::Literal(UiIrValue::Float(number)) => {
            format!("::obscur_view::view::ViewKey::from({})", string_literal(&number.to_string()))
        }
        UiIrExpression::Literal(UiIrValue::Null) => {
            "::obscur_view::view::ViewKey::from(\"null\")".to_owned()
        }
        UiIrExpression::Call { name, .. } => {
            format!("::obscur_view::view::ViewKey::from({})", string_literal(name))
        }
    };
    let _ = write!(out, ".with_key({key_expr})");
}

/// Emits the `ViewValue` expression for a property's value expression.
///
/// A dynamic binding emits its path as placeholder text (its real value needs
/// runtime resolution); a call emits `Null`. Mirrors the runtime bridge.
fn emit_value(expression: &UiIrExpression) -> String {
    match expression {
        UiIrExpression::Literal(value) => emit_literal(value),
        UiIrExpression::Binding(path) => {
            format!("::obscur_view::view::ViewValue::Text({}.into())", string_literal(path))
        }
        UiIrExpression::Call { .. } => "::obscur_view::view::ViewValue::Null".to_owned(),
    }
}

/// Emits the `ViewValue` expression for a literal.
fn emit_literal(value: &UiIrValue) -> String {
    match value {
        UiIrValue::Null => "::obscur_view::view::ViewValue::Null".to_owned(),
        UiIrValue::Bool(flag) => format!("::obscur_view::view::ViewValue::Bool({flag})"),
        UiIrValue::Int(number) => format!("::obscur_view::view::ViewValue::Int({number})"),
        UiIrValue::Float(number) => format!("::obscur_view::view::ViewValue::Float({number:?})"),
        UiIrValue::Text(text) => {
            format!("::obscur_view::view::ViewValue::Text({}.into())", string_literal(text))
        }
    }
}

/// Renders `text` as a valid Rust string literal (Debug formatting escapes
/// quotes, backslashes and control characters correctly).
fn string_literal(text: &str) -> String {
    format!("{text:?}")
}

/// Returns `indent` levels of four-space indentation.
fn indentation(indent: usize) -> String {
    "    ".repeat(indent)
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_ui_ir::document::{UiIrEvent, UiIrProperty};

    fn expr(root: UiIrNode) -> String {
        ViewCodegen::document_to_view_expr(&UiIrDocument::new(root))
    }

    #[test]
    fn button_emits_its_real_component_id() {
        let out = expr(UiIrNode::component(0, "Button"));
        // Button::COMPONENT_ID == 0x00D0_0001.
        assert!(out.contains("ComponentId::from_raw(0xd00001)"), "{out}");
    }

    #[test]
    fn generic_div_emits_the_container_component_id() {
        let out = expr(UiIrNode::element(0, "div"));
        // Container::COMPONENT_ID == 0x00C0_0001.
        assert!(out.contains("ComponentId::from_raw(0xc00001)"), "{out}");
    }

    #[test]
    fn img_src_emits_the_reserved_src_symbol() {
        let out = expr(
            UiIrNode::element(0, "img").with_property(UiIrProperty::text("src", "logo.png")),
        );
        // Image::COMPONENT_ID == 0x00C0_0003; SRC symbol == 0x4.
        assert!(out.contains("ComponentId::from_raw(0xc00003)"), "{out}");
        assert!(out.contains("InternedSymbol::from_raw(0x4)"), "{out}");
        assert!(out.contains(r#"ViewValue::Text("logo.png".into())"#), "{out}");
    }

    #[test]
    fn click_event_emits_the_reserved_event_symbol() {
        let out = expr(
            UiIrNode::component(0, "Button").with_event(UiIrEvent::new("click", "submit")),
        );
        // EVENT_CLICK symbol == 0x101.
        assert!(out.contains("with_event"), "{out}");
        assert!(out.contains("InternedSymbol::from_raw(0x101)"), "{out}");
    }

    #[test]
    fn text_child_is_a_real_text_node() {
        let out = expr(UiIrNode::element(0, "span").with_child(UiIrNode::text(1, "hi")));
        assert!(out.contains(r#"ViewNode::text("hi")"#), "{out}");
        assert!(out.contains("with_children"), "{out}");
    }

    #[test]
    fn literal_property_kinds_emit_their_variants() {
        let out = expr(
            UiIrNode::element(0, "div")
                .with_property(UiIrProperty::literal("count", UiIrValue::Int(5)))
                .with_property(UiIrProperty::literal("ratio", UiIrValue::Float(1.5)))
                .with_property(UiIrProperty::literal("on", UiIrValue::Bool(true))),
        );
        assert!(out.contains("ViewValue::Int(5)"), "{out}");
        assert!(out.contains("ViewValue::Float(1.5)"), "{out}");
        assert!(out.contains("ViewValue::Bool(true)"), "{out}");
    }

    #[test]
    fn dynamic_binding_emits_its_path_as_placeholder_text() {
        let out = expr(
            UiIrNode::element(0, "div").with_property(UiIrProperty::binding("title", "user.name")),
        );
        assert!(out.contains(r#"ViewValue::Text("user.name".into())"#), "{out}");
    }

    #[test]
    fn fragment_root_splices_its_children() {
        let out = expr(
            UiIrNode::fragment(0)
                .with_child(UiIrNode::element(1, "a"))
                .with_child(UiIrNode::element(2, "b")),
        );
        // Two element expressions spliced directly under the root vec!.
        assert_eq!(out.matches("ViewNode::element").count(), 2, "{out}");
    }

    #[test]
    fn string_literals_are_escaped() {
        let out = expr(UiIrNode::text(0, "say \"hi\"\n"));
        // The quotes and newline are escaped into a valid Rust literal.
        assert!(out.contains(r#"say \"hi\"\n"#), "{out}");
    }

    #[test]
    fn fn_wrapper_wraps_the_expression() {
        let src = ViewCodegen::document_to_view_fn(&UiIrDocument::new(UiIrNode::element(0, "div")), "app_view");
        assert!(src.contains("pub fn app_view() -> ::obscur_view::view::View {"), "{src}");
        assert!(src.trim_end().ends_with('}'), "{src}");
    }
}
