//! `ContractValidator`: checks an element against a component contract (spec
//! §2.13), plus the colocated [`ComponentContract`] that describes one.

use obscur_okui_syntax::ast::Element;

use crate::validation::ValidationDiagnostic;

/// The usage contract of a component: what an instance of it must and may carry
/// (spec §2.13).
///
/// A contract names the `component` it constrains and records:
///
/// - `required_props` — attribute names every instance must set;
/// - `allowed_children` — the child element tags permitted inside an instance;
///   an **empty** list means *any* child element is allowed (the contract does
///   not restrict children).
///
/// Contracts are built fluently, for example:
///
/// ```
/// use obscur_okui_compiler::validation::ComponentContract;
/// let contract = ComponentContract::new("Tabs")
///     .requiring_prop("selected")
///     .allowing_child("Tab");
/// assert_eq!(contract.component(), "Tabs");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentContract {
    component: Box<str>,
    required_props: Vec<Box<str>>,
    allowed_children: Vec<Box<str>>,
}

impl ComponentContract {
    /// Creates a contract for `component` with no required props and no child
    /// restriction (any child allowed).
    #[must_use]
    pub fn new(component: impl Into<Box<str>>) -> Self {
        Self {
            component: component.into(),
            required_props: Vec::new(),
            allowed_children: Vec::new(),
        }
    }

    /// Adds `name` to the contract's required props (fluent).
    #[must_use]
    pub fn requiring_prop(mut self, name: impl Into<Box<str>>) -> Self {
        self.required_props.push(name.into());
        self
    }

    /// Adds `tag` to the contract's allowed child tags (fluent).
    ///
    /// Adding any allowed child turns on child restriction: once the list is
    /// non-empty, only the listed tags are permitted.
    #[must_use]
    pub fn allowing_child(mut self, tag: impl Into<Box<str>>) -> Self {
        self.allowed_children.push(tag.into());
        self
    }

    /// Returns the constrained component's name.
    #[must_use]
    pub fn component(&self) -> &str {
        &self.component
    }

    /// Returns the required prop names.
    #[must_use]
    pub fn required_props(&self) -> &[Box<str>] {
        &self.required_props
    }

    /// Returns the allowed child tags; an empty slice means any child is
    /// allowed.
    #[must_use]
    pub fn allowed_children(&self) -> &[Box<str>] {
        &self.allowed_children
    }
}

/// Validates a component instance against its [`ComponentContract`] (spec
/// §2.13).
///
/// The validator checks the two structural obligations a contract expresses:
/// that every required prop is present as an attribute, and that every child
/// element is among the allowed tags when the contract restricts children.
#[derive(Debug, Clone, Default)]
pub struct ContractValidator;

impl ContractValidator {
    /// Creates a new contract validator.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Validates `element` against `contract`, returning one diagnostic per
    /// violation (spec §2.13).
    ///
    /// The checks are:
    ///
    /// - **required props** — every name in
    ///   [`required_props`](ComponentContract::required_props) must appear as an
    ///   attribute on `element`; a missing one yields
    ///   `"component '<c>' requires prop '<p>'"`;
    /// - **allowed children** — if
    ///   [`allowed_children`](ComponentContract::allowed_children) is non-empty,
    ///   every child *element*'s tag must be listed; a disallowed tag yields
    ///   `"'<tag>' is not an allowed child of '<c>'"`.
    ///
    /// Every diagnostic is anchored at the element's byte offset.
    #[must_use]
    pub fn validate_element(
        &self,
        element: &Element,
        contract: &ComponentContract,
    ) -> Vec<ValidationDiagnostic> {
        let mut diagnostics = Vec::new();
        let offset = element.syntax().offset().unwrap_or(0);
        let component = contract.component();

        for prop in contract.required_props() {
            if element.attribute(prop).is_none() {
                diagnostics.push(ValidationDiagnostic::new(
                    format!("component '{component}' requires prop '{prop}'"),
                    offset,
                ));
            }
        }

        if !contract.allowed_children().is_empty() {
            for child in element.children() {
                let Some(child_element) = Element::cast(child) else {
                    continue;
                };
                let Some(tag) = child_element.tag_name() else {
                    continue;
                };
                let child_offset = child_element.syntax().offset().unwrap_or(offset);
                if !contract
                    .allowed_children()
                    .iter()
                    .any(|allowed| allowed.as_ref() == tag)
                {
                    diagnostics.push(ValidationDiagnostic::new(
                        format!("'{tag}' is not an allowed child of '{component}'"),
                        child_offset,
                    ));
                }
            }
        }

        diagnostics
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
        assert_eq!(contract.allowed_children(), &["Tab".into()]);
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
}
