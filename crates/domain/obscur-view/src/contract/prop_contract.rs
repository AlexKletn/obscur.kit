#![doc = include_str!("prop_contract.doc/_module.md")]

use obscur_identity::InternedSymbol;

#[doc = include_str!("prop_contract.doc/PropContract.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropContract {
    #[doc = include_str!("prop_contract.doc/name.md")]
    pub name: InternedSymbol,
    #[doc = include_str!("prop_contract.doc/value_type.md")]
    pub value_type: InternedSymbol,
    #[doc = include_str!("prop_contract.doc/required.md")]
    pub required: bool,
    #[doc = include_str!("prop_contract.doc/has_default.md")]
    pub has_default: bool,
}

impl PropContract {
    #[doc = include_str!("prop_contract.doc/new.md")]
    #[must_use]
    pub const fn new(name: InternedSymbol, value_type: InternedSymbol) -> Self {
        Self {
            name,
            value_type,
            required: false,
            has_default: false,
        }
    }

    #[doc = include_str!("prop_contract.doc/required_2.md")]
    #[must_use]
    pub const fn required(mut self) -> Self {
        self.required = true;
        self
    }

    #[doc = include_str!("prop_contract.doc/with_default.md")]
    #[must_use]
    pub const fn with_default(mut self) -> Self {
        self.has_default = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_prop_is_optional_and_defaultless() {
        let prop = PropContract::new(InternedSymbol::from_raw(1), InternedSymbol::from_raw(2));
        assert!(!prop.required);
        assert!(!prop.has_default);
    }

    #[test]
    fn builder_sets_required_and_default() {
        let prop = PropContract::new(InternedSymbol::from_raw(1), InternedSymbol::from_raw(2))
            .required()
            .with_default();
        assert!(prop.required);
        assert!(prop.has_default);
        assert_eq!(prop.name, InternedSymbol::from_raw(1));
        assert_eq!(prop.value_type, InternedSymbol::from_raw(2));
    }
}
