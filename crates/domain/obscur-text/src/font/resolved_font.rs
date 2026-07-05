//! The outcome of resolving a font for one itemized run (spec §5.10, §5.18).

use crate::font::{FontFace, FontId};

/// The font chosen for one itemized run: a [`FontId`] paired with the concrete
/// [`FontFace`] it names (spec §5.10 font fallback, §5.18 resolution).
///
/// A [`FontResolver`](crate::font::FontResolver) walks a fallback chain and
/// returns a `ResolvedFont` for the first family that can render the run. The
/// [`font`](Self::font) tags the run's glyphs so later stages can look the face
/// back up in whatever registry issued the id, while the [`face`](Self::face) is
/// handed straight to the shaper. Because [`FontFace`] embeds `f32` metrics this
/// type derives `PartialEq` but not `Eq`.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedFont {
    /// The id the resolved run's glyph indices belong to.
    font: FontId,
    /// The concrete face to shape the run with.
    face: FontFace,
}

impl ResolvedFont {
    /// Creates a resolution pairing `font` with the concrete `face` it names.
    #[must_use]
    pub const fn new(font: FontId, face: FontFace) -> Self {
        Self { font, face }
    }

    /// Returns the id the resolved run's glyph indices belong to.
    #[must_use]
    pub const fn font(&self) -> FontId {
        self.font
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
}
