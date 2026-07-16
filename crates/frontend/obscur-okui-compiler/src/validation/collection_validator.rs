//! `CollectionValidator`: the **file** front end's adapter onto the §2.11
//! collection rule.

use obscur_okui_syntax::ast::{Document, Element};

use crate::validation::{ValidationDiagnostic, check_collection};

/// Checks every collection construct in a `.okui` document against the §2.11
/// collection rule (spec §2.11, §4.9).
///
/// The rule itself is [`check_collection`](crate::validation::check_collection) —
/// the same function the `okui!` macro is checked with. This type only walks the
/// syntax tree and attaches a byte offset to each refusal, the mirror of what
/// [`ExpressionValidator`](crate::validation::ExpressionValidator) does for
/// §2.10.
///
/// Today it refuses exactly one tag, `<VirtualFor>`; see the
/// [`collection_rule`](crate::validation::collection_rule) module docs for why a
/// refusal is better than the silent `<For>` it used to compile to.
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
    /// The walk does not stop at a refusal: a document with two `<VirtualFor>`s
    /// reports both, the same report-them-all policy `Compiler::compile` follows.
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
    use crate::validation::collection_rule::VIRTUAL_FOR_MESSAGE;
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
    fn a_virtual_for_is_refused_at_its_own_offset() {
        let src = "<div><VirtualFor each={items} key={id}><Row/></VirtualFor></div>";
        let diagnostics = CollectionValidator::new().validate(&document(src));
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message(), VIRTUAL_FOR_MESSAGE);
        // Pointed at the `<VirtualFor>` itself, not at the document root.
        assert_eq!(
            diagnostics[0].offset() as usize,
            src.find("<VirtualFor").expect("the tag is in the source")
        );
    }

    #[test]
    fn every_virtual_for_is_reported_not_just_the_first() {
        let doc = document(
            "<div>\
               <VirtualFor each={a} key={id}><Row/></VirtualFor>\
               <VirtualFor each={b} key={id}><Row/></VirtualFor>\
             </div>",
        );
        assert_eq!(CollectionValidator::new().validate(&doc).len(), 2);
    }
}
