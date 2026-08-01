#![doc = include_str!("custom_property.doc/_module.md")]

use crate::property::PropertyValue;
use obscur_identity::InternedSymbol;

#[doc = include_str!("custom_property.doc/CustomProperty.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct CustomProperty {
    #[doc = include_str!("custom_property.doc/name.md")]
    name: InternedSymbol,
    #[doc = include_str!("custom_property.doc/value.md")]
    value: PropertyValue,
}

impl CustomProperty {
    #[doc = include_str!("custom_property.doc/new.md")]
    #[must_use]
    pub const fn new(name: InternedSymbol, value: PropertyValue) -> Self {
        Self { name, value }
    }

    #[doc = include_str!("custom_property.doc/name_2.md")]
    #[must_use]
    pub const fn name(&self) -> InternedSymbol {
        self.name
    }

    #[doc = include_str!("custom_property.doc/value_2.md")]
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
