use std::sync::Arc;

use super::{FontAttributes, FontFamily};

/// A face the system offered for a character nothing registered could draw.
#[derive(Debug, Clone)]
pub struct FallbackFace {
    family: FontFamily,
    attributes: FontAttributes,
    bytes: Arc<[u8]>,
}

impl FallbackFace {
    /// Creates a fallback face from the family the system named it under and its
    /// raw bytes.
    #[must_use]
    pub const fn new(family: FontFamily, bytes: Arc<[u8]>) -> Self {
        Self {
            family,
            attributes: FontAttributes::new(),
            bytes,
        }
    }

    #[doc = include_str!("../../docs/api/font/system_font_fallback.doc/with_attributes.md")]
    #[must_use]
    pub const fn with_attributes(mut self, attributes: FontAttributes) -> Self {
        self.attributes = attributes;
        self
    }

    #[doc = include_str!("../../docs/api/font/system_font_fallback.doc/attributes.md")]
    #[must_use]
    pub const fn attributes(&self) -> &FontAttributes {
        &self.attributes
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

#[cfg(test)]
mod tests {
    use super::*;

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
