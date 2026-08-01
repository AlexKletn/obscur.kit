#![doc = include_str!("ui_ir_validator.doc/_module.md")]

use crate::document::{UiIrDocument, UiIrExpression, UiIrNode, UiIrNodeKind};
use crate::validation::{UiIrValidationError, UiIrValidationResult, ValidationContext};

#[doc = include_str!("ui_ir_validator.doc/UiIrValidator.md")]
#[derive(Debug, Default, Clone, Copy)]
pub struct UiIrValidator;

impl UiIrValidator {
    #[doc = include_str!("ui_ir_validator.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[doc = include_str!("ui_ir_validator.doc/validate.md")]
    #[must_use]
    pub fn validate(&self, doc: &UiIrDocument) -> UiIrValidationResult {
        let mut result = UiIrValidationResult::new();
        let mut ctx = ValidationContext::new();

        // Declared-capability well-formedness: an empty capability id is an
        // undeclared capability.
        for cap in doc.required_capabilities() {
            if cap.as_str().is_empty() {
                result.push_error(UiIrValidationError::UndeclaredCapability(Box::from("")));
            }
        }

        self.validate_node(doc.root(), &mut ctx, &mut result);
        result
    }

    #[doc = include_str!("ui_ir_validator.doc/validate_node.md")]
    fn validate_node(
        &self,
        node: &UiIrNode,
        ctx: &mut ValidationContext,
        result: &mut UiIrValidationResult,
    ) {
        // (a) Duplicate ids: `record_id` returns false when already seen.
        if !ctx.record_id(node.id()) {
            result.push_error(UiIrValidationError::DuplicateNodeId(node.id()));
        }

        // (b) Empty tag on an element or component.
        if matches!(node.kind(), UiIrNodeKind::Element | UiIrNodeKind::Component)
            && node.tag().is_empty()
        {
            result.push_error(UiIrValidationError::EmptyElementTag(node.id()));
        }

        // (c) A text node must be a leaf.
        if node.kind() == UiIrNodeKind::Text && !node.children().is_empty() {
            result.push_error(UiIrValidationError::TextNodeHasChildren(node.id()));
        }

        // (d) A property bound to an empty path.
        for property in node.properties() {
            if let UiIrExpression::Binding(path) = property.value() {
                if path.is_empty() {
                    result.push_error(UiIrValidationError::EmptyBindingPath {
                        node: node.id(),
                        property: Box::from(property.name()),
                    });
                }
            }
        }

        for child in node.children() {
            self.validate_node(child, ctx, result);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::UiIrProperty;

    #[test]
    fn clean_document_is_valid() {
        let root = UiIrNode::element(1, "div")
            .with_child(UiIrNode::text(2, "hello"))
            .with_child(UiIrNode::element(3, "span").with_child(UiIrNode::text(4, "world")));
        let doc = UiIrDocument::new(root);
        let result = UiIrValidator::new().validate(&doc);
        assert!(
            result.is_valid(),
            "unexpected errors: {:?}",
            result.errors()
        );
    }

    #[test]
    fn duplicate_ids_are_reported() {
        let root = UiIrNode::element(1, "div").with_child(UiIrNode::text(1, "dup"));
        let doc = UiIrDocument::new(root);
        let result = UiIrValidator::new().validate(&doc);
        assert!(!result.is_valid());
        assert!(
            result
                .errors()
                .contains(&UiIrValidationError::DuplicateNodeId(1))
        );
    }

    #[test]
    fn empty_element_tag_is_reported() {
        let doc = UiIrDocument::new(UiIrNode::element(1, ""));
        let result = UiIrValidator::new().validate(&doc);
        assert_eq!(result.errors(), &[UiIrValidationError::EmptyElementTag(1)]);
    }

    #[test]
    fn text_node_with_children_is_reported() {
        let root = UiIrNode::text(1, "leaf").with_child(UiIrNode::text(2, "child"));
        let doc = UiIrDocument::new(root);
        let result = UiIrValidator::new().validate(&doc);
        assert!(
            result
                .errors()
                .contains(&UiIrValidationError::TextNodeHasChildren(1))
        );
    }

    #[test]
    fn empty_binding_path_is_reported() {
        let root = UiIrNode::element(1, "div").with_property(UiIrProperty::binding("title", ""));
        let doc = UiIrDocument::new(root);
        let result = UiIrValidator::new().validate(&doc);
        assert!(
            result
                .errors()
                .contains(&UiIrValidationError::EmptyBindingPath {
                    node: 1,
                    property: "title".into(),
                })
        );
    }

    #[test]
    fn empty_required_capability_is_reported() {
        use obscur_capability::CapabilityId;
        let doc = UiIrDocument::new(UiIrNode::element(1, "div")).requiring(CapabilityId::new(""));
        let result = UiIrValidator::new().validate(&doc);
        assert!(
            result
                .errors()
                .contains(&UiIrValidationError::UndeclaredCapability(Box::from("")))
        );
    }
}
