#![doc = include_str!("system_font_fallback.doc/_module.md")]

use std::sync::Arc;

use super::{FontAttributes, FontFamily};

#[doc = include_str!("system_font_fallback.doc/FallbackFace.md")]
#[derive(Debug, Clone)]
pub struct FallbackFace {
    family: FontFamily,
    attributes: FontAttributes,
    bytes: Arc<[u8]>,
}

impl FallbackFace {
    #[doc = include_str!("system_font_fallback.doc/new.md")]
    #[must_use]
    pub const fn new(family: FontFamily, bytes: Arc<[u8]>) -> Self {
        Self {
            family,
            attributes: FontAttributes::new(),
            bytes,
        }
    }

    #[doc = include_str!("system_font_fallback.doc/with_attributes.md")]
    #[must_use]
    pub const fn with_attributes(mut self, attributes: FontAttributes) -> Self {
        self.attributes = attributes;
        self
    }

    #[doc = include_str!("system_font_fallback.doc/attributes.md")]
    #[must_use]
    pub const fn attributes(&self) -> &FontAttributes {
        &self.attributes
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

    #[doc = include_str!("system_font_fallback.doc/face_for_family.md")]
    #[must_use]
    fn face_for_family(&self, name: &str, attributes: &FontAttributes) -> Option<FallbackFace> {
        // Defaulted rather than required: a host that can only be asked
        // "who covers this character" is still a complete implementation of
        // the older half of this port, and saying so here is what keeps
        // adding the named half from breaking one.
        let _ = (name, attributes);
        None
    }
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
    fn a_host_that_only_knows_characters_answers_no_family() {
        // The defaulted method: an implementation written before the named half
        // existed keeps compiling and keeps its old behaviour.
        assert!(
            Never
                .face_for_family("Figtree", &FontAttributes::new())
                .is_none()
        );
    }

    #[test]
    fn a_face_states_the_cut_it_is() {
        let bytes: Arc<[u8]> = Arc::from(&b"stub"[..]);
        let bold = FontAttributes::new().with_weight(crate::span::FontWeight::BOLD);
        let face = FallbackFace::new(FontFamily::named("Figtree"), bytes).with_attributes(bold);
        assert_eq!(face.attributes(), &bold);
        // Unstated is the regular upright cut, not "unknown".
        let plain = FallbackFace::new(FontFamily::named("Figtree"), Arc::from(&b"stub"[..]));
        assert_eq!(plain.attributes(), &FontAttributes::new());
    }

    #[test]
    fn a_face_carries_the_family_it_was_found_under() {
        let bytes: Arc<[u8]> = Arc::from(&b"stub"[..]);
        let face = FallbackFace::new(FontFamily::named("DejaVu Sans"), Arc::clone(&bytes));
        assert_eq!(face.family().name(), "DejaVu Sans");
        assert_eq!(face.bytes().as_ref(), b"stub");
    }
}
