//! [`FontconfigFallback`]: the host's per-character font lookup, wired to the
//! text stack (spec §5.10 font fallback, §5.18).

use std::sync::Arc;

use crate::font_fontconfig::source::FontconfigSource;
use crate::text::font::{FallbackFace, FontFamily, SystemFontFallback};

/// Answers "which installed face can draw this character?" through fontconfig.
///
/// This is the seam that stops a character outside the application's own font
/// chain from rendering as nothing at all. The chain is the families an author
/// names plus the three generics a host installs; between them they cover Latin,
/// Cyrillic and Greek, so a CJK file name, an emoji, or a `⌫` on a key resolves
/// to no face and paints no glyphs — a blank, not a box.
///
/// Consulted only when every family in the chain has already declined a run, and
/// the answer is cached by the caller, so the cost is one `fc-match` per new
/// character rather than per run or per frame.
///
/// # What it does not do
///
/// It reads the face fontconfig names and nothing else: no `.ttc` collections
/// (the loader takes `.ttf`/`.otf` only, so a host whose CJK font is a
/// collection still draws nothing), and no attribute matching — a fallback face
/// is taken at its default weight and slant, because the question asked was
/// coverage, not style.
#[derive(Debug, Clone, Default)]
pub struct FontconfigFallback {
    source: FontconfigSource,
}

impl FontconfigFallback {
    /// Creates a fallback over the conventional `fc-match`.
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
