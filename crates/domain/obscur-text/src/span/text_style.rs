#![doc = include_str!("text_style.doc/_module.md")]

#[doc = include_str!("text_style.doc/FontWeight.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FontWeight(u16);

impl FontWeight {
    #[doc = include_str!("text_style.doc/THIN.md")]
    pub const THIN: FontWeight = FontWeight(100);
    #[doc = include_str!("text_style.doc/EXTRA_LIGHT.md")]
    pub const EXTRA_LIGHT: FontWeight = FontWeight(200);
    #[doc = include_str!("text_style.doc/LIGHT.md")]
    pub const LIGHT: FontWeight = FontWeight(300);
    #[doc = include_str!("text_style.doc/NORMAL.md")]
    pub const NORMAL: FontWeight = FontWeight(400);
    #[doc = include_str!("text_style.doc/MEDIUM.md")]
    pub const MEDIUM: FontWeight = FontWeight(500);
    #[doc = include_str!("text_style.doc/SEMI_BOLD.md")]
    pub const SEMI_BOLD: FontWeight = FontWeight(600);
    #[doc = include_str!("text_style.doc/BOLD.md")]
    pub const BOLD: FontWeight = FontWeight(700);
    #[doc = include_str!("text_style.doc/EXTRA_BOLD.md")]
    pub const EXTRA_BOLD: FontWeight = FontWeight(800);
    #[doc = include_str!("text_style.doc/BLACK.md")]
    pub const BLACK: FontWeight = FontWeight(900);

    #[doc = include_str!("text_style.doc/new.md")]
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

    #[doc = include_str!("text_style.doc/value.md")]
    #[must_use]
    pub const fn value(self) -> u16 {
        self.0
    }
}

impl Default for FontWeight {
    #[doc = include_str!("text_style.doc/default.md")]
    fn default() -> Self {
        Self::NORMAL
    }
}

#[doc = include_str!("text_style.doc/FontSlant.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FontSlant {
    #[doc = include_str!("text_style.doc/Normal.md")]
    Normal,
    #[doc = include_str!("text_style.doc/Italic.md")]
    Italic,
    #[doc = include_str!("text_style.doc/Oblique.md")]
    Oblique,
}

impl Default for FontSlant {
    #[doc = include_str!("text_style.doc/default_2.md")]
    fn default() -> Self {
        Self::Normal
    }
}

#[doc = include_str!("text_style.doc/TextStyle.md")]
//
// Color is intentionally NOT modeled here: text color is applied at paint time
// via the scene, keeping text layout color-agnostic.
#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    #[doc = include_str!("text_style.doc/font_family.md")]
    pub font_family: Box<str>,
    #[doc = include_str!("text_style.doc/font_size.md")]
    pub font_size: f32,
    #[doc = include_str!("text_style.doc/weight.md")]
    pub weight: FontWeight,
    #[doc = include_str!("text_style.doc/slant.md")]
    pub slant: FontSlant,
    #[doc = include_str!("text_style.doc/letter_spacing.md")]
    pub letter_spacing: f32,
    #[doc = include_str!("text_style.doc/word_spacing.md")]
    pub word_spacing: f32,
    #[doc = include_str!("text_style.doc/line_height.md")]
    pub line_height: Option<f32>,
}

impl TextStyle {
    #[doc = include_str!("text_style.doc/DEFAULT_FONT_SIZE.md")]
    pub const DEFAULT_FONT_SIZE: f32 = 16.0;

    #[doc = include_str!("text_style.doc/new_2.md")]
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

    #[doc = include_str!("text_style.doc/with_size.md")]
    #[must_use]
    pub fn with_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self
    }

    #[doc = include_str!("text_style.doc/with_weight.md")]
    #[must_use]
    pub fn with_weight(mut self, weight: FontWeight) -> Self {
        self.weight = weight;
        self
    }

    #[doc = include_str!("text_style.doc/with_slant.md")]
    #[must_use]
    pub fn with_slant(mut self, slant: FontSlant) -> Self {
        self.slant = slant;
        self
    }

    #[doc = include_str!("text_style.doc/with_letter_spacing.md")]
    #[must_use]
    pub fn with_letter_spacing(mut self, letter_spacing: f32) -> Self {
        self.letter_spacing = letter_spacing;
        self
    }

    #[doc = include_str!("text_style.doc/with_word_spacing.md")]
    #[must_use]
    pub fn with_word_spacing(mut self, word_spacing: f32) -> Self {
        self.word_spacing = word_spacing;
        self
    }

    #[doc = include_str!("text_style.doc/with_line_height.md")]
    #[must_use]
    pub fn with_line_height(mut self, line_height: f32) -> Self {
        self.line_height = Some(line_height);
        self
    }

    #[doc = include_str!("text_style.doc/font_family_2.md")]
    #[must_use]
    pub fn font_family(&self) -> &str {
        &self.font_family
    }

    #[doc = include_str!("text_style.doc/font_size_2.md")]
    #[must_use]
    pub const fn font_size(&self) -> f32 {
        self.font_size
    }

    #[doc = include_str!("text_style.doc/weight_2.md")]
    #[must_use]
    pub const fn weight(&self) -> FontWeight {
        self.weight
    }

    #[doc = include_str!("text_style.doc/slant_2.md")]
    #[must_use]
    pub const fn slant(&self) -> FontSlant {
        self.slant
    }

    #[doc = include_str!("text_style.doc/letter_spacing_2.md")]
    #[must_use]
    pub const fn letter_spacing(&self) -> f32 {
        self.letter_spacing
    }

    #[doc = include_str!("text_style.doc/word_spacing_2.md")]
    #[must_use]
    pub const fn word_spacing(&self) -> f32 {
        self.word_spacing
    }

    #[doc = include_str!("text_style.doc/line_height_2.md")]
    #[must_use]
    pub const fn line_height(&self) -> Option<f32> {
        self.line_height
    }
}

impl Default for TextStyle {
    #[doc = include_str!("text_style.doc/default_3.md")]
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
