#![doc = include_str!("theme_preset.doc/_module.md")]

use std::collections::BTreeMap;

use obscur_scene::color::Color;

use crate::token::{ThemeToken, TokenValue};

#[doc = include_str!("theme_preset.doc/ThemeMode.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeMode {
    #[doc = include_str!("theme_preset.doc/Light.md")]
    #[default]
    Light,
    #[doc = include_str!("theme_preset.doc/Dark.md")]
    Dark,
    #[doc = include_str!("theme_preset.doc/HighContrast.md")]
    HighContrast,
}

impl ThemeMode {
    #[doc = include_str!("theme_preset.doc/is_high_contrast.md")]
    #[must_use]
    pub const fn is_high_contrast(self) -> bool {
        matches!(self, Self::HighContrast)
    }
}

#[doc = include_str!("theme_preset.doc/ThemePreset.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct ThemePreset {
    #[doc = include_str!("theme_preset.doc/name.md")]
    pub name: Box<str>,
    #[doc = include_str!("theme_preset.doc/mode.md")]
    pub mode: ThemeMode,
    #[doc = include_str!("theme_preset.doc/tokens.md")]
    pub tokens: BTreeMap<Box<str>, TokenValue>,
}

impl ThemePreset {
    #[doc = include_str!("theme_preset.doc/new.md")]
    #[must_use]
    pub fn new(name: impl Into<Box<str>>, mode: ThemeMode) -> Self {
        Self {
            name: name.into(),
            mode,
            tokens: BTreeMap::new(),
        }
    }

    #[doc = include_str!("theme_preset.doc/insert.md")]
    pub fn insert(&mut self, token: ThemeToken) {
        self.tokens.insert(token.name, token.value);
    }

    #[doc = include_str!("theme_preset.doc/with.md")]
    #[must_use]
    pub fn with(mut self, token: ThemeToken) -> Self {
        self.insert(token);
        self
    }

    #[doc = include_str!("theme_preset.doc/name_2.md")]
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[doc = include_str!("theme_preset.doc/mode_2.md")]
    #[must_use]
    pub const fn mode(&self) -> ThemeMode {
        self.mode
    }

    #[doc = include_str!("theme_preset.doc/get.md")]
    #[must_use]
    pub fn get(&self, token_name: &str) -> Option<&TokenValue> {
        self.tokens.get(token_name)
    }

    #[doc = include_str!("theme_preset.doc/color.md")]
    #[must_use]
    pub fn color(&self, token_name: &str) -> Option<Color> {
        match self.get(token_name) {
            Some(&TokenValue::Color(color)) => Some(color),
            _ => None,
        }
    }

    #[doc = include_str!("theme_preset.doc/length.md")]
    #[must_use]
    pub fn length(&self, token_name: &str) -> Option<f32> {
        match self.get(token_name) {
            Some(&TokenValue::Length(length)) => Some(length),
            _ => None,
        }
    }

    #[doc = include_str!("theme_preset.doc/contains.md")]
    #[must_use]
    pub fn contains(&self, token_name: &str) -> bool {
        self.tokens.contains_key(token_name)
    }

    #[doc = include_str!("theme_preset.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    #[doc = include_str!("theme_preset.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    #[doc = include_str!("theme_preset.doc/tokens_2.md")]
    pub fn tokens(&self) -> impl Iterator<Item = (&str, &TokenValue)> {
        self.tokens.iter().map(|(name, value)| (&**name, value))
    }

    #[doc = include_str!("theme_preset.doc/light.md")]
    #[must_use]
    pub fn light() -> Self {
        Self::new("Light", ThemeMode::Light)
            .with(ThemeToken::color(
                "color.background",
                Color::rgb(0.98, 0.98, 0.98),
            ))
            .with(ThemeToken::color(
                "color.foreground",
                Color::rgb(0.1, 0.1, 0.1),
            ))
            .with(ThemeToken::color(
                "color.primary",
                Color::rgb(0.2, 0.45, 0.9),
            ))
            .with(ThemeToken::length("space.sm", 4.0))
            .with(ThemeToken::length("space.md", 8.0))
            .with(ThemeToken::length("space.lg", 16.0))
            .with(ThemeToken::length("radius.sm", 4.0))
            .with(ThemeToken::font_family("font.body", "sans-serif"))
    }

    #[doc = include_str!("theme_preset.doc/dark.md")]
    #[must_use]
    pub fn dark() -> Self {
        Self::new("Dark", ThemeMode::Dark)
            .with(ThemeToken::color(
                "color.background",
                Color::rgb(0.11, 0.11, 0.12),
            ))
            .with(ThemeToken::color(
                "color.foreground",
                Color::rgb(0.92, 0.92, 0.92),
            ))
            .with(ThemeToken::color(
                "color.primary",
                Color::rgb(0.4, 0.65, 1.0),
            ))
            .with(ThemeToken::length("space.sm", 4.0))
            .with(ThemeToken::length("space.md", 8.0))
            .with(ThemeToken::length("space.lg", 16.0))
            .with(ThemeToken::length("radius.sm", 4.0))
            .with(ThemeToken::font_family("font.body", "sans-serif"))
    }

    #[doc = include_str!("theme_preset.doc/high_contrast.md")]
    #[must_use]
    pub fn high_contrast() -> Self {
        Self::new("High Contrast", ThemeMode::HighContrast)
            .with(ThemeToken::color("color.background", Color::BLACK))
            .with(ThemeToken::color("color.foreground", Color::WHITE))
            .with(ThemeToken::color(
                "color.primary",
                Color::rgb(1.0, 0.85, 0.0),
            ))
            .with(ThemeToken::length("space.sm", 4.0))
            .with(ThemeToken::length("space.md", 8.0))
            .with(ThemeToken::length("space.lg", 16.0))
            .with(ThemeToken::length("radius.sm", 4.0))
            .with(ThemeToken::font_family("font.body", "sans-serif"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_default_and_high_contrast() {
        assert_eq!(ThemeMode::default(), ThemeMode::Light);
        assert!(ThemeMode::HighContrast.is_high_contrast());
        assert!(!ThemeMode::Light.is_high_contrast());
    }

    #[test]
    fn insert_get_and_typed_resolve() {
        let mut preset = ThemePreset::new("Custom", ThemeMode::Light);
        assert!(preset.is_empty());

        preset.insert(ThemeToken::color(
            "color.primary",
            Color::rgb(0.2, 0.4, 0.8),
        ));
        preset.insert(ThemeToken::length("space.md", 8.0));

        assert!(!preset.is_empty());
        assert_eq!(preset.len(), 2);
        assert!(preset.contains("color.primary"));
        assert!(!preset.contains("color.missing"));

        assert_eq!(
            preset.color("color.primary"),
            Some(Color::rgb(0.2, 0.4, 0.8))
        );
        assert_eq!(preset.length("space.md"), Some(8.0));
        // Typed resolve rejects a mismatched kind.
        assert_eq!(preset.color("space.md"), None);
        assert_eq!(preset.length("color.primary"), None);
        assert_eq!(
            preset.get("color.primary"),
            Some(&TokenValue::Color(Color::rgb(0.2, 0.4, 0.8)))
        );
        assert_eq!(preset.get("absent"), None);
    }

    #[test]
    fn with_is_fluent_and_replaces() {
        let preset = ThemePreset::new("Custom", ThemeMode::Dark)
            .with(ThemeToken::length("space.md", 4.0))
            .with(ThemeToken::length("space.md", 8.0));
        // Later insert of the same name wins.
        assert_eq!(preset.length("space.md"), Some(8.0));
        assert_eq!(preset.len(), 1);
    }

    #[test]
    fn tokens_iter_is_deterministic() {
        let preset = ThemePreset::new("Custom", ThemeMode::Light)
            .with(ThemeToken::length("space.md", 8.0))
            .with(ThemeToken::length("radius.sm", 4.0))
            .with(ThemeToken::length("space.sm", 4.0));
        let names: Vec<&str> = preset.tokens().map(|(name, _)| name).collect();
        assert_eq!(names, ["radius.sm", "space.md", "space.sm"]);
    }

    #[test]
    fn builtin_presets_expose_core_tokens() {
        for preset in [
            ThemePreset::light(),
            ThemePreset::dark(),
            ThemePreset::high_contrast(),
        ] {
            assert!(preset.color("color.background").is_some());
            assert!(preset.color("color.foreground").is_some());
            assert_eq!(preset.length("space.md"), Some(8.0));
        }
    }

    #[test]
    fn builtin_modes_and_names() {
        assert_eq!(ThemePreset::light().mode(), ThemeMode::Light);
        assert_eq!(ThemePreset::dark().mode(), ThemeMode::Dark);
        assert_eq!(ThemePreset::high_contrast().mode(), ThemeMode::HighContrast);
        assert_eq!(ThemePreset::light().name(), "Light");
    }

    #[test]
    fn light_and_dark_backgrounds_differ() {
        let light_bg = ThemePreset::light().color("color.background");
        let dark_bg = ThemePreset::dark().color("color.background");
        assert!(light_bg.is_some());
        assert!(dark_bg.is_some());
        assert_ne!(light_bg, dark_bg);
    }

    #[test]
    fn high_contrast_uses_pure_black_and_white() {
        let preset = ThemePreset::high_contrast();
        assert_eq!(preset.color("color.background"), Some(Color::BLACK));
        assert_eq!(preset.color("color.foreground"), Some(Color::WHITE));
    }
}
