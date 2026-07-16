//! The font set a host hands the toolkit, and the handles the install gives back
//! (spec §5.18).
//!
//! A self-shaping toolkit needs font **bytes**. It cannot take a CSS family name
//! and hope the platform draws it: the shaper reads `cmap`/`GSUB`/`GPOS` itself, so
//! "use the system font" is only meaningful once something has read a file off the
//! host's disk. That is why the byte-carrying [`FontSet`] is the input to
//! [`KitBootstrap::install_text_stack`](super::KitBootstrap::install_text_stack)
//! and why the kit has no bundled fallback set — see
//! [`crate::bootstrap`](super)'s note on the §5.18 deviation.

use std::sync::Arc;

use obscur_text::font::{FontAttributes, FontFamily, FontId};

/// One face a host offers the toolkit: the family and attributes it answers to,
/// plus the raw bytes to shape and rasterize it from.
#[derive(Debug, Clone)]
pub struct FontFaceBytes {
    family: FontFamily,
    attributes: FontAttributes,
    bytes: Arc<[u8]>,
}

impl FontFaceBytes {
    /// Declares `bytes` as a face of `family` with `attributes`.
    ///
    /// The bytes are not parsed here — [`install_text_stack`](super::KitBootstrap::install_text_stack)
    /// parses them once, and bytes that are not a loadable font simply never cover
    /// a run (they are not an error, and never a panic).
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

/// The ordered set of faces a host offers the toolkit (spec §5.18).
///
/// Order is the fallback order: resolution walks the requested family chain and,
/// within a family, prefers the face whose attributes sit nearest the request,
/// breaking ties by the order faces were pushed here.
#[derive(Debug, Clone, Default)]
pub struct FontSet {
    faces: Vec<FontFaceBytes>,
}

impl FontSet {
    /// Creates an empty set. Installing it wires the real shaping stack over *no*
    /// faces, which resolves nothing and paints no text — see
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

/// What [`install_text_stack`](super::KitBootstrap::install_text_stack) hands
/// back: the scene font handle assigned to each installed face, paired with the
/// bytes behind it.
///
/// # Why the install cannot finish the job
///
/// The runtime resolves and shapes; it never rasterizes. The renderer does, and it
/// is not the runtime's to reach — a host owns it (and its GPU device). So the
/// shaper's emitted glyph **indices** are only meaningful to a renderer that has
/// been given the *same* bytes under the *same* handle. Feed this to the renderer
/// before the first frame:
///
/// ```ignore
/// let fonts = boot.install_text_stack(&mut runtime, &font_set);
/// for (handle, bytes) in fonts.iter() {
///     renderer.register_font(handle, bytes.to_vec());
/// }
/// ```
///
/// The two failure modes differ, and neither is silent-but-fine. Skip the loop and
/// the renderer has no bytes under the handle, so it falls back to drawing **box
/// marks** (`draw_glyph_boxes` in the CPU backend) — visibly tofu. Register
/// *different* bytes under the same handle and it is worse: the halves disagree
/// about what glyph `7` is, and real-looking but **wrong outlines** paint. Nothing
/// here can force the hand-off; this table is what makes it one loop.
#[derive(Debug, Clone, Default)]
pub struct InstalledFonts {
    faces: Vec<(u32, Arc<[u8]>)>,
}

impl InstalledFonts {
    /// Creates the handle table for `faces`, in installed order.
    pub(super) fn new(faces: Vec<(u32, Arc<[u8]>)>) -> Self {
        Self { faces }
    }

    /// Iterates `(scene font handle, bytes)` in installed order.
    pub fn iter(&self) -> impl Iterator<Item = (u32, &Arc<[u8]>)> {
        self.faces.iter().map(|(handle, bytes)| (*handle, bytes))
    }

    /// The scene font handle assigned to the face at `index` in the installed
    /// [`FontSet`], or `None` when the set had no such face.
    #[must_use]
    pub fn handle(&self, index: usize) -> Option<u32> {
        self.faces.get(index).map(|(handle, _)| *handle)
    }

    /// The number of installed faces.
    #[must_use]
    pub fn len(&self) -> usize {
        self.faces.len()
    }

    /// Whether nothing was installed — the state of an empty [`FontSet`], in which
    /// no text will paint.
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

/// The scene font handle (`u32`) a resolved [`FontId`] paints under.
///
/// The registry hands ids out densely from `0` in registration order, and the
/// scene's font handle is that index — the identity every hand-wired host in this
/// repo re-derived for itself.
pub(super) fn font_handle(id: FontId) -> u32 {
    u32::try_from(id.index()).unwrap_or(0)
}
