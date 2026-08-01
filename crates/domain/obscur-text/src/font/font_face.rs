#![doc = include_str!("font_face.doc/_module.md")]

use crate::font::{FontAttributes, FontFamily, FontSource};

#[doc = include_str!("font_face.doc/FontFace.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct FontFace {
    #[doc = include_str!("font_face.doc/family.md")]
    family: FontFamily,
    #[doc = include_str!("font_face.doc/attributes.md")]
    attributes: FontAttributes,
    #[doc = include_str!("font_face.doc/source.md")]
    source: FontSource,
    #[doc = include_str!("font_face.doc/metrics.md")]
    metrics: FontMetrics,
}

impl FontFace {
    #[doc = include_str!("font_face.doc/new.md")]
    #[must_use]
    pub const fn new(
        family: FontFamily,
        attributes: FontAttributes,
        source: FontSource,
        metrics: FontMetrics,
    ) -> Self {
        Self {
            family,
            attributes,
            source,
            metrics,
        }
    }

    #[doc = include_str!("font_face.doc/family_2.md")]
    #[must_use]
    pub const fn family(&self) -> &FontFamily {
        &self.family
    }

    #[doc = include_str!("font_face.doc/attributes_2.md")]
    #[must_use]
    pub const fn attributes(&self) -> &FontAttributes {
        &self.attributes
    }

    #[doc = include_str!("font_face.doc/source_2.md")]
    #[must_use]
    pub const fn source(&self) -> &FontSource {
        &self.source
    }

    #[doc = include_str!("font_face.doc/metrics_2.md")]
    #[must_use]
    pub const fn metrics(&self) -> &FontMetrics {
        &self.metrics
    }
}

#[doc = include_str!("font_face.doc/FontMetrics.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FontMetrics {
    #[doc = include_str!("font_face.doc/units_per_em.md")]
    pub units_per_em: u16,
    #[doc = include_str!("font_face.doc/ascent.md")]
    pub ascent: f32,
    #[doc = include_str!("font_face.doc/descent.md")]
    pub descent: f32,
    #[doc = include_str!("font_face.doc/line_gap.md")]
    pub line_gap: f32,
    #[doc = include_str!("font_face.doc/cap_height.md")]
    pub cap_height: Option<f32>,
    #[doc = include_str!("font_face.doc/x_height.md")]
    pub x_height: Option<f32>,
}

impl FontMetrics {
    #[doc = include_str!("font_face.doc/new_2.md")]
    #[must_use]
    pub const fn new(units_per_em: u16, ascent: f32, descent: f32, line_gap: f32) -> Self {
        Self {
            units_per_em,
            ascent,
            descent,
            line_gap,
            cap_height: None,
            x_height: None,
        }
    }

    #[doc = include_str!("font_face.doc/with_cap_height.md")]
    #[must_use]
    pub const fn with_cap_height(mut self, cap_height: f32) -> Self {
        self.cap_height = Some(cap_height);
        self
    }

    #[doc = include_str!("font_face.doc/with_x_height.md")]
    #[must_use]
    pub const fn with_x_height(mut self, x_height: f32) -> Self {
        self.x_height = Some(x_height);
        self
    }

    #[doc = include_str!("font_face.doc/line_height.md")]
    #[must_use]
    pub fn line_height(&self) -> f32 {
        self.ascent + self.descent.abs() + self.line_gap
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::GenericFamily;
    use crate::span::FontWeight;

    #[test]
    fn line_height_uses_descent_magnitude() {
        // Descent given as a negative value per font conventions.
        let metrics = FontMetrics::new(1000, 800.0, -200.0, 100.0);
        assert_eq!(metrics.line_height(), 1100.0);
        // A positive descent yields the same magnitude-based result.
        let positive = FontMetrics::new(1000, 800.0, 200.0, 100.0);
        assert_eq!(positive.line_height(), 1100.0);
    }

    #[test]
    fn optional_metrics_default_none_then_set() {
        let bare = FontMetrics::new(2048, 1500.0, -400.0, 0.0);
        assert_eq!(bare.cap_height, None);
        assert_eq!(bare.x_height, None);
        let full = bare.with_cap_height(1400.0).with_x_height(1000.0);
        assert_eq!(full.cap_height, Some(1400.0));
        assert_eq!(full.x_height, Some(1000.0));
    }

    #[test]
    fn face_accessors() {
        let family = FontFamily::named("Inter");
        let attributes = FontAttributes::new().with_weight(FontWeight::BOLD);
        let source = FontSource::system("Inter-Bold");
        let metrics = FontMetrics::new(1000, 800.0, -200.0, 0.0);
        let face = FontFace::new(family.clone(), attributes, source.clone(), metrics);
        assert_eq!(face.family(), &family);
        assert_eq!(face.attributes(), &attributes);
        assert_eq!(face.source(), &source);
        assert_eq!(face.metrics().units_per_em, 1000);
        assert_ne!(family, FontFamily::generic(GenericFamily::Serif));
    }
}
