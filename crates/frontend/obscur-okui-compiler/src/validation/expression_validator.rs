//! `ExpressionValidator`: enforces the `.okui` expression restrictions (spec
//! §2.10), plus the shared [`ValidationDiagnostic`] used across the validation
//! module.

use obscur_okui_syntax::ast::{Document, Element};
use syn::{BinOp, Expr};

/// A validation issue found while checking a document (spec §2.10, §2.13).
///
/// This is the common diagnostic reported by every validator in this module —
/// the [`ExpressionValidator`], the
/// [`CapabilityValidator`](crate::validation::CapabilityValidator) and the
/// [`ContractValidator`](crate::validation::ContractValidator). It pairs a human
/// readable `message` with the byte `offset` into the source where the issue was
/// detected, so a caller can render a pointed diagnostic. It is deliberately
/// untyped (a message string rather than an error enum) because the validation
/// rules are open-ended and reported uniformly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationDiagnostic {
    message: Box<str>,
    offset: u32,
}

impl ValidationDiagnostic {
    /// Creates a diagnostic with `message` at byte `offset`.
    #[must_use]
    pub fn new(message: impl Into<Box<str>>, offset: u32) -> Self {
        Self {
            message: message.into(),
            offset,
        }
    }

    /// Returns the diagnostic message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the byte offset into the source where the issue was detected.
    #[must_use]
    pub const fn offset(&self) -> u32 {
        self.offset
    }
}

/// Rejects a Rust expression outside the pure-read subset §2.10 allows, returning
/// why.
///
/// # One rule, both front ends
///
/// This is *the* §2.10 check. The `okui!` macro calls it with the
/// [`syn::Expr`](syn::Expr) it already parsed; a compiled `.okui` file reaches it
/// through [`ExpressionValidator::validate`], which parses the attribute's text
/// first. Two copies of a security rule is one copy too many — they drift, and the
/// looser one is the one that decides.
///
/// # A positive allow-list, not a deny-list
///
/// The match names what an expression **may** be and refuses everything else, so a
/// shape nobody thought about is refused rather than admitted. That direction is
/// the point: §2.10's forbidden side names *effects* — "I/O", "direct service
/// access" — and no syntactic list can enumerate those. A list of what is allowed
/// is only ever as good as the spec; a list of what is forbidden is only ever as
/// good as the imagination of whoever wrote it.
///
/// It has teeth only since bindings became real Rust. A binding used to be emitted
/// as a string, so `{do_io()}` produced the harmless text `"do_io()"` — the rule
/// was policing a language that never evaluated.
///
/// # What is allowed, and why each
///
/// * **paths** and **field access** — §2.10's props, read-only state, loop locals
///   and field access. The whole point of a binding.
/// * **literals** — a constant is a read of nothing.
/// * **comparisons and logical operators**, and `!` / `-` — named by §2.10.
/// * **arithmetic** — named by neither side, and allowed because every §2.10
///   *forbidden* item is an effect and arithmetic has none. Refusing `{a + b}`
///   while allowing `{a > b}` would be a rule about spelling rather than about
///   what an expression does.
/// * **parentheses**, **indexing**, **casts**, **borrows** — grouping and reads by
///   another name.
/// * **method calls** — the gap, below.
///
/// Everything else is refused, which by construction covers §2.10's forbidden
/// list: assignment and compound assignment, `await` / `async`, closures,
/// `unsafe`, macros, loops, `?`, and **free function calls** — the "unconstrained
/// arbitrary function calls" it names, refused until something declares which
/// functions are its "explicitly exported pure" ones.
///
/// # The gap this does not close
///
/// A **method call** is admitted. §2.10 permits Option/null checks (`.is_some()`),
/// formatting and translations, and all three are method calls in Rust — but so is
/// `.write_to_disk()`, and nothing here can tell them apart. Refusing them
/// wholesale would forbid what the spec allows. The receiver and the arguments are
/// still walked, so a free call cannot hide inside one: the gap is exactly one
/// level wide. It waits on the same registry free calls do.
///
/// # Errors
///
/// Returns the reason `expr` is outside §2.10's subset — [`FORBIDDEN_MESSAGE`]
/// for a shape that is not a pure read, or a pointed message for an assignment.
/// The `Err` is a sentence for a caller to render, since the two callers report
/// it differently: the macro attaches a span, the file path a byte offset.
pub fn check_expression(expr: &Expr) -> Result<(), String> {
    match expr {
        // A read of a name, a field, or a constant.
        Expr::Path(_) | Expr::Lit(_) => Ok(()),
        Expr::Field(inner) => check_expression(&inner.base),
        Expr::Paren(inner) => check_expression(&inner.expr),
        Expr::Index(inner) => {
            check_expression(&inner.expr)?;
            check_expression(&inner.index)
        }
        Expr::Reference(inner) => check_expression(&inner.expr),
        Expr::Unary(inner) => check_expression(&inner.expr),
        // `syn` files compound assignment (`+=`) under `Binary`, not under
        // `Assign` — so admitting every binary operator would admit the mutation
        // §2.10 forbids. The operator is checked, not just the operands.
        Expr::Binary(inner) => {
            if is_assigning_op(inner.op) {
                return Err("assignment is not allowed in an expression (spec §2.10)".to_owned());
            }
            check_expression(&inner.left)?;
            check_expression(&inner.right)
        }
        Expr::Cast(inner) => check_expression(&inner.expr),
        // See "the gap this does not close".
        Expr::MethodCall(inner) => {
            check_expression(&inner.receiver)?;
            inner.args.iter().try_for_each(check_expression)
        }
        _ => Err(FORBIDDEN_MESSAGE.to_owned()),
    }
}

/// What a caller is told when an expression is outside §2.10's subset.
pub const FORBIDDEN_MESSAGE: &str =
    "this expression is outside what `.okui` allows (spec §2.10): an expression may \
     read props, state and loop locals, access fields, compare, and combine with \
     logical operators — it may not assign, mutate, await, or call a function that \
     has not been declared pure";

/// Whether `op` mutates its left operand (`+=`, `-=`, …).
///
/// A separate question from "is it a binary operator", because `syn` gives the
/// compound-assignment operators the same `Expr::Binary` node as `+` and `==`.
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

/// Validates that an `.okui` expression stays within the pure-read subset the
/// spec allows (spec §2.10).
///
/// A thin wrapper: it parses the attribute's text as a Rust expression and hands
/// it to [`check_expression`], which is the rule. The `okui!` macro applies the
/// same function to the `syn::Expr` it already has, so a document and an inline
/// invocation are held to one standard.
///
/// # It used to be a lexical scan
///
/// It looked for a bare `=` and a handful of keywords in the raw text, which is
/// all a validator can do without parsing — and it let `{load_config()}` through,
/// because nothing in that string is a forbidden *word*. The expression is real
/// Rust (that is what it lowers to), so it can simply be parsed; there was never a
/// reason for the file path to know less about an expression than the macro does.
#[derive(Debug, Clone, Default)]
pub struct ExpressionValidator;

impl ExpressionValidator {
    /// Creates a new expression validator.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Checks `expr_text` against §2.10, returning one diagnostic per violation,
    /// anchored at `offset`.
    ///
    /// Text that is not a Rust expression at all is itself a diagnostic: an
    /// `.okui` binding lowers to Rust, so anything rustc could not parse could
    /// never have run — reporting it here beats emitting it and letting the error
    /// surface inside generated code.
    ///
    /// `offset` is the expression's byte offset in the source and is attached to
    /// every diagnostic; resolving a column *within* the expression would need the
    /// span mapping this text-based seam does not carry.
    #[must_use]
    pub fn validate(&self, expr_text: &str, offset: u32) -> Vec<ValidationDiagnostic> {
        let Ok(expr) = syn::parse_str::<Expr>(expr_text) else {
            return vec![ValidationDiagnostic::new(
                "this is not a valid expression (spec §2.10)",
                offset,
            )];
        };
        match check_expression(&expr) {
            Ok(()) => Vec::new(),
            Err(message) => vec![ValidationDiagnostic::new(message, offset)],
        }
    }

    /// Validates every dynamic-attribute expression in `doc`, recursively
    /// (spec §2.10).
    ///
    /// Each element's attributes are inspected; every attribute that carries a
    /// dynamic `{expr}` value has its expression text validated at the
    /// expression's byte offset, then the element's children are walked the same
    /// way. Literal string attributes have no expression and are skipped.
    #[must_use]
    pub fn validate_document(&self, doc: &Document) -> Vec<ValidationDiagnostic> {
        let mut diagnostics = Vec::new();
        for element in doc.elements() {
            self.validate_element(&element, &mut diagnostics);
        }
        diagnostics
    }

    /// Validates one element's dynamic-attribute expressions, then recurses into
    /// its element-shaped children.
    fn validate_element(&self, element: &Element, diagnostics: &mut Vec<ValidationDiagnostic>) {
        for attribute in element.attributes() {
            if let Some(expression) = attribute.value_expression() {
                let offset = expression.syntax().offset().unwrap_or(0);
                diagnostics.extend(self.validate(&expression.text(), offset));
            }
        }
        for child in element.children() {
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
        assert!(diagnostics[0].message().contains("§2.10"), "{:?}", diagnostics[0]);
    }

    #[test]
    fn validate_document_recurses_into_children() {
        let validator = ExpressionValidator::new();
        let doc = document("<Root><Child y={load_config()}/></Root>");
        let diagnostics = validator.validate_document(&doc);
        assert_eq!(diagnostics.len(), 1, "the nested element is walked");
        assert!(diagnostics[0].message().contains("§2.10"), "{:?}", diagnostics[0]);
    }
}
