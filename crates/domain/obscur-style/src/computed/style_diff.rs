#![doc = include_str!("../../docs/api/computed/style_diff.doc/_module.md")]
use obscur_identity::FastSet;

use crate::computed::ComputedStyle;
use crate::property::PropertyId;
use std::collections::HashSet;

#[doc = include_str!("../../docs/api/computed/style_diff.doc/StyleDiff.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct StyleDiff {
    changed: Vec<PropertyId>,
    custom_properties_changed: bool,
}

impl StyleDiff {
    #[doc = include_str!("../../docs/api/computed/style_diff.doc/between.md")]
    #[must_use]
    pub fn between(old: &ComputedStyle, new: &ComputedStyle) -> Self {
        let mut properties_in_either: FastSet<PropertyId> = HashSet::default();
        for (property, _) in old {
            properties_in_either.insert(*property);
        }
        for (property, _) in new {
            properties_in_either.insert(*property);
        }

        let mut changed: Vec<PropertyId> = properties_in_either
            .into_iter()
            .filter(|&property| old.get(property) != new.get(property))
            .collect();
        // Stable, deterministic order: sort by canonical property name.
        changed.sort_by_key(|property| property.name());

        Self {
            changed,
            custom_properties_changed: Self::custom_changed(old, new),
        }
    }

    /// Returns the changed properties in sorted order.
    #[must_use]
    pub fn changed_properties(&self) -> &[PropertyId] {
        &self.changed
    }

    /// Returns `true` when any custom property was added, removed, or changed.
    #[must_use]
    pub fn custom_properties_changed(&self) -> bool {
        self.custom_properties_changed
    }

    /// Returns `true` when nothing changed — no property and no custom property.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.changed.is_empty() && !self.custom_properties_changed
    }

    /// Returns `true` when `property` is among the changed properties.
    #[must_use]
    pub fn contains(&self, property: PropertyId) -> bool {
        self.changed.contains(&property)
    }

    /// Compares the two styles' effective custom properties.
    fn custom_changed(old: &ComputedStyle, new: &ComputedStyle) -> bool {
        let mut names_in_either: HashSet<_> = HashSet::default();
        for (name, _) in old.custom_properties() {
            names_in_either.insert(*name);
        }
        for (name, _) in new.custom_properties() {
            names_in_either.insert(*name);
        }
        names_in_either
            .into_iter()
            .any(|name| old.custom_property(name) != new.custom_property(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::computed::ComputedValue;
    use crate::property::{PropertyValue, StyleColor};
    use obscur_identity::InternedSymbol;

    #[test]
    fn identical_styles_have_empty_diff() {
        let mut style = ComputedStyle::new();
        style.set(PropertyId::Opacity, ComputedValue::number(1.0));
        let diff = StyleDiff::between(&style, &style);
        assert!(diff.is_empty());
        assert!(diff.changed_properties().is_empty());
    }

    #[test]
    fn detects_added_removed_and_differing() {
        let mut old = ComputedStyle::new();
        old.set(PropertyId::Opacity, ComputedValue::number(1.0)); // differs
        old.set(PropertyId::Width, ComputedValue::number(1.0)); // removed

        let mut new = ComputedStyle::new();
        new.set(PropertyId::Opacity, ComputedValue::number(0.5)); // differs
        new.set(PropertyId::Color, ComputedValue::color(StyleColor::BLACK)); // added

        let diff = StyleDiff::between(&old, &new);
        assert!(!diff.is_empty());
        assert!(diff.contains(PropertyId::Opacity));
        assert!(diff.contains(PropertyId::Width));
        assert!(diff.contains(PropertyId::Color));
        // Sorted by name: background/color... here "color" < "opacity" < "width".
        assert_eq!(
            diff.changed_properties(),
            &[PropertyId::Color, PropertyId::Opacity, PropertyId::Width]
        );
    }

    #[test]
    fn detects_custom_property_change() {
        let name = InternedSymbol::from_raw(1);
        let mut old = ComputedStyle::new();
        old.set_custom_property(name, PropertyValue::Number(1.0));
        let mut new = ComputedStyle::new();
        new.set_custom_property(name, PropertyValue::Number(2.0));

        let diff = StyleDiff::between(&old, &new);
        assert!(diff.custom_properties_changed());
        assert!(!diff.is_empty());
        // No computed property changed, only a custom property.
        assert!(diff.changed_properties().is_empty());
    }
}
