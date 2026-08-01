#![doc = include_str!("inherited_property.doc/_module.md")]

use crate::property::{Inheritance, PropertyId};

#[doc = include_str!("inherited_property.doc/InheritedProperty.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InheritedProperty;

impl InheritedProperty {
    #[doc = include_str!("inherited_property.doc/of.md")]
    #[must_use]
    pub const fn of(property: PropertyId) -> Inheritance {
        property.default_inheritance()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_inherited_property() {
        assert_eq!(
            InheritedProperty::of(PropertyId::Color),
            Inheritance::Inherited
        );
    }

    #[test]
    fn classifies_non_inherited_property() {
        assert_eq!(
            InheritedProperty::of(PropertyId::Opacity),
            Inheritance::NotInherited
        );
    }
}
