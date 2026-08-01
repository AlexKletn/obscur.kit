#![doc = include_str!("system_font_fallback.doc/_module.md")]

use std::sync::Arc;

use super::FontFamily;

#[doc = include_str!("system_font_fallback.doc/FallbackFace.md")]
#[derive(Debug, Clone)]
pub struct FallbackFace {
    family: FontFamily,
    bytes: Arc<[u8]>,
}

impl FallbackFace {
    #[doc = include_str!("system_font_fallback.doc/new.md")]
    #[must_use]
    pub const fn new(family: FontFamily, bytes: Arc<[u8]>) -> Self {
        Self { family, bytes }
    }

    #[doc = include_str!("system_font_fallback.doc/family.md")]
    #[must_use]
    pub const fn family(&self) -> &FontFamily {
        &self.family
    }

    #[doc = include_str!("system_font_fallback.doc/bytes.md")]
    #[must_use]
    pub const fn bytes(&self) -> &Arc<[u8]> {
        &self.bytes
    }
}

#[doc = include_str!("system_font_fallback.doc/SystemFontFallback.md")]
pub trait SystemFontFallback {
    #[doc = include_str!("system_font_fallback.doc/face_for_char.md")]
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
