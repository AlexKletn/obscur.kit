//! `PropContract`: the typed contract of a single component prop (spec §2.13).

use obscur_identity::InternedSymbol;

#[doc = include_str!("../../docs/api/contract/prop_contract.doc/PropContract.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropContract {
    /// Interned name of the prop as it appears on the component.
    pub name: InternedSymbol,
    #[doc = include_str!("../../docs/api/contract/prop_contract.doc/value_type.md")]
    pub value_type: InternedSymbol,
    /// Whether a value for this prop must be supplied by the caller.
    pub required: bool,
    #[doc = include_str!("../../docs/api/contract/prop_contract.doc/has_default.md")]
    pub has_default: bool,
}

impl PropContract {
    #[doc = include_str!("../../docs/api/contract/prop_contract.doc/new.md")]
    #[must_use]
    pub const fn new(name: InternedSymbol, value_type: InternedSymbol) -> Self {
        Self {
            name,
            value_type,
            required: false,
            has_default: false,
        }
    }

    /// Marks the prop as required, returning the updated contract.
    #[must_use]
    pub const fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Records that the component declares a default for this prop, returning
    /// the updated contract.
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
