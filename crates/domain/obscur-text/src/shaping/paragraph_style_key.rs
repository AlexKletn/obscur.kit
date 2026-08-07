#![doc = include_str!("paragraph_style_key.doc/_module.md")]

use crate::font::{FontAttributes, FontFace, FontFallback, FontFamily};
use crate::shaping::TextDirection;
use crate::shaping::face_source_key::FaceSourceKey;

#[doc = include_str!("paragraph_style_key.doc/ParagraphStyleKey.md")]
pub(crate) struct ParagraphStyleKey {
    #[doc = include_str!("paragraph_style_key.doc/font_size_bits.md")]
    font_size_bits: u32,
    #[doc = include_str!("paragraph_style_key.doc/base_direction.md")]
    base_direction: TextDirection,
    #[doc = include_str!("paragraph_style_key.doc/families.md")]
    families: Box<[FontFamily]>,
    #[doc = include_str!("paragraph_style_key.doc/attributes.md")]
    attributes: FontAttributes,
    #[doc = include_str!("paragraph_style_key.doc/default_family.md")]
    default_family: FontFamily,
    #[doc = include_str!("paragraph_style_key.doc/default_attributes.md")]
    default_attributes: FontAttributes,
    #[doc = include_str!("paragraph_style_key.doc/default_source.md")]
    default_source: FaceSourceKey,
}

impl ParagraphStyleKey {
    #[doc = include_str!("paragraph_style_key.doc/new.md")]
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

    #[doc = include_str!("paragraph_style_key.doc/matches.md")]
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
