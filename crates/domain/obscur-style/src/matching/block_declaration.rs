#![doc = include_str!("block_declaration.doc/_module.md")]

use crate::cascade::ImportantFlag;
use crate::property::{PropertyId, PropertyValue};

#[doc = include_str!("block_declaration.doc/BlockDeclaration.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct BlockDeclaration {
    property: PropertyId,
    value: PropertyValue,
    important: ImportantFlag,
}

impl BlockDeclaration {
    #[doc = include_str!("block_declaration.doc/new.md")]
    #[must_use]
    pub const fn new(property: PropertyId, value: PropertyValue, important: ImportantFlag) -> Self {
        Self {
            property,
            value,
            important,
        }
    }

    #[doc = include_str!("block_declaration.doc/normal.md")]
    #[must_use]
    pub const fn normal(property: PropertyId, value: PropertyValue) -> Self {
        Self::new(property, value, ImportantFlag::Normal)
    }

    #[doc = include_str!("block_declaration.doc/important.md")]
    #[must_use]
    pub const fn important(property: PropertyId, value: PropertyValue) -> Self {
        Self::new(property, value, ImportantFlag::Important)
    }

    #[doc = include_str!("block_declaration.doc/property.md")]
    #[must_use]
    pub const fn property(&self) -> PropertyId {
        self.property
    }

    #[doc = include_str!("block_declaration.doc/value.md")]
    #[must_use]
    pub const fn value(&self) -> &PropertyValue {
        &self.value
    }

    #[doc = include_str!("block_declaration.doc/important_flag.md")]
    #[must_use]
    pub const fn important_flag(&self) -> ImportantFlag {
        self.important
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::property::StyleColor;

    #[test]
    fn normal_sets_normal_flag() {
        let decl =
            BlockDeclaration::normal(PropertyId::Color, PropertyValue::Color(StyleColor::BLACK));
        assert_eq!(decl.property(), PropertyId::Color);
        assert_eq!(*decl.value(), PropertyValue::Color(StyleColor::BLACK));
        assert_eq!(decl.important_flag(), ImportantFlag::Normal);
    }

    #[test]
    fn important_sets_important_flag() {
        let decl = BlockDeclaration::important(PropertyId::Opacity, PropertyValue::Number(0.5));
        assert_eq!(decl.important_flag(), ImportantFlag::Important);
    }

    #[test]
    fn new_round_trips_all_fields() {
        let decl = BlockDeclaration::new(
            PropertyId::Width,
            PropertyValue::Number(4.0),
            ImportantFlag::Important,
        );
        assert_eq!(decl.property(), PropertyId::Width);
        assert_eq!(*decl.value(), PropertyValue::Number(4.0));
        assert_eq!(decl.important_flag(), ImportantFlag::Important);
    }
}
