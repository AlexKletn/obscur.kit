#![doc = include_str!("background_layer.doc/_module.md")]

use crate::property::{ConicGradient, LinearGradient, PropertyValue, RadialGradient};

#[doc = include_str!("background_layer.doc/BackgroundLayer.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum BackgroundLayer {
    #[doc = include_str!("background_layer.doc/Linear.md")]
    Linear(LinearGradient),
    #[doc = include_str!("background_layer.doc/Radial.md")]
    Radial(RadialGradient),
    #[doc = include_str!("background_layer.doc/Conic.md")]
    Conic(ConicGradient),
    #[doc = include_str!("background_layer.doc/Url.md")]
    Url(Box<str>),
}

impl BackgroundLayer {
    #[doc = include_str!("background_layer.doc/uses_ch.md")]
    #[must_use]
    pub const fn uses_ch(&self) -> bool {
        match self {
            Self::Radial(gradient) => gradient.uses_ch(),
            Self::Conic(gradient) => gradient.uses_ch(),
            Self::Linear(_) | Self::Url(_) => false,
        }
    }

    #[doc = include_str!("background_layer.doc/into_property_value.md")]
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
