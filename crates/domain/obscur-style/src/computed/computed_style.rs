#![doc = include_str!("computed_style.doc/_module.md")]

use crate::computed::ComputedValue;
use crate::property::{PropertyId, PropertyValue};
use obscur_identity::InternedSymbol;
use std::collections::HashMap;
use std::collections::hash_map::Iter;

#[doc = include_str!("computed_style.doc/ComputedStyle.md")]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ComputedStyle {
    values: HashMap<PropertyId, ComputedValue>,
    custom_properties: HashMap<InternedSymbol, PropertyValue>,
}

impl ComputedStyle {
    #[doc = include_str!("computed_style.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            custom_properties: HashMap::new(),
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
        self.custom_properties.insert(name, value)
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
