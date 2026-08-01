#![doc = include_str!("contract_validator.doc/_module.md")]

// xtask:allow(multiple-entities) — the validator and the `ComponentContract`
// it validates against; see tools/xtask/src/rule/main_entity.rs.

use obscur_okui_syntax::ast::{Attribute, Element};
use obscur_okui_syntax::syntax::SyntaxKind;
use obscur_view::contract::PropTypeSurface;

use crate::lowering::event_local_name;
use crate::validation::contract_rule::{
    AttributeUse, ElementUse, MemberArity, MemberUse, SlotUse, ViolationAnchor, check_contract,
    is_reserved_attribute,
};
use crate::validation::{ChildPolicy, SlotRule, ValidationDiagnostic};

#[doc = include_str!("contract_validator.doc/ComponentContract.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentContract {
    component: Box<str>,
    required_props: Vec<Box<str>>,
    allowed_props: Vec<Box<str>>,
    allowed_events: Vec<Box<str>>,
    children: ChildPolicy,
    prop_types: Vec<(Box<str>, PropTypeSurface)>,
    slots: Vec<SlotRule>,
}

impl ComponentContract {
    #[doc = include_str!("contract_validator.doc/new.md")]
    #[must_use]
    pub fn new(component: impl Into<Box<str>>) -> Self {
        Self {
            component: component.into(),
            required_props: Vec::new(),
            allowed_props: Vec::new(),
            allowed_events: Vec::new(),
            children: ChildPolicy::Any,
            prop_types: Vec::new(),
            slots: Vec::new(),
        }
    }

    #[doc = include_str!("contract_validator.doc/with_prop_type.md")]
    #[must_use]
    pub fn with_prop_type(mut self, name: impl Into<Box<str>>, ty: PropTypeSurface) -> Self {
        self.prop_types.push((name.into(), ty));
        self
    }

    #[doc = include_str!("contract_validator.doc/prop_type.md")]
    #[must_use]
    pub fn prop_type(&self, name: &str) -> Option<PropTypeSurface> {
        self.prop_types
            .iter()
            .find(|(prop, _)| prop.as_ref() == name)
            .map(|(_, ty)| *ty)
    }

    #[doc = include_str!("contract_validator.doc/requiring_prop.md")]
    #[must_use]
    pub fn requiring_prop(mut self, name: impl Into<Box<str>>) -> Self {
        self.required_props.push(name.into());
        self
    }

    #[doc = include_str!("contract_validator.doc/allowing_prop.md")]
    #[must_use]
    pub fn allowing_prop(mut self, name: impl Into<Box<str>>) -> Self {
        self.allowed_props.push(name.into());
        self
    }

    #[doc = include_str!("contract_validator.doc/allowing_event.md")]
    #[must_use]
    pub fn allowing_event(mut self, name: impl Into<Box<str>>) -> Self {
        self.allowed_events.push(name.into());
        self
    }

    #[doc = include_str!("contract_validator.doc/allowing_child.md")]
    #[must_use]
    pub fn allowing_child(mut self, tag: impl Into<Box<str>>) -> Self {
        self.children.allow(tag);
        self
    }

    #[doc = include_str!("contract_validator.doc/no_children.md")]
    #[must_use]
    pub fn no_children(mut self) -> Self {
        self.children = ChildPolicy::None;
        self
    }

    #[doc = include_str!("contract_validator.doc/with_slot.md")]
    #[must_use]
    pub fn with_slot(mut self, slot: SlotRule) -> Self {
        self.slots.push(slot);
        self
    }

    #[doc = include_str!("contract_validator.doc/component.md")]
    #[must_use]
    pub fn component(&self) -> &str {
        &self.component
    }

    #[doc = include_str!("contract_validator.doc/slots.md")]
    #[must_use]
    pub fn slots(&self) -> &[SlotRule] {
        &self.slots
    }

    #[doc = include_str!("contract_validator.doc/slot.md")]
    #[must_use]
    pub fn slot(&self, name: &str) -> Option<&SlotRule> {
        self.slots.iter().find(|slot| slot.name() == name)
    }

    #[doc = include_str!("contract_validator.doc/required_props.md")]
    #[must_use]
    pub fn required_props(&self) -> &[Box<str>] {
        &self.required_props
    }

    #[doc = include_str!("contract_validator.doc/allowed_props.md")]
    #[must_use]
    pub fn allowed_props(&self) -> &[Box<str>] {
        &self.allowed_props
    }

    #[doc = include_str!("contract_validator.doc/allowed_events.md")]
    #[must_use]
    pub fn allowed_events(&self) -> &[Box<str>] {
        &self.allowed_events
    }

    #[doc = include_str!("contract_validator.doc/children.md")]
    #[must_use]
    pub const fn children(&self) -> &ChildPolicy {
        &self.children
    }

    #[doc = include_str!("contract_validator.doc/permits_prop.md")]
    pub(crate) fn permits_prop(&self, name: &str) -> bool {
        is_reserved_attribute(name)
            || self.required_props.iter().any(|p| p.as_ref() == name)
            || self.allowed_props.iter().any(|p| p.as_ref() == name)
    }

    #[doc = include_str!("contract_validator.doc/permits_event.md")]
    pub(crate) fn permits_event(&self, event: &str) -> bool {
        self.allowed_events.iter().any(|e| e.as_ref() == event)
    }
}

#[doc = include_str!("contract_validator.doc/ContractValidator.md")]
#[derive(Debug, Clone, Default)]
pub struct ContractValidator;

impl ContractValidator {
    #[doc = include_str!("contract_validator.doc/new_2.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[doc = include_str!("contract_validator.doc/validate_element.md")]
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

    #[doc = include_str!("contract_validator.doc/check_prop_types.md")]
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

    #[doc = include_str!("contract_validator.doc/check_string_dispatch.md")]
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

#[doc = include_str!("contract_validator.doc/AstInstance.md")]
struct AstInstance {
    tag: String,
    attributes: Vec<(String, bool)>,
    attribute_offsets: Vec<u32>,
    members: Vec<(String, Option<Slot>, MemberArity)>,
    member_offsets: Vec<u32>,
    #[doc = include_str!("contract_validator.doc/content.md")]
    content: bool,
}

#[doc = include_str!("contract_validator.doc/Slot.md")]
enum Slot {
    #[doc = include_str!("contract_validator.doc/Named.md")]
    Named(String),
    #[doc = include_str!("contract_validator.doc/Dynamic.md")]
    Dynamic,
}

impl AstInstance {
    #[doc = include_str!("contract_validator.doc/of.md")]
    fn of(element: &Element, offset: u32) -> Self {
        let mut instance = Self {
            tag: element.tag_name().unwrap_or_default(),
            attributes: Vec::new(),
            attribute_offsets: Vec::new(),
            members: Vec::new(),
            member_offsets: Vec::new(),
            content: false,
        };
        for attribute in element.attributes() {
            let event = attribute.is_event();
            let name = if event {
                event_local_name(&attribute)
            } else {
                attribute.name().unwrap_or_default()
            };
            instance.attributes.push((name, event));
            instance
                .attribute_offsets
                .push(attribute.syntax().offset().unwrap_or(offset));
        }
        instance.collect_members(element, MemberArity::Once, offset);
        instance
    }

    #[doc = include_str!("contract_validator.doc/collect_members.md")]
    fn collect_members(&mut self, element: &Element, arity: MemberArity, offset: u32) {
        for child in element.children() {
            if matches!(child.kind(), SyntaxKind::Expression)
                || (child.kind() == SyntaxKind::TextNode && !child.text().trim().is_empty())
            {
                self.content = true;
            }
            let Some(child) = Element::cast(child) else {
                continue;
            };
            let Some(tag) = child.tag_name() else {
                continue;
            };
            let child_offset = child.syntax().offset().unwrap_or(offset);
            if child.is_conditional() {
                self.collect_members(
                    &child,
                    degrade(arity, MemberArity::AtMostOnce),
                    child_offset,
                );
                continue;
            }
            if child.is_loop() {
                self.collect_members(&child, MemberArity::Any, child_offset);
                continue;
            }
            if arity == MemberArity::Any && matches!(tag.as_str(), "Empty" | "Separator") {
                self.collect_members(&child, MemberArity::Any, child_offset);
                continue;
            }
            let slot = child
                .attribute("slot")
                .map(|attribute| attribute.string_value().map_or(Slot::Dynamic, Slot::Named));
            self.members.push((tag, slot, arity));
            self.member_offsets.push(child_offset);
        }
    }

    #[doc = include_str!("contract_validator.doc/as_use.md")]
    fn as_use(&self) -> ElementUse<'_> {
        ElementUse {
            tag: &self.tag,
            attributes: self
                .attributes
                .iter()
                .map(|(name, event)| AttributeUse {
                    name,
                    event: *event,
                })
                .collect(),
            members: self
                .members
                .iter()
                .map(|(tag, slot, arity)| MemberUse {
                    tag,
                    slot: slot.as_ref().map(|slot| match slot {
                        Slot::Named(name) => SlotUse::Named(name),
                        Slot::Dynamic => SlotUse::Dynamic,
                    }),
                    arity: *arity,
                })
                .collect(),
            content: self.content,
        }
    }

    #[doc = include_str!("contract_validator.doc/offset_of.md")]
    fn offset_of(&self, anchor: ViolationAnchor, element: u32) -> u32 {
        match anchor {
            ViolationAnchor::Element => element,
            ViolationAnchor::Attribute(index) => self
                .attribute_offsets
                .get(index)
                .copied()
                .unwrap_or(element),
            ViolationAnchor::Member(index) => {
                self.member_offsets.get(index).copied().unwrap_or(element)
            }
        }
    }
}

#[doc = include_str!("contract_validator.doc/degrade.md")]
const fn degrade(arity: MemberArity, inner: MemberArity) -> MemberArity {
    match arity {
        MemberArity::Any => MemberArity::Any,
        MemberArity::Once | MemberArity::AtMostOnce => inner,
    }
}

#[doc = include_str!("contract_validator.doc/AttributeValueType.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AttributeValueType {
    #[doc = include_str!("contract_validator.doc/String.md")]
    String,
    #[doc = include_str!("contract_validator.doc/Int.md")]
    Int,
    #[doc = include_str!("contract_validator.doc/Float.md")]
    Float,
    #[doc = include_str!("contract_validator.doc/Bool.md")]
    Bool,
    #[doc = include_str!("contract_validator.doc/Unknown.md")]
    Unknown,
}

#[doc = include_str!("contract_validator.doc/attribute_value_type.md")]
fn attribute_value_type(attribute: &Attribute) -> AttributeValueType {
    if let Some(expression) = attribute.value_expression() {
        let Ok(expr) = syn::parse_str::<syn::Expr>(&expression.text()) else {
            return AttributeValueType::Unknown;
        };
        return literal_expr_type(&expr);
    }
    if attribute.string_value().is_some() {
        return AttributeValueType::String;
    }
    AttributeValueType::Unknown
}

#[doc = include_str!("contract_validator.doc/literal_expr_type.md")]
fn literal_expr_type(expr: &syn::Expr) -> AttributeValueType {
    match expr {
        syn::Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Str(_) => AttributeValueType::String,
            syn::Lit::Int(_) => AttributeValueType::Int,
            syn::Lit::Float(_) => AttributeValueType::Float,
            syn::Lit::Bool(_) => AttributeValueType::Bool,
            _ => AttributeValueType::Unknown,
        },
        // A negative numeric literal is `-` applied to the literal.
        syn::Expr::Unary(unary) if matches!(unary.op, syn::UnOp::Neg(_)) => {
            match literal_expr_type(&unary.expr) {
                ty @ (AttributeValueType::Int | AttributeValueType::Float) => ty,
                _ => AttributeValueType::Unknown,
            }
        }
        _ => AttributeValueType::Unknown,
    }
}

#[doc = include_str!("contract_validator.doc/value_type_matches.md")]
fn value_type_matches(expected: PropTypeSurface, found: AttributeValueType) -> bool {
    if found == AttributeValueType::Unknown || expected == PropTypeSurface::Other {
        return true;
    }
    match expected {
        PropTypeSurface::String => found == AttributeValueType::String,
        PropTypeSurface::Int => found == AttributeValueType::Int,
        PropTypeSurface::Float => {
            matches!(found, AttributeValueType::Float | AttributeValueType::Int)
        }
        PropTypeSurface::Bool => found == AttributeValueType::Bool,
        PropTypeSurface::Other => true,
    }
}

#[doc = include_str!("contract_validator.doc/describe_prop_type.md")]
fn describe_prop_type(ty: PropTypeSurface) -> &'static str {
    match ty {
        PropTypeSurface::String => "a string",
        PropTypeSurface::Int => "an integer",
        PropTypeSurface::Float => "a number",
        PropTypeSurface::Bool => "a boolean",
        PropTypeSurface::Other => "a value",
    }
}

#[doc = include_str!("contract_validator.doc/describe_value_type.md")]
fn describe_value_type(ty: AttributeValueType) -> &'static str {
    match ty {
        AttributeValueType::String => "a string",
        AttributeValueType::Int => "an integer",
        AttributeValueType::Float => "a number",
        AttributeValueType::Bool => "a boolean",
        AttributeValueType::Unknown => "an expression",
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
