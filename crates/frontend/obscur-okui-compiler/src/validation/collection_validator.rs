//! `CollectionValidator`: the **file** front end's adapter onto the §2.11
//! collection rule.

use obscur_okui_syntax::{
    ast::{Document, Element},
    syntax::SyntaxKind,
};

use crate::validation::{
    DUPLICATE_KEY, ValidationDiagnostic, check_collection, check_condition_attribute,
    check_conditional_branch, duplicate_collection_slots,
};

#[doc = include_str!("../../docs/api/validation/collection_validator.doc/CollectionValidator.md")]
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
        let mut chain_open = false;
        for element in doc.elements() {
            Self::validate_element(&element, &mut chain_open, &mut diagnostics);
        }
        diagnostics
    }

    #[doc = include_str!("../../docs/api/validation/collection_validator.doc/validate_element.md")]
    fn validate_element(
        element: &Element,
        chain_open: &mut bool,
        diagnostics: &mut Vec<ValidationDiagnostic>,
    ) {
        let tag = element.tag_name().unwrap_or_default();
        let offset = element.syntax().offset().unwrap_or(0);
        match check_conditional_branch(&tag, *chain_open) {
            Ok(open) => *chain_open = open,
            Err(message) => diagnostics.push(ValidationDiagnostic::new(message, offset)),
        }
        if matches!(tag.as_str(), "If" | "ElseIf" | "Else") {
            let count = element
                .attributes()
                .filter(|attr| !attr.is_event() && attr.name().as_deref() == Some("cond"))
                .count();
            if let Err(message) = check_condition_attribute(&tag, count) {
                diagnostics.push(ValidationDiagnostic::new(message, offset));
            }
        }
        if let Err(message) = check_collection(&tag) {
            diagnostics.push(ValidationDiagnostic::new(message, offset));
        }
        let keys = element
            .attributes()
            .filter(|attr| !attr.is_event() && attr.name().as_deref() == Some("key"));
        for (index, attr) in keys.enumerate() {
            let offset = attr.syntax().offset().unwrap_or(offset);
            if index > 0 {
                diagnostics.push(ValidationDiagnostic::new(DUPLICATE_KEY, offset));
            } else if attr.value_expression().is_none() && attr.string_value().is_none() {
                diagnostics.push(ValidationDiagnostic::new(
                    crate::validation::KEY_REQUIRES_VALUE,
                    offset,
                ));
            }
        }
        if element.is_loop() {
            let tags = element
                .children()
                .filter_map(Element::cast)
                .filter_map(|child| {
                    Some((child.tag_name()?, child.syntax().offset().unwrap_or(0)))
                });
            diagnostics.extend(
                duplicate_collection_slots(tags)
                    .map(|(offset, message)| ValidationDiagnostic::new(message, offset)),
            );
        }
        let mut child_chain_open = false;
        for child in element.children() {
            if child.kind() == SyntaxKind::TextNode && child.text().trim().is_empty() {
                continue;
            }
            if let Some(child_element) = Element::cast(child) {
                Self::validate_element(&child_element, &mut child_chain_open, diagnostics);
            } else {
                child_chain_open = false;
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
    fn repeated_slots_are_reported_at_the_second_declaration() {
        let source = "<VirtualFor each={items} key={item}><Empty/><Loading/><Empty/></VirtualFor>";
        let diagnostics = CollectionValidator::new().validate(&document(source));
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message().contains("only one `<Empty>`"));
        assert_eq!(
            diagnostics[0].offset() as usize,
            source.rfind("<Empty").unwrap()
        );
    }

    #[test]
    fn repeated_keys_are_reported_at_the_repeated_attribute() {
        let validator = CollectionValidator::new();
        for source in [
            r#"<span key="first" key="second"/>"#,
            "<VirtualFor each={rows} key={item.id} key={index}/>",
        ] {
            let diagnostics = validator.validate(&document(source));
            assert_eq!(diagnostics.len(), 1);
            assert!(diagnostics[0].message().contains("only one `key`"));
            assert_eq!(
                diagnostics[0].offset() as usize,
                source.rfind("key=").unwrap()
            );
        }
    }

    #[test]
    fn bare_keys_are_rejected() {
        for source in ["<span key/>", "<For each={rows} key/>"] {
            let diagnostics = CollectionValidator::new().validate(&document(source));
            assert_eq!(diagnostics.len(), 1, "{source}");
            assert!(diagnostics[0].message().contains("explicit value"));
        }
    }

    #[test]
    fn conditional_chain_requires_adjacent_branches() {
        let validator = CollectionValidator::new();
        assert!(
            validator
                .validate(&document("<div><If cond={true}/>\n <Else/></div>"))
                .is_empty()
        );
        for source in [
            "<Else/>",
            "<div><If cond={true}/>text<Else/></div>",
            "<div><If cond={true}/><Else/><ElseIf cond={true}/></div>",
        ] {
            let diagnostics = validator.validate(&document(source));
            assert_eq!(diagnostics.len(), 1, "{source}");
            assert!(diagnostics[0].message().contains("must immediately follow"));
        }
    }

    #[test]
    fn conditional_branches_validate_condition_attributes() {
        let validator = CollectionValidator::new();
        for (source, expected) in [
            ("<If condition={true}/>", "requires a"),
            ("<If cond={true}/><ElseIf/>", "requires a"),
            ("<If cond={true} cond={false}/>", "only one"),
            ("<If cond={true}/><Else cond={false}/>", "cannot have"),
        ] {
            let diagnostics = validator.validate(&document(source));
            assert_eq!(diagnostics.len(), 1, "{source}");
            assert!(
                diagnostics[0].message().contains(expected),
                "{diagnostics:?}"
            );
        }
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
