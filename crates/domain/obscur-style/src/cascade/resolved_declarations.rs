//! The per-property cascaded values produced by the cascade:
//! [`ResolvedDeclarations`] (§4.14).

use crate::cascade::CascadePriority;
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
    /// The priority the winning declaration for each property had.
    ///
    /// Kept because a *logical* property and its physical counterpart are two
    /// separate `PropertyId`s that cascade independently, yet CSS says they
    /// compete: `margin-left` and `margin-inline-start` on one element are decided
    /// by ordinary cascade order. By the time the winner-per-property is picked,
    /// that order is gone — so the priority is retained and the computed-value step
    /// compares the two winners (spec §4.13 logical properties, §4.14 cascade).
    ///
    /// Absent for a value inserted directly (e.g. by a test or a runtime override),
    /// which is why every read of it is optional.
    priorities: HashMap<PropertyId, CascadePriority>,
}

impl ResolvedDeclarations {
    /// Creates an empty set of resolved declarations.
    #[must_use]
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            priorities: HashMap::new(),
        }
    }

    /// Wraps a pre-built map of cascaded values.
    #[must_use]
    pub fn from_map(values: HashMap<PropertyId, PropertyValue>) -> Self {
        // No priorities: a map handed in directly did not come from a cascade, so
        // there is no ordering to record. `priority_of` answers `None`, and the
        // logical/physical comparison falls back to preferring the physical — the
        // only honest answer when nothing states which was declared later.
        Self {
            values,
            priorities: HashMap::new(),
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
        self.priorities.insert(property, priority);
        self.values.insert(property, value)
    }

    /// The priority that won `property`, if it was recorded with one.
    #[must_use]
    pub fn priority_of(&self, property: PropertyId) -> Option<CascadePriority> {
        self.priorities.get(&property).copied()
    }

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
