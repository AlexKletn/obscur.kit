//! A concrete, resolvable font face (spec §5.18).

use crate::font::{FontAttributes, FontFamily, FontSource};

use super::FontMetrics;

#[doc = include_str!("../../docs/api/font/font_face.doc/FontFace.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct FontFace {
    /// The family this face belongs to.
    family: FontFamily,
    /// The attributes (weight/slant/width) this face provides.
    attributes: FontAttributes,
    /// Where this face's bytes come from.
    source: FontSource,
    /// This face's design metrics.
    metrics: FontMetrics,
}

impl FontFace {
    /// Creates a face from its `family`, `attributes`, `source`, and `metrics`.
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

    /// Returns the family this face belongs to.
    #[must_use]
    pub const fn family(&self) -> &FontFamily {
        &self.family
    }

    /// Returns the attributes this face provides.
    #[must_use]
    pub const fn attributes(&self) -> &FontAttributes {
        &self.attributes
    }

    /// Returns the source of this face's bytes.
    #[must_use]
    pub const fn source(&self) -> &FontSource {
        &self.source
    }

    /// Returns this face's design metrics.
    #[must_use]
    pub const fn metrics(&self) -> &FontMetrics {
        &self.metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::GenericFamily;
    use crate::span::FontWeight;

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
