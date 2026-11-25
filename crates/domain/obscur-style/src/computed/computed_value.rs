#![doc = include_str!("../../docs/api/computed/computed_value.doc/_module.md")]

use crate::property::{
    BackgroundLayer, BackgroundPosition, BackgroundRepeat, BackgroundSize, BoxShadow,
    ConicGradient, FilterOp, Length, LengthExpr, LinearGradient, ObjectFit, PropertyValue,
    RadialGradient, StyleColor, Transform, TransformOrigin,
};
use obscur_identity::InternedSymbol;

#[doc = include_str!("../../docs/api/computed/computed_value.doc/ComputedValue.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct ComputedValue(PropertyValue);

impl ComputedValue {
    #[doc = include_str!("../../docs/api/computed/computed_value.doc/from_concrete.md")]
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
            | PropertyValue::BackgroundSizes(_)
            | PropertyValue::BackgroundPosition(_)
            | PropertyValue::BackgroundPositions(_)
            | PropertyValue::BackgroundRepeat(_) | PropertyValue::BackgroundRepeats(_)
            | PropertyValue::BackgroundBlendModes(_)
            | PropertyValue::ClipPolygon { .. }
            | PropertyValue::ClipEllipse { .. }
            | PropertyValue::ClipRectangle { .. }
            | PropertyValue::ClipBox(_)
            | PropertyValue::ClipSvgPath { .. }
            | PropertyValue::BackgroundClips(_)
            | PropertyValue::BackgroundOrigins(_)
            | PropertyValue::BackgroundAttachments(_)
            | PropertyValue::ObjectFit(_)
            | PropertyValue::BoxShadow(_)
            | PropertyValue::Filter(_)
            | PropertyValue::FontFamilies(_)
            | PropertyValue::GridTracks(_)
            | PropertyValue::GridLine(_)
            | PropertyValue::GridAreas(_)
            | PropertyValue::GridAreaName(_)
            // A name identifies something registered with the runtime, and it
            // survives to computed value as the TEXT it was written as. A
            // `Keyword` could not: its symbol is minted by the interner of the
            // stylesheet that happened to be compiling, so the same word in two
            // sheets is two different symbols and neither matches a registry.
            | PropertyValue::ContainerNames(_)
            | PropertyValue::Name(_)
            | PropertyValue::Transform(_)
            // The two specified-value list forms are concrete in the same sense
            // a `Calc` is: they are a real value the cascade can carry, waiting
            // for a length context. The resolver folds them into a `Transform` /
            // `Filter` as it absolutizes, which is only reachable from HERE —
            // rejecting them would drop `transform: translate(2em)` before the
            // one pass that can resolve it ever runs.
            | PropertyValue::TransformFunctions(_)
            | PropertyValue::FilterFunctions(_)
            | PropertyValue::TransformOrigin(_) => Some(Self(value)),
            // `content` is a generated-content decoration, never a computed value
            // of a real element's box, so it has no place in a `ComputedValue`.
            PropertyValue::String(_)
            | PropertyValue::ContainerRef { .. }
            | PropertyValue::ContainerTemplate(_)
            | PropertyValue::FlexFlowTemplate(_)
            | PropertyValue::GapTemplate(_)
            | PropertyValue::VariableTemplate(_)
            | PropertyValue::CustomRef(_)
            // A custom property's declared text is a pre-substitution
            // placeholder in exactly the sense a `CustomRef` is: no element's
            // computed value is a token sequence.
            | PropertyValue::RawTokens(_)
            | PropertyValue::Inherit
            | PropertyValue::Initial
            | PropertyValue::Unset => None,
        }
    }

    /// Constructs a computed color value.
    #[must_use]
    pub const fn color(color: StyleColor) -> Self {
        Self(PropertyValue::Color(color))
    }

    /// Constructs a computed length value.
    #[must_use]
    pub const fn length(length: Length) -> Self {
        Self(PropertyValue::Length(length))
    }

    /// Constructs a computed `calc()` / `min()` / `max()` / `clamp()` value.
    #[must_use]
    pub const fn calc(expr: LengthExpr) -> Self {
        Self(PropertyValue::Calc(expr))
    }

    /// Constructs a computed bare-number value (for example `opacity`).
    #[must_use]
    pub const fn number(value: f32) -> Self {
        Self(PropertyValue::Number(value))
    }

    #[doc = include_str!("../../docs/api/computed/computed_value.doc/map_value.md")]
    pub(super) fn map_value(self, f: impl FnOnce(PropertyValue) -> PropertyValue) -> Self {
        Self(f(self.0))
    }

    /// Constructs a computed keyword value from its interned symbol.
    #[must_use]
    pub const fn keyword(symbol: InternedSymbol) -> Self {
        Self(PropertyValue::Keyword(symbol))
    }

    /// Returns the underlying concrete [`PropertyValue`].
    #[must_use]
    pub const fn value(&self) -> &PropertyValue {
        &self.0
    }

    /// Returns the color if this computed value is a color.
    #[must_use]
    pub const fn as_color(&self) -> Option<StyleColor> {
        self.0.as_color()
    }

    /// Returns the length if this computed value is a length.
    #[must_use]
    pub const fn as_length(&self) -> Option<Length> {
        self.0.as_length()
    }

    /// Returns the linear gradient if this computed value is a `linear-gradient`.
    #[must_use]
    pub const fn as_linear_gradient(&self) -> Option<&LinearGradient> {
        self.0.as_linear_gradient()
    }

    /// Returns the radial gradient if this computed value is a `radial-gradient`.
    #[must_use]
    pub const fn as_radial_gradient(&self) -> Option<&RadialGradient> {
        self.0.as_radial_gradient()
    }

    /// Returns the conic gradient if this computed value is a `conic-gradient`.
    #[must_use]
    pub const fn as_conic_gradient(&self) -> Option<&ConicGradient> {
        self.0.as_conic_gradient()
    }

    /// Returns the length expression if this computed value is a `calc()` /
    /// `min()` / `max()` / `clamp()`.
    #[must_use]
    pub const fn as_calc(&self) -> Option<&LengthExpr> {
        self.0.as_calc()
    }

    /// Returns the raw `url(...)` string if this computed value is a
    /// `background-image: url(...)` reference (unresolved).
    #[must_use]
    pub fn as_background_image_url(&self) -> Option<&str> {
        self.0.as_background_image_url()
    }

    /// Returns the multi-layer `background-image` list if this computed value is a
    /// [`PropertyValue::BackgroundLayers`] (two or more comma-separated layers, in
    /// author order). A single-layer background stays a flat gradient/url value.
    #[must_use]
    pub fn as_background_layers(&self) -> Option<&[BackgroundLayer]> {
        self.0.as_background_layers()
    }

    /// Returns the `box-shadow` list if this computed value is a `box-shadow`.
    #[must_use]
    pub fn as_box_shadows(&self) -> Option<&[BoxShadow]> {
        self.0.as_box_shadows()
    }

    /// Returns the `background-size` if this computed value is one.
    #[must_use]
    pub const fn as_background_size(&self) -> Option<BackgroundSize> {
        self.0.as_background_size()
    }

    /// Returns the `background-position` if this computed value is one.
    #[must_use]
    pub const fn as_background_position(&self) -> Option<BackgroundPosition> {
        self.0.as_background_position()
    }

    /// Returns the `background-repeat` if this computed value is one.
    #[must_use]
    pub const fn as_background_repeat(&self) -> Option<BackgroundRepeat> {
        self.0.as_background_repeat()
    }

    /// Returns the `object-fit` if this computed value is one.
    #[must_use]
    pub const fn as_object_fit(&self) -> Option<ObjectFit> {
        self.0.as_object_fit()
    }

    /// Returns the `filter` op list if this computed value is a `filter`.
    #[must_use]
    pub fn as_filter_ops(&self) -> Option<&[FilterOp]> {
        self.0.as_filter_ops()
    }

    /// Returns the resolved 2D `transform` if this computed value is a transform.
    #[must_use]
    pub const fn as_transform(&self) -> Option<Transform> {
        self.0.as_transform()
    }

    /// Returns the `transform-origin` if this computed value is a transform
    /// origin.
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
