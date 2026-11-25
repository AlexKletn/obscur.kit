use obscur_motion::value::MotionValue;
use obscur_style::property::{Length, PropertyValue, StyleColor, Transform};

#[doc = include_str!("../../docs/api/orchestration/animation_binding.doc/AnimatedKind.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimatedKind {
    /// A bare number (e.g. `opacity`, `flex-grow`).
    Number,
    /// A pixel length (e.g. `width`, `height`, margins, padding).
    LengthPx,
    /// A 2D `transform` affine `[a, b, c, d, e, f]` (spec §4.17 post-layout
    /// motion; CSS Transforms §14 decomposition interpolation).
    Transform,
    /// An sRGB `color`/`background-color` etc., interpolated with premultiplied
    /// alpha from a [`Color`](MotionValue::Color) (css-color-4 §13.3).
    Color,
}

impl AnimatedKind {
    /// Lowers a sampled [`MotionValue`] into the concrete [`PropertyValue`] this
    /// kind represents, or [`None`] when the sample's variant does not match the
    /// kind (a scalar for `Number`/`LengthPx`, a transform matrix for
    /// `Transform`).
    #[must_use]
    pub fn lower(self, value: MotionValue) -> Option<PropertyValue> {
        match self {
            Self::Number => value.as_scalar().map(PropertyValue::Number),
            Self::LengthPx => value
                .as_scalar()
                .map(|s| PropertyValue::Length(Length::px(s))),
            Self::Transform => value
                .as_transform()
                .map(|matrix| PropertyValue::Transform(Transform::new(matrix))),
            Self::Color => value
                .as_color()
                .map(|[r, g, b, a]| PropertyValue::Color(StyleColor::rgba(r, g, b, a))),
        }
    }
}
