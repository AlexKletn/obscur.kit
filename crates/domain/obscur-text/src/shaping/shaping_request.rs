#![doc = include_str!("shaping_request.doc/_module.md")]

use crate::font::FontFace;
use crate::shaping::TextDirection;

#[doc = include_str!("shaping_request.doc/ShapingRequest.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct ShapingRequest {
    #[doc = include_str!("shaping_request.doc/text.md")]
    text: Box<str>,
    #[doc = include_str!("shaping_request.doc/face.md")]
    face: FontFace,
    #[doc = include_str!("shaping_request.doc/font_size.md")]
    font_size: f32,
    #[doc = include_str!("shaping_request.doc/direction.md")]
    direction: TextDirection,
    #[doc = include_str!("shaping_request.doc/language.md")]
    language: Option<Box<str>>,
}

impl ShapingRequest {
    #[doc = include_str!("shaping_request.doc/new.md")]
    #[must_use]
    pub fn new(text: impl Into<Box<str>>, face: FontFace, font_size: f32) -> Self {
        Self {
            text: text.into(),
            face,
            font_size,
            direction: TextDirection::LeftToRight,
            language: None,
        }
    }

    #[doc = include_str!("shaping_request.doc/with_direction.md")]
    #[must_use]
    pub fn with_direction(mut self, direction: TextDirection) -> Self {
        self.direction = direction;
        self
    }

    #[doc = include_str!("shaping_request.doc/with_language.md")]
    #[must_use]
    pub fn with_language(mut self, language: impl Into<Box<str>>) -> Self {
        self.language = Some(language.into());
        self
    }

    #[doc = include_str!("shaping_request.doc/text_2.md")]
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    #[doc = include_str!("shaping_request.doc/face_2.md")]
    #[must_use]
    pub const fn face(&self) -> &FontFace {
        &self.face
    }

    #[doc = include_str!("shaping_request.doc/font_size_2.md")]
    #[must_use]
    pub const fn font_size(&self) -> f32 {
        self.font_size
    }

    #[doc = include_str!("shaping_request.doc/direction_2.md")]
    #[must_use]
    pub const fn direction(&self) -> TextDirection {
        self.direction
    }

    #[doc = include_str!("shaping_request.doc/language_2.md")]
    #[must_use]
    pub fn language(&self) -> Option<&str> {
        self.language.as_deref()
    }

    #[doc = include_str!("shaping_request.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{FontAttributes, FontFamily, FontMetrics, FontSource, GenericFamily};

    fn mono_face() -> FontFace {
        FontFace::new(
            FontFamily::generic(GenericFamily::Monospace),
            FontAttributes::default(),
            FontSource::system("mono"),
            FontMetrics::new(1000, 800.0, -200.0, 0.0),
        )
    }

    #[test]
    fn new_applies_defaults() {
        let request = ShapingRequest::new("hi", mono_face(), 16.0);
        assert_eq!(request.text(), "hi");
        assert_eq!(request.font_size(), 16.0);
        assert_eq!(request.direction(), TextDirection::LeftToRight);
        assert_eq!(request.language(), None);
        assert!(!request.is_empty());
    }

    #[test]
    fn builders_override_defaults() {
        let request = ShapingRequest::new("שלום", mono_face(), 20.0)
            .with_direction(TextDirection::RightToLeft)
            .with_language("he");
        assert_eq!(request.direction(), TextDirection::RightToLeft);
        assert_eq!(request.language(), Some("he"));
    }

    #[test]
    fn is_empty_tracks_text() {
        assert!(ShapingRequest::new("", mono_face(), 12.0).is_empty());
        assert!(!ShapingRequest::new("x", mono_face(), 12.0).is_empty());
    }
}
