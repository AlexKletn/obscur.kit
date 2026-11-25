//! Where a font face's bytes come from (spec §5.18).

use std::sync::Arc;

#[doc = include_str!("../../docs/api/font/font_source.doc/FontSource.md")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FontSource {
    /// A system-installed face, referenced by its PostScript or family name and
    /// resolved by the platform provider.
    System(Box<str>),
    /// A key into the bundled fallback set used for WASM and isolated
    /// environments.
    Bundled(Box<str>),
    /// In-memory font data, shared cheaply via [`Arc`] and compared by content.
    Memory(Arc<[u8]>),
}

impl FontSource {
    /// Creates a [`FontSource::System`] referencing the face named `name`.
    #[must_use]
    pub fn system(name: impl Into<Box<str>>) -> Self {
        Self::System(name.into())
    }

    /// Creates a [`FontSource::Bundled`] keyed by `key` in the bundled fallback
    /// set.
    #[must_use]
    pub fn bundled(key: impl Into<Box<str>>) -> Self {
        Self::Bundled(key.into())
    }

    /// Creates a [`FontSource::Memory`] holding in-memory font `data`.
    #[must_use]
    pub fn memory(data: impl Into<Arc<[u8]>>) -> Self {
        Self::Memory(data.into())
    }

    /// Returns whether this is a [`FontSource::System`] source.
    #[must_use]
    pub const fn is_system(&self) -> bool {
        matches!(self, Self::System(_))
    }

    /// Returns whether this is a [`FontSource::Bundled`] source.
    #[must_use]
    pub const fn is_bundled(&self) -> bool {
        matches!(self, Self::Bundled(_))
    }

    /// Returns whether this is a [`FontSource::Memory`] source.
    #[must_use]
    pub const fn is_memory(&self) -> bool {
        matches!(self, Self::Memory(_))
    }

    /// Returns the in-memory font bytes for a [`FontSource::Memory`] source, or
    /// `None` for a `System` or `Bundled` source whose bytes the provider must
    /// resolve (spec §5.18).
    #[must_use]
    pub fn data(&self) -> Option<&[u8]> {
        match self {
            Self::Memory(data) => Some(data),
            Self::System(_) | Self::Bundled(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_predicates_and_no_data() {
        let source = FontSource::system("Helvetica");
        assert!(source.is_system());
        assert!(!source.is_bundled());
        assert!(!source.is_memory());
        assert_eq!(source.data(), None);
    }

    #[test]
    fn bundled_predicates_and_no_data() {
        let source = FontSource::bundled("noto-sans");
        assert!(source.is_bundled());
        assert!(!source.is_system());
        assert!(!source.is_memory());
        assert_eq!(source.data(), None);
    }

    #[test]
    fn memory_predicates_and_data() {
        let source = FontSource::memory(vec![1u8, 2, 3]);
        assert!(source.is_memory());
        assert!(!source.is_system());
        assert!(!source.is_bundled());
        assert_eq!(source.data(), Some(&[1u8, 2, 3][..]));
    }

    #[test]
    fn memory_compares_by_content() {
        assert_eq!(
            FontSource::memory(vec![9u8, 8]),
            FontSource::memory(vec![9u8, 8])
        );
        assert_ne!(
            FontSource::memory(vec![9u8, 8]),
            FontSource::memory(vec![9u8, 7])
        );
    }
}
