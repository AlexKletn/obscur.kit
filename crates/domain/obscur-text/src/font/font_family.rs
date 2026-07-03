//! A requested font family — a specific name or a CSS generic (spec §5.18).

/// A requested font family: either a specific named family or a CSS generic
/// family keyword (spec §5.18).
///
/// This models what an application *asks for*, not a concrete resolved face. A
/// [`FontProvider`](crate::font::FontProvider) turns a `FontFamily` (together
/// with [`FontAttributes`](crate::font::FontAttributes)) into a
/// [`FontFace`](crate::font::FontFace); a generic family names an abstract
/// category the provider maps onto whatever concrete family it has for that
/// role.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FontFamily {
    /// A specific family requested by name (e.g. `"Inter"`, `"Helvetica"`).
    Named(Box<str>),
    /// A CSS generic family keyword the provider resolves to a concrete family.
    Generic(GenericFamily),
}

impl FontFamily {
    /// Creates a [`FontFamily::Named`] from `name`.
    #[must_use]
    pub fn named(name: impl Into<Box<str>>) -> Self {
        Self::Named(name.into())
    }

    /// Creates a [`FontFamily::Generic`] from `generic`.
    #[must_use]
    pub const fn generic(generic: GenericFamily) -> Self {
        Self::Generic(generic)
    }

    /// Parses a family from a name, mapping the well-known CSS generic keywords
    /// (`"serif"`, `"sans-serif"`, `"monospace"`, `"cursive"`, `"fantasy"`,
    /// `"system-ui"`) to [`FontFamily::Generic`] and everything else to
    /// [`FontFamily::Named`] (spec §5.18).
    ///
    /// Matching is case-sensitive against the canonical CSS spellings.
    #[must_use]
    pub fn from_name(name: &str) -> Self {
        match name {
            "serif" => Self::Generic(GenericFamily::Serif),
            "sans-serif" => Self::Generic(GenericFamily::SansSerif),
            "monospace" => Self::Generic(GenericFamily::Monospace),
            "cursive" => Self::Generic(GenericFamily::Cursive),
            "fantasy" => Self::Generic(GenericFamily::Fantasy),
            "system-ui" => Self::Generic(GenericFamily::SystemUi),
            other => Self::Named(other.into()),
        }
    }

    /// Returns the family name: the specific name for [`FontFamily::Named`], or
    /// the CSS keyword ([`GenericFamily::fallback_name`]) for
    /// [`FontFamily::Generic`] (spec §5.18).
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Named(name) => name,
            Self::Generic(generic) => generic.fallback_name(),
        }
    }
}

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
    fn from_name_maps_generics() {
        assert_eq!(
            FontFamily::from_name("serif"),
            FontFamily::Generic(GenericFamily::Serif)
        );
        assert_eq!(
            FontFamily::from_name("sans-serif"),
            FontFamily::Generic(GenericFamily::SansSerif)
        );
        assert_eq!(
            FontFamily::from_name("monospace"),
            FontFamily::Generic(GenericFamily::Monospace)
        );
        assert_eq!(
            FontFamily::from_name("cursive"),
            FontFamily::Generic(GenericFamily::Cursive)
        );
        assert_eq!(
            FontFamily::from_name("fantasy"),
            FontFamily::Generic(GenericFamily::Fantasy)
        );
        assert_eq!(
            FontFamily::from_name("system-ui"),
            FontFamily::Generic(GenericFamily::SystemUi)
        );
    }

    #[test]
    fn from_name_keeps_specific_names() {
        assert_eq!(
            FontFamily::from_name("Inter"),
            FontFamily::Named("Inter".into())
        );
        // Non-canonical spellings are treated as named, not generic.
        assert_eq!(
            FontFamily::from_name("Sans-Serif"),
            FontFamily::Named("Sans-Serif".into())
        );
    }

    #[test]
    fn name_accessor() {
        assert_eq!(FontFamily::named("Helvetica").name(), "Helvetica");
        assert_eq!(
            FontFamily::generic(GenericFamily::Monospace).name(),
            "monospace"
        );
    }

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
