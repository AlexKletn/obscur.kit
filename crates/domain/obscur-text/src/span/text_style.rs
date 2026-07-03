//! The typography of a rich text span (spec §5.10).

/// A CSS-like font weight on the `1..=1000` axis (spec §5.10).
///
/// This is a small, self-contained value type; the text domain models
/// typography without depending on the CSS/style domain. The named constants
/// mirror the common CSS keyword weights (`THIN`, `NORMAL`, `BOLD`, `BLACK`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FontWeight(u16);

impl FontWeight {
    /// Thin weight (`100`).
    pub const THIN: FontWeight = FontWeight(100);
    /// Extra-light weight (`200`).
    pub const EXTRA_LIGHT: FontWeight = FontWeight(200);
    /// Light weight (`300`).
    pub const LIGHT: FontWeight = FontWeight(300);
    /// Normal (regular) weight (`400`); the default.
    pub const NORMAL: FontWeight = FontWeight(400);
    /// Medium weight (`500`).
    pub const MEDIUM: FontWeight = FontWeight(500);
    /// Semi-bold weight (`600`).
    pub const SEMI_BOLD: FontWeight = FontWeight(600);
    /// Bold weight (`700`).
    pub const BOLD: FontWeight = FontWeight(700);
    /// Extra-bold weight (`800`).
    pub const EXTRA_BOLD: FontWeight = FontWeight(800);
    /// Black (heavy) weight (`900`).
    pub const BLACK: FontWeight = FontWeight(900);

    /// Creates a weight from a raw value, clamped to the valid `1..=1000` axis.
    ///
    /// Values of `0` or above `1000` are clamped rather than rejected, so this
    /// never panics.
    #[must_use]
    pub const fn new(value: u16) -> Self {
        let clamped = if value < 1 {
            1
        } else if value > 1000 {
            1000
        } else {
            value
        };
        FontWeight(clamped)
    }

    /// Returns the raw weight value on the `1..=1000` axis.
    #[must_use]
    pub const fn value(self) -> u16 {
        self.0
    }
}

impl Default for FontWeight {
    /// The default weight is [`FontWeight::NORMAL`].
    fn default() -> Self {
        Self::NORMAL
    }
}

/// The slant (posture) of a font (spec §5.10).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FontSlant {
    /// Upright glyphs.
    Normal,
    /// A true italic design (distinct glyph shapes).
    Italic,
    /// A slanted (obliqued) rendering of the upright design.
    Oblique,
}

impl Default for FontSlant {
    /// The default slant is [`FontSlant::Normal`].
    fn default() -> Self {
        Self::Normal
    }
}

/// The typography of a rich text span (spec §5.10).
///
/// A `TextStyle` describes how a run of text is drawn: its font family, size,
/// weight, slant, spacing, and line height. It is a pure, backend-agnostic value
/// type independent of the CSS/style domain; the spec's CSS typography
/// integration lives in an adapter that maps a computed style onto this type,
/// not the other way around.
///
/// Because it holds `f32` fields it derives only `Debug`, `Clone`, and
/// `PartialEq` (floats preclude `Eq`).
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
    fn font_weight_consts() {
        assert_eq!(FontWeight::THIN.value(), 100);
        assert_eq!(FontWeight::NORMAL.value(), 400);
        assert_eq!(FontWeight::BOLD.value(), 700);
        assert_eq!(FontWeight::BLACK.value(), 900);
        assert_eq!(FontWeight::default(), FontWeight::NORMAL);
    }

    #[test]
    fn font_weight_new_clamps() {
        assert_eq!(FontWeight::new(0).value(), 1);
        assert_eq!(FontWeight::new(1).value(), 1);
        assert_eq!(FontWeight::new(450).value(), 450);
        assert_eq!(FontWeight::new(1000).value(), 1000);
        assert_eq!(FontWeight::new(5000).value(), 1000);
    }

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

    #[test]
    fn font_slant_default() {
        assert_eq!(FontSlant::default(), FontSlant::Normal);
    }
}
