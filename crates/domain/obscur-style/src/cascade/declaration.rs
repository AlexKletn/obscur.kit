//! A single declaration participating in the cascade: [`Declaration`] (§4.14).

use crate::cascade::{CascadeOrigin, CascadePriority, ImportantFlag};
use crate::property::{PropertyId, PropertyValue};
use crate::selector::{SelectorId, SelectorSpecificity};

/// One declaration that applies to a node and competes in the cascade (§4.14).
///
/// A declaration pairs a target [`PropertyId`] with the [`PropertyValue`] to
/// assign, together with everything needed to rank it against the other
/// declarations that matched the same node: its `!important` flag, its
/// [`CascadeOrigin`], the [`SelectorSpecificity`] of the rule that produced it,
/// and its `source_order` position within that origin's stream.
///
/// The optional [`selector`](Declaration::selector) records which compiled
/// selector produced the declaration, purely as provenance for diagnostics; it
/// plays no part in ranking.
///
/// Because [`PropertyValue`] carries `f32` payloads it is not [`Eq`]; this type
/// therefore derives [`PartialEq`] but not [`Eq`].
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
    /// Creates a declaration from its cascade inputs.
    ///
    /// `source_order` is the declaration's position within its origin stream;
    /// larger means later, and later wins ties within a cascade band.
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

    /// Returns whether this declaration is `!important`.
    #[must_use]
    pub const fn important(&self) -> ImportantFlag {
        self.important
    }

    /// Returns the origin this declaration comes from.
    #[must_use]
    pub const fn origin(&self) -> CascadeOrigin {
        self.origin
    }

    /// Returns the specificity of the rule that produced this declaration.
    #[must_use]
    pub const fn specificity(&self) -> SelectorSpecificity {
        self.specificity
    }

    /// Returns the declaration's position within its origin stream.
    #[must_use]
    pub const fn source_order(&self) -> u32 {
        self.source_order
    }

    /// Returns the compiled selector that produced this declaration, if any
    /// (provenance only; not used for ranking).
    #[must_use]
    pub const fn selector(&self) -> Option<SelectorId> {
        self.selector
    }

    /// Returns this declaration's [`CascadePriority`] — its total-order sort key
    /// in the cascade.
    ///
    /// The declaration with the greatest priority wins for its property.
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
