//! The stable, cheap cache identity of a [`FontSource`] (spec §5.10).

use crate::font::FontSource;

/// The stable identity of a [`FontSource`] used inside a cache key.
///
/// System and bundled sources are keyed by their name (tagged so a `System`
/// name never collides with an identical `Bundled` name). In-memory font bytes
/// are keyed by the backing allocation's pointer and length — a cheap, stable
/// identity — instead of hashing the whole byte buffer on every lookup. Two
/// `Memory` sources that share the same `Arc<[u8]>` (the common case: one face
/// shaped repeatedly) therefore hit the cache, and distinct allocations never
/// collide even if their lengths match.
///
/// Shared by every shaping cache in this module (the per-run
/// [`CachingShaper`](crate::shaping::CachingShaper) and the paragraph cache) so
/// they compute face identity identically.
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
}
