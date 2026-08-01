#![doc = include_str!("view_property.doc/_module.md")]

use obscur_identity::InternedSymbol;

use super::ViewValue;

#[doc = include_str!("view_property.doc/ViewProperty.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct ViewProperty {
    #[doc = include_str!("view_property.doc/name.md")]
    name: InternedSymbol,
    #[doc = include_str!("view_property.doc/value.md")]
    value: ViewValue,
}

impl ViewProperty {
    #[doc = include_str!("view_property.doc/new.md")]
    #[must_use]
    pub fn new(name: InternedSymbol, value: ViewValue) -> Self {
        Self { name, value }
    }

    #[doc = include_str!("view_property.doc/name_2.md")]
    #[must_use]
    pub const fn name(&self) -> InternedSymbol {
        self.name
    }

    #[doc = include_str!("view_property.doc/value_2.md")]
    #[must_use]
    pub const fn value(&self) -> &ViewValue {
        &self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessors_return_fields() {
        let name = InternedSymbol::from_raw(42);
        let prop = ViewProperty::new(name, ViewValue::Int(9));
        assert_eq!(prop.name(), name);
        assert_eq!(prop.value(), &ViewValue::Int(9));
    }
}
