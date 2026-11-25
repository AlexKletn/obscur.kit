//! The non-text half of a shaped paragraph's identity: [`ParagraphStyleKey`].

use crate::font::{FontAttributes, FontFace, FontFallback, FontFamily};
use crate::shaping::TextDirection;
use crate::shaping::face_source_key::FaceSourceKey;

#[doc = include_str!("../../docs/api/shaping/paragraph_style_key.doc/ParagraphStyleKey.md")]
pub(crate) struct ParagraphStyleKey {
    /// The used font size, as raw bits so the key compares by exact bit pattern
    /// rather than by `f32` equality (which has no `Eq`, and would make `NaN`
    /// mismatch itself).
    font_size_bits: u32,
    /// The paragraph's base direction, which the itemizer reads to resolve neutrals.
    base_direction: TextDirection,
    /// The requested fallback chain, in order. The resolution of a run depends on
    /// the whole chain, not just the family it happened to land on.
    families: Box<[FontFamily]>,
    /// The requested weight / style / stretch, which select among the faces of a
    /// family and drive synthesis.
    attributes: FontAttributes,
    /// The family of the default face used when nothing in the chain matches.
    default_family: FontFamily,
    /// The attributes of that default face.
    default_attributes: FontAttributes,
    /// Where that default face's bytes come from, as a
    /// [`FaceSourceKey`](crate::shaping::face_source_key::FaceSourceKey) — an
    /// identity that never hashes or compares the font file itself.
    default_source: FaceSourceKey,
}

impl ParagraphStyleKey {
    /// Captures the style half of a shaping call. Called only on a cache **miss**,
    /// so the clones it makes are paid once per style group, not once per probe.
    pub(crate) fn new(
        families: &FontFallback,
        attributes: FontAttributes,
        default_face: &FontFace,
        font_size: f32,
        base_direction: TextDirection,
    ) -> Self {
        Self {
            font_size_bits: font_size.to_bits(),
            base_direction,
            families: families.families().into(),
            attributes,
            default_family: default_face.family().clone(),
            default_attributes: *default_face.attributes(),
            default_source: FaceSourceKey::from_source(default_face.source()),
        }
    }

    #[doc = include_str!("../../docs/api/shaping/paragraph_style_key.doc/matches.md")]
    pub(crate) fn matches(
        &self,
        families: &FontFallback,
        attributes: FontAttributes,
        default_face: &FontFace,
        font_size: f32,
        base_direction: TextDirection,
    ) -> bool {
        self.font_size_bits == font_size.to_bits()
            && self.base_direction == base_direction
            && self.attributes == attributes
            && self.default_attributes == *default_face.attributes()
            && self.default_family == *default_face.family()
            && *self.families == *families.families()
            && self.default_source.matches(default_face.source())
    }
}
