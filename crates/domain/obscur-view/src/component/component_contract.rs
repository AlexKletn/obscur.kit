//! Typed component contract (spec §2.13).

use obscur_diagnostics::DiagnosticSubsystem;
use obscur_identity::{ComponentId, InternedSymbol};

use crate::component::StyleScope;
use crate::contract::{
    CapabilityContract, ChildContract, EventContract, PropContract, SlotContract, StylePartContract,
};

/// The typed, exported contract of a component (spec §2.13).
///
/// A contract is the component's public surface, aggregating the sub-contracts
/// from the [`contract`](crate::contract) module: the props it accepts (and
/// which carry defaults), the events it exposes, the slots and allowed
/// children, its style parts, and the capabilities it depends on. It also
/// carries diagnostics metadata: the [`DiagnosticSubsystem`] failures are
/// attributed to (defaulting to [`DiagnosticSubsystem::View`]) and an optional
/// stable component name.
///
/// Everything here is plain, comparable data — it derives `Eq`, so no field may
/// embed a float (the sub-contract types are all `Eq`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentContract {
    /// Stable component identity.
    pub id: ComponentId,
    /// Optional stable, interned name for the component (for diagnostics and
    /// `DevTools`).
    pub name: Option<InternedSymbol>,
    /// The props this component accepts, including default/required metadata.
    pub props: Vec<PropContract>,
    /// The events this component exposes.
    pub events: Vec<EventContract>,
    /// The named slots this component exports.
    pub slots: Vec<SlotContract>,
    /// Which direct children the component accepts.
    pub allowed_children: ChildContract,
    /// The style parts (and their states) this component exports.
    pub style_parts: Vec<StylePartContract>,
    /// Whether the document's ordinary selectors reach this component's
    /// rendered output (§4.15). [`StyleScope::Scoped`] by default — the spec's
    /// "component stylesheets are scoped by default" — and
    /// [`StyleScope::Open`] for its "explicitly open/global scopes are
    /// possible".
    pub style_scope: StyleScope,
    /// The capabilities this component depends on.
    pub capabilities: Vec<CapabilityContract>,
    /// The subsystem failures in this component are attributed to.
    pub subsystem: DiagnosticSubsystem,
}

impl ComponentContract {
    /// Creates an empty-but-valid contract for `id`.
    ///
    /// It exposes no props/events/slots/style-parts, requires no capabilities,
    /// accepts no children ([`ChildContract::None`]), has no name, and is
    /// attributed to [`DiagnosticSubsystem::View`]. Callers refine it with the
    /// `with_*`/`allowing_children` builders.
    #[must_use]
    pub const fn new(id: ComponentId) -> Self {
        Self {
            id,
            name: None,
            props: Vec::new(),
            events: Vec::new(),
            slots: Vec::new(),
            allowed_children: ChildContract::None,
            style_parts: Vec::new(),
            style_scope: StyleScope::Scoped,
            capabilities: Vec::new(),
            subsystem: DiagnosticSubsystem::View,
        }
    }

    /// Sets the component's stable interned name.
    #[must_use]
    pub fn with_name(mut self, name: InternedSymbol) -> Self {
        self.name = Some(name);
        self
    }

    /// Adds a prop to the contract.
    #[must_use]
    pub fn with_prop(mut self, prop: PropContract) -> Self {
        self.props.push(prop);
        self
    }

    /// Adds an event to the contract.
    #[must_use]
    pub fn with_event(mut self, event: EventContract) -> Self {
        self.events.push(event);
        self
    }

    /// Adds a slot to the contract.
    #[must_use]
    pub fn with_slot(mut self, slot: SlotContract) -> Self {
        self.slots.push(slot);
        self
    }

    /// Adds a style part to the contract.
    #[must_use]
    pub fn with_style_part(mut self, style_part: StylePartContract) -> Self {
        self.style_parts.push(style_part);
        self
    }

    /// Declares whether the document's ordinary selectors reach this
    /// component's rendered output (§4.15).
    ///
    /// Unstated it is [`StyleScope::Scoped`], so an existing contract's
    /// encapsulation is unchanged; a component that wants to be styled by the
    /// application's sheets says so here.
    #[must_use]
    pub fn with_style_scope(mut self, style_scope: StyleScope) -> Self {
        self.style_scope = style_scope;
        self
    }

    /// Adds a capability dependency to the contract.
    #[must_use]
    pub fn with_capability(mut self, capability: CapabilityContract) -> Self {
        self.capabilities.push(capability);
        self
    }

    /// Sets the rule for which direct children the component accepts.
    #[must_use]
    pub fn allowing_children(mut self, allowed_children: ChildContract) -> Self {
        self.allowed_children = allowed_children;
        self
    }

    /// Sets the subsystem failures in this component are attributed to.
    #[must_use]
    pub fn with_subsystem(mut self, subsystem: DiagnosticSubsystem) -> Self {
        self.subsystem = subsystem;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::ComponentContract;
    use crate::contract::{
        CapabilityContract, ChildContract, EventContract, PropContract, SlotCardinality,
        SlotContract, StylePartContract,
    };
    use obscur_capability::CapabilityId;
    use obscur_diagnostics::DiagnosticSubsystem;
    use obscur_identity::{ComponentId, InternedSymbol};

    #[test]
    fn new_is_empty_but_valid() {
        let contract = ComponentContract::new(ComponentId::from_raw(1));
        assert_eq!(contract.id, ComponentId::from_raw(1));
        assert!(contract.name.is_none());
        assert!(contract.props.is_empty());
        assert!(contract.events.is_empty());
        assert!(contract.slots.is_empty());
        assert!(contract.style_parts.is_empty());
        assert!(contract.capabilities.is_empty());
        assert_eq!(contract.allowed_children, ChildContract::None);
        assert_eq!(contract.subsystem, DiagnosticSubsystem::View);
    }

    #[test]
    fn builders_accumulate_every_facet() {
        let contract = ComponentContract::new(ComponentId::from_raw(2))
            .with_name(InternedSymbol::from_raw(9))
            .with_prop(PropContract::new(
                InternedSymbol::from_raw(1),
                InternedSymbol::from_raw(2),
            ))
            .with_event(EventContract::new(
                InternedSymbol::from_raw(3),
                InternedSymbol::from_raw(4),
            ))
            .with_slot(SlotContract::new(
                InternedSymbol::from_raw(5),
                SlotCardinality::Many,
                ChildContract::Any,
            ))
            .with_style_part(StylePartContract::stateless(InternedSymbol::from_raw(6)))
            .with_capability(CapabilityContract::required(CapabilityId::new(
                "clipboard.read",
            )))
            .allowing_children(ChildContract::Any)
            .with_subsystem(DiagnosticSubsystem::Tree);

        assert_eq!(contract.name, Some(InternedSymbol::from_raw(9)));
        assert_eq!(contract.props.len(), 1);
        assert_eq!(contract.events.len(), 1);
        assert_eq!(contract.slots.len(), 1);
        assert_eq!(contract.style_parts.len(), 1);
        assert_eq!(contract.capabilities.len(), 1);
        assert_eq!(contract.allowed_children, ChildContract::Any);
        assert_eq!(contract.subsystem, DiagnosticSubsystem::Tree);
    }
}
