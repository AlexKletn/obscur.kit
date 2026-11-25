/// Screen media preferences used by the style matcher. The default is light with no motion or contrast preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MediaPreferences {
    /// Whether the host prefers the dark color scheme.
    pub dark: bool,
    /// Whether the host requests less animation.
    pub reduced_motion: bool,
    /// Whether the host requests increased contrast.
    pub increased_contrast: bool,
}
