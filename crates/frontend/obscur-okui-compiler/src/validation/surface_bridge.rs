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
//! event [`allowing_event`](ComponentContract::allowing_event). Children cross
//! whole, case for case, into the [`ChildPolicy`] that mirrors this very enum:
//! [`Only`](ChildrenSurface::Only) to
//! [`Only`](crate::validation::ChildPolicy::Only),
//! [`Any`](ChildrenSurface::Any) to [`Any`](crate::validation::ChildPolicy::Any),
//! and [`None`](ChildrenSurface::None) to
//! [`no_children`](ComponentContract::no_children).
//!
//! That last case used to be **flattened into `Any`**: the string contract had a
//! `Vec` whose empty state already meant "any child", so a derived
//! `#[contract]` declaring a leaf component crossed this boundary and came out
//! unrestricted, and `<MyIcon>text</MyIcon>` compiled clean against a contract
//! that had said the opposite. The declaration was not lost in the derive or on
//! the domain surface — only here, on the last hop before the checker. With
//! [`ChildPolicy`] carrying all three states, the bridge is lossless.
//!
//! Each declared [`SlotSurface`] becomes one [`SlotRule`] (§2.12), carrying its
//! name, its [`SlotCardinality`](obscur_view::contract::SlotCardinality) — the
//! very same type, not a re-spelling — and its own child rule. A **slot's**
//! children are still `Only`-maps-and-nothing-else, because [`SlotRule`] keeps a
//! tag list of its own: a slot with no declared children is a slot that admits
//! any, and a slot declaring `None` is a slot nothing may be assigned to, which
//! the cardinality already expresses better.
//!
//! [`ChildPolicy`]: crate::validation::ChildPolicy

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
        for (name, ty) in surface.prop_types() {
            contract = contract.with_prop_type(*name, *ty);
        }
        match surface.children() {
            ChildrenSurface::Only(tags) => {
                for tag in tags {
                    contract = contract.allowing_child(*tag);
                }
            }
            ChildrenSurface::None => contract = contract.no_children(),
            // The contract's own default; matched rather than defaulted so a new
            // case on the domain surface has to be answered here.
            ChildrenSurface::Any => {}
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
        assert_eq!(contract.children().tags(), &["Tab".into()]);
    }

    #[test]
    fn a_declared_leaf_crosses_as_a_leaf() {
        // The lossy hop, pinned: `None` used to arrive as `Any`, so a derived leaf
        // component accepted children its own contract had refused.
        use crate::validation::ChildPolicy;

        let any = ComponentContract::from(&ComponentSurface::new("A").any_children());
        assert_eq!(any.children(), &ChildPolicy::Any);
        let none = ComponentContract::from(&ComponentSurface::new("B").no_children());
        assert_eq!(none.children(), &ChildPolicy::None);
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
