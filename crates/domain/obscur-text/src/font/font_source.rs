#![doc = include_str!("font_source.doc/_module.md")]

use std::sync::Arc;

#[doc = include_str!("font_source.doc/FontSource.md")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FontSource {
    #[doc = include_str!("font_source.doc/System.md")]
    System(Box<str>),
    #[doc = include_str!("font_source.doc/Bundled.md")]
    Bundled(Box<str>),
    #[doc = include_str!("font_source.doc/Memory.md")]
    Memory(Arc<[u8]>),
}

impl FontSource {
    #[doc = include_str!("font_source.doc/system.md")]
    #[must_use]
    pub fn system(name: impl Into<Box<str>>) -> Self {
        Self::System(name.into())
    }

    #[doc = include_str!("font_source.doc/bundled.md")]
    #[must_use]
    pub fn bundled(key: impl Into<Box<str>>) -> Self {
        Self::Bundled(key.into())
    }

    #[doc = include_str!("font_source.doc/memory.md")]
    #[must_use]
    pub fn memory(data: impl Into<Arc<[u8]>>) -> Self {
        Self::Memory(data.into())
    }

    #[doc = include_str!("font_source.doc/is_system.md")]
    #[must_use]
    pub const fn is_system(&self) -> bool {
        matches!(self, Self::System(_))
    }

    #[doc = include_str!("font_source.doc/is_bundled.md")]
    #[must_use]
    pub const fn is_bundled(&self) -> bool {
        matches!(self, Self::Bundled(_))
    }

    #[doc = include_str!("font_source.doc/is_memory.md")]
    #[must_use]
    pub const fn is_memory(&self) -> bool {
        matches!(self, Self::Memory(_))
    }

    #[doc = include_str!("font_source.doc/data.md")]
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
