//! `DslMacro`: expansion logic for the TSX-like Rust DSL (spec §2.8).
//!
//! This module parses a JSX/TSX-like element tree into a
//! [`UiIrDocument`] **while the macro runs**. The document is not emitted or
//! interpreted: `okui!` hands it straight to `ViewCodegen`, which lowers it to
//! ordinary `View`-building Rust, so no IR survives into the built crate.
//!
//! # Grammar
//!
//! - Element: `<Tag ...attrs> ...children </Tag>` or self-closing
//!   `<Tag ...attrs />`.
//! - Tag naming (TSX convention): an UPPERCASE-initial identifier (`Button`) is
//!   a component, a lowercase-initial identifier (`div`) is an element.
//! - Attributes:
//!   - `name="literal"` → [`UiIrProperty::text`](::obscur_ui_ir::document::UiIrProperty).
//!   - `name={expr}` → a literal when `expr` is a literal, a
//!     [`binding`](::obscur_ui_ir::document::UiIrProperty::binding) when it is a
//!     single ident or dotted field-access path, else a `binding` whose path is
//!     the stringified expression.
//!   - `name` (bare) → boolean-true literal property.
//!   - `on:click={handler}` → an [`UiIrEvent`](::obscur_ui_ir::document::UiIrEvent).
//! - Children (in order): nested elements, bare string literals (text nodes),
//!   and `{expr}` interpolations.
//!
//! # Representation choices
//!
//! - Binding/handler paths are normalised to a dotted, space-free string:
//!   `{user.name}` becomes the path `"user.name"` (not `"user . name"`).
//! - Interpolated `{expr}` children lower to an
//!   [`interpolation`](::obscur_ui_ir::document::UiIrNode::interpolation) node
//!   carrying the expression — the *same* node the `.okui` file lowering builds,
//!   so `ViewCodegen` emits the same text node from either front end. They used
//!   to disagree: this one built a literal text node holding the path's spelling,
//!   so `<div>{user.name}</div>` rendered the characters `user.name`.
//! - A lone top-level element is emitted as the document root directly; any
//!   other top level (multiple siblings, or a single text/interpolation node)
//!   is wrapped in a [`fragment`](::obscur_ui_ir::document::UiIrNode::fragment)
//!   root.
//!
//! # Id scheme
//!
//! Every node is assigned a stable sequential `u64` id during a single
//! pre-order walk (root = `0`, incrementing).

use obscur_okui_compiler::compile::ContractRegistry;
use obscur_okui_compiler::validation::{
    AttributeUse, ElementUse, MemberArity, MemberUse, SlotUse, ViolationAnchor, check_collection,
    check_contract,
};
use obscur_ui_ir::document::{
    UiIrDocument, UiIrEvent, UiIrExpression, UiIrNode, UiIrProperty, UiIrValue,
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream, Parser};
use syn::{
    Expr, ExprField, ExprLit, ExprPath, Ident, Lit, LitStr, Member, PathArguments, Token, braced,
    token,
};

/// Expander for the TSX-like Rust DSL (spec §2.8).
///
/// [`DslMacro::to_document`] parses the DSL into a real [`UiIrDocument`] during
/// expansion; `okui!` then lowers that document through `ViewCodegen`, so the
/// emitted code builds a `View` directly and no IR exists at runtime.
#[derive(Debug)]
pub(crate) struct DslMacro;

impl DslMacro {
    /// Parses the DSL token stream into a [`UiIrDocument`] at expansion time.
    ///
    /// The document is built, not emitted, so the caller can lower it further
    /// while the macro is still running.
    ///
    /// # Errors
    ///
    /// Returns a [`syn::Error`] on empty input, on malformed element syntax, on
    /// a mismatched closing tag, when a braced value is not a [`syn::Expr`], or
    /// when an element breaks its component's contract (§2.12, §2.13).
    pub(crate) fn to_document(input: TokenStream) -> Result<UiIrDocument, syn::Error> {
        let nodes = parse_nodes(input)?;
        validate_contracts(&nodes)?;
        let mut counter: u64 = 0;
        let root = if nodes.len() == 1 && matches!(nodes[0], DslNode::Element(_)) {
            build_node(&nodes[0], &mut counter)
        } else {
            let mut fragment = UiIrNode::fragment(take_id(&mut counter));
            for node in &nodes {
                fragment = fragment.with_child(build_node(node, &mut counter));
            }
            fragment
        };
        Ok(UiIrDocument::new(root))
    }
}

/// Parses the whole DSL token stream into its top-level nodes.
///
/// # Errors
///
/// Returns a [`syn::Error`] on empty input or malformed element syntax.
fn parse_nodes(input: TokenStream) -> Result<Vec<DslNode>, syn::Error> {
    let parser = |stream: ParseStream| -> syn::Result<Vec<DslNode>> {
        let mut nodes = Vec::new();
        while !stream.is_empty() {
            nodes.push(stream.parse::<DslNode>()?);
        }
        Ok(nodes)
    };
    let nodes = parser.parse2(input)?;
    if nodes.is_empty() {
        return Err(syn::Error::new(
            Span::call_site(),
            "expected a DSL element, but the input was empty",
        ));
    }
    Ok(nodes)
}

/// Whether `tag` names a conditional control-flow construct rather than an
/// element (spec §2.11).
///
/// The same three names the `.okui` file parser reserves, so the inline macro and
/// a compiled file classify a document identically — two front ends, one meaning.
fn is_conditional_tag(tag: &str) -> bool {
    matches!(tag, "If" | "ElseIf" | "Else")
}

/// Whether `tag` names a collection construct rather than an element
/// (spec §2.11).
///
/// `VirtualFor` is still classified with `For` — §2.11 says it is a collection,
/// and misclassifying it as an element would render a node called `VirtualFor`.
/// What it no longer does is *compile*:
/// [`check_collection`](obscur_okui_compiler::validation::check_collection)
/// refuses it in [`validate_element`], which runs before any of this, because the
/// emitter treats the two alike and a `<VirtualFor>` that silently materializes
/// every item is a performance contract nobody keeps. This predicate keeps both
/// names so the refusal reads as "a loop we will not compile" rather than "an
/// unknown component".
fn is_loop_tag(tag: &str) -> bool {
    matches!(tag, "For" | "VirtualFor")
}

/// Builds the [`UiIrNode`] for `node`, threading `counter` to assign pre-order
/// ids (root = 0, incrementing).
fn build_node(node: &DslNode, counter: &mut u64) -> UiIrNode {
    match node {
        DslNode::Text(content) => UiIrNode::text(take_id(counter), content.as_str()),
        DslNode::Interpolation(value) => {
            UiIrNode::interpolation(take_id(counter), build_expression(value))
        }
        DslNode::Element(element) => {
            let id = take_id(counter);
            let mut ir = if is_conditional_tag(element.tag.as_str()) {
                // `<If>` / `<ElseIf>` / `<Else>` are control flow, not elements —
                // the same classification the `.okui` file parser makes. Building
                // them as elements is what used to make the macro render every
                // branch of a chain: three `<span>`-like nodes named `If`,
                // `ElseIf` and `Else`, none of which conditioned anything.
                UiIrNode::conditional(id, element.tag.as_str())
            } else if is_loop_tag(element.tag.as_str()) {
                UiIrNode::loop_node(id, element.tag.as_str())
            } else if element.is_component {
                UiIrNode::component(id, element.tag.as_str())
            } else {
                UiIrNode::element(id, element.tag.as_str())
            };
            for attr in &element.attrs {
                ir = match attr {
                    DslAttr::Event { event, handler, .. } => {
                        ir.with_event(UiIrEvent::new(event.as_str(), handler.as_str()))
                    }
                    // A `<For>`'s mandatory `key` is carried on the Loop node
                    // itself (spec §2.11), not as a property — the same split the
                    // `.okui` file lowering makes. Left as a property it reaches
                    // no consumer, and every item in the list renders unkeyed.
                    DslAttr::Prop { name, value, .. }
                        if name == "key" && is_loop_tag(element.tag.as_str()) =>
                    {
                        ir.with_key(build_expression(value))
                    }
                    DslAttr::Prop { name, value, .. } => {
                        ir.with_property(build_property(name, value))
                    }
                };
            }
            for child in &element.children {
                ir = ir.with_child(build_node(child, counter));
            }
            ir
        }
    }
}

/// Builds the [`UiIrProperty`] for one attribute.
/// Builds the [`UiIrExpression`] for a bare attribute value, with no property
/// name around it — for the places a value stands alone, such as a `<For>`'s key.
fn build_expression(value: &PropValue) -> UiIrExpression {
    match value {
        PropValue::BareTrue => UiIrExpression::literal(UiIrValue::Bool(true)),
        PropValue::Text(text) => UiIrExpression::literal(UiIrValue::Text(text.as_str().into())),
        PropValue::Bool(flag) => UiIrExpression::literal(UiIrValue::Bool(*flag)),
        PropValue::Int(value) => UiIrExpression::literal(UiIrValue::Int(*value)),
        PropValue::Float(value) => UiIrExpression::literal(UiIrValue::Float(*value)),
        PropValue::Binding(path) => UiIrExpression::binding(path.as_str()),
    }
}

fn build_property(name: &str, value: &PropValue) -> UiIrProperty {
    match value {
        PropValue::BareTrue => UiIrProperty::literal(name, UiIrValue::Bool(true)),
        PropValue::Text(text) => UiIrProperty::text(name, text.as_str()),
        PropValue::Bool(flag) => UiIrProperty::literal(name, UiIrValue::Bool(*flag)),
        PropValue::Int(value) => UiIrProperty::literal(name, UiIrValue::Int(*value)),
        PropValue::Float(value) => UiIrProperty::literal(name, UiIrValue::Float(*value)),
        PropValue::Binding(path) => UiIrProperty::binding(name, path.as_str()),
    }
}

/// One parsed node of the DSL tree.
enum DslNode {
    /// A nested element or component.
    Element(DslElement),
    /// A bare string literal child, lowered to a literal text node.
    Text(String),
    /// A `{expr}` interpolation child, holding the resolved expression the
    /// interpolation node carries.
    Interpolation(PropValue),
}

/// A parsed element (or component instance) with its attributes and children.
struct DslElement {
    /// Whether the tag names a component (uppercase-initial) rather than an
    /// element.
    is_component: bool,
    /// The tag or component name.
    tag: String,
    /// The tag identifier's span, so a contract violation about the element as a
    /// whole lands on the tag the author wrote.
    span: Span,
    /// The parsed attributes, in source order.
    attrs: Vec<DslAttr>,
    /// The parsed children, in source order.
    children: Vec<DslNode>,
}

/// A parsed attribute: either a property or an `on:` event binding.
enum DslAttr {
    /// A named property with a resolved value.
    Prop {
        /// The property name.
        name: String,
        /// The name identifier's span.
        span: Span,
        /// The resolved property value.
        value: PropValue,
    },
    /// An `on:<event>={handler}` event binding.
    Event {
        /// The event name (the part after `on:`).
        event: String,
        /// The event identifier's span.
        span: Span,
        /// The handler name, normalised to a string.
        handler: String,
    },
}

/// A resolved attribute value.
enum PropValue {
    /// A bare attribute (`disabled`), meaning boolean-true.
    BareTrue,
    /// A literal string value.
    Text(String),
    /// A literal boolean value.
    Bool(bool),
    /// A literal integer value.
    Int(i64),
    /// A literal float value.
    Float(f64),
    /// A dynamic binding to a state/prop path (or stringified expression).
    Binding(String),
}

impl Parse for DslNode {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![<]) {
            Ok(Self::Element(input.parse()?))
        } else if input.peek(LitStr) {
            let literal: LitStr = input.parse()?;
            Ok(Self::Text(literal.value()))
        } else if input.peek(token::Brace) {
            let content;
            braced!(content in input);
            let expr: Expr = content.parse()?;
            // Through `classify_prop_value`, so a body `{expr}` faces §2.10's
            // allow-list exactly as an attribute's does. It never did before,
            // because it never evaluated: it was carried as text. Now it becomes
            // real Rust, and a binding that evaluates must be one that was checked.
            Ok(Self::Interpolation(classify_prop_value(&expr)?))
        } else {
            Err(input.error("expected an element `<..>`, a string literal, or `{ expr }`"))
        }
    }
}

impl Parse for DslElement {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![<]>()?;
        let tag_ident = Ident::parse_any(input)?;
        let tag = tag_ident.to_string();
        let is_component = tag.chars().next().is_some_and(char::is_uppercase);

        let mut attrs = Vec::new();
        while !input.peek(Token![/]) && !input.peek(Token![>]) {
            if input.is_empty() {
                return Err(syn::Error::new(
                    tag_ident.span(),
                    format!("unexpected end of input inside the opening tag `<{tag}>`"),
                ));
            }
            attrs.push(parse_attr(input)?);
        }

        if input.peek(Token![/]) {
            input.parse::<Token![/]>()?;
            input.parse::<Token![>]>()?;
            return Ok(Self {
                is_component,
                tag,
                span: tag_ident.span(),
                attrs,
                children: Vec::new(),
            });
        }
        input.parse::<Token![>]>()?;

        let mut children = Vec::new();
        loop {
            if input.peek(Token![<]) && input.peek2(Token![/]) {
                break;
            }
            if input.is_empty() {
                return Err(syn::Error::new(
                    tag_ident.span(),
                    format!("missing closing tag `</{tag}>`"),
                ));
            }
            children.push(input.parse::<DslNode>()?);
        }

        input.parse::<Token![<]>()?;
        input.parse::<Token![/]>()?;
        let close = Ident::parse_any(input)?;
        if close != tag_ident {
            return Err(syn::Error::new(
                close.span(),
                format!("mismatched closing tag: expected `</{tag}>`, found `</{close}>`"),
            ));
        }
        input.parse::<Token![>]>()?;

        Ok(Self {
            is_component,
            tag,
            span: tag_ident.span(),
            attrs,
            children,
        })
    }
}

/// Parses a single attribute (property or `on:` event) from `input`.
fn parse_attr(input: ParseStream) -> syn::Result<DslAttr> {
    let name_ident = Ident::parse_any(input)?;

    if name_ident == "on" && input.peek(Token![:]) {
        input.parse::<Token![:]>()?;
        let event_ident = Ident::parse_any(input)?;
        input.parse::<Token![=]>()?;
        let content;
        braced!(content in input);
        let handler: Expr = content.parse()?;
        return Ok(DslAttr::Event {
            event: event_ident.to_string(),
            span: event_ident.span(),
            handler: binding_or_string(&handler),
        });
    }

    let name = name_ident.to_string();
    let span = name_ident.span();
    if input.peek(Token![=]) {
        input.parse::<Token![=]>()?;
        if input.peek(LitStr) {
            let literal: LitStr = input.parse()?;
            Ok(DslAttr::Prop {
                name,
                span,
                value: PropValue::Text(literal.value()),
            })
        } else if input.peek(token::Brace) {
            let content;
            braced!(content in input);
            let expr: Expr = content.parse()?;
            Ok(DslAttr::Prop {
                name,
                span,
                value: classify_prop_value(&expr)?,
            })
        } else {
            Err(input.error("expected a string literal or `{ expr }` after `=`"))
        }
    } else {
        Ok(DslAttr::Prop {
            name,
            span,
            value: PropValue::BareTrue,
        })
    }
}

/// Checks every element in the parsed tree against its component's contract
/// (spec §2.12, §2.13), reporting each violation at the token that caused it.
///
/// # This front end used to check nothing
///
/// `okui!` calls itself the spec's *primary* authoring surface, and it ran **no**
/// contract validation at all: `okui! { <Button laebl="OK"/> }` compiled clean
/// while the very same document in a `.okui` file was rejected. §2.13's "unknown
/// attributes, events and string method names are compile errors" held on the
/// fallback path only. The rule was never the problem — it was written against
/// the file parser's AST, which this front end does not have — so it now lives in
/// [`check_contract`] over plain data and both front ends reach it.
///
/// # What a proc-macro can and cannot see
///
/// The registry is [`ContractRegistry::builtins`]: the primitives and shipped
/// widgets, whose surfaces are compiled into this macro. A **user** component's
/// contract cannot be checked here and no amount of wiring will change that —
/// `MyProps::contract_surface()` is a function that runs when the built crate
/// runs, and this code runs while that crate is still being compiled. The file
/// front end takes its registry as an argument and so can be handed one
/// ([`ContractRegistry::register_surface`]); a macro has no argument to take it
/// in. That asymmetry is structural, and it is why the `.okui` path is not
/// simply redundant.
fn validate_contracts(nodes: &[DslNode]) -> syn::Result<()> {
    let registry = ContractRegistry::builtins();
    let mut error: Option<syn::Error> = None;
    for node in nodes {
        if let DslNode::Element(element) = node {
            validate_element(element, &registry, &mut error);
        }
    }
    error.map_or(Ok(()), Err)
}

/// Checks `element` and its element descendants, accumulating every violation
/// into `error` rather than stopping at the first — the same
/// report-them-all policy `Compiler::compile` follows.
///
/// Two rules run here, and both are the compiler's own: the §2.12/§2.13 contract
/// ([`check_contract`]) and the §2.11 collection rule ([`check_collection`]),
/// which refuses a `<VirtualFor>`. The macro adds the [`Span`], which is the
/// whole reason this front end reports better than the file one — the author's
/// build stops on the tag they wrote.
fn validate_element(
    element: &DslElement,
    registry: &ContractRegistry,
    error: &mut Option<syn::Error>,
) {
    if let Err(message) = check_collection(&element.tag) {
        push_error(error, syn::Error::new(element.span, message));
    }
    if let Some(contract) = registry.get(&element.tag) {
        let instance = DslInstance::of(element);
        for violation in check_contract(&instance.as_use(), contract) {
            push_error(
                error,
                syn::Error::new(
                    instance.span_of(violation.anchor(), element.span),
                    violation.message(contract.component()),
                ),
            );
        }
    }
    for child in &element.children {
        if let DslNode::Element(child) = child {
            validate_element(child, registry, error);
        }
    }
}

/// Folds `reported` into the accumulated `error`, so one expansion reports every
/// violation it found rather than only the first.
fn push_error(error: &mut Option<syn::Error>, reported: syn::Error) {
    match error {
        Some(existing) => existing.combine(reported),
        None => *error = Some(reported),
    }
}

/// The strings and spans one [`DslElement`] contributes to an [`ElementUse`].
///
/// The mirror of the file front end's `AstInstance`, and for the same two
/// reasons: the rule borrows `&str` from storage that must outlive it, and a
/// [`ViolationAnchor`] has to be resolved back to something the author can be
/// pointed at — a [`Span`] here, a byte offset there.
struct DslInstance<'a> {
    tag: &'a str,
    attributes: Vec<(&'a str, bool)>,
    attribute_spans: Vec<Span>,
    members: Vec<(&'a str, Option<SlotUse<'a>>, MemberArity)>,
    member_spans: Vec<Span>,
}

impl<'a> DslInstance<'a> {
    /// Describes `element` for the rule.
    fn of(element: &'a DslElement) -> Self {
        let mut instance = Self {
            tag: element.tag.as_str(),
            attributes: Vec::new(),
            attribute_spans: Vec::new(),
            members: Vec::new(),
            member_spans: Vec::new(),
        };
        for attr in &element.attrs {
            let (name, event, span) = match attr {
                DslAttr::Prop { name, span, .. } => (name.as_str(), false, *span),
                DslAttr::Event { event, span, .. } => (event.as_str(), true, *span),
            };
            instance.attributes.push((name, event));
            instance.attribute_spans.push(span);
        }
        instance.collect_members(element, MemberArity::Once);
        instance
    }

    /// Appends `element`'s members, looking through the control flow that
    /// splices into it — the same walk the file front end makes, because the
    /// emitter makes no distinction between the two front ends' output.
    fn collect_members(&mut self, element: &'a DslElement, arity: MemberArity) {
        for child in &element.children {
            let DslNode::Element(child) = child else {
                continue;
            };
            if is_conditional_tag(&child.tag) {
                self.collect_members(child, degrade(arity, MemberArity::AtMostOnce));
                continue;
            }
            if is_loop_tag(&child.tag) {
                self.collect_members(child, MemberArity::Any);
                continue;
            }
            // A `<For>`'s `<Empty>`/`<Separator>` are §2.11 slots whose bodies
            // reach the loop's parent; the wrappers themselves are never nodes.
            if arity == MemberArity::Any && matches!(child.tag.as_str(), "Empty" | "Separator") {
                self.collect_members(child, MemberArity::Any);
                continue;
            }
            self.members
                .push((child.tag.as_str(), slot_of(child), arity));
            self.member_spans.push(child.span);
        }
    }

    /// Borrows this instance as the plain [`ElementUse`] the rule takes.
    fn as_use(&self) -> ElementUse<'_> {
        ElementUse {
            tag: self.tag,
            attributes: self
                .attributes
                .iter()
                .map(|(name, event)| AttributeUse {
                    name,
                    event: *event,
                })
                .collect(),
            members: self
                .members
                .iter()
                .map(|(tag, slot, arity)| MemberUse {
                    tag,
                    slot: *slot,
                    arity: *arity,
                })
                .collect(),
        }
    }

    /// Resolves an anchor to the span its error is reported at.
    fn span_of(&self, anchor: ViolationAnchor, element: Span) -> Span {
        match anchor {
            ViolationAnchor::Element => element,
            ViolationAnchor::Attribute(index) => {
                self.attribute_spans.get(index).copied().unwrap_or(element)
            }
            ViolationAnchor::Member(index) => {
                self.member_spans.get(index).copied().unwrap_or(element)
            }
        }
    }
}

/// Returns `element`'s slot assignment, if it carries one.
///
/// Only a string literal is a slot *name*; `slot={expr}` is a name that does not
/// exist until runtime, and the rule says so rather than this front end deciding
/// it alone.
fn slot_of(element: &DslElement) -> Option<SlotUse<'_>> {
    element.attrs.iter().find_map(|attr| match attr {
        DslAttr::Prop { name, value, .. } if name == "slot" => Some(match value {
            PropValue::Text(text) => SlotUse::Named(text.as_str()),
            _ => SlotUse::Dynamic,
        }),
        _ => None,
    })
}

/// Weakens `arity` to `inner`, keeping the weaker of the two: nesting only ever
/// loosens the bound.
const fn degrade(arity: MemberArity, inner: MemberArity) -> MemberArity {
    match arity {
        MemberArity::Any => MemberArity::Any,
        MemberArity::Once | MemberArity::AtMostOnce => inner,
    }
}

/// Rejects an expression outside the pure-read subset §2.10 allows, as a
/// `syn::Error` spanned at the offending expression.
///
/// The rule itself lives in the compiler
/// ([`check_expression`](::obscur_okui_compiler::validation::check_expression)) —
/// the same function a compiled `.okui` file is checked with. Two copies of a
/// restriction is one too many: they drift, and the looser one is the one that
/// decides. What this adds is the span, which is the whole reason the macro path
/// reports better than the file path can: the author's build stops on the
/// expression they wrote.
fn check_expression(expr: &Expr) -> syn::Result<()> {
    ::obscur_okui_compiler::validation::check_expression(expr)
        .map_err(|message| syn::Error::new_spanned(expr, message))
}

/// Classifies a braced attribute expression into a resolved [`PropValue`].
fn classify_prop_value(expr: &Expr) -> syn::Result<PropValue> {
    check_expression(expr)?;
    if let Expr::Lit(ExprLit { lit, .. }) = expr {
        return Ok(match lit {
            Lit::Str(value) => PropValue::Text(value.value()),
            Lit::Bool(value) => PropValue::Bool(value.value),
            Lit::Int(value) => PropValue::Int(value.base10_parse::<i64>()?),
            Lit::Float(value) => PropValue::Float(value.base10_parse::<f64>()?),
            _ => PropValue::Binding(stringify_expr(expr)),
        });
    }
    Ok(PropValue::Binding(
        path_string(expr).unwrap_or_else(|| stringify_expr(expr)),
    ))
}

/// Returns a normalised binding path for a simple ident or dotted field-access
/// expression, else `None`.
fn path_string(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Path(ExprPath {
            qself: None, path, ..
        }) if path.leading_colon.is_none() && path.segments.len() == 1 => {
            let segment = &path.segments[0];
            matches!(segment.arguments, PathArguments::None).then(|| segment.ident.to_string())
        }
        Expr::Field(ExprField { base, member, .. }) => {
            let base = path_string(base)?;
            let member = match member {
                Member::Named(ident) => ident.to_string(),
                Member::Unnamed(index) => index.index.to_string(),
            };
            Some(format!("{base}.{member}"))
        }
        _ => None,
    }
}

/// Returns the normalised binding path for `expr`, falling back to its
/// stringified token form for anything that is not a simple path.
fn binding_or_string(expr: &Expr) -> String {
    path_string(expr).unwrap_or_else(|| stringify_expr(expr))
}

/// Stringifies an expression to its (space-separated) token form.
fn stringify_expr(expr: &Expr) -> String {
    quote!(#expr).to_string()
}

/// Takes the current id from `counter` and advances it.
fn take_id(counter: &mut u64) -> u64 {
    let id = *counter;
    *counter += 1;
    id
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_ui_ir::document::{UiIrExpression, UiIrNodeKind};
    use quote::quote;

    /// Lowers the DSL to the IR document the `okui!` macro hands to `ViewCodegen`.
    fn doc(input: TokenStream) -> UiIrDocument {
        DslMacro::to_document(input).expect("expansion should succeed")
    }

    /// The named property's expression on `node`.
    fn prop<'a>(node: &'a UiIrNode, name: &str) -> &'a UiIrExpression {
        node.property(name)
            .unwrap_or_else(|| panic!("property {name:?} is present"))
    }

    #[test]
    fn self_closing_element_lowers_to_a_single_element_node() {
        let document = doc(quote! { <div/> });
        let root = document.root();
        assert_eq!(root.kind(), UiIrNodeKind::Element);
        assert_eq!(root.tag(), "div");
        assert_eq!(root.id(), 0);
        assert!(root.is_leaf());
    }

    #[test]
    fn uppercase_tag_is_a_component_and_lowercase_is_an_element() {
        // `label` is not decoration: `<Button/>` is now a contract violation on
        // this front end too, which is the whole of §2.13's promise.
        assert_eq!(
            doc(quote! { <Button label="OK"/> }).root().kind(),
            UiIrNodeKind::Component
        );
        assert_eq!(doc(quote! { <Button label="OK"/> }).root().tag(), "Button");
        assert_eq!(
            doc(quote! { <container/> }).root().kind(),
            UiIrNodeKind::Element
        );
    }

    #[test]
    fn string_property_lowers_to_a_text_literal() {
        let document = doc(quote! { <div id="x"/> });
        assert_eq!(
            prop(document.root(), "id"),
            &UiIrExpression::Literal(UiIrValue::Text("x".into()))
        );
    }

    #[test]
    fn literal_properties_keep_their_types() {
        let document = doc(quote! { <div enabled={true} count={5} ratio={1.5}/> });
        let root = document.root();
        assert_eq!(
            prop(root, "enabled"),
            &UiIrExpression::Literal(UiIrValue::Bool(true))
        );
        assert_eq!(
            prop(root, "count"),
            &UiIrExpression::Literal(UiIrValue::Int(5))
        );
        assert_eq!(
            prop(root, "ratio"),
            &UiIrExpression::Literal(UiIrValue::Float(1.5))
        );
    }

    #[test]
    fn bare_attribute_is_boolean_true() {
        let document = doc(quote! { <div disabled/> });
        assert_eq!(
            prop(document.root(), "disabled"),
            &UiIrExpression::Literal(UiIrValue::Bool(true))
        );
    }

    #[test]
    fn binding_property_uses_a_normalised_dotted_path() {
        // `{user.name}` must normalise to "user.name", not the spaced token form.
        let document = doc(quote! { <div title={user.name} value={count}/> });
        let root = document.root();
        assert_eq!(
            prop(root, "title"),
            &UiIrExpression::Binding("user.name".into())
        );
        assert_eq!(
            prop(root, "value"),
            &UiIrExpression::Binding("count".into())
        );
    }

    #[test]
    fn event_handler_lowers_to_an_event_binding() {
        let document = doc(quote! { <Button label="OK" on:click={on_ok}/> });
        let events = document.root().events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event(), "click");
        assert_eq!(events[0].handler(), "on_ok");
    }

    #[test]
    fn a_string_literal_child_lowers_to_a_text_node() {
        let document = doc(quote! { <div>"hi"</div> });
        let child = &document.root().children()[0];
        assert_eq!(child.kind(), UiIrNodeKind::Text);
        assert_eq!(child.content(), "hi");
        assert_eq!(child.id(), 1);
    }

    #[test]
    fn an_interpolation_child_carries_a_live_binding_not_its_spelling() {
        // Inverted. This test used to assert `content() == "user.name"` — the
        // *spelling* of the path as literal text — and it was the clearest
        // statement of the bug: the assertion said the document renders its own
        // source code. An interpolation now carries the expression, exactly as an
        // attribute binding does, and `ViewCodegen` emits it as a text node's
        // content.
        let document = doc(quote! { <div>{user.name}</div> });
        let child = &document.root().children()[0];
        assert_eq!(
            child.interpolated_value(),
            Some(&UiIrExpression::Binding("user.name".into()))
        );
        assert_eq!(child.content(), "", "no literal content survives");
    }

    #[test]
    fn a_literal_interpolation_child_stays_a_literal() {
        let document = doc(quote! { <div>{5}</div> });
        assert_eq!(
            document.root().children()[0].interpolated_value(),
            Some(&UiIrExpression::Literal(UiIrValue::Int(5)))
        );
    }

    #[test]
    fn an_interpolation_faces_the_same_restrictions_as_an_attribute() {
        // §2.10 is an allow-list, and a body `{expr}` is now emitted as real Rust,
        // so it must be checked like any other binding. It was not, while it was
        // inert.
        let error = DslMacro::to_document(quote! { <div>{std::fs::read(p)}</div> })
            .expect_err("a call is outside §2.10's allow-list");
        assert!(
            error.to_string().contains("2.10"),
            "the §2.10 rule reports: {error}"
        );
    }

    #[test]
    fn nested_children_get_incrementing_preorder_ids() {
        let document = doc(quote! {
            <div>
                <span>"a"</span>
                <span>"b"</span>
            </div>
        });
        let root = document.root();
        assert_eq!(root.id(), 0);
        let ids: Vec<u64> = root.children().iter().map(UiIrNode::id).collect();
        assert_eq!(
            ids,
            vec![1, 3],
            "pre-order: each span's text takes the next id"
        );
        assert_eq!(root.children()[0].children()[0].id(), 2);
        assert_eq!(root.children()[1].children()[0].id(), 4);
        assert_eq!(document.root().node_count(), 5);
    }

    #[test]
    fn multiple_roots_are_wrapped_in_a_fragment() {
        let document = doc(quote! { <div/><span/> });
        let root = document.root();
        assert_eq!(root.kind(), UiIrNodeKind::Fragment);
        assert_eq!(root.id(), 0);
        assert_eq!(root.children().len(), 2);
        assert_eq!(root.children()[0].tag(), "div");
        assert_eq!(root.children()[1].tag(), "span");
    }

    #[test]
    fn a_lone_text_root_is_also_wrapped_in_a_fragment() {
        // Only a lone ELEMENT becomes the root directly.
        assert_eq!(doc(quote! { "hi" }).root().kind(), UiIrNodeKind::Fragment);
    }

    #[test]
    fn component_with_property_event_and_text_child() {
        let document = doc(quote! {
            <Button label="OK" on:click={on_ok}>"OK"</Button>
        });
        let root = document.root();
        assert_eq!(root.kind(), UiIrNodeKind::Component);
        assert_eq!(root.tag(), "Button");
        assert_eq!(
            prop(root, "label"),
            &UiIrExpression::Literal(UiIrValue::Text("OK".into()))
        );
        assert_eq!(root.events()[0].handler(), "on_ok");
        assert_eq!(root.children()[0].content(), "OK");
    }

    #[test]
    fn malformed_input_is_an_error_not_a_panic() {
        assert!(DslMacro::to_document(quote! {}).is_err(), "empty input");
        assert!(
            DslMacro::to_document(quote! { <a></b> }).is_err(),
            "mismatched tag"
        );
        assert!(
            DslMacro::to_document(quote! { <div> }).is_err(),
            "missing closing tag"
        );
    }

    /// Whether `source` parses as an expression §2.10 admits.
    fn admits(source: &str) -> bool {
        let expr: Expr = syn::parse_str(source).expect("the probe itself must parse");
        check_expression(&expr).is_ok()
    }

    #[test]
    fn the_allow_list_admits_every_read_shape_the_spec_names() {
        // §2.10: props, read-only state, loop/template locals, field access,
        // comparisons, logical operators, Option/null checks.
        for source in [
            "open",              // a prop / state / loop local
            "user.name",         // field access
            "user.profile.city", // nested field access
            "42",                // a literal
            "count > 0",         // a comparison
            "a == b",            // another
            "open && !closed",   // logical operators
            "(a || b) && c",     // grouping
            "items[0]",          // indexing
            "maybe.is_some()",   // an Option check — a method call in Rust
            "count + 1",         // arithmetic: pure, and named by neither side
            "&user.name",        // a borrow of a read is a read
        ] {
            assert!(admits(source), "§2.10 allows `{source}`");
        }
    }

    #[test]
    fn the_allow_list_refuses_every_effect_the_spec_forbids() {
        // §2.10's forbidden list: assignment, arbitrary mutation, I/O, async work,
        // unconstrained arbitrary function calls, direct service access. Each is
        // refused by refusing its *shape* — there is no way to spell the effect.
        for source in [
            "count = 1",           // assignment
            "count += 1",          // compound assignment
            "fetch().await",       // async work
            "load_config()",       // an unconstrained free call
            "std::fs::read(path)", // I/O, which is a free call
            "|x| x + 1",           // a closure
            "vec![1, 2]",          // a macro
            "unsafe { ptr() }",    // unsafe
            "maybe?",              // `?`
            "loop { }",            // not an expression anyone reads
        ] {
            assert!(!admits(source), "§2.10 forbids `{source}`");
        }
    }

    /// Every message a rejected document produces, joined. A combined
    /// `syn::Error` displays only its first, and the checks accumulate.
    fn rejection(input: TokenStream) -> String {
        DslMacro::to_document(input)
            .expect_err("the document breaks its contract")
            .into_iter()
            .map(|error| error.to_string())
            .collect::<Vec<_>>()
            .join("; ")
    }

    #[test]
    fn the_primary_authoring_surface_now_enforces_a_contract_at_all() {
        // §2.13's headline promise, which this front end did not keep: the very
        // same document was rejected by the `.okui` file path and compiled clean
        // here.
        assert!(
            rejection(quote! { <Button laebl="OK"/> }).contains("unknown attribute 'laebl'"),
            "a typo'd prop is a compile error on the macro path too"
        );
        assert!(
            rejection(quote! { <Button label="OK" on:hover={f}/> })
                .contains("unknown event 'hover'"),
            "and so is an event the widget does not expose"
        );
        assert!(
            rejection(quote! { <Image/> }).contains("requires prop 'src'"),
            "and so is an omitted required prop"
        );
    }

    #[test]
    fn a_slot_on_a_component_that_declares_none_is_refused() {
        // No shipped widget declares a §2.12 slot, so every `slot="…"` written
        // against one names nothing — which is exactly what it now says, rather
        // than lowering to a property no consumer reads.
        assert!(
            rejection(quote! { <Button label="OK"><span slot="icon"/></Button> })
                .contains("declares no slots"),
            "a slot assignment must name a declared slot"
        );
    }

    #[test]
    fn virtual_for_is_refused_rather_than_compiled_as_a_plain_for() {
        // §2-D: `<VirtualFor>` parsed, classified and lowered with its tag, and
        // `emit_loop` never read the tag — so a performance contract was accepted
        // and kept by nothing. It is now a compile error on the author's tag.
        let message = rejection(quote! {
            <div><VirtualFor each={rows} as={row} key={row.id}><span/></VirtualFor></div>
        });
        assert!(
            message.contains("`<VirtualFor>` is not implemented"),
            "the author is told the tag is refused, got: {message}"
        );
        assert!(
            message.contains("Use `<For>`"),
            "and what to write instead, got: {message}"
        );
    }

    #[test]
    fn a_plain_for_still_compiles() {
        // The other half: the refusal must be about `<VirtualFor>` alone. A
        // rule that also broke `<For>` would be breakage, not enforcement.
        assert!(
            DslMacro::to_document(quote! {
                <div><For each={rows} as={row} key={row.id}><span/></For></div>
            })
            .is_ok(),
            "`<For>` is the collection construct that is really implemented"
        );
    }

    #[test]
    fn a_contract_kept_is_a_contract_that_compiles() {
        // The other half of the check: the widgets' real surfaces must admit
        // their real usage, or the enforcement is just breakage.
        for input in [
            quote! { <Button label="Save" on:click={submit}/> },
            quote! { <Image src="logo.png" alt="Logo"/> },
            quote! { <Input value={text} on:input={edit}/> },
            quote! { <container class="row"><span id="x"/></container> },
        ] {
            assert!(
                DslMacro::to_document(input.clone()).is_ok(),
                "`{input}` keeps its contract"
            );
        }
    }

    #[test]
    fn control_flow_is_looked_through_to_find_the_members_it_splices() {
        // `<If>` splices its body into the enclosing child list, so a `slot=`
        // inside one is addressed to `Button`, not to the `If`. Checking direct
        // children only would let it through.
        assert!(
            rejection(quote! {
                <Button label="OK">
                    <If cond={open}><span slot="icon"/></If>
                </Button>
            })
            .contains("declares no slots"),
            "a member behind a conditional is still the component's member"
        );
    }

    #[test]
    fn a_method_call_is_the_gap_the_allow_list_does_not_close() {
        // Recorded, not hidden. §2.10 allows Option/null checks, formatting and
        // translations — all method calls in Rust — so refusing method calls
        // wholesale would forbid what the spec permits. Nothing here can tell
        // `.is_some()` from `.write_to_disk()`, and guessing would be worse than
        // saying so: both are admitted until a registry of exported pure
        // functions exists, which is the same thing free calls wait on.
        assert!(admits("maybe.is_some()"), "which §2.10 allows");
        assert!(
            admits("services.db().query()"),
            "and this, which it does not — the gap, stated"
        );
        // The receiver and the arguments are still walked, so the gap is exactly
        // one level wide: a *free* call cannot hide inside a method call.
        assert!(!admits("thing.method(load_config())"));
        assert!(!admits("load_config().method()"));
    }

    #[test]
    fn an_effect_nested_inside_a_read_is_still_refused() {
        // The walk is recursive, so hiding a call under a comparison does not
        // launder it — which is the difference between checking the shape and
        // checking the top of the shape.
        assert!(!admits("count > load_limit()"));
        assert!(!admits("user.name == fetch().await"));
        assert!(!admits("(a && do_io())"));
        assert!(!admits("items[index()]"));
    }

    #[test]
    fn a_forbidden_expression_is_a_compile_error_at_the_authors_code() {
        // Not a diagnostic collected somewhere: the macro fails, so the author's
        // build stops on the line they wrote.
        let error = DslMacro::to_document(quote! { <span class={load_config()}/> })
            .expect_err("a free call must be refused");
        assert!(
            error.to_string().contains("§2.10"),
            "the error should say which rule: {error}"
        );
    }

    #[test]
    fn an_ordinary_binding_still_compiles() {
        // The regression guard: the overwhelming majority of bindings are reads.
        assert!(DslMacro::to_document(quote! { <span class={label}/> }).is_ok());
        assert!(DslMacro::to_document(quote! { <span class={user.name}/> }).is_ok());
    }
}
