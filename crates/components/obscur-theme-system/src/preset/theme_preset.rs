//! A named bundle of tokens for a visual mode (spec §2.6).

use std::collections::BTreeMap;

use obscur_scene::color::Color;

use crate::token::{ThemeToken, TokenValue};

/// The visual mode a [`ThemePreset`] targets (spec §2.6).
///
/// These mirror the built-in presets the theme system ships: an ordinary light
/// theme, a dark theme, and an accessibility-oriented high-contrast theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeMode {
    /// A light theme: light background, dark foreground.
    #[default]
    Light,
    /// A dark theme: dark background, light foreground.
    Dark,
    /// A high-contrast theme for accessibility.
    HighContrast,
}

impl ThemeMode {
    /// Returns whether this is the high-contrast mode.
    #[must_use]
    pub const fn is_high_contrast(self) -> bool {
        matches!(self, Self::HighContrast)
    }
}

/// A named bundle of design tokens realising a visual mode (spec §2.6).
///
/// A preset is the unit an application selects to theme its UI: it pairs a human
/// name and a [`ThemeMode`] with a set of [`ThemeToken`]s keyed by name. Tokens
/// are stored in a [`BTreeMap`] so iteration and equality are deterministic
/// regardless of insertion order. The built-in [`light`](Self::light),
/// [`dark`](Self::dark) and [`high_contrast`](Self::high_contrast) presets each
/// seed a small, sensible token set.
///
/// The token values carry [`Color`] and `f32` payloads, so this type derives
/// `Debug`, `Clone`, and `PartialEq` only.
#[derive(Debug, Clone, PartialEq)]
pub struct ThemePreset {
    /// The preset's human-readable name (e.g. `"Light"`).
    pub name: Box<str>,
    /// The visual mode this preset targets.
    pub mode: ThemeMode,
    /// The tokens keyed by their namespaced name, in deterministic order.
    pub tokens: BTreeMap<Box<str>, TokenValue>,
}

impl ThemePreset {
    /// Creates an empty preset with the given name and mode.
    #[must_use]
    pub fn new(name: impl Into<Box<str>>, mode: ThemeMode) -> Self {
        Self {
            name: name.into(),
            mode,
            tokens: BTreeMap::new(),
        }
    }

    /// Inserts a token, keyed by its name; replaces any existing token of that name.
    pub fn insert(&mut self, token: ThemeToken) {
        self.tokens.insert(token.name, token.value);
    }

    /// Inserts a token and returns the preset, for fluent construction.
    #[must_use]
    pub fn with(mut self, token: ThemeToken) -> Self {
        self.insert(token);
        self
    }

    /// Returns the preset's name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the preset's visual mode.
    #[must_use]
    pub const fn mode(&self) -> ThemeMode {
        self.mode
    }

    /// Resolves a token by name.
    #[must_use]
    pub fn get(&self, token_name: &str) -> Option<&TokenValue> {
        self.tokens.get(token_name)
    }

    /// Resolves a token as a color, or `None` if absent or not a color.
    #[must_use]
    pub fn color(&self, token_name: &str) -> Option<Color> {
        match self.get(token_name) {
            Some(&TokenValue::Color(color)) => Some(color),
            _ => None,
        }
    }

    /// Resolves a token as a length, or `None` if absent or not a length.
    #[must_use]
    pub fn length(&self, token_name: &str) -> Option<f32> {
        match self.get(token_name) {
            Some(&TokenValue::Length(length)) => Some(length),
            _ => None,
        }
    }

    /// Returns whether a token with the given name is present.
    #[must_use]
    pub fn contains(&self, token_name: &str) -> bool {
        self.tokens.contains_key(token_name)
    }

    /// Returns the number of tokens in the preset.
    #[must_use]
    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    /// Returns whether the preset has no tokens.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    /// Iterates the tokens as `(name, value)` pairs in deterministic name order.
    pub fn tokens(&self) -> impl Iterator<Item = (&str, &TokenValue)> {
        self.tokens.iter().map(|(name, value)| (&**name, value))
    }

    /// The built-in light preset: a light background with a dark foreground.
    #[must_use]
    pub fn light() -> Self {
        Self::new("Light", ThemeMode::Light)
            .with(ThemeToken::color("color.background", Color::rgb(0.98, 0.98, 0.98)))
            .with(ThemeToken::color("color.foreground", Color::rgb(0.1, 0.1, 0.1)))
            .with(ThemeToken::color("color.primary", Color::rgb(0.2, 0.45, 0.9)))
            .with(ThemeToken::length("space.sm", 4.0))
            .with(ThemeToken::length("space.md", 8.0))
            .with(ThemeToken::length("space.lg", 16.0))
            .with(ThemeToken::length("radius.sm", 4.0))
            .with(ThemeToken::font_family("font.body", "sans-serif"))
    }

    /// The built-in dark preset: a dark background with a light foreground.
    #[must_use]
    pub fn dark() -> Self {
        Self::new("Dark", ThemeMode::Dark)
            .with(ThemeToken::color("color.background", Color::rgb(0.11, 0.11, 0.12)))
            .with(ThemeToken::color("color.foreground", Color::rgb(0.92, 0.92, 0.92)))
            .with(ThemeToken::color("color.primary", Color::rgb(0.4, 0.65, 1.0)))
            .with(ThemeToken::length("space.sm", 4.0))
            .with(ThemeToken::length("space.md", 8.0))
            .with(ThemeToken::length("space.lg", 16.0))
            .with(ThemeToken::length("radius.sm", 4.0))
            .with(ThemeToken::font_family("font.body", "sans-serif"))
    }

    /// The built-in high-contrast preset: pure black/white with a saturated primary.
    #[must_use]
    pub fn high_contrast() -> Self {
        Self::new("High Contrast", ThemeMode::HighContrast)
            .with(ThemeToken::color("color.background", Color::BLACK))
            .with(ThemeToken::color("color.foreground", Color::WHITE))
            .with(ThemeToken::color("color.primary", Color::rgb(1.0, 0.85, 0.0)))
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

        preset.insert(ThemeToken::color("color.primary", Color::rgb(0.2, 0.4, 0.8)));
        preset.insert(ThemeToken::length("space.md", 8.0));

        assert!(!preset.is_empty());
        assert_eq!(preset.len(), 2);
        assert!(preset.contains("color.primary"));
        assert!(!preset.contains("color.missing"));

        assert_eq!(preset.color("color.primary"), Some(Color::rgb(0.2, 0.4, 0.8)));
        assert_eq!(preset.length("space.md"), Some(8.0));
        // Typed resolve rejects a mismatched kind.
        assert_eq!(preset.color("space.md"), None);
        assert_eq!(preset.length("color.primary"), None);
        assert_eq!(preset.get("color.primary"), Some(&TokenValue::Color(Color::rgb(0.2, 0.4, 0.8))));
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
        for preset in [ThemePreset::light(), ThemePreset::dark(), ThemePreset::high_contrast()] {
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
