//! Per-property inheritance classification for the cascade.
//!
//! [`InheritedProperty`] is a thin, stateless classifier: given a
//! [`PropertyId`], [`InheritedProperty::of`] answers whether that property
//! inherits from its parent by default (§4.14). Modelling this as a classifier
//! (rather than a stored `(PropertyId, Inheritance)` pair) keeps a single source
//! of truth — [`PropertyId::default_inheritance`] — and avoids the risk of a
//! stored pair drifting out of sync with the property's real default.

use crate::property::{Inheritance, PropertyId};

/// Classifier that reports a property's default inheritance behaviour.
///
/// It carries no state; it exists only as a named entry point the cascade calls
/// to decide whether a property inherits by default. `Copy + Eq`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InheritedProperty;

impl InheritedProperty {
    /// Returns the default [`Inheritance`] of the given property (§4.14).
    ///
    /// This delegates to [`PropertyId::default_inheritance`] so there is exactly
    /// one authoritative table of per-property inheritance.
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
