//! `SlotContract`: a named slot exported by a component (spec §2.11, §2.12).

use obscur_identity::InternedSymbol;

use crate::contract::{ChildContract, SlotCardinality};

/// The exported contract for one named slot of a component (spec §2.11,
/// §2.12).
///
/// A slot names a position where children may be placed. It pairs an interned
/// slot name with a [`SlotCardinality`] (how many children it accepts) and a
/// [`ChildContract`] (which children it accepts). Both the shape and the
/// membership rules are therefore fully typed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotContract {
    /// Interned name of the slot.
    pub name: InternedSymbol,
    /// How many children the slot may hold.
    pub cardinality: SlotCardinality,
    /// Which child components the slot accepts.
    pub allowed_children: ChildContract,
}

impl SlotContract {
    /// Creates a slot contract with the given name, cardinality and child rule.
    #[must_use]
    pub const fn new(
        name: InternedSymbol,
        cardinality: SlotCardinality,
        allowed_children: ChildContract,
    ) -> Self {
        Self {
            name,
            cardinality,
            allowed_children,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::ComponentId;

    #[test]
    fn constructs_with_all_fields() {
        let slot = SlotContract::new(
            InternedSymbol::from_raw(5),
            SlotCardinality::Many,
            ChildContract::Only(vec![ComponentId::from_raw(1)]),
        );
        assert_eq!(slot.name, InternedSymbol::from_raw(5));
        assert_eq!(slot.cardinality, SlotCardinality::Many);
        assert!(slot.allowed_children.allows(ComponentId::from_raw(1)));
    }
}
