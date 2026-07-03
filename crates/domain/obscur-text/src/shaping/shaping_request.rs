//! The input describing one uniformly-styled run to be shaped (spec §5.10).

use crate::font::FontFace;
use crate::shaping::TextDirection;

/// The input to shape a single run of uniformly-styled text (spec §5.10).
///
/// A request carries the text plus everything a shaper needs to turn it into
/// glyphs: the resolved [`FontFace`], the font size in logical pixels, the base
/// [`TextDirection`], and an optional BCP-47 language hint. The run is assumed to
/// be **already itemized**: a single face and a single direction apply to the
/// whole text. Splitting mixed-script, mixed-style, or mixed-direction text into
/// uniform runs (itemization and bidi splitting) happens *before* shaping, so the
/// shaper never has to re-segment its input.
///
/// Embedding an `f32` font size means this type derives `PartialEq` but not
/// `Eq`.
#[derive(Debug, Clone, PartialEq)]
pub struct ShapingRequest {
    /// The run's source text, owned so the request is self-contained.
    text: Box<str>,
    /// The resolved face to shape with.
    face: FontFace,
    /// The font size, in logical pixels.
    font_size: f32,
    /// The base (paragraph) direction of the run.
    direction: TextDirection,
    /// An optional BCP-47 language tag hint; `None` means auto-detect.
    language: Option<Box<str>>,
}

impl ShapingRequest {
    /// Creates a request to shape `text` with `face` at `font_size` logical
    /// pixels, defaulting the direction to [`TextDirection::LeftToRight`] and the
    /// language hint to `None`.
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

    /// Returns this request with its base [`TextDirection`] set to `direction`.
    #[must_use]
    pub fn with_direction(mut self, direction: TextDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Returns this request with its BCP-47 `language` hint set.
    #[must_use]
    pub fn with_language(mut self, language: impl Into<Box<str>>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// Returns the run's source text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the resolved face to shape with.
    #[must_use]
    pub const fn face(&self) -> &FontFace {
        &self.face
    }

    /// Returns the font size, in logical pixels.
    #[must_use]
    pub const fn font_size(&self) -> f32 {
        self.font_size
    }

    /// Returns the base (paragraph) direction of the run.
    #[must_use]
    pub const fn direction(&self) -> TextDirection {
        self.direction
    }

    /// Returns the BCP-47 language tag hint, or `None` for auto-detect.
    #[must_use]
    pub fn language(&self) -> Option<&str> {
        self.language.as_deref()
    }

    /// Returns whether the run's source text is empty.
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
