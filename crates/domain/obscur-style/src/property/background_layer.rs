#![doc = include_str!("../../docs/api/property/background_layer.doc/_module.md")]

use crate::property::{ConicGradient, LinearGradient, PropertyValue, RadialGradient};

#[doc = include_str!("../../docs/api/property/background_layer.doc/BackgroundLayer.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum BackgroundLayer {
    /// A layer position with no image to paint.
    None,
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
    #[doc = include_str!("../../docs/api/property/background_layer.doc/uses_ch.md")]
    #[must_use]
    pub fn uses_ch(&self) -> bool {
        match self {
            Self::Linear(gradient) => gradient.uses_ch(),
            Self::Radial(gradient) => gradient.uses_ch(),
            Self::Conic(gradient) => gradient.uses_ch(),
            Self::None | Self::Url(_) => false,
        }
    }

    #[doc = include_str!("../../docs/api/property/background_layer.doc/into_property_value.md")]
    #[must_use]
    pub fn into_property_value(self) -> PropertyValue {
        match self {
            Self::None => PropertyValue::Initial,
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
