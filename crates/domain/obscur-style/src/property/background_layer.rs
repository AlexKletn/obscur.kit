//! One layer of a multi-layer `background-image` list: [`BackgroundLayer`].
//!
//! CSS `background-image` accepts a **comma-separated list** of image layers,
//! painted back-to-front (the first-listed layer is the topmost, the last-listed
//! the bottommost). Each layer is one gradient (`linear-gradient(...)`,
//! `radial-gradient(...)`, `conic-gradient(...)`, including the `repeating-` forms)
//! or a `url(...)` reference.
//!
//! The single-layer case keeps the flat
//! [`PropertyValue::LinearGradient`](crate::property::PropertyValue::LinearGradient)
//! / [`RadialGradient`](crate::property::PropertyValue::RadialGradient) /
//! [`ConicGradient`](crate::property::PropertyValue::ConicGradient) /
//! [`BackgroundImageUrl`](crate::property::PropertyValue::BackgroundImageUrl)
//! variants (byte-identical to before this type existed); a list of two or more
//! layers is carried as
//! [`PropertyValue::BackgroundLayers`](crate::property::PropertyValue::BackgroundLayers)
//! holding an ordered slice of these layers, in author (top→bottom) order.

use crate::property::{ConicGradient, LinearGradient, PropertyValue, RadialGradient};

/// A single `background-image` layer: a gradient or a `url(...)` reference.
///
/// Layers are stored in CSS author order (first = top). The scene coordinator
/// paints them back-to-front (last first) so the first-listed layer ends up on
/// top (spec §5.2).
#[derive(Debug, Clone, PartialEq)]
pub enum BackgroundLayer {
    /// A `linear-gradient(...)` / `repeating-linear-gradient(...)` layer.
    Linear(LinearGradient),
    /// A `radial-gradient(...)` layer.
    Radial(RadialGradient),
    /// A `conic-gradient(...)` layer.
    Conic(ConicGradient),
    /// A `url(...)` image layer, carrying the **raw** authored URL/path string
    /// (unresolved), exactly as the single
    /// [`PropertyValue::BackgroundImageUrl`](crate::property::PropertyValue::BackgroundImageUrl)
    /// variant does.
    Url(Box<str>),
}

impl BackgroundLayer {
    /// Collapses a single layer back to the flat one-value
    /// [`PropertyValue`] it corresponds to.
    ///
    /// This is how a one-element layer list stays **byte-identical** to the
    /// historical single-value `background-image` result: the CSS lowering keeps
    /// the flat variant for one layer and only emits
    /// [`PropertyValue::BackgroundLayers`] for two or more.
    #[must_use]
    pub fn into_property_value(self) -> PropertyValue {
        match self {
            Self::Linear(gradient) => PropertyValue::LinearGradient(gradient),
            Self::Radial(gradient) => PropertyValue::RadialGradient(gradient),
            Self::Conic(gradient) => PropertyValue::ConicGradient(gradient),
            Self::Url(url) => PropertyValue::BackgroundImageUrl(url),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::property::{GradientStop, StyleColor};

    fn stops() -> Vec<GradientStop> {
        vec![
            GradientStop::new(StyleColor::BLACK, 0.0),
            GradientStop::new(StyleColor::rgba(1.0, 0.0, 0.0, 1.0), 1.0),
        ]
    }

    #[test]
    fn into_property_value_round_trips_each_kind() {
        let linear = LinearGradient::new(90.0, stops());
        assert_eq!(
            BackgroundLayer::Linear(linear.clone()).into_property_value(),
            PropertyValue::LinearGradient(linear)
        );
        assert_eq!(
            BackgroundLayer::Url("x.png".into()).into_property_value(),
            PropertyValue::BackgroundImageUrl("x.png".into())
        );
    }
}
