//! `ContractValidator`: checks an element against a component contract (spec
//! §2.13), plus the colocated [`ComponentContract`] that describes one.

use obscur_okui_syntax::ast::{Attribute, Element};
use obscur_okui_syntax::syntax::SyntaxKind;
use obscur_view::contract::PropTypeSurface;

use crate::validation::contract_rule::{
    AttributeUse, ElementUse, MemberArity, MemberUse, SlotUse, ViolationAnchor, check_contract,
    is_reserved_attribute,
};
use crate::validation::{SlotRule, ValidationDiagnostic};

/// The usage contract of a component: what an instance of it must and may carry
/// (spec §2.13).
///
/// A contract names the `component` it constrains and records:
///
/// - `required_props` — attribute names every instance must set;
/// - `allowed_props` — additional attribute names an instance *may* set. The
///   prop surface is **closed** once the contract declares *any* prop — required
///   or allowed: an attribute is then permitted only if it is in `allowed_props`
///   or `required_props`, is a [reserved](is_reserved_attribute) structural
///   attribute, or is an `on:*` event (events are governed by `allowed_events`);
///   every other attribute is an unknown-attribute violation. A contract that
///   declares **no** props at all leaves the surface *open* (any attribute is
///   accepted, mirroring an empty `allowed_children`);
/// - `allowed_events` — the `on:<event>` names an instance may bind; an
///   **empty** list means *no event restriction*. A non-empty list closes the
///   event surface: an `on:<event>` binding is permitted only if `<event>` is
///   listed;
/// - `allowed_children` — the child element tags permitted inside an instance;
///   an **empty** list means *any* child element is allowed (the contract does
///   not restrict children);
/// - `slots` — the named slots the component declares (§2.12). An **empty** list
///   closes the slot surface rather than opening it: a `slot="…"` on a child of
///   a component that declares none names nothing, and there is no permissive
///   reading of that to fall back to.
///
/// Contracts are built fluently, for example:
///
/// ```
/// use obscur_okui_compiler::validation::ComponentContract;
/// let contract = ComponentContract::new("Tabs")
///     .requiring_prop("selected")
///     .allowing_prop("orientation")
///     .allowing_event("change")
///     .allowing_child("Tab");
/// assert_eq!(contract.component(), "Tabs");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentContract {
    component: Box<str>,
    required_props: Vec<Box<str>>,
    allowed_props: Vec<Box<str>>,
    allowed_events: Vec<Box<str>>,
    allowed_children: Vec<Box<str>>,
    prop_types: Vec<(Box<str>, PropTypeSurface)>,
    slots: Vec<SlotRule>,
}

impl ComponentContract {
    /// Creates a contract for `component` with no required props and no prop,
    /// event or child restriction (any attribute, event and child allowed).
    #[must_use]
    pub fn new(component: impl Into<Box<str>>) -> Self {
        Self {
            component: component.into(),
            required_props: Vec::new(),
            allowed_props: Vec::new(),
            allowed_events: Vec::new(),
            allowed_children: Vec::new(),
            prop_types: Vec::new(),
            slots: Vec::new(),
        }
    }

    /// Records the declared type category of prop `name` (fluent; spec §2.8), so
    /// [`validate_element`](ContractValidator::validate_element) can reject a
    /// literal of the wrong kind. Bridged from a
    /// [`ComponentSurface`](obscur_view::contract::ComponentSurface); a prop with
    /// no recorded type, or one recorded as [`PropTypeSurface::Other`], is not
    /// type-checked.
    #[must_use]
    pub fn with_prop_type(mut self, name: impl Into<Box<str>>, ty: PropTypeSurface) -> Self {
        self.prop_types.push((name.into(), ty));
        self
    }

    /// Returns the declared type category of prop `name`, if one was recorded.
    #[must_use]
    pub fn prop_type(&self, name: &str) -> Option<PropTypeSurface> {
        self.prop_types
            .iter()
            .find(|(prop, _)| prop.as_ref() == name)
            .map(|(_, ty)| *ty)
    }

    /// Adds `name` to the contract's required props (fluent).
    #[must_use]
    pub fn requiring_prop(mut self, name: impl Into<Box<str>>) -> Self {
        self.required_props.push(name.into());
        self
    }

    /// Adds `name` to the contract's allowed (optional) props (fluent).
    ///
    /// The prop surface is closed once the contract declares any prop (required
    /// or allowed): only listed props, required props, reserved structural
    /// attributes and `on:*` events are then permitted; every other attribute is
    /// an unknown-attribute violation.
    #[must_use]
    pub fn allowing_prop(mut self, name: impl Into<Box<str>>) -> Self {
        self.allowed_props.push(name.into());
        self
    }

    /// Adds `name` to the contract's allowed `on:<event>` names (fluent).
    ///
    /// Adding any allowed event closes the event surface: once the list is
    /// non-empty, only listed events are permitted; every other `on:*` binding
    /// is an unknown-event violation.
    #[must_use]
    pub fn allowing_event(mut self, name: impl Into<Box<str>>) -> Self {
        self.allowed_events.push(name.into());
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

    /// Declares a named slot on the contract (fluent; spec §2.12).
    ///
    /// Declaring any slot is what *opens* the component's slot surface: until
    /// one is declared, a child's `slot="…"` has nothing to name.
    #[must_use]
    pub fn with_slot(mut self, slot: SlotRule) -> Self {
        self.slots.push(slot);
        self
    }

    /// Returns the constrained component's name.
    #[must_use]
    pub fn component(&self) -> &str {
        &self.component
    }

    /// Returns the declared named slots (spec §2.12); an empty slice means the
    /// component declares none and accepts no slot assignment at all.
    #[must_use]
    pub fn slots(&self) -> &[SlotRule] {
        &self.slots
    }

    /// Returns the declared slot called `name`, if any.
    #[must_use]
    pub fn slot(&self, name: &str) -> Option<&SlotRule> {
        self.slots.iter().find(|slot| slot.name() == name)
    }

    /// Returns the required prop names.
    #[must_use]
    pub fn required_props(&self) -> &[Box<str>] {
        &self.required_props
    }

    /// Returns the allowed (optional) prop names. Props are unrestricted only
    /// when the contract declares no props at all (this slice *and*
    /// [`required_props`](Self::required_props) both empty).
    #[must_use]
    pub fn allowed_props(&self) -> &[Box<str>] {
        &self.allowed_props
    }

    /// Returns the allowed `on:<event>` names; an empty slice means events are
    /// unrestricted.
    #[must_use]
    pub fn allowed_events(&self) -> &[Box<str>] {
        &self.allowed_events
    }

    /// Returns the allowed child tags; an empty slice means any child is
    /// allowed.
    #[must_use]
    pub fn allowed_children(&self) -> &[Box<str>] {
        &self.allowed_children
    }

    /// Returns whether `name` is an accepted attribute: a required prop, an
    /// allowed prop, or a reserved structural attribute. Only consulted when the
    /// prop surface is closed (`allowed_props` non-empty).
    pub(crate) fn permits_prop(&self, name: &str) -> bool {
        is_reserved_attribute(name)
            || self.required_props.iter().any(|p| p.as_ref() == name)
            || self.allowed_props.iter().any(|p| p.as_ref() == name)
    }

    /// Returns whether `event` is an accepted `on:<event>` name. Only consulted
    /// when the event surface is closed (`allowed_events` non-empty).
    pub(crate) fn permits_event(&self, event: &str) -> bool {
        self.allowed_events.iter().any(|e| e.as_ref() == event)
    }
}

/// Validates a component instance against its [`ComponentContract`] (spec
/// §2.12, §2.13).
///
/// This is the **file** front end's adapter onto the shared rule in
/// [`contract_rule`](crate::validation::contract_rule): it describes a parsed
/// [`Element`] as a plain [`ElementUse`], runs [`check_contract`], and resolves
/// each violation's [`ViolationAnchor`] to a source byte offset. The rule itself
/// lives elsewhere precisely so the `okui!` macro — which has no syntax tree —
/// can reach the same verdict from `syn` tokens.
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
    /// - **unknown attributes** — when the prop surface is closed (the contract
    ///   declares any required or allowed prop), every non-event attribute must
    ///   be a required/allowed prop or a reserved structural attribute
    ///   (`key`/`class`/`id`/`requires`); an unrecognised one yields
    ///   `"unknown attribute '<a>' on component '<c>'"`;
    /// - **unknown events** — when the event surface is closed
    ///   ([`allowed_events`](ComponentContract::allowed_events) non-empty),
    ///   every `on:<event>` binding's event must be listed; an unrecognised one
    ///   yields `"unknown event '<e>' on component '<c>'"`;
    /// - **allowed children** — if
    ///   [`allowed_children`](ComponentContract::allowed_children) is non-empty,
    ///   every child *element*'s tag must be listed; a disallowed tag yields
    ///   `"'<tag>' is not an allowed child of '<c>'"`;
    /// - **slots** (§2.12) — a child's `slot="…"` must name a slot the contract
    ///   declares and satisfy that slot's own child rule, and every declared
    ///   slot's cardinality must hold.
    ///
    /// Attribute-, event-, child- and slot-assignment diagnostics are anchored at
    /// the offending attribute's or child's byte offset; the required-prop and
    /// cardinality diagnostics are anchored at the element's own.
    ///
    /// **String-based method dispatch** (§2.3, §2.13) *is* modelled, at the one
    /// place the `.okui` AST spells it: an `on:<event>` whose value is a string
    /// literal — `on:pressed="handle_submit"` — is a named-method reference, which
    /// §2.3 ("Events are typed. String-based method dispatch is not used.")
    /// forbids. It is refused whether or not the event is declared, because the
    /// *form* is the fault: a handler is bound with `{…}`, never named by string.
    /// See [`check_string_dispatch`](Self::check_string_dispatch).
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

    /// Appends a diagnostic for every prop attribute given a literal of the wrong
    /// type — a `String` prop handed an integer, an `i64` prop handed a string
    /// (spec §2.8, "typed props … compile-time validation").
    ///
    /// One diagnostic per offending attribute, anchored at that attribute, so two
    /// wrong props on one element are two pointed errors, not one merged one. A
    /// prop the contract records no type for (or records as
    /// [`PropTypeSurface::Other`]), an attribute whose value is not a literal
    /// (`count={total}` — a binding whose type is unknown here), and every event
    /// attribute are all left alone.
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

    /// Appends a diagnostic for every `on:<event>` handler given as a string
    /// literal — string-based method dispatch, which §2.3/§2.13 forbid.
    ///
    /// A handler is bound as an expression (`on:pressed={handle_submit}`) or, on
    /// the file path, a bare reference (`on:pressed=handle_submit`); a *quoted*
    /// value names a method by string, and that is the form refused. A prop
    /// attribute's string value is untouched — only an event's is a dispatch
    /// reference.
    fn check_string_dispatch(
        element: &Element,
        contract: &ComponentContract,
        element_offset: u32,
        diagnostics: &mut Vec<ValidationDiagnostic>,
    ) {
        for attribute in element.attributes() {
            if attribute.is_event() && attribute.string_value().is_some() {
                let event = event_name(&attribute);
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

/// The owned strings one parsed [`Element`] contributes to an [`ElementUse`],
/// plus the byte offset each part is reported at.
///
/// It exists because the rule borrows `&str` while the AST hands out `String`:
/// the storage must outlive the borrow, and the offsets must travel with it so a
/// [`ViolationAnchor`] can be resolved back to a source position.
struct AstInstance {
    tag: String,
    attributes: Vec<(String, bool)>,
    attribute_offsets: Vec<u32>,
    members: Vec<(String, Option<Slot>, MemberArity)>,
    member_offsets: Vec<u32>,
}

/// A member's slot assignment, owned. The borrowed
/// [`SlotUse`](crate::validation::contract_rule::SlotUse) the rule takes cannot
/// hold a `String` the AST just minted.
enum Slot {
    /// `slot="header"`.
    Named(String),
    /// `slot={expr}`.
    Dynamic,
}

impl AstInstance {
    /// Describes `element`, defaulting any part with no recorded position to
    /// `offset` (the element's own).
    fn of(element: &Element, offset: u32) -> Self {
        let mut instance = Self {
            tag: element.tag_name().unwrap_or_default(),
            attributes: Vec::new(),
            attribute_offsets: Vec::new(),
            members: Vec::new(),
            member_offsets: Vec::new(),
        };
        for attribute in element.attributes() {
            let event = attribute.is_event();
            let name = if event {
                event_name(&attribute)
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

    /// Appends `element`'s members, looking through the control flow that
    /// splices into it (see the [`contract_rule`](crate::validation::contract_rule)
    /// module docs).
    ///
    /// `<Empty>` / `<Separator>` inside a `<For>` are looked through for the same
    /// reason: they are §2.11 slots whose *bodies* reach the loop's parent, not
    /// nodes of their own.
    fn collect_members(&mut self, element: &Element, arity: MemberArity, offset: u32) {
        for child in element.children() {
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

    /// Borrows this instance as the plain [`ElementUse`] the rule takes.
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
        }
    }

    /// Resolves an anchor to the byte offset its diagnostic is reported at.
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

/// Weakens `arity` to `inner`, keeping the weaker of the two.
///
/// Nesting only ever loosens the bound: a member already inside a `<For>` stays
/// unbounded when an `<If>` wraps it.
const fn degrade(arity: MemberArity, inner: MemberArity) -> MemberArity {
    match arity {
        MemberArity::Any => MemberArity::Any,
        MemberArity::Once | MemberArity::AtMostOnce => inner,
    }
}

/// The kind of literal an attribute was given, as far as it can be told at
/// compile time (spec §2.8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AttributeValueType {
    /// A quoted `attr="…"` value, or a braced string literal `attr={"…"}`.
    String,
    /// A braced integer literal `attr={5}` (or its negation).
    Int,
    /// A braced float literal `attr={1.5}` (or its negation).
    Float,
    /// A braced boolean literal `attr={true}`.
    Bool,
    /// A binding, a computed expression, or a bare word — nothing a literal type
    /// can be read from, so it is not type-checked.
    Unknown,
}

/// Classifies the literal an attribute carries, for the §2.8 type check.
///
/// A quoted value is a string; a braced value is parsed as a Rust expression and
/// classified only when it is a literal (or a negated numeric literal). Anything
/// else — a binding `{count}`, an arithmetic expression `{a + b}`, a bare word —
/// is [`Unknown`](AttributeValueType::Unknown) and left unchecked.
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

/// Classifies a braced expression that is a literal (or a negated numeric
/// literal), else [`Unknown`](AttributeValueType::Unknown).
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

/// Whether a value of type `found` satisfies a prop declared `expected`.
///
/// An [`Unknown`](AttributeValueType::Unknown) value and an
/// [`Other`](PropTypeSurface::Other) prop are always accepted — neither carries
/// enough information to prove a mismatch. An integer literal is accepted for a
/// float prop (`opacity={1}`), since it is a valid float; every other pairing
/// must match exactly.
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

/// The human phrase for a declared prop type, for a diagnostic.
fn describe_prop_type(ty: PropTypeSurface) -> &'static str {
    match ty {
        PropTypeSurface::String => "a string",
        PropTypeSurface::Int => "an integer",
        PropTypeSurface::Float => "a number",
        PropTypeSurface::Bool => "a boolean",
        PropTypeSurface::Other => "a value",
    }
}

/// The human phrase for the literal an attribute was given, for a diagnostic.
fn describe_value_type(ty: AttributeValueType) -> &'static str {
    match ty {
        AttributeValueType::String => "a string",
        AttributeValueType::Int => "an integer",
        AttributeValueType::Float => "a number",
        AttributeValueType::Bool => "a boolean",
        AttributeValueType::Unknown => "an expression",
    }
}

/// Returns an event attribute's local event name — the identifier after the
/// `:` in `on:click` (`"click"`), falling back to the attribute's own name when
/// no trailing identifier is present. Mirrors the lowering pass's convention.
fn event_name(attribute: &Attribute) -> String {
    let mut seen_colon = false;
    for token in attribute.syntax().tokens() {
        match token.kind() {
            SyntaxKind::Colon => seen_colon = true,
            SyntaxKind::Ident if seen_colon => return token.text().to_owned(),
            _ => {}
        }
    }
    attribute.name().unwrap_or_default()
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
