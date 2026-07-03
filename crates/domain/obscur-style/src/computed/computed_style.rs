//! The fully-resolved computed style of a node: [`ComputedStyle`] (§4.14).
//!
//! A [`ComputedStyle`] is what [`crate::computed::StyleResolver`] produces for a
//! node once the cascade has run: every property this crate computes maps to a
//! concrete [`ComputedValue`], and the node's **effective custom properties**
//! (the ones a child inherits) are stored alongside so resolution can flow down
//! the tree.
//!
//! Custom properties are kept as raw [`PropertyValue`]s (already `var()`-
//! substituted and inherited, per §4.14) rather than [`ComputedValue`]s: they are
//! substitution inputs for descendants, not per-property computed values in their
//! own right.

use crate::computed::ComputedValue;
use crate::property::{PropertyId, PropertyValue};
use obscur_identity::InternedSymbol;
use std::collections::HashMap;
use std::collections::hash_map::Iter;

/// The fully-resolved computed style of a single node (§4.14).
///
/// `values` holds the concrete [`ComputedValue`] for each [`PropertyId`] the
/// resolver computed for the node. `custom_properties` holds the effective,
/// inheritable custom properties (parent's, overridden by the node's own) so a
/// child's resolution can start from them.
///
/// Because [`ComputedValue`] and [`PropertyValue`] may carry `f32` data, this
/// derives [`PartialEq`] but not [`Eq`]/[`Hash`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ComputedStyle {
    values: HashMap<PropertyId, ComputedValue>,
    custom_properties: HashMap<InternedSymbol, PropertyValue>,
}

impl ComputedStyle {
    /// Creates an empty computed style (no properties, no custom properties).
    ///
    /// This is the root's implicit parent: resolving against it makes every
    /// inherited property fall back to its initial value.
    #[must_use]
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            custom_properties: HashMap::new(),
        }
    }

    /// Returns the computed value for `property`, if the resolver produced one.
    #[must_use]
    pub fn get(&self, property: PropertyId) -> Option<&ComputedValue> {
        self.values.get(&property)
    }

    /// Inserts or replaces the computed value for a property, returning the
    /// previous value if one was present.
    ///
    /// Used by [`crate::computed::StyleResolver`] to build up a style.
    pub fn set(&mut self, property: PropertyId, value: ComputedValue) -> Option<ComputedValue> {
        self.values.insert(property, value)
    }

    /// Returns the effective custom property named `name`, if present.
    ///
    /// This is the value a descendant substitutes for `var(--name)`.
    #[must_use]
    pub fn custom_property(&self, name: InternedSymbol) -> Option<&PropertyValue> {
        self.custom_properties.get(&name)
    }

    /// Inserts or replaces an effective custom property, returning the previous
    /// value if one was present.
    ///
    /// Used by [`crate::computed::StyleResolver`] while assembling the node's
    /// inheritable custom properties.
    pub fn set_custom_property(
        &mut self,
        name: InternedSymbol,
        value: PropertyValue,
    ) -> Option<PropertyValue> {
        self.custom_properties.insert(name, value)
    }

    /// Iterates over the `(property, computed value)` pairs in unspecified order.
    #[must_use]
    pub fn iter(&self) -> Iter<'_, PropertyId, ComputedValue> {
        self.values.iter()
    }

    /// Iterates over the effective `(name, custom property value)` pairs in
    /// unspecified order.
    #[must_use]
    pub fn custom_properties(&self) -> Iter<'_, InternedSymbol, PropertyValue> {
        self.custom_properties.iter()
    }

    /// Returns the number of computed properties stored.
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns `true` when no property has a computed value.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Convenience getter for the computed `opacity` as a number, if resolved.
    ///
    /// Reads [`PropertyId::Opacity`] and returns the bare number it holds; it
    /// returns [`None`] when opacity was not computed or (defensively) is not a
    /// number.
    #[must_use]
    pub fn opacity(&self) -> Option<f32> {
        match self.get(PropertyId::Opacity)?.value() {
            PropertyValue::Number(value) => Some(*value),
            _ => None,
        }
    }
}

impl<'a> IntoIterator for &'a ComputedStyle {
    type Item = (&'a PropertyId, &'a ComputedValue);
    type IntoIter = Iter<'a, PropertyId, ComputedValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::property::StyleColor;

    #[test]
    fn empty_by_default() {
        let style = ComputedStyle::new();
        assert!(style.is_empty());
        assert_eq!(style.len(), 0);
        assert_eq!(style.get(PropertyId::Color), None);
        assert_eq!(style.opacity(), None);
        assert_eq!(ComputedStyle::default(), style);
    }

    #[test]
    fn set_and_get_property() {
        let mut style = ComputedStyle::new();
        assert_eq!(
            style.set(PropertyId::Color, ComputedValue::color(StyleColor::BLACK)),
            None
        );
        assert_eq!(
            style.get(PropertyId::Color),
            Some(&ComputedValue::color(StyleColor::BLACK))
        );
        assert_eq!(style.len(), 1);
    }

    #[test]
    fn opacity_getter_reads_number() {
        let mut style = ComputedStyle::new();
        style.set(PropertyId::Opacity, ComputedValue::number(0.25));
        assert_eq!(style.opacity(), Some(0.25));
    }

    #[test]
    fn custom_property_round_trip() {
        let mut style = ComputedStyle::new();
        let name = InternedSymbol::from_raw(9);
        style.set_custom_property(name, PropertyValue::Number(2.0));
        assert_eq!(style.custom_property(name), Some(&PropertyValue::Number(2.0)));
        assert_eq!(style.custom_properties().count(), 1);
    }
}
