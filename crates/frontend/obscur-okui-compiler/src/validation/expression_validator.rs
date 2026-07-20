//! `ExpressionValidator`: enforces the `.okui` expression restrictions (spec
//! §2.10), plus the shared [`ValidationDiagnostic`] used across the validation
//! module.

use obscur_okui_syntax::ast::{Document, Element, Expression};
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
/// # The method-call gap, and the two edges of it §2.10 *does* let us cut
///
/// A method call is where Rust spells three things §2.10 *allows* — Option/null
/// checks (`.is_some()`), formatting and translations — and several it *forbids*.
/// Nothing syntactic tells `.format()` from `.write_to_disk()` in general, so the
/// receiver is admitted by default. But two of the forbidden bullets have a shape
/// this can recognise, and refusing them wholesale would not forbid anything the
/// spec allows:
///
/// * **arbitrary mutation** — a method whose name is a standard in-place mutator
///   (`push`, `sort`, `insert`, `clear`, …; see [`is_mutating_method`]). An
///   author reads state in an expression; they do not mutate it there.
/// * **direct service access** — a method invoked on a value reached *through* a
///   field (`services.http.get(...)`), as opposed to called directly on a read
///   binding (`user.is_some()`). This is a structural approximation, not a proof:
///   it is the shape "navigate into an object, then call into it" that §2.10's
///   "direct service access" names, and it leaves the direct-on-binding calls the
///   spec allows untouched. See [`SERVICE_ACCESS_MESSAGE`].
///
/// The receiver and the arguments are still walked, so a free call cannot hide
/// inside one.
///
/// # Free calls and the exported-pure exception
///
/// A **free function call** (`Expr::Call`) is §2.10's "unconstrained arbitrary
/// function calls" — refused, *unless* the callee is a bare name that a `Function`
/// symbol declared in scope, which is §2.10's Allowed "explicitly exported pure
/// functions." The set of such names is [`functions`](check_expression_in); the
/// context-free [`check_expression`] passes an empty one, so the macro front end
/// (which has no way to declare a pure function) refuses every free call, as it
/// did before this exception existed.
///
/// # Errors
///
/// Returns the reason `expr` is outside §2.10's subset — a pointed message for an
/// assignment, mutation, service access or refused call, else
/// [`FORBIDDEN_MESSAGE`]. The `Err` is a sentence for a caller to render, since
/// the two callers report it differently: the macro attaches a span, the file
/// path a byte offset.
pub fn check_expression(expr: &Expr) -> Result<(), String> {
    check_expression_in(expr, &[])
}

/// The §2.10 check with the set of `.okui`-declared **explicitly exported pure
/// functions** in scope, so a call to one of them is admitted where an
/// unconstrained free call is refused (spec §2.10).
///
/// [`check_expression`] is this with an empty `functions` — every free call
/// refused — which is what the macro front end wants and what the rule did before
/// the exported-pure exception.
///
/// # Errors
///
/// As [`check_expression`].
pub fn check_expression_in(expr: &Expr, functions: &[Box<str>]) -> Result<(), String> {
    match expr {
        // A read of a name, a field, or a constant.
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
                return Err(SERVICE_ACCESS_MESSAGE.to_owned());
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

/// The bare identifier a call's callee names, if it is a single-segment path
/// (`fmt_price`), else `None` (`std::fs::read`, `make()()`, a method call).
///
/// Only a bare name can match a `.okui`-declared pure function, and only a bare
/// name can be reported by name; a qualified or computed callee is refused with
/// the generic [`FORBIDDEN_FREE_CALL`] message.
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

/// The message for a bare call to a function no `Function` symbol declared: it
/// names the refused callee and attributes the refusal to §2.10's exported-pure
/// rule.
fn refused_call_message(name: &str) -> String {
    format!(
        "`{name}` is not an explicitly exported pure function, so it may not be \
         called in an expression (spec §2.10)"
    )
}

/// What a caller is told when a qualified or computed free call is refused.
pub const FORBIDDEN_FREE_CALL: &str =
    "only explicitly exported pure functions may be called in an expression (spec §2.10)";

/// What a caller is told when a method is invoked on a value reached through a
/// field — the shape §2.10's "direct service access" names.
pub const SERVICE_ACCESS_MESSAGE: &str =
    "direct service access is not allowed in an expression (spec §2.10)";

/// Whether `method` is a standard in-place mutator — a method whose whole purpose
/// is to change its receiver (spec §2.10, "arbitrary mutation").
///
/// This is a denylist of the common `Vec`/`String`/collection mutators, not a
/// proof of purity: a method absent from it is admitted (the method-call gap). It
/// names only methods that mutate *by convention* in the standard library, so a
/// pure method that returns a new value — `str::replace`, `Vec::iter` — is not on
/// it and stays allowed.
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

/// What a caller is told when an expression is outside §2.10's subset.
pub const FORBIDDEN_MESSAGE: &str = "this expression is outside what `.okui` allows (spec §2.10): an expression may \
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
pub struct ExpressionValidator {
    /// The names of `.okui`-declared **explicitly exported pure functions**, so a
    /// call to one is admitted where an unconstrained free call is refused
    /// (spec §2.10). Empty by default — [`new`](Self::new) — which is what the
    /// macro front end wants, since it has no way to declare a pure function.
    functions: Vec<Box<str>>,
}

impl ExpressionValidator {
    /// Creates a new expression validator with no exported-pure functions in
    /// scope (every free call refused).
    #[must_use]
    pub const fn new() -> Self {
        Self {
            functions: Vec::new(),
        }
    }

    /// Returns the validator with `functions` declared as the
    /// explicitly-exported pure functions a call may name (fluent; spec §2.10).
    #[must_use]
    pub fn with_functions(mut self, functions: impl IntoIterator<Item = Box<str>>) -> Self {
        self.functions = functions.into_iter().collect();
        self
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
        match check_expression_in(&expr, &self.functions) {
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
