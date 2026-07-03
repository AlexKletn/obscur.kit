//! A concrete, resolvable font face (spec §5.18).

use crate::font::{FontAttributes, FontFamily, FontSource};

/// A concrete, resolvable font face: a family and attributes paired with the
/// source of its bytes and its design metrics (spec §5.18).
///
/// A [`FontProvider`](crate::font::FontProvider) produces a `FontFace` when it
/// resolves a [`FontFamily`] and [`FontAttributes`] request. Because it embeds
/// `f32` [`FontMetrics`] it derives only `Debug`, `Clone`, and `PartialEq`
/// (floats preclude `Eq`).
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

/// A font's design metrics, in font design units (spec §5.18).
///
/// Distances are expressed in the font's own coordinate space, whose scale is
/// [`units_per_em`](FontMetrics::units_per_em); divide by it and multiply by a
/// pixel size to obtain pixels. Following font conventions, `descent` is
/// typically **negative** (below the baseline); it is stored exactly as given,
/// and [`line_height`](FontMetrics::line_height) takes its magnitude.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FontMetrics {
    /// The number of design units per em; the scale for every other field.
    pub units_per_em: u16,
    /// The ascent above the baseline, in font units (typically positive).
    pub ascent: f32,
    /// The descent below the baseline, in font units (typically negative).
    pub descent: f32,
    /// Extra leading between lines, in font units.
    pub line_gap: f32,
    /// The height of a flat capital, in font units, if the font reports it.
    pub cap_height: Option<f32>,
    /// The height of a lowercase `x`, in font units, if the font reports it.
    pub x_height: Option<f32>,
}

impl FontMetrics {
    /// Creates metrics from the required fields, leaving
    /// [`cap_height`](Self::cap_height) and [`x_height`](Self::x_height) unset.
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

    /// Returns these metrics with [`cap_height`](Self::cap_height) set.
    #[must_use]
    pub const fn with_cap_height(mut self, cap_height: f32) -> Self {
        self.cap_height = Some(cap_height);
        self
    }

    /// Returns these metrics with [`x_height`](Self::x_height) set.
    #[must_use]
    pub const fn with_x_height(mut self, x_height: f32) -> Self {
        self.x_height = Some(x_height);
        self
    }

    /// Returns the natural line height in font units:
    /// `ascent + |descent| + line_gap` (spec §5.18).
    ///
    /// The descent's magnitude is used so a font's conventional negative descent
    /// still adds to the line height rather than subtracting from it.
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
        let face = FontFace::new(
            family.clone(),
            attributes,
            source.clone(),
            metrics,
        );
        assert_eq!(face.family(), &family);
        assert_eq!(face.attributes(), &attributes);
        assert_eq!(face.source(), &source);
        assert_eq!(face.metrics().units_per_em, 1000);
        assert_ne!(family, FontFamily::generic(GenericFamily::Serif));
    }
}
