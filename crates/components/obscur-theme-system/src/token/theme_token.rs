#![doc = include_str!("theme_token.doc/_module.md")]

use obscur_scene::color::Color;

#[doc = include_str!("theme_token.doc/TokenValue.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum TokenValue {
    #[doc = include_str!("theme_token.doc/Color.md")]
    Color(Color),
    #[doc = include_str!("theme_token.doc/Length.md")]
    Length(f32),
    #[doc = include_str!("theme_token.doc/Number.md")]
    Number(f32),
    #[doc = include_str!("theme_token.doc/Text.md")]
    Text(Box<str>),
    #[doc = include_str!("theme_token.doc/FontFamily.md")]
    FontFamily(Box<str>),
    #[doc = include_str!("theme_token.doc/DurationMillis.md")]
    DurationMillis(u32),
}

#[doc = include_str!("theme_token.doc/ThemeToken.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct ThemeToken {
    #[doc = include_str!("theme_token.doc/name.md")]
    pub name: Box<str>,
    #[doc = include_str!("theme_token.doc/value.md")]
    pub value: TokenValue,
}

impl ThemeToken {
    #[doc = include_str!("theme_token.doc/new.md")]
    #[must_use]
    pub fn new(name: impl Into<Box<str>>, value: TokenValue) -> Self {
        Self {
            name: name.into(),
            value,
        }
    }

    #[doc = include_str!("theme_token.doc/color.md")]
    #[must_use]
    pub fn color(name: impl Into<Box<str>>, color: Color) -> Self {
        Self::new(name, TokenValue::Color(color))
    }

    #[doc = include_str!("theme_token.doc/length.md")]
    #[must_use]
    pub fn length(name: impl Into<Box<str>>, length: f32) -> Self {
        Self::new(name, TokenValue::Length(length))
    }

    #[doc = include_str!("theme_token.doc/number.md")]
    #[must_use]
    pub fn number(name: impl Into<Box<str>>, number: f32) -> Self {
        Self::new(name, TokenValue::Number(number))
    }

    #[doc = include_str!("theme_token.doc/text.md")]
    #[must_use]
    pub fn text(name: impl Into<Box<str>>, text: impl Into<Box<str>>) -> Self {
        Self::new(name, TokenValue::Text(text.into()))
    }

    #[doc = include_str!("theme_token.doc/font_family.md")]
    #[must_use]
    pub fn font_family(name: impl Into<Box<str>>, family: impl Into<Box<str>>) -> Self {
        Self::new(name, TokenValue::FontFamily(family.into()))
    }

    #[doc = include_str!("theme_token.doc/duration_millis.md")]
    #[must_use]
    pub fn duration_millis(name: impl Into<Box<str>>, millis: u32) -> Self {
        Self::new(name, TokenValue::DurationMillis(millis))
    }

    #[doc = include_str!("theme_token.doc/name_2.md")]
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[doc = include_str!("theme_token.doc/value_2.md")]
    #[must_use]
    pub fn value(&self) -> &TokenValue {
        &self.value
    }

    #[doc = include_str!("theme_token.doc/as_color.md")]
    #[must_use]
    pub fn as_color(&self) -> Option<Color> {
        match self.value {
            TokenValue::Color(color) => Some(color),
            _ => None,
        }
    }

    #[doc = include_str!("theme_token.doc/as_length.md")]
    #[must_use]
    pub fn as_length(&self) -> Option<f32> {
        match self.value {
            TokenValue::Length(length) => Some(length),
            _ => None,
        }
    }

    #[doc = include_str!("theme_token.doc/as_text.md")]
    #[must_use]
    pub fn as_text(&self) -> Option<&str> {
        match &self.value {
            TokenValue::Text(text) | TokenValue::FontFamily(text) => Some(text),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_constructor_and_getter() {
        let token = ThemeToken::color("color.primary", Color::rgb(0.1, 0.2, 0.3));
        assert_eq!(token.name(), "color.primary");
        assert_eq!(token.as_color(), Some(Color::rgb(0.1, 0.2, 0.3)));
        assert_eq!(token.as_length(), None);
        assert_eq!(token.as_text(), None);
        assert_eq!(token.value(), &TokenValue::Color(Color::rgb(0.1, 0.2, 0.3)));
    }

    #[test]
    fn length_and_number_constructors() {
        let length = ThemeToken::length("space.md", 8.0);
        assert_eq!(length.as_length(), Some(8.0));
        assert_eq!(length.as_color(), None);

        let number = ThemeToken::number("scale", 1.5);
        assert_eq!(number.value(), &TokenValue::Number(1.5));
        // A number is not a length.
        assert_eq!(number.as_length(), None);
    }

    #[test]
    fn text_and_font_family_getters() {
        let text = ThemeToken::text("label", "hello");
        assert_eq!(text.as_text(), Some("hello"));
        assert_eq!(text.as_color(), None);

        let font = ThemeToken::font_family("font.body", "sans-serif");
        assert_eq!(font.as_text(), Some("sans-serif"));
    }

    #[test]
    fn duration_and_new() {
        let duration = ThemeToken::duration_millis("motion.fast", 120);
        assert_eq!(duration.value(), &TokenValue::DurationMillis(120));
        assert_eq!(duration.as_length(), None);

        let explicit = ThemeToken::new("radius.sm", TokenValue::Length(4.0));
        assert_eq!(explicit.as_length(), Some(4.0));
    }
}
