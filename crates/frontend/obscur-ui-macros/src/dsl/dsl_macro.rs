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
//! - Interpolated `{expr}` children lower to a
//!   [`text`](::obscur_ui_ir::document::UiIrNode::text) node whose *content* is
//!   the binding path (or stringified expression), because `UiIrNode::text`
//!   only accepts literal content — the expression cannot be carried as a live
//!   binding on a text node. Callers needing a dynamic text binding should bind
//!   a property on an enclosing element instead.
//! - A lone top-level element is emitted as the document root directly; any
//!   other top level (multiple siblings, or a single text/interpolation node)
//!   is wrapped in a [`fragment`](::obscur_ui_ir::document::UiIrNode::fragment)
//!   root.
//!
//! # Id scheme
//!
//! Every node is assigned a stable sequential `u64` id during a single
//! pre-order walk (root = `0`, incrementing).

use obscur_ui_ir::document::{UiIrDocument, UiIrEvent, UiIrNode, UiIrProperty, UiIrValue};
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
    /// a mismatched closing tag, or when a braced value is not a [`syn::Expr`].
    pub(crate) fn to_document(input: TokenStream) -> Result<UiIrDocument, syn::Error> {
        let nodes = parse_nodes(input)?;
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

/// Builds the [`UiIrNode`] for `node`, threading `counter` to assign pre-order
/// ids (root = 0, incrementing).
fn build_node(node: &DslNode, counter: &mut u64) -> UiIrNode {
    match node {
        DslNode::Text(content) | DslNode::Interpolation(content) => {
            UiIrNode::text(take_id(counter), content.as_str())
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
            } else if element.is_component {
                UiIrNode::component(id, element.tag.as_str())
            } else {
                UiIrNode::element(id, element.tag.as_str())
            };
            for attr in &element.attrs {
                ir = match attr {
                    DslAttr::Event { event, handler } => {
                        ir.with_event(UiIrEvent::new(event.as_str(), handler.as_str()))
                    }
                    DslAttr::Prop { name, value } => ir.with_property(build_property(name, value)),
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
    /// A `{expr}` interpolation child, lowered to a text node carrying the
    /// binding path (see the module docs for the limitation).
    Interpolation(String),
}

/// A parsed element (or component instance) with its attributes and children.
struct DslElement {
    /// Whether the tag names a component (uppercase-initial) rather than an
    /// element.
    is_component: bool,
    /// The tag or component name.
    tag: String,
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
        /// The resolved property value.
        value: PropValue,
    },
    /// An `on:<event>={handler}` event binding.
    Event {
        /// The event name (the part after `on:`).
        event: String,
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
            Ok(Self::Interpolation(binding_or_string(&expr)))
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
            handler: binding_or_string(&handler),
        });
    }

    let name = name_ident.to_string();
    if input.peek(Token![=]) {
        input.parse::<Token![=]>()?;
        if input.peek(LitStr) {
            let literal: LitStr = input.parse()?;
            Ok(DslAttr::Prop {
                name,
                value: PropValue::Text(literal.value()),
            })
        } else if input.peek(token::Brace) {
            let content;
            braced!(content in input);
            let expr: Expr = content.parse()?;
            Ok(DslAttr::Prop {
                name,
                value: classify_prop_value(&expr)?,
            })
        } else {
            Err(input.error("expected a string literal or `{ expr }` after `=`"))
        }
    } else {
        Ok(DslAttr::Prop {
            name,
            value: PropValue::BareTrue,
        })
    }
}

/// Classifies a braced attribute expression into a resolved [`PropValue`].
fn classify_prop_value(expr: &Expr) -> syn::Result<PropValue> {
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
        assert_eq!(doc(quote! { <Button/> }).root().kind(), UiIrNodeKind::Component);
        assert_eq!(doc(quote! { <Button/> }).root().tag(), "Button");
        assert_eq!(doc(quote! { <container/> }).root().kind(), UiIrNodeKind::Element);
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
        assert_eq!(prop(root, "enabled"), &UiIrExpression::Literal(UiIrValue::Bool(true)));
        assert_eq!(prop(root, "count"), &UiIrExpression::Literal(UiIrValue::Int(5)));
        assert_eq!(prop(root, "ratio"), &UiIrExpression::Literal(UiIrValue::Float(1.5)));
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
        assert_eq!(prop(root, "title"), &UiIrExpression::Binding("user.name".into()));
        assert_eq!(prop(root, "value"), &UiIrExpression::Binding("count".into()));
    }

    #[test]
    fn event_handler_lowers_to_an_event_binding() {
        let document = doc(quote! { <Button on:click={on_ok}/> });
        let events = document.root().events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event(), "click");
        assert_eq!(events[0].handler(), "on_ok");
    }

    #[test]
    fn text_and_interpolation_children_lower_to_text_nodes() {
        let document = doc(quote! { <div>"hi"</div> });
        let child = &document.root().children()[0];
        assert_eq!(child.kind(), UiIrNodeKind::Text);
        assert_eq!(child.content(), "hi");
        assert_eq!(child.id(), 1);

        // An interpolation carries its binding PATH as literal content (see the
        // module docs: `UiIrNode::text` cannot hold a live binding).
        let document = doc(quote! { <div>{user.name}</div> });
        assert_eq!(document.root().children()[0].content(), "user.name");
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
        assert_eq!(ids, vec![1, 3], "pre-order: each span's text takes the next id");
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
        assert!(DslMacro::to_document(quote! { <a></b> }).is_err(), "mismatched tag");
        assert!(DslMacro::to_document(quote! { <div> }).is_err(), "missing closing tag");
    }
}
