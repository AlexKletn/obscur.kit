#![doc = include_str!("resolved_declarations.doc/_module.md")]
use obscur_identity::FastMap;

use crate::cascade::CascadePriority;
use crate::property::{PropertyId, PropertyValue};
use std::collections::HashMap;
use std::collections::hash_map::Iter;

#[doc = include_str!("resolved_declarations.doc/ResolvedDeclarations.md")]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ResolvedDeclarations {
    values: FastMap<PropertyId, PropertyValue>,
    #[doc = include_str!("resolved_declarations.doc/logical.md")]
    logical: bool,
    #[doc = include_str!("resolved_declarations.doc/priorities.md")]
    priorities: FastMap<PropertyId, CascadePriority>,
}

impl ResolvedDeclarations {
    #[doc = include_str!("resolved_declarations.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            values: HashMap::default(),
            logical: false,
            priorities: HashMap::default(),
        }
    }

    #[doc = include_str!("resolved_declarations.doc/from_map.md")]
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

    #[doc = include_str!("resolved_declarations.doc/insert_with_priority.md")]
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

    #[doc = include_str!("resolved_declarations.doc/priority_of.md")]
    #[must_use]
    pub fn priority_of(&self, property: PropertyId) -> Option<CascadePriority> {
        self.priorities.get(&property).copied()
    }

    #[doc = include_str!("resolved_declarations.doc/insert.md")]
    pub fn insert(&mut self, property: PropertyId, value: PropertyValue) -> Option<PropertyValue> {
        self.logical |= property.is_logical();
        self.values.insert(property, value)
    }

    #[doc = include_str!("resolved_declarations.doc/has_logical.md")]
    #[must_use]
    pub const fn has_logical(&self) -> bool {
        self.logical
    }

    #[doc = include_str!("resolved_declarations.doc/get.md")]
    #[must_use]
    pub fn get(&self, property: PropertyId) -> Option<&PropertyValue> {
        self.values.get(&property)
    }

    #[doc = include_str!("resolved_declarations.doc/iter.md")]
    #[must_use]
    pub fn iter(&self) -> Iter<'_, PropertyId, PropertyValue> {
        self.values.iter()
    }

    #[doc = include_str!("resolved_declarations.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    #[doc = include_str!("resolved_declarations.doc/is_empty.md")]
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
