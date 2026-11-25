use std::sync::Arc;

use obscur_text::font::{FontAttributes, FontFamily};

use super::FontFaceBytes;

#[doc = include_str!("../../docs/api/bootstrap/text_stack.doc/FontSet.md")]
#[derive(Debug, Clone, Default)]
pub struct FontSet {
    faces: Vec<FontFaceBytes>,
}

impl FontSet {
    /// Creates an empty set. Installing it wires the real shaping stack with no
    /// host faces, and text is served from the bundled last-resort face the install
    /// appends — see
    /// [`install_text_stack`](super::KitBootstrap::install_text_stack).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds `face` to the end of the fallback order.
    pub fn push(&mut self, face: FontFaceBytes) {
        self.faces.push(face);
    }

    /// Builder form of [`push`](Self::push).
    #[must_use]
    pub fn with_face(mut self, face: FontFaceBytes) -> Self {
        self.push(face);
        self
    }

    /// Builder form of [`push`](Self::push) that names the face inline.
    #[must_use]
    pub fn with(
        self,
        family: FontFamily,
        attributes: FontAttributes,
        bytes: impl Into<Arc<[u8]>>,
    ) -> Self {
        self.with_face(FontFaceBytes::new(family, attributes, bytes))
    }

    /// The faces, in fallback order.
    #[must_use]
    pub fn faces(&self) -> &[FontFaceBytes] {
        &self.faces
    }

    /// The number of faces.
    #[must_use]
    pub fn len(&self) -> usize {
        self.faces.len()
    }

    /// Whether the set offers no faces at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.faces.is_empty()
    }
}
