#![doc = include_str!("collection_validator.doc/_module.md")]

use obscur_okui_syntax::ast::{Document, Element};

use crate::validation::{ValidationDiagnostic, check_collection};

#[doc = include_str!("collection_validator.doc/CollectionValidator.md")]
#[derive(Debug, Clone, Copy, Default)]
pub struct CollectionValidator;

impl CollectionValidator {
    #[doc = include_str!("collection_validator.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[doc = include_str!("collection_validator.doc/validate.md")]
    #[must_use]
    pub fn validate(&self, doc: &Document) -> Vec<ValidationDiagnostic> {
        let mut diagnostics = Vec::new();
        for element in doc.elements() {
            Self::validate_element(&element, &mut diagnostics);
        }
        diagnostics
    }

    #[doc = include_str!("collection_validator.doc/validate_element.md")]
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
