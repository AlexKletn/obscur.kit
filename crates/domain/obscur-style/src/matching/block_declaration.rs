//! One authored declaration inside a rule block, before cascade context:
//! [`BlockDeclaration`].

use crate::cascade::ImportantFlag;
use crate::property::{PropertyId, PropertyValue};

/// A single authored declaration as written inside a style rule's block, e.g.
/// `color: red` or `width: 10px !important`.
///
/// This is the declaration *before* it acquires any cascade context: it carries
/// only the target [`PropertyId`], the [`PropertyValue`] to assign, and whether
/// it was written `!important`. Its [`CascadeOrigin`](crate::cascade::CascadeOrigin),
/// [`SelectorSpecificity`](crate::selector::SelectorSpecificity) and
/// `source_order` are not known here — they are supplied at match time by the
/// [`StyleMatcher`](crate::matching::StyleMatcher), which pairs the origin of
/// the containing stylesheet, the specificity of the matched selector, and the
/// declaration's document position to build a full
/// [`Declaration`](crate::cascade::Declaration).
#[derive(Debug, Clone, PartialEq)]
pub struct BlockDeclaration {
    property: PropertyId,
    value: PropertyValue,
    important: ImportantFlag,
}

impl BlockDeclaration {
    /// Creates an authored declaration from its property, value and
    /// `!important` flag.
    #[must_use]
    pub const fn new(property: PropertyId, value: PropertyValue, important: ImportantFlag) -> Self {
        Self {
            property,
            value,
            important,
        }
    }

    /// Creates a normal (non-`!important`) authored declaration.
    #[must_use]
    pub const fn normal(property: PropertyId, value: PropertyValue) -> Self {
        Self::new(property, value, ImportantFlag::Normal)
    }

    /// Creates an `!important` authored declaration.
    #[must_use]
    pub const fn important(property: PropertyId, value: PropertyValue) -> Self {
        Self::new(property, value, ImportantFlag::Important)
    }

    /// Returns the property this declaration assigns.
    #[must_use]
    pub const fn property(&self) -> PropertyId {
        self.property
    }

    /// Returns the value this declaration assigns.
    #[must_use]
    pub const fn value(&self) -> &PropertyValue {
        &self.value
    }

    /// Returns whether this declaration was written `!important`.
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
        let decl = BlockDeclaration::normal(PropertyId::Color, PropertyValue::Color(StyleColor::BLACK));
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
