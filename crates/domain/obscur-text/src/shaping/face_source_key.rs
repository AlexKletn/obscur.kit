//! The stable, cheap cache identity of a [`FontSource`] (spec §5.10).

use crate::font::FontSource;

#[doc = include_str!("../../docs/api/shaping/face_source_key.doc/FaceSourceKey.md")]
#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) enum FaceSourceKey {
    /// A system face, keyed by its resolver name.
    System(Box<str>),
    /// A bundled fallback face, keyed by its bundle key.
    Bundled(Box<str>),
    /// In-memory bytes, keyed by allocation pointer and length.
    Memory { ptr: usize, len: usize },
}

impl FaceSourceKey {
    /// Builds a stable, cheap key from a [`FontSource`] without copying font
    /// bytes.
    pub(crate) fn from_source(source: &FontSource) -> Self {
        match source {
            FontSource::System(name) => Self::System(name.clone()),
            FontSource::Bundled(name) => Self::Bundled(name.clone()),
            FontSource::Memory(data) => {
                // Thin the fat `*const [u8]` to a byte pointer, then to an integer
                // identity — a cheap, stable key that never hashes the font bytes.
                let ptr = std::sync::Arc::as_ptr(data).cast::<u8>() as usize;
                Self::Memory {
                    ptr,
                    len: data.len(),
                }
            }
        }
    }

    #[doc = include_str!("../../docs/api/shaping/face_source_key.doc/matches.md")]
    pub(crate) fn matches(&self, source: &FontSource) -> bool {
        match (self, source) {
            (Self::System(name), FontSource::System(other))
            | (Self::Bundled(name), FontSource::Bundled(other)) => name == other,
            (Self::Memory { ptr, len }, FontSource::Memory(data)) => {
                *ptr == std::sync::Arc::as_ptr(data).cast::<u8>() as usize && *len == data.len()
            }
            _ => false,
        }
    }
}
