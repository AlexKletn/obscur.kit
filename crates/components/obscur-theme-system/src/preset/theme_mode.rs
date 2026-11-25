#[doc = include_str!("../../docs/api/preset/theme_preset.doc/ThemeMode.md")]
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
