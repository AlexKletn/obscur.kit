//! `ExpressionValidator`: enforces the `.okui` expression restrictions (spec
//! §2.10), plus the shared [`ValidationDiagnostic`] used across the validation
//! module.

use obscur_okui_syntax::ast::{Document, Element};

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

/// The keywords that may not appear in an `.okui` expression (spec §2.10).
///
/// An expression is a pure, side-effect-free read: assignment, mutation, I/O,
/// async and control-flow keywords are all forbidden.
const FORBIDDEN_KEYWORDS: [&str; 6] = ["await", "async", "mut", "fn", "let", "return"];

/// Validates that an `.okui` expression stays within the pure-expression subset
/// the spec allows (spec §2.10).
///
/// Per §2.10 an expression may **not** contain assignment, mutation, I/O, async
/// or arbitrary function definitions / control flow. Because the syntax layer
/// captures each expression as raw text (there is no expression AST yet), this
/// validator performs a *lexical* scan of that text for the forbidden
/// constructs it can recognise without parsing:
///
/// - an assignment `=` that is not part of a comparison operator (`==`, `!=`,
///   `>=`, `<=`);
/// - a whole-word occurrence of a forbidden keyword (`await`, `async`, `mut`,
///   `fn`, `let`, `return`).
///
/// # Boundary
///
/// It does **not** attempt to validate arbitrary function *calls* beyond the
/// keyword list: distinguishing an allowed pure call from a forbidden effectful
/// one requires a real expression parser and semantic knowledge of the callee,
/// which is out of scope here. That check is deferred to a later pass once an
/// expression AST exists.
#[derive(Debug, Clone, Default)]
pub struct ExpressionValidator;

impl ExpressionValidator {
    /// Creates a new expression validator.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Scans the raw `expr_text` for forbidden constructs, returning one
    /// diagnostic per violation, each anchored at `offset` (spec §2.10).
    ///
    /// The scan flags an assignment `=` (a `=` that is not part of `==`, `!=`,
    /// `>=` or `<=`) and any whole-word forbidden keyword. `offset` is the byte
    /// offset of the expression in the source and is attached to every
    /// diagnostic; per-violation column resolution needs an expression parser
    /// and is out of scope.
    #[must_use]
    pub fn validate(&self, expr_text: &str, offset: u32) -> Vec<ValidationDiagnostic> {
        let mut diagnostics = Vec::new();

        if contains_assignment(expr_text) {
            diagnostics.push(ValidationDiagnostic::new(
                "assignment is not allowed in expressions",
                offset,
            ));
        }

        for keyword in FORBIDDEN_KEYWORDS {
            if word_present(expr_text, keyword) {
                diagnostics.push(ValidationDiagnostic::new(
                    format!("'{keyword}' is not allowed in expressions"),
                    offset,
                ));
            }
        }

        diagnostics
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

/// Returns whether `text` contains an assignment `=` — a `=` that is not part of
/// a comparison operator (`==`, `!=`, `>=`, `<=`).
///
/// A `=` preceded by `!`, `<`, `>` or another `=`, or immediately followed by
/// another `=`, is part of a comparison; any remaining lone `=` is an
/// assignment.
fn contains_assignment(text: &str) -> bool {
    let bytes = text.as_bytes();
    for (index, &byte) in bytes.iter().enumerate() {
        if byte != b'=' {
            continue;
        }
        let prev = index.checked_sub(1).map(|i| bytes[i]);
        let next = bytes.get(index + 1).copied();
        let is_comparison =
            matches!(prev, Some(b'!' | b'<' | b'>' | b'=')) || matches!(next, Some(b'='));
        if !is_comparison {
            return true;
        }
    }
    false
}

/// Returns whether `word` occurs in `text` as a whole word — bounded on each
/// side by a non-identifier character (or the start/end of the text).
///
/// Identifier characters are ASCII letters, digits and `_`; a match embedded in
/// a longer identifier (for example `mut` inside `mutable`) is not a whole-word
/// occurrence and is not reported.
fn word_present(text: &str, word: &str) -> bool {
    if word.is_empty() {
        return false;
    }
    let bytes = text.as_bytes();
    let word_bytes = word.as_bytes();
    let mut search_from = 0;
    while let Some(relative) = find_subslice(&bytes[search_from..], word_bytes) {
        let start = search_from + relative;
        let end = start + word_bytes.len();
        let before_ok = start
            .checked_sub(1)
            .is_none_or(|i| !is_identifier_byte(bytes[i]));
        let after_ok = bytes
            .get(end)
            .copied()
            .is_none_or(|b| !is_identifier_byte(b));
        if before_ok && after_ok {
            return true;
        }
        search_from = start + 1;
    }
    false
}

/// Returns the index of the first occurrence of `needle` in `haystack`, or
/// `None` when absent (`needle` is assumed non-empty).
fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.len() > haystack.len() {
        return None;
    }
    (0..=haystack.len() - needle.len())
        .find(|&start| &haystack[start..start + needle.len()] == needle)
}

/// Returns whether `byte` is an identifier character (ASCII letter, digit or
/// `_`).
fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
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
    fn assignment_is_flagged() {
        let validator = ExpressionValidator::new();
        let diagnostics = validator.validate("a = b", 0);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].message(),
            "assignment is not allowed in expressions"
        );
    }

    #[test]
    fn forbidden_keyword_is_flagged() {
        let validator = ExpressionValidator::new();
        let diagnostics = validator.validate("await x", 0);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].message(),
            "'await' is not allowed in expressions"
        );
    }

    #[test]
    fn keyword_embedded_in_identifier_is_not_flagged() {
        let validator = ExpressionValidator::new();
        // `mutable` and `returned` merely contain the keywords as substrings.
        assert!(validator.validate("mutable", 0).is_empty());
        assert!(validator.validate("returned", 0).is_empty());
    }

    #[test]
    fn validate_document_walks_dynamic_attributes() {
        let validator = ExpressionValidator::new();
        let doc = document("<T x={a = b}/>");
        let diagnostics = validator.validate_document(&doc);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].message(),
            "assignment is not allowed in expressions"
        );
    }

    #[test]
    fn validate_document_recurses_into_children() {
        let validator = ExpressionValidator::new();
        let doc = document("<Root><Child y={let z}/></Root>");
        let diagnostics = validator.validate_document(&doc);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].message(),
            "'let' is not allowed in expressions"
        );
    }
}
