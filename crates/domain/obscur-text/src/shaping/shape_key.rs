use crate::font::{FontAttributes, FontFamily};
use crate::shaping::{ShapingRequest, TextDirection, face_source_key::FaceSourceKey};

#[doc = include_str!("../../docs/api/shaping/caching_shaper.doc/ShapeKey.md")]
#[derive(Clone, PartialEq, Eq, Hash)]
pub(super) struct ShapeKey {
    text: Box<str>,
    font_size_bits: u32,
    direction: TextDirection,
    language: Option<Box<str>>,
    family: FontFamily,
    attributes: FontAttributes,
    source: FaceSourceKey,
}

impl ShapeKey {
    /// Extracts a key from a [`ShapingRequest`], cloning only cheap owned data
    /// (never the font bytes).
    pub(super) fn from_request(request: &ShapingRequest) -> Self {
        let face = request.face();
        Self {
            text: request.text().into(),
            font_size_bits: request.font_size().to_bits(),
            direction: request.direction(),
            language: request.language().map(Into::into),
            family: face.family().clone(),
            attributes: *face.attributes(),
            source: FaceSourceKey::from_source(face.source()),
        }
    }
}
