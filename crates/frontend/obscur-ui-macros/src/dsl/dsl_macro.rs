//! `DslMacro`: expansion logic for the TSX-like Rust DSL (spec §2.8).
//!
//! This module turns a JSX/TSX-like element tree into Rust tokens that build an
//! [`::obscur_ui_ir::document::UiIrDocument`] at runtime. It is pure token-in /
//! token-out logic: the macro crate emits leading-`::` paths as tokens and does
//! **not** depend on `obscur-ui-ir`; the generated code references those paths.
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
//! pre-order walk (root = `0`, incrementing); the integer literals are baked
//! into the generated constructor calls.

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
/// Call [`DslMacro::expand`] with the DSL token stream to obtain the generated
/// block expression building an [`::obscur_ui_ir::document::UiIrDocument`], or a
/// [`syn::Error`] for malformed input.
#[derive(Debug)]
pub struct DslMacro;

impl DslMacro {
    /// Expands the TSX-like DSL token stream into Rust tokens that build an
    /// obscur-ui-ir [`::obscur_ui_ir::document::UiIrDocument`].
    ///
    /// # Errors
    ///
    /// Returns a [`syn::Error`] on empty input, on malformed element syntax
    /// (missing `>`, unexpected tokens), on a mismatched closing tag, or when a
    /// braced value cannot be parsed as a [`syn::Expr`].
    pub fn expand(input: TokenStream) -> Result<TokenStream, syn::Error> {
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

        let mut counter: u64 = 0;
        let root = if nodes.len() == 1 && matches!(nodes[0], DslNode::Element(_)) {
            gen_node(&nodes[0], &mut counter)
        } else {
            let id = take_id(&mut counter);
            let mut ts = quote! { ::obscur_ui_ir::document::UiIrNode::fragment(#id) };
            for node in &nodes {
                let child = gen_node(node, &mut counter);
                ts = quote! { #ts.with_child(#child) };
            }
            ts
        };

        Ok(quote! {
            {
                ::obscur_ui_ir::document::UiIrDocument::new(#root)
            }
        })
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

/// Generates the constructor-call tokens for `node`, threading `counter` to
/// assign pre-order ids.
fn gen_node(node: &DslNode, counter: &mut u64) -> TokenStream {
    match node {
        DslNode::Text(content) | DslNode::Interpolation(content) => {
            let id = take_id(counter);
            quote! { ::obscur_ui_ir::document::UiIrNode::text(#id, #content) }
        }
        DslNode::Element(element) => {
            let id = take_id(counter);
            let ctor = Ident::new(
                if element.is_component {
                    "component"
                } else {
                    "element"
                },
                Span::call_site(),
            );
            let tag = &element.tag;
            let mut ts = quote! { ::obscur_ui_ir::document::UiIrNode::#ctor(#id, #tag) };
            for attr in &element.attrs {
                let attr_ts = gen_attr(attr);
                ts = quote! { #ts #attr_ts };
            }
            for child in &element.children {
                let child_ts = gen_node(child, counter);
                ts = quote! { #ts.with_child(#child_ts) };
            }
            ts
        }
    }
}

/// Generates the builder-call tokens (`.with_property(..)` / `.with_event(..)`)
/// for a single attribute.
fn gen_attr(attr: &DslAttr) -> TokenStream {
    match attr {
        DslAttr::Event { event, handler } => quote! {
            .with_event(::obscur_ui_ir::document::UiIrEvent::new(#event, #handler))
        },
        DslAttr::Prop { name, value } => {
            let property = match value {
                PropValue::BareTrue => quote! {
                    ::obscur_ui_ir::document::UiIrProperty::literal(
                        #name,
                        ::obscur_ui_ir::document::UiIrValue::Bool(true)
                    )
                },
                PropValue::Text(text) => quote! {
                    ::obscur_ui_ir::document::UiIrProperty::text(#name, #text)
                },
                PropValue::Bool(flag) => quote! {
                    ::obscur_ui_ir::document::UiIrProperty::literal(
                        #name,
                        ::obscur_ui_ir::document::UiIrValue::Bool(#flag)
                    )
                },
                PropValue::Int(value) => quote! {
                    ::obscur_ui_ir::document::UiIrProperty::literal(
                        #name,
                        ::obscur_ui_ir::document::UiIrValue::Int(#value)
                    )
                },
                PropValue::Float(value) => quote! {
                    ::obscur_ui_ir::document::UiIrProperty::literal(
                        #name,
                        ::obscur_ui_ir::document::UiIrValue::Float(#value)
                    )
                },
                PropValue::Binding(path) => quote! {
                    ::obscur_ui_ir::document::UiIrProperty::binding(#name, #path)
                },
            };
            quote! { .with_property(#property) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    fn expand(input: TokenStream) -> String {
        DslMacro::expand(input).expect("expansion should succeed").to_string()
    }

    #[test]
    fn self_closing_element_emits_single_element_node() {
        let out = expand(quote! { <div/> });
        assert!(out.contains(r#"UiIrNode :: element (0u64 , "div")"#), "{out}");
        assert!(out.contains("UiIrDocument :: new"), "{out}");
    }

    #[test]
    fn uppercase_tag_is_a_component() {
        let out = expand(quote! { <Button/> });
        assert!(out.contains(r#"UiIrNode :: component (0u64 , "Button")"#), "{out}");
    }

    #[test]
    fn lowercase_tag_is_an_element() {
        let out = expand(quote! { <container/> });
        assert!(out.contains(r#"UiIrNode :: element (0u64 , "container")"#), "{out}");
    }

    #[test]
    fn string_property_emits_text_property() {
        let out = expand(quote! { <div id="x"/> });
        assert!(out.contains(r#"UiIrProperty :: text ("id" , "x")"#), "{out}");
    }

    #[test]
    fn bool_literal_property() {
        let out = expand(quote! { <div enabled={true}/> });
        assert!(
            out.contains(r#"UiIrProperty :: literal ("enabled" , :: obscur_ui_ir :: document :: UiIrValue :: Bool (true))"#),
            "{out}"
        );
    }

    #[test]
    fn int_literal_property() {
        let out = expand(quote! { <div count={5}/> });
        assert!(out.contains(r"UiIrValue :: Int (5i64)"), "{out}");
    }

    #[test]
    fn float_literal_property() {
        let out = expand(quote! { <div ratio={1.5}/> });
        assert!(out.contains(r"UiIrValue :: Float (1.5f64)"), "{out}");
    }

    #[test]
    fn bare_attribute_is_boolean_true() {
        let out = expand(quote! { <div disabled/> });
        assert!(
            out.contains(r#"UiIrProperty :: literal ("disabled" , :: obscur_ui_ir :: document :: UiIrValue :: Bool (true))"#),
            "{out}"
        );
    }

    #[test]
    fn binding_property_uses_normalised_dotted_path() {
        let out = expand(quote! { <div title={user.name}/> });
        assert!(out.contains(r#"UiIrProperty :: binding ("title" , "user.name")"#), "{out}");
    }

    #[test]
    fn single_ident_binding_property() {
        let out = expand(quote! { <div value={count}/> });
        assert!(out.contains(r#"UiIrProperty :: binding ("value" , "count")"#), "{out}");
    }

    #[test]
    fn event_handler_emits_event() {
        let out = expand(quote! { <Button on:click={on_ok}/> });
        assert!(out.contains(r#"with_event (:: obscur_ui_ir :: document :: UiIrEvent :: new ("click" , "on_ok"))"#), "{out}");
    }

    #[test]
    fn text_child_emits_text_node_with_next_id() {
        let out = expand(quote! { <div>"hi"</div> });
        assert!(out.contains(r#"UiIrNode :: element (0u64 , "div")"#), "{out}");
        assert!(out.contains(r#"UiIrNode :: text (1u64 , "hi")"#), "{out}");
    }

    #[test]
    fn interpolation_child_lowers_to_text_binding_path() {
        let out = expand(quote! { <div>{user.name}</div> });
        assert!(out.contains(r#"UiIrNode :: text (1u64 , "user.name")"#), "{out}");
    }

    #[test]
    fn nested_children_get_incrementing_preorder_ids() {
        let out = expand(quote! {
            <div>
                <span>"a"</span>
                <span>"b"</span>
            </div>
        });
        assert!(out.contains(r#"UiIrNode :: element (0u64 , "div")"#), "{out}");
        assert!(out.contains(r#"UiIrNode :: element (1u64 , "span")"#), "{out}");
        assert!(out.contains(r#"UiIrNode :: text (2u64 , "a")"#), "{out}");
        assert!(out.contains(r#"UiIrNode :: element (3u64 , "span")"#), "{out}");
        assert!(out.contains(r#"UiIrNode :: text (4u64 , "b")"#), "{out}");
    }

    #[test]
    fn multiple_roots_are_wrapped_in_a_fragment() {
        let out = expand(quote! { <div/><span/> });
        assert!(out.contains(r"UiIrNode :: fragment (0u64)"), "{out}");
        assert!(out.contains(r#"UiIrNode :: element (1u64 , "div")"#), "{out}");
        assert!(out.contains(r#"UiIrNode :: element (2u64 , "span")"#), "{out}");
    }

    #[test]
    fn component_with_property_event_and_text_child() {
        let out = expand(quote! {
            <Button label="OK" on:click={on_ok}>"OK"</Button>
        });
        assert!(out.contains(r#"UiIrNode :: component (0u64 , "Button")"#), "{out}");
        assert!(out.contains(r#"UiIrProperty :: text ("label" , "OK")"#), "{out}");
        assert!(out.contains(r#"UiIrEvent :: new ("click" , "on_ok")"#), "{out}");
        assert!(out.contains(r#"UiIrNode :: text (1u64 , "OK")"#), "{out}");
    }

    #[test]
    fn empty_input_is_an_error() {
        assert!(DslMacro::expand(quote! {}).is_err());
    }

    #[test]
    fn mismatched_closing_tag_is_an_error() {
        assert!(DslMacro::expand(quote! { <a></b> }).is_err());
    }

    #[test]
    fn missing_closing_tag_is_an_error() {
        assert!(DslMacro::expand(quote! { <div> }).is_err());
    }
}
