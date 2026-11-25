use proc_macro2::Span;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, Token};

use super::{
    dsl_attr::{DslAttr, parse_attr},
    dsl_node::DslNode,
};

/// A parsed element (or component instance) with its attributes and children.
pub(super) struct DslElement {
    /// Whether the tag names a component (uppercase-initial) rather than an
    /// element.
    pub(super) is_component: bool,
    /// The tag or component name.
    pub(super) tag: String,
    /// The tag identifier's span, so a contract violation about the element as a
    /// whole lands on the tag the author wrote.
    pub(super) span: Span,
    /// The parsed attributes, in source order.
    pub(super) attrs: Vec<DslAttr>,
    /// The parsed children, in source order.
    pub(super) children: Vec<DslNode>,
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
