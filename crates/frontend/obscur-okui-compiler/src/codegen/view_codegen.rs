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
//! A tag naming **no** built-in resolves by the node's kind: a `<div>` is the
//! `Container` box, and a `<Card>` is
//! [`user_component_id("Card")`](name_binding::user_component_id) — the id
//! `#[derive(Contract)]` computes for the same name, so a host that registers
//! `CardProps::contract().id` mounts the author's component under the authored
//! tag. Codegen never asks the §2.14 registry what `Card` is and could not: that
//! registry is a runtime value of `Box<dyn ComponentFactory>` trait objects,
//! while this runs during the host crate's own compilation. The name is what the
//! two sides share.
//!
//! What that does not buy is a diagnostic: a tag naming a component nobody
//! registered emits an id nothing mounts, and — since no part of matching,
//! layout or paint reads a `ComponentId` — still renders as the same box it
//! silently rendered as before. See [`name_binding`]'s module docs for why that
//! rule cannot be stated once for both front ends.
//!
//! # A binding is a Rust path
//!
//! `{open}` in a `.okui` document emits the Rust expression `open`, verbatim.
//! There is no evaluator here and there does not need to be one: the emitted code
//! lands in a scope and **Rust resolves it**. A path that names nothing is a Rust
//! compile error, in the caller's build, pointing at the caller's code. A field
//! whose type cannot become a [`ViewValue`](obscur_view::view::ViewValue) — or,
//! for a body `{expr}`, cannot `Display` — is the same. That is the whole type
//! system of the expression language, borrowed rather than rebuilt.
//!
//! This holds in `<div>{user.name}</div>` too, and that took a second commit: the
//! body case kept emitting the path's *spelling* as literal text long after the
//! attribute case stopped. See [`emit_text_content`].
//!
//! This is also what makes §2.10's restrictions meaningful rather than
//! decorative: the allow-list polices what may appear in a path that is about to
//! become real Rust.
//!
//! # Which scope, and the gap for a compiled file
//!
//! For the inline `okui!` macro the scope is the **caller's**, and a binding reads
//! the caller's locals — that is the path the tests exercise end to end. For a
//! compiled file, [`document_to_view_fn`](ViewCodegen::document_to_view_fn) emits
//! a **nullary** `pub fn <name>() -> View`, so the only scope a binding can
//! resolve in is whatever the generated module has around it (a `const`, a
//! `static`, an import). A file whose bindings read per-call state does not
//! compile — the function has no parameters to read it from — and giving it them
//! needs a decision about the props signature that nothing here makes yet. The
//! emitted expression is right on both front ends; only the file front end's
//! function shape limits what a binding can name.
//!
//! # Conditionals are real conditionals
//!
//! `<If cond={open}>` emits `if open { … }`. `<ElseIf>` / `<Else>` following it
//! join the same chain, so exactly one branch renders — the point of having them.
//!
//! Emitting a real `if` is why the children of a fragment are collected into a
//! `Vec` with `push` rather than written as a `vec![…]` literal: a conditional is
//! a *statement*, and there is no way to spell one as an element of a list
//! literal. The shape of the generated code follows from the language, not from
//! preference.
//!
//! # Loops are real loops
//!
//! `<For each={items} as={item} key={item.id}>` emits `for item in &items { … }`.
//! `as` names the loop variable, and it is a Rust binding like any other — which
//! is what lets the body's `{item.name}` be an ordinary path. The collection is
//! borrowed, so a `<For>` never consumes the caller's data.
//!
//! `<Empty>` and `<Separator>` children are the §2.11 slots, not template nodes;
//! `<Loading>` is deliberately absent — see [`loop_slot`].

use std::fmt::Write as _;

use obscur_ui_ir::document::{UiIrDocument, UiIrExpression, UiIrNode, UiIrNodeKind, UiIrValue};
use obscur_ui_ir::name_binding::{self, TagPosition};

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
        let nodes = emit_nodes_block(std::slice::from_ref(&document.root()), 1);
        format!(
            "    ::obscur_view::view::View::from_fragment(\n\
             \x20       ::obscur_view::view::ViewFragment::from_nodes({nodes}),\n\
             \x20   )"
        )
    }
}

/// Emits a block expression evaluating to the `Vec<ViewNode>` that `nodes`
/// lower to, at `indent` levels.
///
/// A `Vec` built with `push` rather than a `vec![…]` literal, because a
/// conditional emits a real `if` — a statement, which no list literal can hold.
/// Each block declares its own sink, so a nested fragment's pushes cannot reach
/// an outer one.
fn emit_nodes_block(nodes: &[&UiIrNode], indent: usize) -> String {
    let pad = indentation(indent);
    let inner = indentation(indent + 1);
    let mut out = String::new();
    out.push_str("{\n");
    let _ = writeln!(
        out,
        "{inner}let mut __nodes: ::std::vec::Vec<::obscur_view::view::ViewNode> = \
         ::std::vec::Vec::new();"
    );
    emit_into(nodes, &mut out, indent + 1, None);
    let _ = writeln!(out, "{inner}__nodes");
    let _ = write!(out, "{pad}}}");
    out
}

/// Appends the statements that push `nodes` into the enclosing `__nodes` sink,
/// at `indent` levels (4 spaces each).
///
/// # Conditionals are grouped, not emitted one by one
///
/// `<If>` / `<ElseIf>` / `<Else>` arrive as *siblings*, but they are one
/// construct: emitting each on its own would produce `if a {…} if b {…} {…}` —
/// three independent statements, the last of which always runs. So a run of them
/// is collected and emitted as a single `if / else if / else` chain, which is the
/// only shape where exactly one branch renders.
///
/// A run ends at the first non-conditional sibling, or after an `<Else>` (nothing
/// may follow the fallback). An `<ElseIf>` / `<Else>` with no `<If>` before it
/// starts no chain and is emitted as its own branch, which is what a stray
/// `<Else>` deserves: it renders unconditionally rather than silently attaching
/// to an unrelated `<If>` further up.
///
/// # The inherited key
///
/// `key` is the identity a `<For>` gives the node it renders for one item. It is
/// threaded down rather than applied at the loop, because the item's root may sit
/// under a conditional: `<For><If cond={x}><Row/></If></For>` must still key the
/// `Row`. A node with a key of its own keeps it; `None` outside a loop leaves
/// every node exactly as before.
fn emit_into(nodes: &[&UiIrNode], out: &mut String, indent: usize, key: Option<&UiIrExpression>) {
    let mut index = 0;
    while index < nodes.len() {
        let node = nodes[index];
        if is_chain_head(node) {
            let end = chain_end(nodes, index);
            emit_conditional_chain(&nodes[index..end], out, indent, key);
            index = end;
            continue;
        }
        emit_node(node, out, indent, key);
        index += 1;
    }
}

/// The reserved child tags of a `<For>` that are slots rather than the item
/// template (spec §2.11).
///
/// `<Loading>` is **not** here. `empty` and `separator` are decidable from the
/// collection itself — it has a length, and iteration has a between — but nothing
/// about a Rust collection says it is still loading: that needs a signal from
/// outside, and the spec does not say what it is. Inventing one (`loading={…}`,
/// say) would be putting words in the spec's mouth, so the slot stays unparsed
/// and `<Loading>` is treated as an ordinary child until the signal is defined.
fn loop_slot(tag: &str) -> bool {
    matches!(tag, "Empty" | "Separator")
}

/// Emits a `<For>` as a real Rust `for` loop, with its `empty` / `separator`
/// slots (spec §2.11).
///
/// The shape:
///
/// ```ignore
/// let mut __first = true;            // only when a slot needs it
/// for item in &items {
///     if !__first { /* separator */ }
///     __first = false;
///     __nodes.push(/* body, keyed */);
/// }
/// if __first { /* empty */ }
/// ```
///
/// One flag serves both slots, and that is not a trick: "no separator before the
/// first item" and "the collection was empty" are the same question — did an
/// iteration happen yet.
///
/// `as={item}` names the loop variable, and it is a Rust binding like any other:
/// the body's `{item.name}` is then an ordinary path (see the module docs).
/// Without `as` the item is bound to `_`, which renders the body once per item
/// with no way to read it — a repeat, which is what a `<For>` that names nothing
/// asks for.
///
/// The collection is walked with `.iter()`, not `&{each}`. Both borrow, so a
/// `<For>` never consumes the caller's data — but `&{each}` breaks the moment the
/// binding is already a reference (`&[T]` becomes `&&[T]`, which is not an
/// iterator), and a caller passing a slice is not doing anything unusual.
/// `.iter()` reads the same on a `Vec`, a slice, an array and a borrow of any of
/// them.
fn emit_loop(node: &UiIrNode, out: &mut String, indent: usize) {
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
            emit_into(&children, out, indent, None);
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
        emit_into(&children, out, indent + 2, None);
        let _ = writeln!(out, "{inner}}}");
    }
    if needs_flag {
        let _ = writeln!(out, "{inner}__first = false;");
    }
    emit_into(&body, out, indent + 1, node.key());
    let _ = writeln!(out, "{pad}}}");

    if let Some(empty) = empty {
        let _ = writeln!(out, "{pad}if __first {{");
        let children: Vec<&UiIrNode> = empty.children().iter().collect();
        emit_into(&children, out, indent + 1, None);
        let _ = writeln!(out, "{pad}}}");
    }
}

/// Whether `node` opens a conditional chain (an `<If>`).
fn is_chain_head(node: &UiIrNode) -> bool {
    node.kind() == UiIrNodeKind::Conditional && node.tag() == "If"
}

/// The exclusive end of the conditional chain starting at `start`: every
/// following `<ElseIf>`, up to and including the first `<Else>`.
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

/// Emits one `if / else if / else` chain, each branch pushing its own body.
fn emit_conditional_chain(
    chain: &[&UiIrNode],
    out: &mut String,
    indent: usize,
    key: Option<&UiIrExpression>,
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
        emit_into(&children, out, indent + 1, key);
        let _ = write!(out, "{pad}}}");
    }
    out.push('\n');
}

/// The Rust condition expression for one branch.
///
/// A missing `cond` on an `<If>` / `<ElseIf>` emits `false`: a condition that was
/// never stated is not true, so the branch does not render. The alternative —
/// rendering a branch whose condition is absent — is the silent-wrong-output
/// behaviour this whole path exists to remove.
fn branch_condition(branch: &UiIrNode) -> String {
    branch
        .property("cond")
        .map_or_else(|| "false".to_owned(), emit_condition)
}

/// Emits an expression in boolean position.
///
/// A binding is its Rust path, verbatim — `cond={open}` is `if open`. A literal
/// `true`/`false` is itself. Anything else (a number, a string, a call) is not a
/// condition, and emits `false` rather than a type error inside generated code
/// the author cannot see.
fn emit_condition(expression: &UiIrExpression) -> String {
    match expression {
        UiIrExpression::Binding(path) => path.to_string(),
        UiIrExpression::Literal(UiIrValue::Bool(flag)) => flag.to_string(),
        UiIrExpression::Literal(_) | UiIrExpression::Call { .. } => "false".to_owned(),
    }
}

/// Appends the statement pushing the single `ViewNode` `node` lowers to.
///
/// A transparent fragment and a (still-spliced) loop push their children instead
/// of a node of their own. An [interpolation](UiIrNode::interpolation) pushes a
/// **text** node — it is a body `{expr}`, and its content is the expression.
fn emit_node(node: &UiIrNode, out: &mut String, indent: usize, key: Option<&UiIrExpression>) {
    if let Some(value) = node.interpolated_value() {
        emit_text(node, out, indent, key, &emit_text_content(value));
        return;
    }
    match node.kind() {
        UiIrNodeKind::Text => {
            let content = string_literal(node.content());
            emit_text(node, out, indent, key, &content);
        }
        UiIrNodeKind::Element => emit_element(node, out, indent, key, TagPosition::Element),
        UiIrNodeKind::Component => emit_element(node, out, indent, key, TagPosition::Component),
        UiIrNodeKind::Loop => emit_loop(node, out, indent),
        UiIrNodeKind::Fragment | UiIrNodeKind::Conditional => {
            let children: Vec<&UiIrNode> = node.children().iter().collect();
            emit_into(&children, out, indent, key);
        }
    }
}

/// Appends the statement pushing a `ViewNode::text` whose content is the
/// already-emitted Rust expression `content`.
fn emit_text(
    node: &UiIrNode,
    out: &mut String,
    indent: usize,
    key: Option<&UiIrExpression>,
    content: &str,
) {
    let pad = indentation(indent);
    let _ = write!(
        out,
        "{pad}__nodes.push(::obscur_view::view::ViewNode::text({content})"
    );
    emit_key(node, out, key);
    out.push_str(");\n");
}

/// Emits the Rust expression for a text node's content.
///
/// A binding is its **Rust path**, displayed: `<div>{user.name}</div>` becomes
/// `ViewNode::text(ToString::to_string(&(user.name)))`. It used to emit the path
/// as a string *literal*, so `{user.name}` rendered the nine characters
/// `user.name` — the document showing its own source. That was fixed for
/// attributes and left live here, on the more common case.
///
/// `Display` is the bound, where [`emit_value`] uses `Into<ViewValue>`, because
/// text content is a string and a body `{count}` must render `7` rather than fail
/// to compile. A type with no `Display` is **rustc's** error in the caller's
/// build — the same borrowed type system every other binding leans on. (The
/// emitted tokens are re-parsed from text, so that error lands on the whole
/// `okui!` invocation, not on the offending expression; only the §2.10 check,
/// which runs on the parsed input, is spanned that precisely.) Displaying by
/// reference, so reading a field for text never moves the caller's data.
///
/// The path is parenthesised because a "path" is only mostly a path: both front
/// ends fall back to a binding on the raw text for anything they do not classify
/// as a literal, so `{a + b}` arrives here as the binding `a + b`, and
/// `&a + b` is not what the author wrote.
///
/// A call emits the empty string for the reason [`emit_value`] emits `Null`:
/// §2.10 allows only "explicitly exported pure functions", and nothing yet decides
/// which those are.
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

/// Appends the statement pushing a single `ViewNode::Element` for `node`.
///
/// `position` comes from the caller's [`UiIrNodeKind`], which is where the front
/// ends already recorded their component-vs-element verdict; it decides only what
/// an unrecognised tag falls back to (a `Container` box, or the component's own
/// name-derived id — see [`name_binding::component_id_for`]).
fn emit_element(
    node: &UiIrNode,
    out: &mut String,
    indent: usize,
    key: Option<&UiIrExpression>,
    position: TagPosition,
) {
    let pad = indentation(indent);
    let inner = indentation(indent + 1);
    let component = name_binding::component_id_for(node.tag(), position).raw();
    let _ = writeln!(
        out,
        "{pad}__nodes.push(::obscur_view::view::ViewNode::element(::obscur_identity::ComponentId::from_raw({component:#x}))"
    );

    for property in node.properties() {
        // Tag-aware, so `<slot name="header"/>` emits the reserved SLOT symbol the
        // runtime's `project_slots` reads rather than a hashed `name` prop. The
        // runtime bridge makes the identical call, which is what keeps an
        // interpreted document and a compiled one the same outlet.
        let symbol = name_binding::attr_symbol_on(node.tag(), property.name()).raw();
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
        let block = emit_nodes_block(&children, indent + 1);
        let _ = writeln!(
            out,
            "{inner}.with_children(::obscur_view::view::ViewFragment::from_nodes({block}))"
        );
    }

    // The key modifier (if any) closes the builder chain before the comma.
    let _ = write!(out, "{inner}");
    emit_key(node, out, key);
    out.push_str(");\n");
}

/// Appends a `.with_key(…)` modifier for `node`'s key, if it has one; a no-op
/// otherwise. Mirrors the runtime bridge's key lowering.
fn emit_key(node: &UiIrNode, out: &mut String, inherited: Option<&UiIrExpression>) {
    let Some(key) = node.key().or(inherited) else {
        return;
    };
    let key_expr = match key {
        UiIrExpression::Literal(UiIrValue::Int(number)) => {
            format!(
                "::obscur_view::view::ViewKey::from({}u64)",
                number.cast_unsigned()
            )
        }
        UiIrExpression::Literal(UiIrValue::Text(text)) => {
            format!(
                "::obscur_view::view::ViewKey::from({})",
                string_literal(text)
            )
        }
        // The key's Rust path, exactly like any other binding: `key={item.id}` is
        // `ViewKey::from(item.id.clone())`. It used to emit the path's spelling,
        // so every item in a list shared the key `"item.id"` — one identity for
        // the whole collection, which is the opposite of what §2.11's mandatory
        // stable key is for.
        UiIrExpression::Binding(path) => {
            format!("::obscur_view::view::ViewKey::from({path}.clone())")
        }
        UiIrExpression::Literal(UiIrValue::Bool(flag)) => format!(
            "::obscur_view::view::ViewKey::from({:?})",
            if *flag { "true" } else { "false" }
        ),
        UiIrExpression::Literal(UiIrValue::Float(number)) => {
            format!(
                "::obscur_view::view::ViewKey::from({})",
                string_literal(&number.to_string())
            )
        }
        UiIrExpression::Literal(UiIrValue::Null) => {
            "::obscur_view::view::ViewKey::from(\"null\")".to_owned()
        }
        UiIrExpression::Call { name, .. } => {
            format!(
                "::obscur_view::view::ViewKey::from({})",
                string_literal(name)
            )
        }
    };
    let _ = write!(out, ".with_key({key_expr})");
}

/// Emits the `ViewValue` expression for a property's value expression.
///
/// A binding emits its **Rust path**, converted through
/// [`ViewValue::from`](obscur_view::view::ViewValue): `value={label}` becomes
/// `ViewValue::from(label.clone())`. It used to emit the path as a string
/// literal, so `value={user.name}` produced the literal text `"user.name"` — the
/// placeholder that made every dynamic `.okui` document silently wrong.
///
/// The `.clone()` is uniform rather than clever: it works for an owned `String`
/// and for a `Copy` number alike, and a binding read into a `ViewValue` cannot
/// move out of the caller's data — a `<For>` body would read the same field on
/// every iteration.
///
/// If the path names nothing, or names something no `ViewValue` can be built
/// from, **rustc says so** in the caller's build. That is the type checking, and
/// it is not reimplemented here.
///
/// A call still emits `Null`: §2.10 allows "explicitly exported pure functions",
/// but nothing yet decides which functions those are, and emitting an arbitrary
/// call would be exactly the unconstrained invocation §2.10 forbids.
fn emit_value(expression: &UiIrExpression) -> String {
    match expression {
        UiIrExpression::Literal(value) => emit_literal(value),
        UiIrExpression::Binding(path) => {
            format!("::obscur_view::view::ViewValue::from({path}.clone())")
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
            format!(
                "::obscur_view::view::ViewValue::Text({}.into())",
                string_literal(text)
            )
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
        );
        assert!(
            src.contains("pub fn app_view() -> ::obscur_view::view::View {"),
            "{src}"
        );
        assert!(src.trim_end().ends_with('}'), "{src}");
    }
}
