#![doc = include_str!("style_diff.doc/_module.md")]

use crate::computed::ComputedStyle;
use crate::property::PropertyId;
use std::collections::HashSet;

#[doc = include_str!("style_diff.doc/StyleDiff.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct StyleDiff {
    changed: Vec<PropertyId>,
    custom_properties_changed: bool,
}

impl StyleDiff {
    #[doc = include_str!("style_diff.doc/between.md")]
    #[must_use]
    pub fn between(old: &ComputedStyle, new: &ComputedStyle) -> Self {
        // Union of property keys present in either style.
        let mut keys: HashSet<PropertyId> = HashSet::new();
        for (property, _) in old {
            keys.insert(*property);
        }
        for (property, _) in new {
            keys.insert(*property);
        }

        let mut changed: Vec<PropertyId> = keys
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

    #[doc = include_str!("style_diff.doc/changed_properties.md")]
    #[must_use]
    pub fn changed_properties(&self) -> &[PropertyId] {
        &self.changed
    }

    #[doc = include_str!("style_diff.doc/custom_properties_changed.md")]
    #[must_use]
    pub fn custom_properties_changed(&self) -> bool {
        self.custom_properties_changed
    }

    #[doc = include_str!("style_diff.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.changed.is_empty() && !self.custom_properties_changed
    }

    #[doc = include_str!("style_diff.doc/contains.md")]
    #[must_use]
    pub fn contains(&self, property: PropertyId) -> bool {
        self.changed.contains(&property)
    }

    #[doc = include_str!("style_diff.doc/custom_changed.md")]
    fn custom_changed(old: &ComputedStyle, new: &ComputedStyle) -> bool {
        // Union of custom-property names present in either style.
        let mut names: HashSet<_> = HashSet::new();
        for (name, _) in old.custom_properties() {
            names.insert(*name);
        }
        for (name, _) in new.custom_properties() {
            names.insert(*name);
        }
        names
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
