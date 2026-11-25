//! The outcome of resolving a font for one itemized run (spec §5.10, §5.18).

use crate::font::{FontFace, FontId};

#[doc = include_str!("../../docs/api/font/resolved_font.doc/ResolvedFont.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedFont {
    /// The id the resolved run's glyph indices belong to.
    font: FontId,
    /// The concrete face to shape the run with.
    face: FontFace,
    /// Whether the run's glyphs should be sheared to synthesize a slant the
    /// matched (upright) face lacks.
    synth_oblique: bool,
    /// Whether the run's glyphs should be thickened to synthesize a bold weight
    /// the matched (lighter) face lacks.
    synth_bold: bool,
}

impl ResolvedFont {
    /// Creates a resolution pairing `font` with the concrete `face` it names,
    /// with no synthesis (an exact-enough match needs none).
    #[must_use]
    pub const fn new(font: FontId, face: FontFace) -> Self {
        Self {
            font,
            face,
            synth_oblique: false,
            synth_bold: false,
        }
    }

    /// Returns this resolution flagged for synthetic styling: `oblique` shears the
    /// glyphs to fake a slant, `bold` thickens them to fake weight (spec §5.18).
    #[must_use]
    pub const fn with_synthesis(mut self, oblique: bool, bold: bool) -> Self {
        self.synth_oblique = oblique;
        self.synth_bold = bold;
        self
    }

    /// Returns the id the resolved run's glyph indices belong to.
    #[must_use]
    pub const fn font(&self) -> FontId {
        self.font
    }

    /// Returns whether the run's glyphs should be sheared to synthesize a slant
    /// the matched face lacks (spec §5.18).
    #[must_use]
    pub const fn synth_oblique(&self) -> bool {
        self.synth_oblique
    }

    /// Returns whether the run's glyphs should be thickened to synthesize a bold
    /// weight the matched face lacks (spec §5.18).
    #[must_use]
    pub const fn synth_bold(&self) -> bool {
        self.synth_bold
    }

    /// Returns the concrete face to shape the run with.
    #[must_use]
    pub const fn face(&self) -> &FontFace {
        &self.face
    }

    /// Consumes the resolution, returning the owned [`FontFace`].
    #[must_use]
    pub fn into_face(self) -> FontFace {
        self.face
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{FontAttributes, FontFamily, FontMetrics, FontSource};

    fn face(name: &str) -> FontFace {
        FontFace::new(
            FontFamily::named(name),
            FontAttributes::default(),
            FontSource::system(name),
            FontMetrics::new(1000, 800.0, -200.0, 0.0),
        )
    }

    #[test]
    fn accessors_report_construction() {
        let resolved = ResolvedFont::new(FontId::new(2), face("Inter"));
        assert_eq!(resolved.font(), FontId::new(2));
        assert_eq!(resolved.face(), &face("Inter"));
    }

    #[test]
    fn into_face_yields_owned_face() {
        let resolved = ResolvedFont::new(FontId::PRIMARY, face("Inter"));
        assert_eq!(resolved.into_face(), face("Inter"));
    }

    #[test]
    fn synthesis_defaults_false_and_builder_sets_it() {
        let plain = ResolvedFont::new(FontId::new(1), face("Inter"));
        assert!(!plain.synth_oblique());
        assert!(!plain.synth_bold());

        let synthesized =
            ResolvedFont::new(FontId::new(1), face("Inter")).with_synthesis(true, true);
        assert!(synthesized.synth_oblique());
        assert!(synthesized.synth_bold());
        // The flags participate in equality (additive fields on the derive).
        assert_ne!(plain, synthesized);

        // Each flag is independent.
        let oblique_only =
            ResolvedFont::new(FontId::new(1), face("Inter")).with_synthesis(true, false);
        assert!(oblique_only.synth_oblique());
        assert!(!oblique_only.synth_bold());
    }
}
