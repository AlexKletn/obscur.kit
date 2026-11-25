use std::sync::Arc;

#[doc = include_str!("../../docs/api/bootstrap/text_stack.doc/InstalledFonts.md")]
#[derive(Debug, Clone, Default)]
pub struct InstalledFonts {
    faces: Vec<(u32, Arc<[u8]>)>,
    /// A clone of the live registry, when one was installed, so faces the host
    /// discovers *later* can still reach the renderer — see
    /// [`take_discovered`](Self::take_discovered).
    #[cfg(all(feature = "text-harfrust", feature = "text-unicode"))]
    registry: Option<crate::text_harfrust::FontRegistry>,
}

impl InstalledFonts {
    /// Creates the handle table for `faces`, in installed order.
    pub(super) fn new(faces: Vec<(u32, Arc<[u8]>)>) -> Self {
        Self {
            faces,
            #[cfg(all(feature = "text-harfrust", feature = "text-unicode"))]
            registry: None,
        }
    }

    /// Attaches the live registry whose per-character fallback may pull in faces
    /// after startup.
    #[cfg(all(feature = "text-harfrust", feature = "text-unicode"))]
    pub(super) fn with_registry(mut self, registry: crate::text_harfrust::FontRegistry) -> Self {
        self.registry = Some(registry);
        self
    }

    #[doc = include_str!("../../docs/api/bootstrap/text_stack.doc/take_discovered.md")]
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

    #[doc = include_str!("../../docs/api/bootstrap/text_stack.doc/take_discovered.md")]
    #[cfg(not(all(feature = "text-harfrust", feature = "text-unicode")))]
    #[must_use]
    pub fn take_discovered(&self) -> Vec<(u32, Arc<[u8]>)> {
        // Nothing discovers a face without the real text stack, and the method
        // exists in this configuration anyway: both runners call it once a
        // frame, so a shape that is missing here does not fail at the call —
        // it fails the whole of `App` for a feature set the runner has no other
        // quarrel with. A surface that draws no text is a real configuration
        // (a plaque, a wallpaper, an indicator), and this is what it costs.
        Vec::new()
    }

    /// Iterates `(scene font handle, bytes)` in installed order.
    pub fn iter(&self) -> impl Iterator<Item = (u32, &Arc<[u8]>)> {
        self.faces.iter().map(|(handle, bytes)| (*handle, bytes))
    }

    /// The scene font handle assigned to the face at `index` in the installed
    /// [`FontSet`](super::FontSet), or `None` when the set had no such face.
    #[must_use]
    pub fn handle(&self, index: usize) -> Option<u32> {
        self.faces.get(index).map(|(handle, _)| *handle)
    }

    /// The number of installed faces.
    #[must_use]
    pub fn len(&self) -> usize {
        self.faces.len()
    }

    /// Whether the hand-off table is empty. Note an empty [`FontSet`](super::FontSet) does **not**
    /// yield an empty table: the install surfaces the bundled last-resort face so
    /// the renderer can paint the isolated-environment fallback (§5.18).
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
