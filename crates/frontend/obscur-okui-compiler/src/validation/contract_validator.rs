//! Checks an element against a component contract (spec §2.13).

mod ast_instance;
mod attribute_value_type;
mod slot;

use obscur_okui_syntax::ast::Element;

use ast_instance::AstInstance;
use attribute_value_type::{
    attribute_value_type, describe_prop_type, describe_value_type, value_type_matches,
};

use crate::lowering::event_local_name;
use crate::validation::contract_rule::check_contract;
use crate::validation::{ComponentContract, ValidationDiagnostic};

#[doc = include_str!("../../docs/api/validation/contract_validator.doc/ContractValidator.md")]
#[derive(Debug, Clone, Default)]
pub struct ContractValidator;

impl ContractValidator {
    /// Creates a new contract validator.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[doc = include_str!("../../docs/api/validation/contract_validator.doc/validate_element.md")]
    #[must_use]
    pub fn validate_element(
        &self,
        element: &Element,
        contract: &ComponentContract,
    ) -> Vec<ValidationDiagnostic> {
        let offset = element.syntax().offset().unwrap_or(0);
        let instance = AstInstance::of(element, offset);
        let mut diagnostics: Vec<ValidationDiagnostic> =
            check_contract(&instance.as_use(), contract)
                .into_iter()
                .map(|violation| {
                    ValidationDiagnostic::new(
                        violation.message(contract.component()),
                        instance.offset_of(violation.anchor(), offset),
                    )
                })
                .collect();
        Self::check_string_dispatch(element, contract, offset, &mut diagnostics);
        Self::check_prop_types(element, contract, offset, &mut diagnostics);
        diagnostics
    }

    #[doc = include_str!("../../docs/api/validation/contract_validator.doc/check_prop_types.md")]
    fn check_prop_types(
        element: &Element,
        contract: &ComponentContract,
        element_offset: u32,
        diagnostics: &mut Vec<ValidationDiagnostic>,
    ) {
        for attribute in element.attributes() {
            if attribute.is_event() {
                continue;
            }
            let Some(name) = attribute.name() else {
                continue;
            };
            let Some(expected) = contract.prop_type(&name) else {
                continue;
            };
            let found = attribute_value_type(&attribute);
            if !value_type_matches(expected, found) {
                let offset = attribute.syntax().offset().unwrap_or(element_offset);
                diagnostics.push(ValidationDiagnostic::new(
                    format!(
                        "prop '{name}' on '{}' expects {} but was given {} (spec §2.8)",
                        contract.component(),
                        describe_prop_type(expected),
                        describe_value_type(found),
                    ),
                    offset,
                ));
            }
        }
    }

    #[doc = include_str!("../../docs/api/validation/contract_validator.doc/check_string_dispatch.md")]
    fn check_string_dispatch(
        element: &Element,
        contract: &ComponentContract,
        element_offset: u32,
        diagnostics: &mut Vec<ValidationDiagnostic>,
    ) {
        for attribute in element.attributes() {
            if attribute.is_event() && attribute.string_value().is_some() {
                let event = event_local_name(&attribute);
                let offset = attribute.syntax().offset().unwrap_or(element_offset);
                diagnostics.push(ValidationDiagnostic::new(
                    format!(
                        "event handler for '{event}' on '{}' is a string literal; \
                         string-based method dispatch is not used (spec §2.3, §2.13) — \
                         bind a handler with {{…}}",
                        contract.component()
                    ),
                    offset,
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_okui_syntax::ast::Document;
    use obscur_okui_syntax::parser::Parser;

    fn first_element(src: &str) -> Element {
        let doc = Document::cast(Parser::new().parse(src).tree().root().clone()).expect("Document");
        doc.elements().next().expect("one element")
    }

    #[test]
    fn contract_accessors_expose_fields() {
        let contract = ComponentContract::new("Tabs")
            .requiring_prop("selected")
            .allowing_child("Tab");
        assert_eq!(contract.component(), "Tabs");
        assert_eq!(contract.required_props(), &["selected".into()]);
        assert_eq!(contract.children().tags(), &["Tab".into()]);
    }

    #[test]
    fn missing_required_prop_is_flagged() {
        let validator = ContractValidator::new();
        let contract = ComponentContract::new("Tabs").requiring_prop("selected");
        let element = first_element("<Tabs/>");
        let diagnostics = validator.validate_element(&element, &contract);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].message(),
            "component 'Tabs' requires prop 'selected'"
        );
    }

    #[test]
    fn disallowed_child_is_flagged() {
        let validator = ContractValidator::new();
        let contract = ComponentContract::new("Tabs").allowing_child("Tab");
        let element = first_element("<Tabs><Panel/></Tabs>");
        let diagnostics = validator.validate_element(&element, &contract);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].message(),
            "'Panel' is not an allowed child of 'Tabs'"
        );
    }

    #[test]
    fn satisfied_contract_passes() {
        let validator = ContractValidator::new();
        let contract = ComponentContract::new("Tabs")
            .requiring_prop("selected")
            .allowing_child("Tab");
        let element = first_element("<Tabs selected=\"0\"><Tab/></Tabs>");
        assert!(validator.validate_element(&element, &contract).is_empty());
    }

    #[test]
    fn empty_allowed_children_permits_any_child() {
        let validator = ContractValidator::new();
        let contract = ComponentContract::new("Box");
        let element = first_element("<Box><Anything/></Box>");
        assert!(validator.validate_element(&element, &contract).is_empty());
    }

    #[test]
    fn unknown_attribute_is_flagged_when_props_closed() {
        let validator = ContractValidator::new();
        let contract = ComponentContract::new("Button").allowing_prop("label");
        let element = first_element("<Button labl=\"Save\"/>");
        let diagnostics = validator.validate_element(&element, &contract);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].message(),
            "unknown attribute 'labl' on component 'Button'"
        );
    }

    #[test]
    fn known_and_required_props_are_not_flagged() {
        let validator = ContractValidator::new();
        let contract = ComponentContract::new("Button")
            .requiring_prop("label")
            .allowing_prop("kind");
        let element = first_element("<Button label=\"Save\" kind=\"primary\"/>");
        assert!(validator.validate_element(&element, &contract).is_empty());
    }

    #[test]
    fn reserved_structural_attributes_are_permitted() {
        let validator = ContractValidator::new();
        let contract = ComponentContract::new("Row").allowing_prop("gap");
        let element = first_element("<Row key=\"1\" class=\"x\" id=\"y\" gap=\"4\"/>");
        assert!(validator.validate_element(&element, &contract).is_empty());
    }

    #[test]
    fn empty_allowed_props_permits_any_attribute() {
        let validator = ContractValidator::new();
        let contract = ComponentContract::new("Box");
        let element = first_element("<Box anything=\"1\" whatever=\"2\"/>");
        assert!(validator.validate_element(&element, &contract).is_empty());
    }

    #[test]
    fn unknown_event_is_flagged_when_events_closed() {
        let validator = ContractValidator::new();
        let contract = ComponentContract::new("Button").allowing_event("click");
        let element = first_element("<Button on:hover=go/>");
        let diagnostics = validator.validate_element(&element, &contract);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].message(),
            "unknown event 'hover' on component 'Button'"
        );
    }

    #[test]
    fn known_event_is_not_flagged() {
        let validator = ContractValidator::new();
        let contract = ComponentContract::new("Button").allowing_event("click");
        let element = first_element("<Button on:click=go/>");
        assert!(validator.validate_element(&element, &contract).is_empty());
    }

    #[test]
    fn empty_allowed_events_permits_any_event() {
        let validator = ContractValidator::new();
        let contract = ComponentContract::new("Button").allowing_prop("label");
        // Prop surface closed, event surface open: the event is not flagged.
        let element = first_element("<Button label=\"x\" on:anything=go/>");
        assert!(validator.validate_element(&element, &contract).is_empty());
    }

    #[test]
    fn accumulates_unknown_attribute_and_event_together() {
        let validator = ContractValidator::new();
        let contract = ComponentContract::new("Button")
            .allowing_prop("label")
            .allowing_event("click");
        let element = first_element("<Button bogus=\"1\" on:hover=go/>");
        let diagnostics = validator.validate_element(&element, &contract);
        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn extended_accessors_expose_props_and_events() {
        let contract = ComponentContract::new("Tabs")
            .allowing_prop("orientation")
            .allowing_event("change");
        assert_eq!(contract.allowed_props(), &["orientation".into()]);
        assert_eq!(contract.allowed_events(), &["change".into()]);
    }
}
