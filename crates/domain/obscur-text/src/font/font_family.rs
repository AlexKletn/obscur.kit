#![doc = include_str!("font_family.doc/_module.md")]

#[doc = include_str!("font_family.doc/FontFamily.md")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FontFamily {
    #[doc = include_str!("font_family.doc/Named.md")]
    Named(Box<str>),
    #[doc = include_str!("font_family.doc/Generic.md")]
    Generic(GenericFamily),
}

impl FontFamily {
    #[doc = include_str!("font_family.doc/named.md")]
    #[must_use]
    pub fn named(name: impl Into<Box<str>>) -> Self {
        Self::Named(name.into())
    }

    #[doc = include_str!("font_family.doc/generic.md")]
    #[must_use]
    pub const fn generic(generic: GenericFamily) -> Self {
        Self::Generic(generic)
    }

    #[doc = include_str!("font_family.doc/from_name.md")]
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

    #[doc = include_str!("font_family.doc/name.md")]
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Named(name) => name,
            Self::Generic(generic) => generic.fallback_name(),
        }
    }
}

#[doc = include_str!("font_family.doc/GenericFamily.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GenericFamily {
    #[doc = include_str!("font_family.doc/Serif.md")]
    Serif,
    #[doc = include_str!("font_family.doc/SansSerif.md")]
    SansSerif,
    #[doc = include_str!("font_family.doc/Monospace.md")]
    Monospace,
    #[doc = include_str!("font_family.doc/Cursive.md")]
    Cursive,
    #[doc = include_str!("font_family.doc/Fantasy.md")]
    Fantasy,
    #[doc = include_str!("font_family.doc/SystemUi.md")]
    SystemUi,
}

impl GenericFamily {
    #[doc = include_str!("font_family.doc/fallback_name.md")]
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
