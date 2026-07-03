//! The per-property cascaded values produced by the cascade:
//! [`ResolvedDeclarations`] (§4.14).

use crate::property::{PropertyId, PropertyValue};
use std::collections::HashMap;
use std::collections::hash_map::Iter;

/// The winning **cascaded value** for each property that participated in the
/// cascade for a node (§4.14).
///
/// This is the output of [`crate::cascade::Cascade::resolve`]: for every
/// [`PropertyId`] that had at least one matching [`crate::cascade::Declaration`],
/// it holds the value of the declaration with the greatest
/// [`crate::cascade::CascadePriority`].
///
/// These are *cascaded* values, **not** computed values. Resolving CSS-wide
/// keywords (`inherit` / `initial` / `unset`), applying inheritance from the
/// parent, and substituting `var()` references all happen in a later sub-step
/// (computed style); none of that is applied here.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ResolvedDeclarations {
    values: HashMap<PropertyId, PropertyValue>,
}

impl ResolvedDeclarations {
    /// Creates an empty set of resolved declarations.
    #[must_use]
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    /// Wraps a pre-built map of cascaded values.
    #[must_use]
    pub fn from_map(values: HashMap<PropertyId, PropertyValue>) -> Self {
        Self { values }
    }

    /// Inserts or replaces the cascaded value for a property, returning the
    /// previous value if one was present.
    pub fn insert(&mut self, property: PropertyId, value: PropertyValue) -> Option<PropertyValue> {
        self.values.insert(property, value)
    }

    /// Returns the cascaded value for `property`, if one was resolved.
    #[must_use]
    pub fn get(&self, property: PropertyId) -> Option<&PropertyValue> {
        self.values.get(&property)
    }

    /// Iterates over the `(property, cascaded value)` pairs in unspecified
    /// order.
    #[must_use]
    pub fn iter(&self) -> Iter<'_, PropertyId, PropertyValue> {
        self.values.iter()
    }

    /// Returns the number of properties that were resolved.
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns `true` when no property was resolved.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_by_default() {
        let resolved = ResolvedDeclarations::new();
        assert!(resolved.is_empty());
        assert_eq!(resolved.len(), 0);
        assert_eq!(resolved.get(PropertyId::Color), None);
    }

    #[test]
    fn insert_and_get() {
        let mut resolved = ResolvedDeclarations::new();
        assert_eq!(
            resolved.insert(PropertyId::Color, PropertyValue::Number(1.0)),
            None
        );
        assert_eq!(
            resolved.get(PropertyId::Color),
            Some(&PropertyValue::Number(1.0))
        );
        assert_eq!(resolved.len(), 1);
        assert!(!resolved.is_empty());
    }

    #[test]
    fn insert_replaces_and_returns_previous() {
        let mut resolved = ResolvedDeclarations::new();
        resolved.insert(PropertyId::Opacity, PropertyValue::Number(0.5));
        let previous = resolved.insert(PropertyId::Opacity, PropertyValue::Number(1.0));
        assert_eq!(previous, Some(PropertyValue::Number(0.5)));
        assert_eq!(resolved.len(), 1);
    }

    #[test]
    fn iter_visits_all_entries() {
        let mut resolved = ResolvedDeclarations::new();
        resolved.insert(PropertyId::Width, PropertyValue::Number(1.0));
        resolved.insert(PropertyId::Height, PropertyValue::Number(2.0));
        assert_eq!(resolved.iter().count(), 2);
    }
}
