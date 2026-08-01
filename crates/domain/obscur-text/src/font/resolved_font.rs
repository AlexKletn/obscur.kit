#![doc = include_str!("resolved_font.doc/_module.md")]

use crate::font::{FontFace, FontId};

#[doc = include_str!("resolved_font.doc/ResolvedFont.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedFont {
    #[doc = include_str!("resolved_font.doc/font.md")]
    font: FontId,
    #[doc = include_str!("resolved_font.doc/face.md")]
    face: FontFace,
    #[doc = include_str!("resolved_font.doc/synth_oblique.md")]
    synth_oblique: bool,
    #[doc = include_str!("resolved_font.doc/synth_bold.md")]
    synth_bold: bool,
}

impl ResolvedFont {
    #[doc = include_str!("resolved_font.doc/new.md")]
    #[must_use]
    pub const fn new(font: FontId, face: FontFace) -> Self {
        Self {
            font,
            face,
            synth_oblique: false,
            synth_bold: false,
        }
    }

    #[doc = include_str!("resolved_font.doc/with_synthesis.md")]
    #[must_use]
    pub const fn with_synthesis(mut self, oblique: bool, bold: bool) -> Self {
        self.synth_oblique = oblique;
        self.synth_bold = bold;
        self
    }

    #[doc = include_str!("resolved_font.doc/font_2.md")]
    #[must_use]
    pub const fn font(&self) -> FontId {
        self.font
    }

    #[doc = include_str!("resolved_font.doc/synth_oblique_2.md")]
    #[must_use]
    pub const fn synth_oblique(&self) -> bool {
        self.synth_oblique
    }

    #[doc = include_str!("resolved_font.doc/synth_bold_2.md")]
    #[must_use]
    pub const fn synth_bold(&self) -> bool {
        self.synth_bold
    }

    #[doc = include_str!("resolved_font.doc/face_2.md")]
    #[must_use]
    pub const fn face(&self) -> &FontFace {
        &self.face
    }

    #[doc = include_str!("resolved_font.doc/into_face.md")]
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
