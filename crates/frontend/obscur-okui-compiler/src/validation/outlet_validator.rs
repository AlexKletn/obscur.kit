//! `OutletValidator`: the **file** front end's adapter onto the §2.12 outlet
//! rule.

use obscur_okui_syntax::ast::{Attribute, Document, Element};
use obscur_okui_syntax::syntax::SyntaxKind;

use crate::validation::ValidationDiagnostic;
use crate::validation::outlet_rule::{OutletName, OutletUse, check_outlet, is_outlet_tag};

/// Checks every authored `<slot …/>` in a `.okui` document against the §2.12
/// outlet rule.
///
/// The rule itself is [`check_outlet`](crate::validation::check_outlet) — the same
/// function the `okui!` macro is checked with. This type only walks the syntax
/// tree and attaches a byte offset to each refusal, the mirror of what
/// [`CollectionValidator`](crate::validation::CollectionValidator) does for §2.11.
///
/// Every diagnostic is anchored at the `<slot>` element itself rather than at the
/// offending attribute. That is deliberate and it is the honest anchor: an outlet
/// is a two-token construct, and every rule here is about *the outlet* — that it
/// has no name, that its name is not decidable, that it carries something it
/// cannot carry. There is no case where the author needs to be pointed inside it.
#[derive(Debug, Clone, Copy, Default)]
pub struct OutletValidator;

impl OutletValidator {
    /// Creates a validator.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Validates every element in `doc`, recursively, reporting each outlet
    /// violation at that element's byte offset.
    #[must_use]
    pub fn validate(&self, doc: &Document) -> Vec<ValidationDiagnostic> {
        let mut diagnostics = Vec::new();
        for element in doc.elements() {
            Self::validate_element(&element, &mut diagnostics);
        }
        diagnostics
    }

    /// Checks one element, then recurses into its element-shaped children.
    ///
    /// The walk does not stop at a refusal, and it does not stop at an outlet
    /// either: a document with two bad `<slot>`s reports both, the same
    /// report-them-all policy `Compiler::compile` follows.
    fn validate_element(element: &Element, diagnostics: &mut Vec<ValidationDiagnostic>) {
        if let Some(tag) = element.tag_name()
            && is_outlet_tag(&tag)
        {
            let offset = element.syntax().offset().unwrap_or(0);
            // The rule borrows `&str`; the AST hands out `String`, so the storage
            // has to outlive the borrow.
            let name = element.attribute("name");
            let name_value = name.as_ref().and_then(Attribute::string_value);
            let extra: Vec<String> = element
                .attributes()
                .filter(|attribute| {
                    attribute.is_event() || attribute.name().is_none_or(|n| n != "name")
                })
                .map(|attribute| authored_spelling(&attribute))
                .collect();
            let outlet = OutletUse {
                extra_attributes: extra.iter().map(String::as_str).collect(),
                has_children: element.children().next().is_some(),
            };
            let classified = match (&name, &name_value) {
                (None, _) => OutletName::Missing,
                (Some(_), Some(value)) => OutletName::Literal(value),
                (Some(_), None) => OutletName::Dynamic,
            };
            for message in check_outlet(&tag, classified, &outlet) {
                diagnostics.push(ValidationDiagnostic::new(message, offset));
            }
        }
        for child in element.children() {
            if let Some(child_element) = Element::cast(child) {
                Self::validate_element(&child_element, diagnostics);
            }
        }
    }
}

/// An attribute's spelling as the author wrote it — `class`, or `on:click` for an
/// event-style one.
///
/// [`Attribute::name`] returns the *first* identifier, which for `on:click` is
/// `on`. Quoting `'on'` back at an author who wrote `on:click` is a message that
/// names a thing they cannot find, so the prefix is put back.
fn authored_spelling(attribute: &Attribute) -> String {
    let name = attribute.name().unwrap_or_default();
    if !attribute.is_event() {
        return name;
    }
    let mut seen_colon = false;
    for token in attribute.syntax().tokens() {
        match token.kind() {
            SyntaxKind::Colon => seen_colon = true,
            SyntaxKind::Ident if seen_colon => return format!("{name}:{}", token.text()),
            _ => {}
        }
    }
    name
}

#[cfg(test)]
mod tests {
    use super::OutletValidator;
    use crate::validation::outlet_rule::{
        DYNAMIC_NAME_MESSAGE, MISSING_NAME_MESSAGE, OUTLET_CHILDREN_MESSAGE,
    };
    use obscur_okui_syntax::ast::Document;
    use obscur_okui_syntax::parser::Parser;

    fn document(src: &str) -> Document {
        Document::cast(Parser::new().parse(src).tree().root().clone()).expect("Document")
    }

    fn messages(src: &str) -> Vec<String> {
        OutletValidator::new()
            .validate(&document(src))
            .into_iter()
            .map(|d| d.message().to_owned())
            .collect()
    }

    #[test]
    fn a_named_outlet_is_accepted() {
        assert!(messages(r#"<div><slot name="header"/></div>"#).is_empty());
    }

    #[test]
    fn an_unnamed_outlet_is_refused_at_its_own_offset() {
        let src = "<div><slot/></div>";
        let diagnostics = OutletValidator::new().validate(&document(src));
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message(), MISSING_NAME_MESSAGE);
        assert_eq!(
            diagnostics[0].offset() as usize,
            src.find("<slot").expect("the tag is in the source")
        );
    }

    #[test]
    fn a_dynamic_name_is_refused() {
        assert_eq!(
            messages("<div><slot name={which}/></div>"),
            [DYNAMIC_NAME_MESSAGE]
        );
    }

    #[test]
    fn an_extra_attribute_is_refused() {
        let found = messages(r#"<slot name="header" class="banner"/>"#);
        assert_eq!(found.len(), 1);
        assert!(found[0].contains("'class'"));
    }

    #[test]
    fn children_are_refused() {
        assert_eq!(
            messages(r#"<slot name="header"><b>fallback</b></slot>"#),
            [OUTLET_CHILDREN_MESSAGE]
        );
    }

    #[test]
    fn every_outlet_is_checked_not_just_the_first() {
        assert_eq!(messages("<div><slot/><slot/></div>").len(), 2);
    }

    #[test]
    fn an_ordinary_tag_with_a_name_is_untouched() {
        // The disjoint half: `name` is an ordinary prop everywhere but here.
        assert!(messages(r#"<div name="x" class="y"><b>hi</b></div>"#).is_empty());
    }
}
