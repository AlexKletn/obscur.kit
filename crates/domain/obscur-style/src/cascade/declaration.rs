#![doc = include_str!("declaration.doc/_module.md")]

use crate::cascade::{CascadeOrigin, CascadePriority, ImportantFlag};
use crate::property::{PropertyId, PropertyValue};
use crate::selector::{SelectorId, SelectorSpecificity};

#[doc = include_str!("declaration.doc/Declaration.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    property: PropertyId,
    value: PropertyValue,
    important: ImportantFlag,
    origin: CascadeOrigin,
    specificity: SelectorSpecificity,
    source_order: u32,
    selector: Option<SelectorId>,
}

impl Declaration {
    #[doc = include_str!("declaration.doc/new.md")]
    #[must_use]
    pub const fn new(
        property: PropertyId,
        value: PropertyValue,
        important: ImportantFlag,
        origin: CascadeOrigin,
        specificity: SelectorSpecificity,
        source_order: u32,
        selector: Option<SelectorId>,
    ) -> Self {
        Self {
            property,
            value,
            important,
            origin,
            specificity,
            source_order,
            selector,
        }
    }

    #[doc = include_str!("declaration.doc/property.md")]
    #[must_use]
    pub const fn property(&self) -> PropertyId {
        self.property
    }

    #[doc = include_str!("declaration.doc/value.md")]
    #[must_use]
    pub const fn value(&self) -> &PropertyValue {
        &self.value
    }

    #[doc = include_str!("declaration.doc/important.md")]
    #[must_use]
    pub const fn important(&self) -> ImportantFlag {
        self.important
    }

    #[doc = include_str!("declaration.doc/origin.md")]
    #[must_use]
    pub const fn origin(&self) -> CascadeOrigin {
        self.origin
    }

    #[doc = include_str!("declaration.doc/specificity.md")]
    #[must_use]
    pub const fn specificity(&self) -> SelectorSpecificity {
        self.specificity
    }

    #[doc = include_str!("declaration.doc/source_order.md")]
    #[must_use]
    pub const fn source_order(&self) -> u32 {
        self.source_order
    }

    #[doc = include_str!("declaration.doc/selector.md")]
    #[must_use]
    pub const fn selector(&self) -> Option<SelectorId> {
        self.selector
    }

    #[doc = include_str!("declaration.doc/priority.md")]
    #[must_use]
    pub const fn priority(&self) -> CascadePriority {
        CascadePriority::new(
            self.origin,
            self.important,
            self.specificity,
            self.source_order,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decl(
        origin: CascadeOrigin,
        important: ImportantFlag,
        specificity: SelectorSpecificity,
        source_order: u32,
    ) -> Declaration {
        Declaration::new(
            PropertyId::Color,
            PropertyValue::Number(1.0),
            important,
            origin,
            specificity,
            source_order,
            None,
        )
    }

    #[test]
    fn accessors_round_trip() {
        let selector = SelectorId::from_raw(3);
        let d = Declaration::new(
            PropertyId::Width,
            PropertyValue::Number(4.0),
            ImportantFlag::Important,
            CascadeOrigin::GlobalApp,
            SelectorSpecificity::new(1, 0, 0),
            7,
            Some(selector),
        );
        assert_eq!(d.property(), PropertyId::Width);
        assert_eq!(*d.value(), PropertyValue::Number(4.0));
        assert_eq!(d.important(), ImportantFlag::Important);
        assert_eq!(d.origin(), CascadeOrigin::GlobalApp);
        assert_eq!(d.specificity(), SelectorSpecificity::new(1, 0, 0));
        assert_eq!(d.source_order(), 7);
        assert_eq!(d.selector(), Some(selector));
    }

    #[test]
    fn priority_reflects_fields() {
        let d = decl(
            CascadeOrigin::Inline,
            ImportantFlag::Normal,
            SelectorSpecificity::new(2, 0, 0),
            5,
        );
        let expected = CascadePriority::new(
            CascadeOrigin::Inline,
            ImportantFlag::Normal,
            SelectorSpecificity::new(2, 0, 0),
            5,
        );
        assert_eq!(d.priority(), expected);
    }

    #[test]
    fn priority_orders_declarations() {
        let normal_inline = decl(
            CascadeOrigin::Inline,
            ImportantFlag::Normal,
            SelectorSpecificity::new(0, 0, 0),
            100,
        );
        let important_theme = decl(
            CascadeOrigin::Theme,
            ImportantFlag::Important,
            SelectorSpecificity::new(0, 0, 0),
            0,
        );
        assert!(important_theme.priority() > normal_inline.priority());
    }
}
