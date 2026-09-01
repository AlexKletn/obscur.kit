#![doc = include_str!("computed_style.doc/_module.md")]

use crate::computed::ComputedValue;
use crate::property::{PropertyId, PropertyValue};
use std::rc::Rc;

use obscur_identity::{FastBuildHasher, FastMap, InternedSymbol};
use std::collections::hash_map::Iter;

#[doc = include_str!("computed_style.doc/ComputedStyle.md")]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ComputedStyle {
    values: FastMap<PropertyId, ComputedValue>,
    custom_properties: Rc<FastMap<InternedSymbol, PropertyValue>>,
}

impl ComputedStyle {
    #[doc = include_str!("computed_style.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            values: FastMap::default(),
            custom_properties: Rc::default(),
        }
    }

    #[doc = include_str!("computed_style.doc/with_capacity.md")]
    #[must_use]
    pub fn with_capacity(properties: usize) -> Self {
        Self {
            values: FastMap::with_capacity_and_hasher(properties, FastBuildHasher::default()),
            // Not sized, because it is usually not built at all: a node that
            // declares no custom property of its own shares its parent's set
            // rather than copying it.
            custom_properties: Rc::default(),
        }
    }

    #[doc = include_str!("computed_style.doc/get.md")]
    #[must_use]
    pub fn get(&self, property: PropertyId) -> Option<&ComputedValue> {
        self.values.get(&property)
    }

    #[doc = include_str!("computed_style.doc/set.md")]
    pub fn set(&mut self, property: PropertyId, value: ComputedValue) -> Option<ComputedValue> {
        self.values.insert(property, value)
    }

    #[doc = include_str!("computed_style.doc/remove.md")]
    pub fn remove(&mut self, property: PropertyId) -> Option<ComputedValue> {
        self.values.remove(&property)
    }

    #[doc = include_str!("computed_style.doc/custom_property.md")]
    #[must_use]
    pub fn custom_property(&self, name: InternedSymbol) -> Option<&PropertyValue> {
        self.custom_properties.get(&name)
    }

    #[doc = include_str!("computed_style.doc/set_custom_property.md")]
    pub fn set_custom_property(
        &mut self,
        name: InternedSymbol,
        value: PropertyValue,
    ) -> Option<PropertyValue> {
        // Copy-on-write: the set is shared with whatever inherited it, and this
        // is the moment it stops being.
        Rc::make_mut(&mut self.custom_properties).insert(name, value)
    }

    #[doc = include_str!("computed_style.doc/shared_custom_properties.md")]
    #[must_use]
    pub fn shared_custom_properties(&self) -> Rc<FastMap<InternedSymbol, PropertyValue>> {
        Rc::clone(&self.custom_properties)
    }

    #[doc = include_str!("computed_style.doc/set_shared_custom_properties.md")]
    pub fn set_shared_custom_properties(
        &mut self,
        properties: Rc<FastMap<InternedSymbol, PropertyValue>>,
    ) {
        self.custom_properties = properties;
    }

    #[doc = include_str!("computed_style.doc/iter.md")]
    #[must_use]
    pub fn iter(&self) -> Iter<'_, PropertyId, ComputedValue> {
        self.values.iter()
    }

    #[doc = include_str!("computed_style.doc/custom_properties.md")]
    #[must_use]
    pub fn custom_properties(&self) -> Iter<'_, InternedSymbol, PropertyValue> {
        self.custom_properties.iter()
    }

    #[doc = include_str!("computed_style.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    #[doc = include_str!("computed_style.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    #[doc = include_str!("computed_style.doc/uses_ch.md")]
    #[must_use]
    pub fn uses_ch(&self) -> bool {
        self.values.values().any(|value| value.value().uses_ch())
    }

    #[doc = include_str!("computed_style.doc/opacity.md")]
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

    #[test]
    fn a_shared_custom_property_set_forks_the_moment_it_is_written() {
        // The sharing is an optimisation and must never be observable. A child
        // that inherits its parent's variables holds the same allocation until it
        // declares one of its own; writing must not reach back into the parent.
        let name = InternedSymbol::named("--tone");
        let other = InternedSymbol::named("--edge");
        let mut parent = ComputedStyle::new();
        parent.set_custom_property(name, PropertyValue::Number(1.0));

        let mut child = ComputedStyle::new();
        child.set_shared_custom_properties(parent.shared_custom_properties());
        assert_eq!(
            child.custom_property(name),
            Some(&PropertyValue::Number(1.0)),
            "an inherited set answers for the child"
        );

        child.set_custom_property(name, PropertyValue::Number(2.0));
        child.set_custom_property(other, PropertyValue::Number(3.0));

        assert_eq!(
            child.custom_property(name),
            Some(&PropertyValue::Number(2.0))
        );
        assert_eq!(
            child.custom_property(other),
            Some(&PropertyValue::Number(3.0))
        );
        assert_eq!(
            parent.custom_property(name),
            Some(&PropertyValue::Number(1.0)),
            "the parent kept its own value: the write forked, it did not reach back"
        );
        assert_eq!(
            parent.custom_property(other),
            None,
            "and the parent never saw a variable only the child declared"
        );
    }

    #[test]
    fn sharing_a_set_does_not_share_the_rest_of_the_style() {
        // Only the custom-property set travels. A child that inherits variables
        // has its own computed values, or every node in a themed tree would read
        // the root's `width`.
        let mut parent = ComputedStyle::new();
        parent.set_custom_property(InternedSymbol::named("--tone"), PropertyValue::Number(1.0));
        parent.set(PropertyId::Opacity, ComputedValue::number(0.5));

        let mut child = ComputedStyle::new();
        child.set_shared_custom_properties(parent.shared_custom_properties());
        assert_eq!(child.get(PropertyId::Opacity), None);
        assert_eq!(parent.opacity(), Some(0.5));
    }
    use crate::property::StyleColor;

    #[test]
    fn uses_ch_spots_a_ch_on_any_property() {
        use crate::property::Length;

        let mut style = ComputedStyle::new();
        style.set(
            PropertyId::Color,
            ComputedValue::from_concrete(PropertyValue::Color(StyleColor::BLACK)).expect("color"),
        );
        assert!(!style.uses_ch(), "an ordinary node needs no measurement");

        style.set(
            PropertyId::Width,
            ComputedValue::from_concrete(PropertyValue::Length(Length::Ch(20.0))).expect("width"),
        );
        assert!(style.uses_ch(), "found on a property other than the first");
    }

    #[test]
    fn an_empty_style_uses_no_ch() {
        assert!(!ComputedStyle::new().uses_ch());
    }

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
        assert_eq!(
            style.custom_property(name),
            Some(&PropertyValue::Number(2.0))
        );
        assert_eq!(style.custom_properties().count(), 1);
    }
}
