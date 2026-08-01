#![doc = include_str!("text_stack.doc/_module.md")]

use std::sync::Arc;

use obscur_text::font::{FontAttributes, FontFamily, FontId};

#[doc = include_str!("text_stack.doc/FontFaceBytes.md")]
#[derive(Debug, Clone)]
pub struct FontFaceBytes {
    family: FontFamily,
    attributes: FontAttributes,
    bytes: Arc<[u8]>,
}

impl FontFaceBytes {
    #[doc = include_str!("text_stack.doc/new.md")]
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

    #[doc = include_str!("text_stack.doc/family.md")]
    #[must_use]
    pub const fn family(&self) -> &FontFamily {
        &self.family
    }

    #[doc = include_str!("text_stack.doc/attributes.md")]
    #[must_use]
    pub const fn attributes(&self) -> &FontAttributes {
        &self.attributes
    }

    #[doc = include_str!("text_stack.doc/bytes.md")]
    #[must_use]
    pub fn bytes(&self) -> &Arc<[u8]> {
        &self.bytes
    }
}

#[doc = include_str!("text_stack.doc/FontSet.md")]
#[derive(Debug, Clone, Default)]
pub struct FontSet {
    faces: Vec<FontFaceBytes>,
}

impl FontSet {
    #[doc = include_str!("text_stack.doc/new_2.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("text_stack.doc/push.md")]
    pub fn push(&mut self, face: FontFaceBytes) {
        self.faces.push(face);
    }

    #[doc = include_str!("text_stack.doc/with_face.md")]
    #[must_use]
    pub fn with_face(mut self, face: FontFaceBytes) -> Self {
        self.push(face);
        self
    }

    #[doc = include_str!("text_stack.doc/with.md")]
    #[must_use]
    pub fn with(
        self,
        family: FontFamily,
        attributes: FontAttributes,
        bytes: impl Into<Arc<[u8]>>,
    ) -> Self {
        self.with_face(FontFaceBytes::new(family, attributes, bytes))
    }

    #[doc = include_str!("text_stack.doc/faces.md")]
    #[must_use]
    pub fn faces(&self) -> &[FontFaceBytes] {
        &self.faces
    }

    #[doc = include_str!("text_stack.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.faces.len()
    }

    #[doc = include_str!("text_stack.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.faces.is_empty()
    }
}

#[doc = include_str!("text_stack.doc/InstalledFonts.md")]
#[derive(Debug, Clone, Default)]
pub struct InstalledFonts {
    faces: Vec<(u32, Arc<[u8]>)>,
    #[doc = include_str!("text_stack.doc/registry.md")]
    #[cfg(all(feature = "text-harfrust", feature = "text-unicode"))]
    registry: Option<crate::text_harfrust::FontRegistry>,
}

impl InstalledFonts {
    #[doc = include_str!("text_stack.doc/new_3.md")]
    pub(super) fn new(faces: Vec<(u32, Arc<[u8]>)>) -> Self {
        Self {
            faces,
            #[cfg(all(feature = "text-harfrust", feature = "text-unicode"))]
            registry: None,
        }
    }

    #[doc = include_str!("text_stack.doc/with_registry.md")]
    #[cfg(all(feature = "text-harfrust", feature = "text-unicode"))]
    pub(super) fn with_registry(mut self, registry: crate::text_harfrust::FontRegistry) -> Self {
        self.registry = Some(registry);
        self
    }

    #[doc = include_str!("text_stack.doc/take_discovered.md")]
    #[cfg(all(feature = "text-harfrust", feature = "text-unicode"))]
    pub fn take_discovered(&self) -> Vec<(u32, Arc<[u8]>)> {
        self.registry.as_ref().map_or_else(Vec::new, |registry| {
            registry
                .take_undelivered_faces()
                .into_iter()
                .map(|(id, bytes)| (super::text_stack::font_handle(id), bytes))
                .collect()
        })
    }

    #[doc = include_str!("text_stack.doc/iter.md")]
    pub fn iter(&self) -> impl Iterator<Item = (u32, &Arc<[u8]>)> {
        self.faces.iter().map(|(handle, bytes)| (*handle, bytes))
    }

    #[doc = include_str!("text_stack.doc/handle.md")]
    #[must_use]
    pub fn handle(&self, index: usize) -> Option<u32> {
        self.faces.get(index).map(|(handle, _)| *handle)
    }

    #[doc = include_str!("text_stack.doc/len_2.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.faces.len()
    }

    #[doc = include_str!("text_stack.doc/is_empty_2.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.faces.is_empty()
    }
}

impl<'a> IntoIterator for &'a InstalledFonts {
    type Item = (u32, &'a Arc<[u8]>);
    type IntoIter = std::vec::IntoIter<(u32, &'a Arc<[u8]>)>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter().collect::<Vec<_>>().into_iter()
    }
}

#[doc = include_str!("text_stack.doc/font_handle.md")]
pub(super) fn font_handle(id: FontId) -> u32 {
    u32::try_from(id.index()).unwrap_or(0)
}
