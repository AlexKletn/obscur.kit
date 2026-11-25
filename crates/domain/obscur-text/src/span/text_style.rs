//! The typography of a rich text span (spec §5.10).

use super::{FontSlant, FontWeight};

#[doc = include_str!("../../docs/api/span/text_style.doc/TextStyle.md")]
//
// Color is intentionally NOT modeled here: text color is applied at paint time
// via the scene, keeping text layout color-agnostic.
#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    /// The primary font family name; the fallback list lives in the font module.
    pub font_family: Box<str>,
    /// The font size in logical pixels (default `16.0`).
    pub font_size: f32,
    /// The font weight (default [`FontWeight::NORMAL`]).
    pub weight: FontWeight,
    /// The font slant (default [`FontSlant::Normal`]).
    pub slant: FontSlant,
    /// Extra spacing added between characters in logical pixels (default `0.0`).
    pub letter_spacing: f32,
    /// Extra spacing added between words in logical pixels (default `0.0`).
    pub word_spacing: f32,
    /// The line height in logical pixels, or `None` to use the font's own
    /// metrics.
    pub line_height: Option<f32>,
}

impl TextStyle {
    /// The default font size in logical pixels.
    pub const DEFAULT_FONT_SIZE: f32 = 16.0;

    /// Creates a style for `font_family` with every other field at its default.
    #[must_use]
    pub fn new(font_family: impl Into<Box<str>>) -> Self {
        Self {
            font_family: font_family.into(),
            font_size: Self::DEFAULT_FONT_SIZE,
            weight: FontWeight::NORMAL,
            slant: FontSlant::Normal,
            letter_spacing: 0.0,
            word_spacing: 0.0,
            line_height: None,
        }
    }

    /// Returns this style with its font size set, in logical pixels.
    #[must_use]
    pub fn with_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self
    }

    /// Returns this style with its font weight set.
    #[must_use]
    pub fn with_weight(mut self, weight: FontWeight) -> Self {
        self.weight = weight;
        self
    }

    /// Returns this style with its font slant set.
    #[must_use]
    pub fn with_slant(mut self, slant: FontSlant) -> Self {
        self.slant = slant;
        self
    }

    /// Returns this style with its letter spacing set, in logical pixels.
    #[must_use]
    pub fn with_letter_spacing(mut self, letter_spacing: f32) -> Self {
        self.letter_spacing = letter_spacing;
        self
    }

    /// Returns this style with its word spacing set, in logical pixels.
    #[must_use]
    pub fn with_word_spacing(mut self, word_spacing: f32) -> Self {
        self.word_spacing = word_spacing;
        self
    }

    /// Returns this style with an explicit line height set, in logical pixels.
    #[must_use]
    pub fn with_line_height(mut self, line_height: f32) -> Self {
        self.line_height = Some(line_height);
        self
    }

    /// Returns the primary font family name.
    #[must_use]
    pub fn font_family(&self) -> &str {
        &self.font_family
    }

    /// Returns the font size in logical pixels.
    #[must_use]
    pub const fn font_size(&self) -> f32 {
        self.font_size
    }

    /// Returns the font weight.
    #[must_use]
    pub const fn weight(&self) -> FontWeight {
        self.weight
    }

    /// Returns the font slant.
    #[must_use]
    pub const fn slant(&self) -> FontSlant {
        self.slant
    }

    /// Returns the letter spacing in logical pixels.
    #[must_use]
    pub const fn letter_spacing(&self) -> f32 {
        self.letter_spacing
    }

    /// Returns the word spacing in logical pixels.
    #[must_use]
    pub const fn word_spacing(&self) -> f32 {
        self.word_spacing
    }

    /// Returns the explicit line height in logical pixels, or `None` to use the
    /// font's own metrics.
    #[must_use]
    pub const fn line_height(&self) -> Option<f32> {
        self.line_height
    }
}

impl Default for TextStyle {
    /// The default style: the `"sans-serif"` family at the default size, normal
    /// weight and slant, no extra spacing, and font-metric line height.
    fn default() -> Self {
        Self::new("sans-serif")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let s = TextStyle::default();
        assert_eq!(s.font_family(), "sans-serif");
        assert_eq!(s.font_size(), 16.0);
        assert_eq!(s.weight(), FontWeight::NORMAL);
        assert_eq!(s.slant(), FontSlant::Normal);
        assert_eq!(s.letter_spacing(), 0.0);
        assert_eq!(s.word_spacing(), 0.0);
        assert_eq!(s.line_height(), None);
    }

    #[test]
    fn builder_chaining() {
        let s = TextStyle::new("Inter")
            .with_size(20.0)
            .with_weight(FontWeight::BOLD)
            .with_slant(FontSlant::Italic)
            .with_letter_spacing(0.5)
            .with_word_spacing(1.5)
            .with_line_height(28.0);

        assert_eq!(s.font_family(), "Inter");
        assert_eq!(s.font_size(), 20.0);
        assert_eq!(s.weight(), FontWeight::BOLD);
        assert_eq!(s.slant(), FontSlant::Italic);
        assert_eq!(s.letter_spacing(), 0.5);
        assert_eq!(s.word_spacing(), 1.5);
        assert_eq!(s.line_height(), Some(28.0));
    }
}
