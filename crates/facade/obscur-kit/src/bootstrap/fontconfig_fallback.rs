#![doc = include_str!("fontconfig_fallback.doc/_module.md")]

use std::sync::Arc;

use crate::font_fontconfig::source::FontconfigSource;
use crate::text::font::{FallbackFace, FontAttributes, FontFamily, SystemFontFallback};

#[doc = include_str!("fontconfig_fallback.doc/FontconfigFallback.md")]
#[derive(Debug, Clone, Default)]
pub struct FontconfigFallback {
    source: FontconfigSource,
}

impl FontconfigFallback {
    #[doc = include_str!("fontconfig_fallback.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            source: FontconfigSource::new(),
        }
    }
}

impl SystemFontFallback for FontconfigFallback {
    fn face_for_char(&self, character: char) -> Option<FallbackFace> {
        let face = self.source.match_char(character)?;
        let bytes = std::fs::read(face.path()).ok()?;
        Some(FallbackFace::new(
            FontFamily::named(face.family()),
            Arc::from(bytes),
        ))
    }

    fn face_for_family(&self, name: &str, attributes: &FontAttributes) -> Option<FallbackFace> {
        // `fc-list` rather than `fc-match`: matching *substitutes*, and a
        // substitute is exactly the wrong answer here — it would report the
        // system sans under the author's name and make an uninstalled font
        // indistinguishable from an installed one. `find_family` keeps only
        // faces whose family really is this name, and picks the nearest cut.
        let face = self.source.find_family(name, attributes)?;
        let bytes = std::fs::read(face.path()).ok()?;
        Some(
            FallbackFace::new(FontFamily::named(face.family()), Arc::from(bytes))
                .with_attributes(*face.attributes()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Latin is covered by the faces already in the chain, but this asks the host
    /// directly: whatever it answers must at least be a *loadable* face, since a
    /// path that cannot be read is dropped rather than returned empty.
    ///
    /// Skipped where fontconfig is absent — there is nothing to ask.
    #[test]
    fn a_host_answer_is_a_real_readable_face() {
        let fallback = FontconfigFallback::new();
        if !FontconfigSource::new().is_available() {
            return;
        }
        if let Some(face) = fallback.face_for_char('A') {
            assert!(
                !face.bytes().is_empty(),
                "a returned face carries its bytes"
            );
            assert!(!face.family().name().is_empty());
        }
    }

    /// A name no machine has must come back as [`None`] rather than as the
    /// substitute `fc-match` would happily hand over — otherwise a font that is
    /// not installed is indistinguishable from one that is.
    #[test]
    fn an_uninstalled_family_is_not_answered_with_a_substitute() {
        let fallback = FontconfigFallback::new();
        if !FontconfigSource::new().is_available() {
            return;
        }
        assert!(
            fallback
                .face_for_family("ThisFamilyDoesNotExist12345", &FontAttributes::new())
                .is_none()
        );
    }
}
