#![doc = include_str!("../../docs/api/property/custom_property.doc/_module.md")]

use crate::property::PropertyValue;
use obscur_identity::InternedSymbol;

#[doc = include_str!("../../docs/api/property/custom_property.doc/CustomProperty.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct CustomProperty {
    /// The interned `--`-prefixed custom-property name.
    name: InternedSymbol,
    /// The declared value: the text it was written with, or the reference it defers
    /// to. See [`CustomProperty`].
    value: PropertyValue,
}

impl CustomProperty {
    /// Creates a custom property from its interned name and declared value.
    #[must_use]
    pub const fn new(name: InternedSymbol, value: PropertyValue) -> Self {
        Self { name, value }
    }

    /// Returns the interned `--`-prefixed name.
    #[must_use]
    pub const fn name(&self) -> InternedSymbol {
        self.name
    }

    /// Returns a reference to the declared value.
    #[must_use]
    pub const fn value(&self) -> &PropertyValue {
        &self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_name_and_value() {
        let name = InternedSymbol::from_raw(7);
        let prop = CustomProperty::new(name, PropertyValue::Number(1.5));
        assert_eq!(prop.name(), name);
        assert_eq!(prop.value(), &PropertyValue::Number(1.5));
    }

    #[test]
    fn value_may_be_unresolved_reference() {
        let name = InternedSymbol::from_raw(1);
        let referenced = InternedSymbol::from_raw(2);
        let prop = CustomProperty::new(name, PropertyValue::CustomRef(referenced));
        assert_eq!(prop.value(), &PropertyValue::CustomRef(referenced));
    }
}
