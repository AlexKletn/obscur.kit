use std::sync::Arc;

use obscur_text::font::{FontAttributes, FontFamily};

/// One face a host offers the toolkit: the family and attributes it answers to,
/// plus the raw bytes to shape and rasterize it from.
#[derive(Debug, Clone)]
pub struct FontFaceBytes {
    family: FontFamily,
    attributes: FontAttributes,
    bytes: Arc<[u8]>,
}

impl FontFaceBytes {
    #[doc = include_str!("../../docs/api/bootstrap/text_stack.doc/new.md")]
    #[must_use]
    pub fn new(
        family: FontFamily,
        attributes: FontAttributes,
        bytes: impl Into<Arc<[u8]>>,
    ) -> Self {
        Self {
            family,
            attributes,
            bytes: bytes.into(),
        }
    }

    /// The family this face answers to.
    #[must_use]
    pub const fn family(&self) -> &FontFamily {
        &self.family
    }

    /// The weight/slant/stretch this face carries.
    #[must_use]
    pub const fn attributes(&self) -> &FontAttributes {
        &self.attributes
    }

    /// The raw font bytes.
    #[must_use]
    pub fn bytes(&self) -> &Arc<[u8]> {
        &self.bytes
    }
}
