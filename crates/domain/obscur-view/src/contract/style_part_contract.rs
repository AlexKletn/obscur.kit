#![doc = include_str!("style_part_contract.doc/_module.md")]

use obscur_identity::InternedSymbol;

#[doc = include_str!("style_part_contract.doc/StylePartContract.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StylePartContract {
    #[doc = include_str!("style_part_contract.doc/name.md")]
    pub name: InternedSymbol,
    #[doc = include_str!("style_part_contract.doc/states.md")]
    pub states: Vec<InternedSymbol>,
}

impl StylePartContract {
    #[doc = include_str!("style_part_contract.doc/new.md")]
    #[must_use]
    pub fn new(name: InternedSymbol, states: Vec<InternedSymbol>) -> Self {
        Self { name, states }
    }

    #[doc = include_str!("style_part_contract.doc/stateless.md")]
    #[must_use]
    pub const fn stateless(name: InternedSymbol) -> Self {
        Self {
            name,
            states: Vec::new(),
        }
    }

    #[doc = include_str!("style_part_contract.doc/exposes.md")]
    #[must_use]
    pub fn exposes(&self, state: InternedSymbol) -> bool {
        self.states.contains(&state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stateless_part_exposes_nothing() {
        let part = StylePartContract::stateless(InternedSymbol::from_raw(9));
        assert!(part.states.is_empty());
        assert!(!part.exposes(InternedSymbol::from_raw(1)));
    }

    #[test]
    fn exposes_declared_state() {
        let hover = InternedSymbol::from_raw(1);
        let part = StylePartContract::new(InternedSymbol::from_raw(9), vec![hover]);
        assert!(part.exposes(hover));
        assert!(!part.exposes(InternedSymbol::from_raw(2)));
    }
}
