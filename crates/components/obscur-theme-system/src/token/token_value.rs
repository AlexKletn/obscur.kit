use obscur_scene::color::Color;

#[doc = include_str!("../../docs/api/token/theme_token.doc/TokenValue.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum TokenValue {
    /// An sRGB color value (e.g. `color.primary`).
    Color(Color),
    /// A length in logical pixels (e.g. `space.md`, `radius.sm`).
    Length(f32),
    /// A unitless number (e.g. a scale factor or opacity).
    Number(f32),
    /// A free-text value (e.g. a label or opaque identifier).
    Text(Box<str>),
    /// A font-family name (e.g. `"sans-serif"`).
    FontFamily(Box<str>),
    /// An animation/transition duration in milliseconds.
    DurationMillis(u32),
}
