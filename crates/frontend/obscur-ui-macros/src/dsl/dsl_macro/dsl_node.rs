use syn::parse::{Parse, ParseStream};
use syn::{Expr, LitStr, Token, braced, token};

use super::{classify_prop_value, dsl_element::DslElement, prop_value::PropValue};

/// One parsed node of the DSL tree.
pub(super) enum DslNode {
    /// A nested element or component.
    Element(DslElement),
    /// A bare string literal child, lowered to a literal text node.
    Text(String),
    /// A `{expr}` interpolation child, holding the resolved expression the
    /// interpolation node carries.
    Interpolation(PropValue),
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
