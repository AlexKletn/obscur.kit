//! `CapabilityValidator`: checks that each element's declared required
//! capabilities are granted (spec §2.13).

use std::collections::BTreeSet;

use obscur_capability::CapabilityId;
use obscur_okui_syntax::ast::{Document, Element};

use crate::validation::ValidationDiagnostic;

/// Validates that every capability an element declares it requires has been
/// granted (spec §2.13).
///
/// # Declaration convention
///
/// An element declares a required capability via a `requires="cap.id"` **string**
/// attribute — for example `<Camera requires="camera"/>`. The validator holds
/// the set of *granted* capability ids and reports an element whose declared
/// requirement is absent from that set. A dynamic `requires={..}` expression is
/// not a static capability id and is ignored (it has no `string_value`).
///
/// The granted set is stored as a [`BTreeSet`] of ids so membership checks are
/// ordered and deduplicated.
#[derive(Debug, Clone, Default)]
pub struct CapabilityValidator {
    granted: BTreeSet<Box<str>>,
}

impl CapabilityValidator {
    /// Creates a validator whose granted set is `granted`.
    #[must_use]
    pub fn new(granted: impl IntoIterator<Item = CapabilityId>) -> Self {
        Self {
            granted: granted.into_iter().map(|id| id.as_str().into()).collect(),
        }
    }

    /// Returns whether the capability id `id` is in the granted set.
    #[must_use]
    pub fn has_capability(&self, id: &str) -> bool {
        self.granted.contains(id)
    }

    /// Validates every element in `doc`, recursively, reporting each declared
    /// requirement that is not granted (spec §2.13).
    ///
    /// For each element carrying a `requires="cap"` string attribute whose value
    /// `cap` is not in the granted set, a diagnostic
    /// `"required capability '<cap>' is not granted"` is produced at the
    /// element's byte offset.
    #[must_use]
    pub fn validate(&self, doc: &Document) -> Vec<ValidationDiagnostic> {
        let mut diagnostics = Vec::new();
        for element in doc.elements() {
            self.validate_element(&element, &mut diagnostics);
        }
        diagnostics
    }

    /// Checks one element's `requires` declaration, then recurses into its
    /// element-shaped children.
    fn validate_element(&self, element: &Element, diagnostics: &mut Vec<ValidationDiagnostic>) {
        if let Some(cap) = element
            .attribute("requires")
            .and_then(|attr| attr.string_value())
            && !self.has_capability(&cap)
        {
            let offset = element.syntax().offset().unwrap_or(0);
            diagnostics.push(ValidationDiagnostic::new(
                format!("required capability '{cap}' is not granted"),
                offset,
            ));
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
    use obscur_okui_syntax::parser::Parser;

    fn document(src: &str) -> Document {
        Document::cast(Parser::new().parse(src).tree().root().clone()).expect("Document")
    }

    #[test]
    fn granted_capability_passes() {
        let validator = CapabilityValidator::new([CapabilityId::new("camera")]);
        assert!(validator.has_capability("camera"));
        let doc = document("<Cam requires=\"camera\"/>");
        assert!(validator.validate(&doc).is_empty());
    }

    #[test]
    fn missing_capability_is_flagged() {
        let validator = CapabilityValidator::default();
        assert!(!validator.has_capability("camera"));
        let doc = document("<Cam requires=\"camera\"/>");
        let diagnostics = validator.validate(&doc);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].message(),
            "required capability 'camera' is not granted"
        );
    }

    #[test]
    fn requirement_on_nested_child_is_checked() {
        let validator = CapabilityValidator::new([CapabilityId::new("audio")]);
        let doc = document("<Root><Mic requires=\"camera\"/></Root>");
        let diagnostics = validator.validate(&doc);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].message(),
            "required capability 'camera' is not granted"
        );
    }
}
