//! The cache key for a whole-paragraph shaping result (spec §5.10).

use crate::font::{FontAttributes, FontFace, FontFallback, FontFamily};
use crate::shaping::face_source_key::FaceSourceKey;
use crate::shaping::TextDirection;

/// The full cache key for [`ParagraphShaper::shape_paragraph`]: everything that
/// can change the itemize → resolve → shape output.
///
/// [`ParagraphShaper::shape_paragraph`](crate::shaping::ParagraphShaper::shape_paragraph)
/// is width-independent — wrapping to a concrete width happens later in
/// `ParagraphLayout` — so its result depends only on the six inputs captured
/// here. Memoizing on this key lets the measure pass (wraps to the constraint
/// width) and the compose pass (wraps to the box width) share one shaped result
/// for an unchanged text node, across both phases and across frames.
///
/// The `f32` font size is stored via [`f32::to_bits`] so the key can derive
/// `Eq` and `Hash` (a raw `f32` cannot). The `families` fallback chain is
/// captured as its ordered list of [`FontFamily`] (each cheap and `Hash`), the
/// `attributes` directly (they are `Copy + Hash`), and the `default_face` by its
/// shaping-relevant identity — family, attributes, and the cheap
/// [`FaceSourceKey`] (never hashing font bytes). Two calls with the same key
/// produce byte-identical runs; any observable difference lands in a different
/// bucket.
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
    /// Extracts a key from a `shape_paragraph` call's inputs, cloning only cheap
    /// owned data (never font bytes).
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
