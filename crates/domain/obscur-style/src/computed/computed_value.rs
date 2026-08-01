#![doc = include_str!("computed_value.doc/_module.md")]

use crate::property::{
    BackgroundLayer, BackgroundPosition, BackgroundRepeat, BackgroundSize, BoxShadow,
    ConicGradient, FilterOp, Length, LengthExpr, LinearGradient, ObjectFit, PropertyValue,
    RadialGradient, StyleColor, Transform, TransformOrigin,
};
use obscur_identity::InternedSymbol;

#[doc = include_str!("computed_value.doc/ComputedValue.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct ComputedValue(PropertyValue);

impl ComputedValue {
    #[doc = include_str!("computed_value.doc/from_concrete.md")]
    #[must_use]
    pub fn from_concrete(value: PropertyValue) -> Option<Self> {
        match value {
            PropertyValue::Keyword(_)
            | PropertyValue::Length(_)
            | PropertyValue::Calc(_)
            | PropertyValue::Number(_)
            | PropertyValue::Color(_)
            | PropertyValue::LinearGradient(_)
            | PropertyValue::RadialGradient(_)
            | PropertyValue::ConicGradient(_)
            | PropertyValue::BackgroundImageUrl(_)
            | PropertyValue::BackgroundLayers(_)
            | PropertyValue::BackgroundSize(_)
            | PropertyValue::BackgroundPosition(_)
            | PropertyValue::BackgroundRepeat(_)
            | PropertyValue::ObjectFit(_)
            | PropertyValue::BoxShadow(_)
            | PropertyValue::Filter(_)
            | PropertyValue::FontFamilies(_)
            | PropertyValue::GridTracks(_)
            | PropertyValue::GridLine(_)
            | PropertyValue::GridAreas(_)
            | PropertyValue::GridAreaName(_)
            | PropertyValue::Transform(_)
            | PropertyValue::TransformOrigin(_) => Some(Self(value)),
            // `content` is a generated-content decoration, never a computed value
            // of a real element's box, so it has no place in a `ComputedValue`.
            PropertyValue::String(_)
            | PropertyValue::CustomRef(_)
            | PropertyValue::Inherit
            | PropertyValue::Initial
            | PropertyValue::Unset => None,
        }
    }

    #[doc = include_str!("computed_value.doc/color.md")]
    #[must_use]
    pub const fn color(color: StyleColor) -> Self {
        Self(PropertyValue::Color(color))
    }

    #[doc = include_str!("computed_value.doc/length.md")]
    #[must_use]
    pub const fn length(length: Length) -> Self {
        Self(PropertyValue::Length(length))
    }

    #[doc = include_str!("computed_value.doc/calc.md")]
    #[must_use]
    pub const fn calc(expr: LengthExpr) -> Self {
        Self(PropertyValue::Calc(expr))
    }

    #[doc = include_str!("computed_value.doc/number.md")]
    #[must_use]
    pub const fn number(value: f32) -> Self {
        Self(PropertyValue::Number(value))
    }

    #[doc = include_str!("computed_value.doc/map_value.md")]
    pub(super) fn map_value(self, f: impl FnOnce(PropertyValue) -> PropertyValue) -> Self {
        Self(f(self.0))
    }

    #[doc = include_str!("computed_value.doc/keyword.md")]
    #[must_use]
    pub const fn keyword(symbol: InternedSymbol) -> Self {
        Self(PropertyValue::Keyword(symbol))
    }

    #[doc = include_str!("computed_value.doc/value.md")]
    #[must_use]
    pub const fn value(&self) -> &PropertyValue {
        &self.0
    }

    #[doc = include_str!("computed_value.doc/as_color.md")]
    #[must_use]
    pub const fn as_color(&self) -> Option<StyleColor> {
        self.0.as_color()
    }

    #[doc = include_str!("computed_value.doc/as_length.md")]
    #[must_use]
    pub const fn as_length(&self) -> Option<Length> {
        self.0.as_length()
    }

    #[doc = include_str!("computed_value.doc/as_linear_gradient.md")]
    #[must_use]
    pub const fn as_linear_gradient(&self) -> Option<&LinearGradient> {
        self.0.as_linear_gradient()
    }

    #[doc = include_str!("computed_value.doc/as_radial_gradient.md")]
    #[must_use]
    pub const fn as_radial_gradient(&self) -> Option<&RadialGradient> {
        self.0.as_radial_gradient()
    }

    #[doc = include_str!("computed_value.doc/as_conic_gradient.md")]
    #[must_use]
    pub const fn as_conic_gradient(&self) -> Option<&ConicGradient> {
        self.0.as_conic_gradient()
    }

    #[doc = include_str!("computed_value.doc/as_calc.md")]
    #[must_use]
    pub const fn as_calc(&self) -> Option<&LengthExpr> {
        self.0.as_calc()
    }

    #[doc = include_str!("computed_value.doc/as_background_image_url.md")]
    #[must_use]
    pub fn as_background_image_url(&self) -> Option<&str> {
        self.0.as_background_image_url()
    }

    #[doc = include_str!("computed_value.doc/as_background_layers.md")]
    #[must_use]
    pub fn as_background_layers(&self) -> Option<&[BackgroundLayer]> {
        self.0.as_background_layers()
    }

    #[doc = include_str!("computed_value.doc/as_box_shadows.md")]
    #[must_use]
    pub fn as_box_shadows(&self) -> Option<&[BoxShadow]> {
        self.0.as_box_shadows()
    }

    #[doc = include_str!("computed_value.doc/as_background_size.md")]
    #[must_use]
    pub const fn as_background_size(&self) -> Option<BackgroundSize> {
        self.0.as_background_size()
    }

    #[doc = include_str!("computed_value.doc/as_background_position.md")]
    #[must_use]
    pub const fn as_background_position(&self) -> Option<BackgroundPosition> {
        self.0.as_background_position()
    }

    #[doc = include_str!("computed_value.doc/as_background_repeat.md")]
    #[must_use]
    pub const fn as_background_repeat(&self) -> Option<BackgroundRepeat> {
        self.0.as_background_repeat()
    }

    #[doc = include_str!("computed_value.doc/as_object_fit.md")]
    #[must_use]
    pub const fn as_object_fit(&self) -> Option<ObjectFit> {
        self.0.as_object_fit()
    }

    #[doc = include_str!("computed_value.doc/as_filter_ops.md")]
    #[must_use]
    pub fn as_filter_ops(&self) -> Option<&[FilterOp]> {
        self.0.as_filter_ops()
    }

    #[doc = include_str!("computed_value.doc/as_transform.md")]
    #[must_use]
    pub const fn as_transform(&self) -> Option<Transform> {
        self.0.as_transform()
    }

    #[doc = include_str!("computed_value.doc/as_transform_origin.md")]
    #[must_use]
    pub const fn as_transform_origin(&self) -> Option<TransformOrigin> {
        self.0.as_transform_origin()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_concrete_accepts_leaf_values() {
        assert!(ComputedValue::from_concrete(PropertyValue::Number(1.0)).is_some());
        assert!(ComputedValue::from_concrete(PropertyValue::Color(StyleColor::BLACK)).is_some());
        assert!(ComputedValue::from_concrete(PropertyValue::Length(Length::Auto)).is_some());
        assert!(
            ComputedValue::from_concrete(PropertyValue::Keyword(InternedSymbol::from_raw(3)))
                .is_some()
        );
    }

    #[test]
    fn background_image_url_is_concrete_and_readable() {
        let value =
            ComputedValue::from_concrete(PropertyValue::BackgroundImageUrl("logo.png".into()))
                .expect("a url background-image is a concrete computed value");
        assert_eq!(value.as_background_image_url(), Some("logo.png"));
    }

    #[test]
    fn background_layers_survive_the_computed_step() {
        use crate::property::{BackgroundLayer, GradientStop, LinearGradient};
        let layers: Box<[BackgroundLayer]> = Box::new([
            BackgroundLayer::Linear(LinearGradient::new(
                0.0,
                vec![
                    GradientStop::new(StyleColor::BLACK, 0.0),
                    GradientStop::new(StyleColor::rgba(1.0, 0.0, 0.0, 1.0), 1.0),
                ],
            )),
            BackgroundLayer::Url("x.png".into()),
        ]);
        let value = ComputedValue::from_concrete(PropertyValue::BackgroundLayers(layers.clone()))
            .expect("a background-layer list is a concrete computed value");
        assert_eq!(value.as_background_layers(), Some(&*layers));
    }

    #[test]
    fn from_concrete_rejects_non_concrete_values() {
        assert!(ComputedValue::from_concrete(PropertyValue::Inherit).is_none());
        assert!(ComputedValue::from_concrete(PropertyValue::Initial).is_none());
        assert!(ComputedValue::from_concrete(PropertyValue::Unset).is_none());
        assert!(
            ComputedValue::from_concrete(PropertyValue::CustomRef(InternedSymbol::from_raw(1)))
                .is_none()
        );
    }

    #[test]
    fn typed_constructors_are_concrete() {
        assert_eq!(
            ComputedValue::color(StyleColor::BLACK).as_color(),
            Some(StyleColor::BLACK)
        );
        assert_eq!(
            ComputedValue::length(Length::px(4.0)).as_length(),
            Some(Length::px(4.0))
        );
        assert_eq!(
            ComputedValue::number(0.5).value(),
            &PropertyValue::Number(0.5)
        );
        let sym = InternedSymbol::from_raw(7);
        assert_eq!(
            ComputedValue::keyword(sym).value(),
            &PropertyValue::Keyword(sym)
        );
    }
}
