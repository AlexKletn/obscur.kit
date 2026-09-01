#![doc = include_str!("expression_validator.doc/_module.md")]

use obscur_okui_syntax::ast::{Document, Element, Expression};
use syn::{BinOp, Expr};

#[doc = include_str!("expression_validator.doc/ValidationDiagnostic.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationDiagnostic {
    message: Box<str>,
    offset: u32,
}

impl ValidationDiagnostic {
    #[doc = include_str!("expression_validator.doc/new.md")]
    #[must_use]
    pub fn new(message: impl Into<Box<str>>, offset: u32) -> Self {
        Self {
            message: message.into(),
            offset,
        }
    }

    #[doc = include_str!("expression_validator.doc/message.md")]
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    #[doc = include_str!("expression_validator.doc/offset.md")]
    #[must_use]
    pub const fn offset(&self) -> u32 {
        self.offset
    }
}

#[doc = include_str!("expression_validator.doc/check_expression.md")]
pub fn check_expression(expr: &Expr) -> Result<(), String> {
    check_expression_in(expr, &[])
}

#[doc = include_str!("expression_validator.doc/check_expression_in.md")]
pub fn check_expression_in(expr: &Expr, functions: &[Box<str>]) -> Result<(), String> {
    match expr {
        Expr::Path(_) | Expr::Lit(_) => Ok(()),
        Expr::Field(inner) => check_expression_in(&inner.base, functions),
        Expr::Paren(inner) => check_expression_in(&inner.expr, functions),
        Expr::Index(inner) => {
            check_expression_in(&inner.expr, functions)?;
            check_expression_in(&inner.index, functions)
        }
        Expr::Reference(inner) => check_expression_in(&inner.expr, functions),
        Expr::Unary(inner) => check_expression_in(&inner.expr, functions),
        // `syn` files compound assignment (`+=`) under `Binary`, not under
        // `Assign` — so admitting every binary operator would admit the mutation
        // §2.10 forbids. The operator is checked, not just the operands.
        Expr::Binary(inner) => {
            if is_assigning_op(inner.op) {
                return Err("assignment is not allowed in an expression (spec §2.10)".to_owned());
            }
            check_expression_in(&inner.left, functions)?;
            check_expression_in(&inner.right, functions)
        }
        Expr::Cast(inner) => check_expression_in(&inner.expr, functions),
        // A free call: refused as an "unconstrained arbitrary function call"
        // unless its callee is a bare name a `Function` symbol declared — an
        // "explicitly exported pure function" (§2.10). Arguments are still walked.
        Expr::Call(inner) => {
            match simple_callee_name(&inner.func) {
                Some(name) if functions.iter().any(|f| f.as_ref() == name) => {}
                Some(name) => return Err(refused_call_message(&name)),
                None => return Err(FORBIDDEN_FREE_CALL.to_owned()),
            }
            inner
                .args
                .iter()
                .try_for_each(|a| check_expression_in(a, functions))
        }
        // See "the method-call gap".
        Expr::MethodCall(inner) => {
            let method = inner.method.to_string();
            if is_mutating_method(&method) {
                return Err(format!(
                    "`{method}` mutates its receiver; an expression may not perform \
                     arbitrary mutation (spec §2.10)"
                ));
            }
            if matches!(&*inner.receiver, Expr::Field(_)) {
                return Err(field_method_message(&method));
            }
            check_expression_in(&inner.receiver, functions)?;
            inner
                .args
                .iter()
                .try_for_each(|a| check_expression_in(a, functions))
        }
        _ => Err(FORBIDDEN_MESSAGE.to_owned()),
    }
}

#[doc = include_str!("expression_validator.doc/simple_callee_name.md")]
fn simple_callee_name(func: &Expr) -> Option<String> {
    let Expr::Path(path) = func else {
        return None;
    };
    if path.qself.is_some() || path.path.leading_colon.is_some() || path.path.segments.len() != 1 {
        return None;
    }
    let segment = &path.path.segments[0];
    matches!(segment.arguments, syn::PathArguments::None).then(|| segment.ident.to_string())
}

#[doc = include_str!("expression_validator.doc/refused_call_message.md")]
fn refused_call_message(name: &str) -> String {
    format!(
        "`{name}` is not an explicitly exported pure function, so it may not be \
         called in an expression (spec §2.10)"
    )
}

#[doc = include_str!("expression_validator.doc/FORBIDDEN_FREE_CALL.md")]
pub const FORBIDDEN_FREE_CALL: &str =
    "only explicitly exported pure functions may be called in an expression (spec §2.10)";

#[doc = include_str!("expression_validator.doc/field_method_message.md")]
fn field_method_message(method: &str) -> String {
    format!(
        "`{method}` is called on a value reached through a field, a shape an \
         expression may not use: it cannot be told apart from the direct service \
         access §2.10 forbids. Compute the value where the view model is built, \
         and read that field here instead."
    )
}

#[doc = include_str!("expression_validator.doc/is_mutating_method.md")]
fn is_mutating_method(method: &str) -> bool {
    matches!(
        method,
        "push"
            | "push_str"
            | "pop"
            | "insert"
            | "remove"
            | "clear"
            | "truncate"
            | "extend"
            | "append"
            | "retain"
            | "sort"
            | "sort_by"
            | "sort_by_key"
            | "sort_unstable"
            | "sort_unstable_by"
            | "reverse"
            | "dedup"
            | "drain"
            | "swap"
            | "swap_remove"
            | "resize"
            | "splice"
            | "split_off"
            | "rotate_left"
            | "rotate_right"
    )
}

#[doc = include_str!("expression_validator.doc/FORBIDDEN_MESSAGE.md")]
pub const FORBIDDEN_MESSAGE: &str = "this expression is outside what `.okui` allows (spec §2.10): an expression may \
     read props, state and loop locals, access fields, compare, and combine with \
     logical operators — it may not assign, mutate, await, or call a function that \
     has not been declared pure";

#[doc = include_str!("expression_validator.doc/is_assigning_op.md")]
fn is_assigning_op(op: BinOp) -> bool {
    matches!(
        op,
        BinOp::AddAssign(_)
            | BinOp::SubAssign(_)
            | BinOp::MulAssign(_)
            | BinOp::DivAssign(_)
            | BinOp::RemAssign(_)
            | BinOp::BitXorAssign(_)
            | BinOp::BitAndAssign(_)
            | BinOp::BitOrAssign(_)
            | BinOp::ShlAssign(_)
            | BinOp::ShrAssign(_)
    )
}

#[doc = include_str!("expression_validator.doc/ExpressionValidator.md")]
#[derive(Debug, Clone, Default)]
pub struct ExpressionValidator {
    #[doc = include_str!("expression_validator.doc/functions.md")]
    functions: Vec<Box<str>>,
}

impl ExpressionValidator {
    #[doc = include_str!("expression_validator.doc/new_2.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            functions: Vec::new(),
        }
    }

    #[doc = include_str!("expression_validator.doc/with_functions.md")]
    #[must_use]
    pub fn with_functions(mut self, functions: impl IntoIterator<Item = Box<str>>) -> Self {
        self.functions = functions.into_iter().collect();
        self
    }

    #[doc = include_str!("expression_validator.doc/validate.md")]
    #[must_use]
    pub fn validate(&self, expr_text: &str, offset: u32) -> Vec<ValidationDiagnostic> {
        let Ok(expr) = syn::parse_str::<Expr>(expr_text) else {
            return vec![ValidationDiagnostic::new(
                "this is not a valid expression (spec §2.10)",
                offset,
            )];
        };
        match check_expression_in(&expr, &self.functions) {
            Ok(()) => Vec::new(),
            Err(message) => vec![ValidationDiagnostic::new(message, offset)],
        }
    }

    #[doc = include_str!("expression_validator.doc/validate_document.md")]
    #[must_use]
    pub fn validate_document(&self, doc: &Document) -> Vec<ValidationDiagnostic> {
        let mut diagnostics = Vec::new();
        for element in doc.elements() {
            self.validate_element(&element, &mut diagnostics);
        }
        diagnostics
    }

    #[doc = include_str!("expression_validator.doc/validate_element.md")]
    fn validate_element(&self, element: &Element, diagnostics: &mut Vec<ValidationDiagnostic>) {
        for attribute in element.attributes() {
            if let Some(expression) = attribute.value_expression() {
                let offset = expression.syntax().offset().unwrap_or(0);
                diagnostics.extend(self.validate(&expression.text(), offset));
            }
        }
        for child in element.children() {
            // A body `{expr}` — `<span>{user.name}</span>` — is an expression like
            // any other, and it is emitted as real Rust exactly like an attribute's
            // (`lower_interpolation` carries it as a `value` binding). Checking only
            // attributes left it unchecked, so `<span>{std::fs::read(p)}</span>`
            // reached generated code with no §2.10 check at all. The hole was not
            // in the rule; the rule was simply never shown half the expressions.
            if let Some(expression) = Expression::cast(child.clone()) {
                let offset = child.offset().unwrap_or(0);
                diagnostics.extend(self.validate(&expression.text(), offset));
                continue;
            }
            if let Some(child_element) = Element::cast(child) {
                self.validate_element(&child_element, diagnostics);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_okui_syntax::ast::Document;
    use obscur_okui_syntax::parser::Parser;

    fn document(src: &str) -> Document {
        Document::cast(Parser::new().parse(src).tree().root().clone()).expect("Document")
    }

    #[test]
    fn diagnostic_accessors_expose_fields() {
        let diag = ValidationDiagnostic::new("boom", 7);
        assert_eq!(diag.message(), "boom");
        assert_eq!(diag.offset(), 7);
    }

    #[test]
    fn comparison_is_allowed() {
        let validator = ExpressionValidator::new();
        assert!(validator.validate("a == b", 0).is_empty());
        assert!(validator.validate("count >= 3", 0).is_empty());
        assert!(validator.validate("a != b", 0).is_empty());
        assert!(validator.validate("a <= b", 0).is_empty());
    }

    #[test]
    fn an_effect_is_refused() {
        let validator = ExpressionValidator::new();
        for source in ["a = b", "count += 1", "fetch().await", "|x| x"] {
            assert_eq!(validator.validate(source, 0).len(), 1, "`{source}`");
        }
    }

    #[test]
    fn a_free_call_is_refused_which_the_lexical_scan_could_not_see() {
        // THE gain from parsing. `load_config()` contains no forbidden *word*, so
        // the old scan admitted it — and a binding now emits as real Rust, so it
        // would have run.
        let validator = ExpressionValidator::new();
        assert_eq!(validator.validate("load_config()", 0).len(), 1);
        assert_eq!(validator.validate("std::fs::read(path)", 0).len(), 1);
        // …and it cannot hide inside an allowed shape.
        assert_eq!(validator.validate("count > load_limit()", 0).len(), 1);
    }

    #[test]
    fn a_body_interpolation_is_checked_like_an_attribute() {
        // THE hole, and it was mine: `check_expression` was total over the AST it
        // was shown, and `validate_element` only showed it attributes. A body
        // `{expr}` is lowered as a real binding (`lower_interpolation`) and emitted
        // as real Rust, so this reached generated code with no §2.10 check at all.
        let validator = ExpressionValidator::new();

        let doc = document("<span>{std::fs::read(p)}</span>");
        let diagnostics = validator.validate_document(&doc);
        assert_eq!(
            diagnostics.len(),
            1,
            "a free call in a body must be refused"
        );
        assert!(diagnostics[0].message().contains("§2.10"));

        // Nested, so the recursion is proven to carry the check down.
        let nested = document("<Root><Child>{load_config()}</Child></Root>");
        assert_eq!(validator.validate_document(&nested).len(), 1);

        // …and an ordinary read in a body still passes.
        assert!(
            validator
                .validate_document(&document("<span>{user.name}</span>"))
                .is_empty()
        );
    }

    #[test]
    fn a_method_on_a_field_is_refused_by_the_rule_it_actually_enforces() {
        // A consumer wrote `<For … key={row.keys.len()}>` and was told "direct
        // service access is not allowed", which sent them looking for a service
        // they never wrote. The refusal is right — the shape is indistinguishable
        // from `services.http.get(url)` — but the message has to say *that*, not
        // assert a `.len()` on a `Vec` is a service.
        let validator = ExpressionValidator::new();

        let diagnostics = validator.validate("row.keys.len()", 0);
        assert_eq!(diagnostics.len(), 1, "the call is still refused");
        let message = diagnostics[0].message();
        assert!(
            message.contains("len"),
            "the message names the call it refused: {message}"
        );
        assert!(
            message.contains("field"),
            "the message names the shape it enforces: {message}"
        );
        assert!(
            !message.contains("direct service access is not allowed"),
            "the message may not claim a `.len()` on a `Vec` *is* service access: \
             {message}"
        );

        // The same through the attribute the consumer actually wrote.
        let doc = document("<For each={rows} as={row} key={row.keys.len()}><T/></For>");
        let from_document = validator.validate_document(&doc);
        assert_eq!(from_document.len(), 1, "{from_document:?}");
        assert_eq!(from_document[0].message(), message);

        // The genuine service call is refused by the same rule, and its message
        // still says so.
        let service = validator.validate("services.http.get(url)", 0);
        assert_eq!(service.len(), 1);
        assert!(service[0].message().contains("service"), "{:?}", service[0]);
    }

    #[test]
    fn a_keyword_inside_an_identifier_is_not_an_effect() {
        // The lexical scan needed a word-boundary rule for this; a parser does not
        // need to be told that `mutable` is an identifier.
        let validator = ExpressionValidator::new();
        assert!(validator.validate("mutable", 0).is_empty());
        assert!(validator.validate("returned", 0).is_empty());
        assert!(validator.validate("awaiting_review", 0).is_empty());
    }

    #[test]
    fn text_that_is_not_an_expression_is_reported_here() {
        // It lowers to Rust, so unparseable text could never have run.
        let validator = ExpressionValidator::new();
        assert_eq!(validator.validate("a +", 0).len(), 1);
        assert_eq!(validator.validate("", 0).len(), 1);
    }

    #[test]
    fn validate_document_walks_dynamic_attributes() {
        let validator = ExpressionValidator::new();
        let doc = document("<T x={a = b}/>");
        let diagnostics = validator.validate_document(&doc);
        assert_eq!(diagnostics.len(), 1);
        assert!(
            diagnostics[0].message().contains("§2.10"),
            "{:?}",
            diagnostics[0]
        );
    }

    #[test]
    fn validate_document_recurses_into_children() {
        let validator = ExpressionValidator::new();
        let doc = document("<Root><Child y={load_config()}/></Root>");
        let diagnostics = validator.validate_document(&doc);
        assert_eq!(diagnostics.len(), 1, "the nested element is walked");
        assert!(
            diagnostics[0].message().contains("§2.10"),
            "{:?}",
            diagnostics[0]
        );
    }
}
