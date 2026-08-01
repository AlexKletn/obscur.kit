#![doc = include_str!("paragraph_key.doc/_module.md")]

use crate::font::{FontAttributes, FontFace, FontFallback, FontFamily};
use crate::shaping::TextDirection;
use crate::shaping::face_source_key::FaceSourceKey;

#[doc = include_str!("paragraph_key.doc/ParagraphKey.md")]
#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct ParagraphKey {
    text: Box<str>,
    font_size_bits: u32,
    base_direction: TextDirection,
    families: Box<[FontFamily]>,
    attributes: FontAttributes,
    default_family: FontFamily,
    default_attributes: FontAttributes,
    default_source: FaceSourceKey,
}

impl ParagraphKey {
    #[doc = include_str!("paragraph_key.doc/new.md")]
    pub(crate) fn new(
        text: &str,
        families: &FontFallback,
        attributes: FontAttributes,
        default_face: &FontFace,
        font_size: f32,
        base_direction: TextDirection,
    ) -> Self {
        Self {
            text: text.into(),
            font_size_bits: font_size.to_bits(),
            base_direction,
            families: families.families().into(),
            attributes,
            default_family: default_face.family().clone(),
            default_attributes: *default_face.attributes(),
            default_source: FaceSourceKey::from_source(default_face.source()),
        }
    }
}
