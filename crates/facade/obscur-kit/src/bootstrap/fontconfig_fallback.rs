#![doc = include_str!("fontconfig_fallback.doc/_module.md")]

use std::sync::Arc;

use crate::font_fontconfig::source::FontconfigSource;
use crate::text::font::{FallbackFace, FontFamily, SystemFontFallback};

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
}
