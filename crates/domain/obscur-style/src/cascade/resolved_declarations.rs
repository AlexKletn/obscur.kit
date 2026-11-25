//! The per-property cascaded values produced by the cascade:
//! [`ResolvedDeclarations`] (§4.14).
use obscur_identity::FastMap;

use crate::cascade::CascadePriority;
use crate::property::{PropertyId, PropertyValue};
use std::collections::HashMap;
use std::collections::hash_map::Iter;

#[doc = include_str!("../../docs/api/cascade/resolved_declarations.doc/ResolvedDeclarations.md")]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ResolvedDeclarations {
    values: FastMap<PropertyId, PropertyValue>,
    /// Whether any declaration here names a logical property.
    logical: bool,
    #[doc = include_str!("../../docs/api/cascade/resolved_declarations.doc/priorities.md")]
    priorities: FastMap<PropertyId, CascadePriority>,
}

impl ResolvedDeclarations {
    /// Creates an empty set of resolved declarations.
    #[must_use]
    pub fn new() -> Self {
        Self {
            values: HashMap::default(),
            logical: false,
            priorities: HashMap::default(),
        }
    }

    /// Wraps a pre-built map of cascaded values.
    #[must_use]
    pub fn from_map(values: FastMap<PropertyId, PropertyValue>) -> Self {
        // No priorities: a map handed in directly did not come from a cascade, so
        // there is no ordering to record. `priority_of` answers `None`, and the
        // logical/physical comparison falls back to preferring the physical — the
        // only honest answer when nothing states which was declared later.
        Self {
            logical: values.keys().any(|property| property.is_logical()),
            values,
            priorities: HashMap::default(),
        }
    }

    /// Inserts or replaces the cascaded value for a property, returning the
    /// previous value if one was present.
    /// Records `property`'s winning value together with the `priority` that won
    /// it, so a later step can compare it against a competing property's winner.
    pub fn insert_with_priority(
        &mut self,
        property: PropertyId,
        value: PropertyValue,
        priority: CascadePriority,
    ) -> Option<PropertyValue> {
        self.logical |= property.is_logical();
        self.priorities.insert(property, priority);
        self.values.insert(property, value)
    }

    /// The priority that won `property`, if it was recorded with one.
    #[must_use]
    pub fn priority_of(&self, property: PropertyId) -> Option<CascadePriority> {
        self.priorities.get(&property).copied()
    }

    #[doc = include_str!("../../docs/api/cascade/resolved_declarations.doc/insert.md")]
    pub fn insert(&mut self, property: PropertyId, value: PropertyValue) -> Option<PropertyValue> {
        self.logical |= property.is_logical();
        self.values.insert(property, value)
    }

    #[doc = include_str!("../../docs/api/cascade/resolved_declarations.doc/has_logical.md")]
    #[must_use]
    pub const fn has_logical(&self) -> bool {
        self.logical
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

impl<'a> IntoIterator for &'a ResolvedDeclarations {
    type Item = (&'a PropertyId, &'a PropertyValue);
    type IntoIter = Iter<'a, PropertyId, PropertyValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
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
