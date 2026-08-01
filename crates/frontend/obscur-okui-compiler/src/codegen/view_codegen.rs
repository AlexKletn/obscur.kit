#![doc = include_str!("view_codegen.doc/_module.md")]

use std::fmt::Write as _;

use obscur_ui_ir::document::{
    UiIrDocument, UiIrExpression, UiIrNode, UiIrNodeKind, UiIrProperty, UiIrValue,
};
use obscur_ui_ir::name_binding::{self, TagPosition};

use crate::codegen::CodegenRoot;

#[doc = include_str!("view_codegen.doc/ViewCodegen.md")]
#[derive(Debug, Clone, Copy, Default)]
pub struct ViewCodegen;

impl ViewCodegen {
    #[doc = include_str!("view_codegen.doc/document_to_view_fn.md")]
    #[must_use]
    pub fn document_to_view_fn(
        document: &UiIrDocument,
        fn_name: &str,
        root: &CodegenRoot,
    ) -> String {
        let expr = Self::document_to_view_expr(document, root);
        let view = root.view();
        format!(
            "/// Generated from a `.okui` document by `obscur-okui-compiler`.\n\
             /// Do not edit by hand.\n\
             // The two lints below fire on the emitted body by construction, and the\n\
             // consumer cannot silence them at the site because the site is generated.\n\
             // `vec_init_then_push`: a conditional is a *statement*, so children are\n\
             // collected into a sink rather than written as a `vec![…]` literal (see the\n\
             // module docs). `unreadable_literal`: every id is a raw hash printed in hex,\n\
             // where digit separators would be noise, not legibility.\n\
             #[allow(clippy::vec_init_then_push, clippy::unreadable_literal)]\n\
             #[must_use]\n\
             pub fn {fn_name}() -> {view}::View {{\n{expr}\n}}\n"
        )
    }

    #[doc = include_str!("view_codegen.doc/document_to_view_expr.md")]
    #[must_use]
    pub fn document_to_view_expr(document: &UiIrDocument, root: &CodegenRoot) -> String {
        let nodes = emit_nodes_block(std::slice::from_ref(&document.root()), 1, root);
        let view = root.view();
        format!(
            "    {view}::View::from_fragment(\n\
             \x20       {view}::ViewFragment::from_nodes({nodes}),\n\
             \x20   )"
        )
    }
}

#[doc = include_str!("view_codegen.doc/emit_nodes_block.md")]
fn emit_nodes_block(nodes: &[&UiIrNode], indent: usize, root: &CodegenRoot) -> String {
    let pad = indentation(indent);
    let inner = indentation(indent + 1);
    let view = root.view();
    let mut out = String::new();
    out.push_str("{\n");
    let _ = writeln!(
        out,
        "{inner}let mut __nodes: ::std::vec::Vec<{view}::ViewNode> = ::std::vec::Vec::new();"
    );
    emit_into(nodes, &mut out, indent + 1, None, root);
    let _ = writeln!(out, "{inner}__nodes");
    let _ = write!(out, "{pad}}}");
    out
}

#[doc = include_str!("view_codegen.doc/emit_into.md")]
fn emit_into(
    nodes: &[&UiIrNode],
    out: &mut String,
    indent: usize,
    key: Option<&UiIrExpression>,
    root: &CodegenRoot,
) {
    let mut index = 0;
    while index < nodes.len() {
        let node = nodes[index];
        if is_chain_head(node) {
            let end = chain_end(nodes, index);
            emit_conditional_chain(&nodes[index..end], out, indent, key, root);
            index = end;
            continue;
        }
        emit_node(node, out, indent, key, root);
        index += 1;
    }
}

#[doc = include_str!("view_codegen.doc/loop_slot.md")]
fn loop_slot(tag: &str) -> bool {
    matches!(tag, "Empty" | "Separator")
}

#[doc = include_str!("view_codegen.doc/emit_loop.md")]
fn emit_loop(node: &UiIrNode, out: &mut String, indent: usize, root: &CodegenRoot) {
    // A `<VirtualFor>` is not a `<For>`: it does not walk the collection at all.
    // It emits a virtual-collection marker the runtime realizes into a
    // viewport-bounded window (spec §4.9), so it is handled apart from the
    // per-item `for` below.
    if node.tag() == "VirtualFor" {
        emit_virtual_collection(node, out, indent, root);
        return;
    }

    let pad = indentation(indent);
    let inner = indentation(indent + 1);

    let (body, slots): (Vec<&UiIrNode>, Vec<&UiIrNode>) =
        node.children().iter().partition(|c| !loop_slot(c.tag()));
    let separator = slots.iter().find(|c| c.tag() == "Separator");
    let empty = slots.iter().find(|c| c.tag() == "Empty");
    let needs_flag = separator.is_some() || empty.is_some();

    let Some(each) = node.property("each").map(emit_condition) else {
        // No collection to walk. Emitting `for _ in &()` would be a type error in
        // generated code the author never wrote, so a `<For each=…>`-less loop
        // renders its `empty` slot — which is what "no items" means — and nothing
        // else.
        if let Some(empty) = empty {
            let children: Vec<&UiIrNode> = empty.children().iter().collect();
            emit_into(&children, out, indent, None, root);
        }
        return;
    };
    let binding = node
        .property("as")
        .map_or_else(|| "_".to_owned(), emit_condition);

    if needs_flag {
        let _ = writeln!(out, "{pad}let mut __first = true;");
    }
    let _ = writeln!(out, "{pad}for {binding} in {each}.iter() {{");
    if let Some(separator) = separator {
        let _ = writeln!(out, "{inner}if !__first {{");
        let children: Vec<&UiIrNode> = separator.children().iter().collect();
        emit_into(&children, out, indent + 2, None, root);
        let _ = writeln!(out, "{inner}}}");
    }
    if needs_flag {
        let _ = writeln!(out, "{inner}__first = false;");
    }
    emit_into(&body, out, indent + 1, node.key(), root);
    let _ = writeln!(out, "{pad}}}");

    if let Some(empty) = empty {
        let _ = writeln!(out, "{pad}if __first {{");
        let children: Vec<&UiIrNode> = empty.children().iter().collect();
        emit_into(&children, out, indent + 1, None, root);
        let _ = writeln!(out, "{pad}}}");
    }
}

#[doc = include_str!("view_codegen.doc/emit_virtual_collection.md")]
fn emit_virtual_collection(node: &UiIrNode, out: &mut String, indent: usize, root: &CodegenRoot) {
    let pad = indentation(indent);
    let inner = indentation(indent + 1);
    let view = root.view();

    let body: Vec<&UiIrNode> = node
        .children()
        .iter()
        .filter(|child| !loop_slot(child.tag()))
        .collect();
    let template = emit_nodes_block(&body, indent + 1, root);

    let _ = writeln!(
        out,
        "{pad}__nodes.push({view}::ViewNode::virtual_collection("
    );
    let _ = writeln!(out, "{inner}{view}::DEFAULT_VIRTUAL_ROW_EXTENT,");
    let _ = writeln!(out, "{inner}{view}::ViewFragment::from_nodes({template}),");
    let _ = writeln!(out, "{pad}));");
}

#[doc = include_str!("view_codegen.doc/is_chain_head.md")]
fn is_chain_head(node: &UiIrNode) -> bool {
    node.kind() == UiIrNodeKind::Conditional && node.tag() == "If"
}

#[doc = include_str!("view_codegen.doc/chain_end.md")]
fn chain_end(nodes: &[&UiIrNode], start: usize) -> usize {
    let mut end = start + 1;
    while end < nodes.len() && nodes[end].kind() == UiIrNodeKind::Conditional {
        let tag = nodes[end].tag();
        if tag == "Else" {
            return end + 1;
        }
        if tag != "ElseIf" {
            break;
        }
        end += 1;
    }
    end
}

#[doc = include_str!("view_codegen.doc/emit_conditional_chain.md")]
fn emit_conditional_chain(
    chain: &[&UiIrNode],
    out: &mut String,
    indent: usize,
    key: Option<&UiIrExpression>,
    root: &CodegenRoot,
) {
    let pad = indentation(indent);
    let _ = write!(out, "{pad}");
    for (position, branch) in chain.iter().enumerate() {
        let head = if branch.tag() == "Else" {
            " else ".to_owned()
        } else if position == 0 {
            format!("if {} ", branch_condition(branch))
        } else {
            format!(" else if {} ", branch_condition(branch))
        };
        out.push_str(&head);
        out.push_str("{\n");
        let children: Vec<&UiIrNode> = branch.children().iter().collect();
        emit_into(&children, out, indent + 1, key, root);
        let _ = write!(out, "{pad}}}");
    }
    out.push('\n');
}

#[doc = include_str!("view_codegen.doc/branch_condition.md")]
fn branch_condition(branch: &UiIrNode) -> String {
    branch
        .property("cond")
        .map_or_else(|| "false".to_owned(), emit_condition)
}

#[doc = include_str!("view_codegen.doc/emit_condition.md")]
fn emit_condition(expression: &UiIrExpression) -> String {
    match expression {
        UiIrExpression::Binding(path) => path.to_string(),
        UiIrExpression::Literal(UiIrValue::Bool(flag)) => flag.to_string(),
        UiIrExpression::Literal(_) | UiIrExpression::Call { .. } => "false".to_owned(),
    }
}

#[doc = include_str!("view_codegen.doc/emit_node.md")]
fn emit_node(
    node: &UiIrNode,
    out: &mut String,
    indent: usize,
    key: Option<&UiIrExpression>,
    root: &CodegenRoot,
) {
    if let Some(value) = node.interpolated_value() {
        emit_text(node, out, indent, key, &emit_text_content(value), root);
        return;
    }
    match node.kind() {
        UiIrNodeKind::Text => {
            let content = string_literal(node.content());
            emit_text(node, out, indent, key, &content, root);
        }
        UiIrNodeKind::Element => {
            emit_element(node, out, indent, key, TagPosition::Element, root);
        }
        UiIrNodeKind::Component => {
            emit_element(node, out, indent, key, TagPosition::Component, root);
        }
        UiIrNodeKind::Loop => emit_loop(node, out, indent, root),
        UiIrNodeKind::Fragment | UiIrNodeKind::Conditional => {
            let children: Vec<&UiIrNode> = node.children().iter().collect();
            emit_into(&children, out, indent, key, root);
        }
    }
}

#[doc = include_str!("view_codegen.doc/emit_text.md")]
fn emit_text(
    node: &UiIrNode,
    out: &mut String,
    indent: usize,
    key: Option<&UiIrExpression>,
    content: &str,
    root: &CodegenRoot,
) {
    let pad = indentation(indent);
    let view = root.view();
    let _ = write!(out, "{pad}__nodes.push({view}::ViewNode::text({content})");
    emit_key(node, out, key, root);
    out.push_str(");\n");
}

#[doc = include_str!("view_codegen.doc/emit_text_content.md")]
fn emit_text_content(expression: &UiIrExpression) -> String {
    match expression {
        UiIrExpression::Binding(path) => format!("::std::string::ToString::to_string(&({path}))"),
        UiIrExpression::Literal(UiIrValue::Text(text)) => string_literal(text),
        UiIrExpression::Literal(UiIrValue::Bool(flag)) => string_literal(&flag.to_string()),
        UiIrExpression::Literal(UiIrValue::Int(number)) => string_literal(&number.to_string()),
        UiIrExpression::Literal(UiIrValue::Float(number)) => string_literal(&number.to_string()),
        UiIrExpression::Literal(UiIrValue::Null) | UiIrExpression::Call { .. } => {
            string_literal("")
        }
    }
}

#[doc = include_str!("view_codegen.doc/emit_element.md")]
fn emit_element(
    node: &UiIrNode,
    out: &mut String,
    indent: usize,
    key: Option<&UiIrExpression>,
    position: TagPosition,
    root: &CodegenRoot,
) {
    let pad = indentation(indent);
    let inner = indentation(indent + 1);
    let view = root.view();
    let identity = root.identity();
    let component = name_binding::component_id_for(node.tag(), position).raw();
    let _ = writeln!(
        out,
        "{pad}__nodes.push({view}::ViewNode::element({identity}::ComponentId::from_raw({component:#x}))"
    );

    for property in node.properties() {
        // Tag-aware, so `<slot name="header"/>` emits the reserved SLOT symbol the
        // runtime's `project_slots` reads rather than a hashed `name` prop. The
        // runtime bridge makes the identical call, which is what keeps an
        // interpreted document and a compiled one the same outlet.
        let symbol = name_binding::attr_symbol_on(node.tag(), property.name()).raw();
        let value = emit_value(property.value(), root);
        let _ = writeln!(
            out,
            "{inner}.with_prop({view}::ViewProperty::new({identity}::InternedSymbol::from_raw({symbol:#x}), {value}))"
        );
    }

    // The tag itself, as the reserved TYPE property — the element's local name, so
    // a `button { … }` / `td { … }` type selector matches what the author wrote
    // (spec §4.13). Emitted *after* the authored properties, so every authored one
    // keeps the index it was written at; skipped entirely when the author bound
    // `type` themselves, which is `local_name_for`'s author-wins rule rather than
    // this call site's. The runtime bridge emits the same property from the same
    // call, which is what keeps an interpreted document and a compiled one the same
    // element.
    if let Some(local_name) = name_binding::local_name_for(
        node.tag(),
        position,
        node.properties().iter().map(UiIrProperty::name),
    ) {
        let symbol = name_binding::TYPE.raw();
        // Through `emit_literal`, so the implicit local name is spelled exactly as
        // an authored `type="button"` would be — one shape for the one symbol.
        let value = emit_literal(&UiIrValue::Text(local_name.into()), root);
        let _ = writeln!(
            out,
            "{inner}.with_prop({view}::ViewProperty::new({identity}::InternedSymbol::from_raw({symbol:#x}), {value}))"
        );
    }

    for event in node.events() {
        let event_symbol = name_binding::event_symbol(event.event()).raw();
        let command = name_binding::fallback_symbol(event.handler()).raw();
        let _ = writeln!(
            out,
            "{inner}.with_event({view}::ViewEventBinding::new({identity}::InternedSymbol::from_raw({event_symbol:#x}), {identity}::InternedSymbol::from_raw({command:#x})))"
        );
    }

    let children: Vec<&UiIrNode> = node.children().iter().collect();
    if !children.is_empty() {
        let block = emit_nodes_block(&children, indent + 1, root);
        let _ = writeln!(
            out,
            "{inner}.with_children({view}::ViewFragment::from_nodes({block}))"
        );
    }

    // The key modifier (if any) closes the builder chain before the comma.
    let _ = write!(out, "{inner}");
    emit_key(node, out, key, root);
    out.push_str(");\n");
}

#[doc = include_str!("view_codegen.doc/emit_key.md")]
fn emit_key(
    node: &UiIrNode,
    out: &mut String,
    inherited: Option<&UiIrExpression>,
    root: &CodegenRoot,
) {
    let Some(key) = node.key().or(inherited) else {
        return;
    };
    let view = root.view();
    let key_expr = match key {
        UiIrExpression::Literal(UiIrValue::Int(number)) => {
            format!("{view}::ViewKey::from({}u64)", number.cast_unsigned())
        }
        UiIrExpression::Literal(UiIrValue::Text(text)) => {
            format!("{view}::ViewKey::from({})", string_literal(text))
        }
        // The key's Rust path, exactly like any other binding: `key={item.id}` is
        // `ViewKey::from(item.id.clone())`. It used to emit the path's spelling,
        // so every item in a list shared the key `"item.id"` — one identity for
        // the whole collection, which is the opposite of what §2.11's mandatory
        // stable key is for.
        UiIrExpression::Binding(path) => {
            format!("{view}::ViewKey::from({path}.clone())")
        }
        UiIrExpression::Literal(UiIrValue::Bool(flag)) => format!(
            "{view}::ViewKey::from({:?})",
            if *flag { "true" } else { "false" }
        ),
        UiIrExpression::Literal(UiIrValue::Float(number)) => {
            format!(
                "{view}::ViewKey::from({})",
                string_literal(&number.to_string())
            )
        }
        UiIrExpression::Literal(UiIrValue::Null) => {
            format!("{view}::ViewKey::from(\"null\")")
        }
        UiIrExpression::Call { name, .. } => {
            format!("{view}::ViewKey::from({})", string_literal(name))
        }
    };
    let _ = write!(out, ".with_key({key_expr})");
}

#[doc = include_str!("view_codegen.doc/emit_value.md")]
fn emit_value(expression: &UiIrExpression, root: &CodegenRoot) -> String {
    let view = root.view();
    match expression {
        UiIrExpression::Literal(value) => emit_literal(value, root),
        UiIrExpression::Binding(path) => {
            format!("{view}::ViewValue::from({path}.clone())")
        }
        UiIrExpression::Call { .. } => format!("{view}::ViewValue::Null"),
    }
}

#[doc = include_str!("view_codegen.doc/emit_literal.md")]
fn emit_literal(value: &UiIrValue, root: &CodegenRoot) -> String {
    let view = root.view();
    match value {
        UiIrValue::Null => format!("{view}::ViewValue::Null"),
        UiIrValue::Bool(flag) => format!("{view}::ViewValue::Bool({flag})"),
        UiIrValue::Int(number) => format!("{view}::ViewValue::Int({number})"),
        UiIrValue::Float(number) => format!("{view}::ViewValue::Float({number:?})"),
        UiIrValue::Text(text) => {
            format!("{view}::ViewValue::Text({}.into())", string_literal(text))
        }
    }
}

#[doc = include_str!("view_codegen.doc/string_literal.md")]
fn string_literal(text: &str) -> String {
    format!("{text:?}")
}

#[doc = include_str!("view_codegen.doc/indentation.md")]
fn indentation(indent: usize) -> String {
    "    ".repeat(indent)
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_ui_ir::document::{UiIrEvent, UiIrProperty};

    /// The emitted expression for `node`, through the root an application gets.
    fn expr(node: UiIrNode) -> String {
        ViewCodegen::document_to_view_expr(&UiIrDocument::new(node), &CodegenRoot::facade())
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
        let out =
            expr(UiIrNode::element(0, "img").with_property(UiIrProperty::text("src", "logo.png")));
        // Image::COMPONENT_ID == 0x00C0_0003; SRC symbol == 0x4.
        assert!(out.contains("ComponentId::from_raw(0xc00003)"), "{out}");
        assert!(out.contains("InternedSymbol::from_raw(0x4)"), "{out}");
        assert!(
            out.contains(r#"ViewValue::Text("logo.png".into())"#),
            "{out}"
        );
    }

    #[test]
    fn an_element_tag_emits_its_local_name_under_the_type_symbol() {
        // The generated code carries the element's own name, so the `button { … }`
        // rules in the UA sheet (and an author's) match what the author wrote.
        // TYPE symbol == 0x3; Button::COMPONENT_ID == 0x00D0_0001, because a
        // lowercase `button` is the same control as `<Button/>`.
        let out = expr(UiIrNode::element(0, "button"));
        assert!(out.contains("ComponentId::from_raw(0xd00001)"), "{out}");
        assert!(
            out.contains(r#"InternedSymbol::from_raw(0x3), ::obscur_kit::view::view::ViewValue::Text("button".into())"#),
            "{out}"
        );
    }

    #[test]
    fn an_authored_type_replaces_the_implicit_local_name() {
        // One property under the shared symbol, carrying the author's value:
        // `<button type="submit">` is HTML's spelling and the author owns the slot.
        let out = expr(
            UiIrNode::element(0, "button").with_property(UiIrProperty::text("type", "submit")),
        );
        assert_eq!(
            out.matches("InternedSymbol::from_raw(0x3)").count(),
            1,
            "{out}"
        );
        assert!(out.contains(r#"ViewValue::Text("submit".into())"#), "{out}");
        assert!(
            !out.contains(r#"ViewValue::Text("button".into())"#),
            "{out}"
        );
    }

    #[test]
    fn a_component_tag_emits_no_local_name() {
        let out = expr(UiIrNode::component(0, "Card"));
        assert!(!out.contains("InternedSymbol::from_raw(0x3)"), "{out}");
    }

    #[test]
    fn the_emitted_local_name_is_what_the_runtime_bridge_lowers() {
        // The two paths must stay byte-for-byte equivalent, and this property is the
        // newest thing they both invent: an interpreted document and a compiled one
        // would otherwise be styleable by type in only one of them. Compared through
        // the *values*, since one path emits Rust text and the other a `ViewNode`.
        for tag in ["button", "div", "td"] {
            let node = UiIrNode::element(0, tag);
            let view =
                crate::bridge::UiIrViewBridge::document_to_view(&UiIrDocument::new(node.clone()));
            let obscur_view::view::ViewNode::Element { props, .. } = &view.root().nodes()[0] else {
                panic!("element");
            };
            let bridged = obscur_view::view::StyleAttributes::extract(props);
            let out = expr(node);
            assert_eq!(bridged.local_name(), Some(tag), "bridge: {tag}");
            assert!(
                out.contains(&format!(r#"ViewValue::Text("{tag}".into())"#)),
                "codegen: {out}"
            );
        }
    }

    #[test]
    fn click_event_emits_the_reserved_event_symbol() {
        let out =
            expr(UiIrNode::component(0, "Button").with_event(UiIrEvent::new("click", "submit")));
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
    fn a_binding_emits_its_rust_path_not_its_spelling() {
        // This test used to assert the opposite, and that is the point of it: a
        // binding emitted `ViewValue::Text("user.name")` — the literal *text* of
        // the path — so every dynamic `.okui` document rendered its own source
        // code. Now the path is Rust, and rustc resolves it in the caller's scope.
        let out = expr(
            UiIrNode::element(0, "div").with_property(UiIrProperty::binding("title", "user.name")),
        );
        assert!(
            out.contains("ViewValue::from(user.name.clone())"),
            "the path must be emitted as an expression: {out}"
        );
        assert!(
            !out.contains(r#""user.name""#),
            "and never as its own spelling: {out}"
        );
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
        let src = ViewCodegen::document_to_view_fn(
            &UiIrDocument::new(UiIrNode::element(0, "div")),
            "app_view",
            &CodegenRoot::facade(),
        );
        assert!(
            src.contains("pub fn app_view() -> ::obscur_kit::view::view::View {"),
            "{src}"
        );
        assert!(src.trim_end().ends_with('}'), "{src}");
    }

    #[test]
    fn the_facade_root_names_no_internal_crate() {
        // THE 1-A regression. Every emitted path has to resolve in a crate whose
        // only dependency is the facade, so `::obscur_view` / `::obscur_identity`
        // — which live in no application's extern prelude — must not appear at
        // all: not in the signature, not in the `Vec<ViewNode>` annotation, not in
        // a property, an event, a key or a value.
        let src = ViewCodegen::document_to_view_fn(
            &UiIrDocument::new(
                UiIrNode::element(0, "div")
                    .with_property(UiIrProperty::text("class", "row"))
                    .with_event(UiIrEvent::new("click", "save"))
                    .with_child(UiIrNode::component(1, "Button"))
                    .with_child(UiIrNode::text(2, "hi")),
            ),
            "app_view",
            &CodegenRoot::facade(),
        );
        assert!(!src.contains("::obscur_view"), "{src}");
        assert!(!src.contains("::obscur_identity"), "{src}");
        assert!(
            src.contains("::obscur_kit::view::view::ViewNode::element"),
            "{src}"
        );
        assert!(
            src.contains("::obscur_kit::identity::ComponentId::from_raw"),
            "{src}"
        );
        assert!(
            src.contains("::obscur_kit::identity::InternedSymbol::from_raw"),
            "{src}"
        );
    }

    #[test]
    fn the_internal_root_still_names_the_crates_directly() {
        // The other half of the same claim: a crate that *does* depend on both
        // internal crates gets the paths that resolve there, and nothing about
        // the facade leaks into them.
        let src = ViewCodegen::document_to_view_fn(
            &UiIrDocument::new(UiIrNode::element(0, "div")),
            "app_view",
            &CodegenRoot::internal(),
        );
        assert!(
            src.contains("pub fn app_view() -> ::obscur_view::view::View {"),
            "{src}"
        );
        assert!(!src.contains("obscur_kit"), "{src}");
    }

    #[test]
    fn a_custom_root_reaches_every_emitted_path() {
        // A root nobody ships, so any path the threading missed shows up as a
        // leftover `obscur_` spelling rather than as a silent default.
        let src = ViewCodegen::document_to_view_fn(
            &UiIrDocument::new(
                UiIrNode::element(0, "div")
                    .with_property(UiIrProperty::literal("count", UiIrValue::Int(1)))
                    .with_event(UiIrEvent::new("click", "save"))
                    .with_child(
                        UiIrNode::text(1, "hi")
                            .with_key(UiIrExpression::Literal(UiIrValue::Text("k".into()))),
                    ),
            ),
            "app_view",
            &CodegenRoot::new("::k::v", "::k::i"),
        );
        assert!(!src.contains("obscur_"), "{src}");
        assert!(src.contains("::k::v::ViewKey::from"), "{src}");
        assert!(src.contains("::k::i::InternedSymbol::from_raw"), "{src}");
    }
}
