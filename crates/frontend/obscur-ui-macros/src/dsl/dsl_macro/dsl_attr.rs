use proc_macro2::Span;
use syn::ext::IdentExt;
use syn::parse::ParseStream;
use syn::{Expr, Ident, LitStr, Token, braced, token};

use super::{binding_or_string, classify_prop_value, prop_value::PropValue};

/// A parsed attribute: either a property or an `on:` event binding.
pub(super) enum DslAttr {
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

#[doc = include_str!("../../../docs/api/dsl/dsl_macro.doc/parse_dashed_name.md")]
fn parse_dashed_name(input: ParseStream) -> syn::Result<(String, Span)> {
    let head = Ident::parse_any(input)?;
    let span = head.span();
    let mut name = head.to_string();
    while input.peek(Token![-]) {
        let dash = input.parse::<Token![-]>()?;
        let segment = Ident::parse_any(input).map_err(|_| {
            syn::Error::new(
                dash.spans[0],
                format!(
                    "expected an identifier after `-` in the attribute name `{name}-`: \
                     a hyphenated name is `ident-ident`, as in `test-id`"
                ),
            )
        })?;
        name.push('-');
        name.push_str(&segment.to_string());
    }
    Ok((name, span))
}

/// Parses a single attribute (property or `on:` event) from `input`.
pub(super) fn parse_attr(input: ParseStream) -> syn::Result<DslAttr> {
    let (name, span) = parse_dashed_name(input)?;

    if name == "on" && input.peek(Token![:]) {
        input.parse::<Token![:]>()?;
        // The event name goes through the same join: the `.okui` lexer makes one
        // Ident of `on:pointer-down`, so this front end must too or the pair
        // disagree about a spelling again.
        let (event, event_span) = parse_dashed_name(input)?;
        input.parse::<Token![=]>()?;
        let content;
        braced!(content in input);
        let handler: Expr = content.parse()?;
        return Ok(DslAttr::Event {
            event,
            span: event_span,
            handler: binding_or_string(&handler),
        });
    }

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
