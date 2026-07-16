//! Bridges a domain [`ComponentSurface`] into the compiler's string
//! [`ComponentContract`] (spec §2.13).
//!
//! `#[derive(Contract)]` emits, alongside the symbol-keyed typed
//! `obscur_view::component::ComponentContract` (the runtime view), a string
//! [`ComponentSurface`] (the tooling view) carrying the very same prop/event/
//! child names as plain strings. This module turns that surface into the
//! string [`ComponentContract`] the [`ContractValidator`](super::ContractValidator)
//! consults, so a user component's usage is enforced exactly as a built-in's
//! is.
//!
//! The mapping is one-to-one with the string contract's semantics: each
//! required prop becomes [`requiring_prop`](ComponentContract::requiring_prop),
//! each allowed prop [`allowing_prop`](ComponentContract::allowing_prop), each
//! event [`allowing_event`](ComponentContract::allowing_event). For children,
//! [`ChildrenSurface::Only`] maps to one
//! [`allowing_child`](ComponentContract::allowing_child) per tag; both
//! [`ChildrenSurface::Any`] and [`ChildrenSurface::None`] leave children
//! unrestricted, because the string contract cannot express a leaf position (an
//! empty allowed-children list already means "any child").
//!
//! Each declared [`SlotSurface`] becomes one [`SlotRule`] (§2.12), carrying its
//! name, its [`SlotCardinality`](obscur_view::contract::SlotCardinality) — the
//! very same type, not a re-spelling — and its own child rule under the same
//! `Only`-maps, `Any`/`None`-do-not mapping.

use obscur_view::contract::{ChildrenSurface, ComponentSurface, SlotSurface};

use crate::validation::{ComponentContract, SlotRule};

impl From<&ComponentSurface> for ComponentContract {
    fn from(surface: &ComponentSurface) -> Self {
        let mut contract = ComponentContract::new(surface.component());
        for prop in surface.required_props() {
            contract = contract.requiring_prop(*prop);
        }
        for prop in surface.allowed_props() {
            contract = contract.allowing_prop(*prop);
        }
        for event in surface.allowed_events() {
            contract = contract.allowing_event(*event);
        }
        if let ChildrenSurface::Only(tags) = surface.children() {
            for tag in tags {
                contract = contract.allowing_child(*tag);
            }
        }
        for slot in surface.slots() {
            contract = contract.with_slot(slot.into());
        }
        contract
    }
}

impl From<&SlotSurface> for SlotRule {
    fn from(surface: &SlotSurface) -> Self {
        let mut rule = Self::new(surface.name(), surface.cardinality());
        if let ChildrenSurface::Only(tags) = surface.children() {
            for tag in tags {
                rule = rule.allowing_child(*tag);
            }
        }
        rule
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_props_events_and_children() {
        let surface = ComponentSurface::new("Tabs")
            .requiring_prop("selected")
            .allowing_prop("orientation")
            .allowing_event("change")
            .allowing_child("Tab");
        let contract = ComponentContract::from(&surface);
        assert_eq!(contract.component(), "Tabs");
        assert_eq!(contract.required_props(), &["selected".into()]);
        assert_eq!(contract.allowed_props(), &["orientation".into()]);
        assert_eq!(contract.allowed_events(), &["change".into()]);
        assert_eq!(contract.allowed_children(), &["Tab".into()]);
    }

    #[test]
    fn any_and_none_children_leave_children_unrestricted() {
        let any = ComponentContract::from(&ComponentSurface::new("A").any_children());
        assert!(any.allowed_children().is_empty());
        let none = ComponentContract::from(&ComponentSurface::new("B").no_children());
        assert!(none.allowed_children().is_empty());
    }

    #[test]
    fn declared_slots_cross_with_their_cardinality_and_child_rule() {
        use obscur_view::contract::SlotCardinality;

        let surface = ComponentSurface::new("Card")
            .with_slot(SlotSurface::new("header", SlotCardinality::ExactlyOne))
            .with_slot(SlotSurface::new("actions", SlotCardinality::Many).allowing_child("Button"));
        let contract = ComponentContract::from(&surface);
        assert_eq!(contract.slots().len(), 2);
        let header = contract.slot("header").expect("the header slot crossed");
        assert_eq!(header.cardinality(), SlotCardinality::ExactlyOne);
        assert!(header.allowed_children().is_empty());
        let actions = contract.slot("actions").expect("the actions slot crossed");
        assert_eq!(actions.allowed_children(), &["Button".into()]);
    }

    #[test]
    fn a_surface_with_no_slots_declares_none() {
        let contract = ComponentContract::from(&ComponentSurface::new("Box"));
        assert!(contract.slots().is_empty());
    }

    #[test]
    fn open_surface_stays_open() {
        // No props declared -> no prop restriction (mirrors the string contract).
        let contract = ComponentContract::from(&ComponentSurface::new("Box"));
        assert!(contract.required_props().is_empty());
        assert!(contract.allowed_props().is_empty());
    }
}
