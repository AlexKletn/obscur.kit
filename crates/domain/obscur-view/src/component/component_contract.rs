#![doc = include_str!("component_contract.doc/_module.md")]

use obscur_diagnostics::DiagnosticSubsystem;
use obscur_identity::{ComponentId, InternedSymbol};

use crate::component::StyleScope;
use crate::contract::{
    CapabilityContract, ChildContract, EventContract, PropContract, SlotContract, StylePartContract,
};

#[doc = include_str!("component_contract.doc/ComponentContract.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentContract {
    #[doc = include_str!("component_contract.doc/id.md")]
    pub id: ComponentId,
    #[doc = include_str!("component_contract.doc/name.md")]
    pub name: Option<InternedSymbol>,
    #[doc = include_str!("component_contract.doc/props.md")]
    pub props: Vec<PropContract>,
    #[doc = include_str!("component_contract.doc/events.md")]
    pub events: Vec<EventContract>,
    #[doc = include_str!("component_contract.doc/slots.md")]
    pub slots: Vec<SlotContract>,
    #[doc = include_str!("component_contract.doc/allowed_children.md")]
    pub allowed_children: ChildContract,
    #[doc = include_str!("component_contract.doc/style_parts.md")]
    pub style_parts: Vec<StylePartContract>,
    #[doc = include_str!("component_contract.doc/style_scope.md")]
    pub style_scope: StyleScope,
    #[doc = include_str!("component_contract.doc/capabilities.md")]
    pub capabilities: Vec<CapabilityContract>,
    #[doc = include_str!("component_contract.doc/subsystem.md")]
    pub subsystem: DiagnosticSubsystem,
}

impl ComponentContract {
    #[doc = include_str!("component_contract.doc/new.md")]
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

    #[doc = include_str!("component_contract.doc/with_name.md")]
    #[must_use]
    pub fn with_name(mut self, name: InternedSymbol) -> Self {
        self.name = Some(name);
        self
    }

    #[doc = include_str!("component_contract.doc/with_prop.md")]
    #[must_use]
    pub fn with_prop(mut self, prop: PropContract) -> Self {
        self.props.push(prop);
        self
    }

    #[doc = include_str!("component_contract.doc/with_event.md")]
    #[must_use]
    pub fn with_event(mut self, event: EventContract) -> Self {
        self.events.push(event);
        self
    }

    #[doc = include_str!("component_contract.doc/with_slot.md")]
    #[must_use]
    pub fn with_slot(mut self, slot: SlotContract) -> Self {
        self.slots.push(slot);
        self
    }

    #[doc = include_str!("component_contract.doc/with_style_part.md")]
    #[must_use]
    pub fn with_style_part(mut self, style_part: StylePartContract) -> Self {
        self.style_parts.push(style_part);
        self
    }

    #[doc = include_str!("component_contract.doc/with_style_scope.md")]
    #[must_use]
    pub fn with_style_scope(mut self, style_scope: StyleScope) -> Self {
        self.style_scope = style_scope;
        self
    }

    #[doc = include_str!("component_contract.doc/with_capability.md")]
    #[must_use]
    pub fn with_capability(mut self, capability: CapabilityContract) -> Self {
        self.capabilities.push(capability);
        self
    }

    #[doc = include_str!("component_contract.doc/allowing_children.md")]
    #[must_use]
    pub fn allowing_children(mut self, allowed_children: ChildContract) -> Self {
        self.allowed_children = allowed_children;
        self
    }

    #[doc = include_str!("component_contract.doc/with_subsystem.md")]
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
