//! The [`SystemFontFallback`] port: a last-resort source of faces for text no
//! registered font can render (spec §5.10 font fallback, §5.18).

use super::{FallbackFace, FontAttributes};

#[doc = include_str!("../../docs/api/font/system_font_fallback.doc/SystemFontFallback.md")]
pub trait SystemFontFallback {
    /// A face covering `character`, or [`None`] when the host has none.
    #[must_use]
    fn face_for_char(&self, character: char) -> Option<FallbackFace>;

    #[doc = include_str!("../../docs/api/font/system_font_fallback.doc/face_for_family.md")]
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
}
