//! `CollectionValidator`: the **file** front end's adapter onto the §2.11
//! collection rule.

use obscur_okui_syntax::ast::{Document, Element};

use crate::validation::{ValidationDiagnostic, check_collection};

/// Checks every collection construct in a `.okui` document against the §2.11
/// collection rule (spec §2.11, §4.9).
///
/// The rule itself is [`check_collection`](crate::validation::check_collection) —
/// the same function the `okui!` macro is checked with. This type only walks the
/// syntax tree and attaches a byte offset to each verdict, the mirror of what
/// [`ExpressionValidator`](crate::validation::ExpressionValidator) does for
/// §2.10.
///
/// Both §2.11 constructs, `<For>` and `<VirtualFor>`, are honoured, so the walk
/// reports nothing today; it is the seam a future collection constraint would
/// live in. See the [`collection_rule`](crate::validation::collection_rule)
/// module docs for how `<VirtualFor>` is realized as a viewport-bounded window
/// rather than the silent `<For>` it used to compile to.
#[derive(Debug, Clone, Copy, Default)]
pub struct CollectionValidator;

impl CollectionValidator {
    /// Creates a validator.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Validates every element in `doc`, recursively, reporting each collection
    /// construct the pipeline does not honour at that element's byte offset.
    #[must_use]
    pub fn validate(&self, doc: &Document) -> Vec<ValidationDiagnostic> {
        let mut diagnostics = Vec::new();
        for element in doc.elements() {
            Self::validate_element(&element, &mut diagnostics);
        }
        diagnostics
    }

    /// Checks one element's tag, then recurses into its element-shaped children.
    ///
    /// The walk does not stop at a diagnostic: it reports every offending
    /// element, the same report-them-all policy `Compiler::compile` follows.
    fn validate_element(element: &Element, diagnostics: &mut Vec<ValidationDiagnostic>) {
        if let Some(tag) = element.tag_name()
            && let Err(message) = check_collection(&tag)
        {
            diagnostics.push(ValidationDiagnostic::new(
                message,
                element.syntax().offset().unwrap_or(0),
            ));
        }
        for child in element.children() {
            if let Some(child_element) = Element::cast(child) {
                Self::validate_element(&child_element, diagnostics);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CollectionValidator;
    use obscur_okui_syntax::ast::Document;
    use obscur_okui_syntax::parser::Parser;

    fn document(src: &str) -> Document {
        Document::cast(Parser::new().parse(src).tree().root().clone()).expect("Document")
    }

    #[test]
    fn a_plain_for_is_accepted() {
        let doc = document("<For each={items} key={id}><Row/></For>");
        assert!(CollectionValidator::new().validate(&doc).is_empty());
    }

    #[test]
    fn a_virtual_for_is_now_accepted() {
        // §2.11's `<VirtualFor>` is a first-class construct realized as a
        // viewport-bounded window (spec §4.9), so the walk reports nothing.
        let src = "<div><VirtualFor each={items} key={id}><Row/></VirtualFor></div>";
        assert!(
            CollectionValidator::new()
                .validate(&document(src))
                .is_empty()
        );
    }

    #[test]
    fn several_virtual_fors_are_all_accepted() {
        let doc = document(
            "<div>\
               <VirtualFor each={a} key={id}><Row/></VirtualFor>\
               <VirtualFor each={b} key={id}><Row/></VirtualFor>\
             </div>",
        );
        assert!(CollectionValidator::new().validate(&doc).is_empty());
    }
}
