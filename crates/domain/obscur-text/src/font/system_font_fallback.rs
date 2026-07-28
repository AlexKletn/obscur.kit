//! The [`SystemFontFallback`] port: a last-resort source of faces for text no
//! registered font can render (spec §5.10 font fallback, §5.18).

use std::sync::Arc;

use super::FontFamily;

/// A face the system offered for a character nothing registered could draw.
#[derive(Debug, Clone)]
pub struct FallbackFace {
    family: FontFamily,
    bytes: Arc<[u8]>,
}

impl FallbackFace {
    /// Creates a fallback face from the family the system named it under and its
    /// raw bytes.
    #[must_use]
    pub const fn new(family: FontFamily, bytes: Arc<[u8]>) -> Self {
        Self { family, bytes }
    }

    /// The family this face was found under.
    #[must_use]
    pub const fn family(&self) -> &FontFamily {
        &self.family
    }

    /// The face's raw bytes.
    #[must_use]
    pub const fn bytes(&self) -> &Arc<[u8]> {
        &self.bytes
    }
}

/// Finds a face on the host that can draw a character the registered fonts
/// cannot (spec §5.10).
///
/// # Why a port rather than a bigger font set
///
/// An application's font chain is the handful of families it names plus the
/// generic ones a host installs — three faces, in this kit's default wiring.
/// Between them they cover Latin, Cyrillic and Greek and essentially nothing
/// else, so a CJK file name, an emoji, or a `⌫` on a key draws **nothing at
/// all**: a glyph that resolves to `.notdef` is skipped rather than boxed, which
/// reads as a broken control rather than as a missing font.
///
/// Registering every font on the machine up front is the obvious alternative and
/// the wrong one: a full CJK face is tens of megabytes, and an application pays
/// for all of them to render one character it may never see. This port asks the
/// question the other way round — *which face covers this character* — so the
/// cost is one lookup and one face, at the moment the character first appears.
///
/// # What an implementation owes
///
/// A **covering** face or [`None`]. Returning a face that does not actually
/// contain the character is worse than returning nothing: the resolver trusts
/// this answer and will register it, so a non-covering face becomes a permanent
/// wrong answer for that character. Implementations are consulted rarely (once
/// per new character, then cached by the caller) and may therefore be slow —
/// shelling out to a system tool is expected.
pub trait SystemFontFallback {
    /// A face covering `character`, or [`None`] when the host has none.
    #[must_use]
    fn face_for_char(&self, character: char) -> Option<FallbackFace>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Never;

    impl SystemFontFallback for Never {
        fn face_for_char(&self, _character: char) -> Option<FallbackFace> {
            None
        }
    }

    #[test]
    fn a_host_with_nothing_to_offer_says_so() {
        assert!(Never.face_for_char('中').is_none());
    }

    #[test]
    fn a_face_carries_the_family_it_was_found_under() {
        let bytes: Arc<[u8]> = Arc::from(&b"stub"[..]);
        let face = FallbackFace::new(FontFamily::named("DejaVu Sans"), Arc::clone(&bytes));
        assert_eq!(face.family().name(), "DejaVu Sans");
        assert_eq!(face.bytes().as_ref(), b"stub");
    }
}
