/// A CSS generic font family: an abstract category a provider resolves to a
/// concrete family (spec §5.18).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GenericFamily {
    /// Fonts with serifs (e.g. Times).
    Serif,
    /// Fonts without serifs (e.g. Helvetica); the usual UI default.
    SansSerif,
    /// Fixed-pitch fonts (e.g. Courier).
    Monospace,
    /// Handwriting-style fonts.
    Cursive,
    /// Decorative / display fonts.
    Fantasy,
    /// The platform's default UI font.
    SystemUi,
}

impl GenericFamily {
    /// Returns the canonical CSS keyword for this generic family (e.g.
    /// [`GenericFamily::SansSerif`] → `"sans-serif"`), used as its fallback name
    /// when no concrete family is resolved (spec §5.18).
    #[must_use]
    pub const fn fallback_name(&self) -> &'static str {
        match self {
            Self::Serif => "serif",
            Self::SansSerif => "sans-serif",
            Self::Monospace => "monospace",
            Self::Cursive => "cursive",
            Self::Fantasy => "fantasy",
            Self::SystemUi => "system-ui",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_names() {
        assert_eq!(GenericFamily::Serif.fallback_name(), "serif");
        assert_eq!(GenericFamily::SansSerif.fallback_name(), "sans-serif");
        assert_eq!(GenericFamily::Monospace.fallback_name(), "monospace");
        assert_eq!(GenericFamily::Cursive.fallback_name(), "cursive");
        assert_eq!(GenericFamily::Fantasy.fallback_name(), "fantasy");
        assert_eq!(GenericFamily::SystemUi.fallback_name(), "system-ui");
    }
}
