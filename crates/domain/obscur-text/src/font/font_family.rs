//! A requested font family — a specific name or a CSS generic (spec §5.18).

use super::GenericFamily;

#[doc = include_str!("../../docs/api/font/font_family.doc/FontFamily.md")]
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

    #[doc = include_str!("../../docs/api/font/font_family.doc/from_name.md")]
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
}
