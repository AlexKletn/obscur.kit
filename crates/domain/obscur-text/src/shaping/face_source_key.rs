#![doc = include_str!("face_source_key.doc/_module.md")]

use crate::font::FontSource;

#[doc = include_str!("face_source_key.doc/FaceSourceKey.md")]
#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) enum FaceSourceKey {
    #[doc = include_str!("face_source_key.doc/System.md")]
    System(Box<str>),
    #[doc = include_str!("face_source_key.doc/Bundled.md")]
    Bundled(Box<str>),
    #[doc = include_str!("face_source_key.doc/Memory.md")]
    Memory { ptr: usize, len: usize },
}

impl FaceSourceKey {
    #[doc = include_str!("face_source_key.doc/from_source.md")]
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
