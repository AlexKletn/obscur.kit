#![doc = include_str!("slot_contract.doc/_module.md")]

use obscur_identity::InternedSymbol;

use crate::contract::{ChildContract, SlotCardinality};

#[doc = include_str!("slot_contract.doc/SlotContract.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotContract {
    #[doc = include_str!("slot_contract.doc/name.md")]
    pub name: InternedSymbol,
    #[doc = include_str!("slot_contract.doc/cardinality.md")]
    pub cardinality: SlotCardinality,
    #[doc = include_str!("slot_contract.doc/allowed_children.md")]
    pub allowed_children: ChildContract,
}

impl SlotContract {
    #[doc = include_str!("slot_contract.doc/new.md")]
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
