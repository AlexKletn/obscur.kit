//! `PropContract`: the typed contract of a single component prop (spec §2.13).

use obscur_identity::InternedSymbol;

/// The exported contract for one prop of a component (spec §2.13).
///
/// A prop is referenced entirely by interned handles: its own name and the
/// name of its declared value type. The value type itself is modelled and
/// interned elsewhere; this contract only names it, so no value/type system is
/// duplicated here. Whether the prop is required and whether the component
/// supplies a default are recorded as plain flags, because there is not yet a
/// value model that could carry the default itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropContract {
    /// Interned name of the prop as it appears on the component.
    pub name: InternedSymbol,
    /// Interned name of the prop's declared value type.
    ///
    /// This is a reference to a type named elsewhere, not an inline type
    /// definition.
    pub value_type: InternedSymbol,
    /// Whether a value for this prop must be supplied by the caller.
    pub required: bool,
    /// Whether the component declares a default value for this prop.
    ///
    /// Modelled as a flag rather than the default value itself, since the
    /// contract layer has no value representation yet.
    pub has_default: bool,
}

impl PropContract {
    /// Creates a prop contract for `name` with declared value type
    /// `value_type`.
    ///
    /// The prop starts out optional and without a default; use
    /// [`PropContract::required`] and [`PropContract::with_default`] to refine
    /// it in a builder style.
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
