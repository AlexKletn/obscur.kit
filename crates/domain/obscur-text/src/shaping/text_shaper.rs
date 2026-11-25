//! The replaceable text-shaping port and its deterministic mock (spec §5.10).

use crate::font::{FontFace, FontId};
use crate::shaping::{GlyphRun, LineMetrics, ShapingRequest};

#[doc = include_str!("../../docs/api/shaping/text_shaper.doc/TextShaper.md")]
pub trait TextShaper {
    #[doc = include_str!("../../docs/api/shaping/text_shaper.doc/shape.md")]
    #[must_use]
    fn shape(&self, request: &ShapingRequest) -> GlyphRun;

    #[doc = include_str!("../../docs/api/shaping/text_shaper.doc/can_shape.md")]
    #[must_use]
    fn can_shape(&self, face: &FontFace) -> bool;

    #[doc = include_str!("../../docs/api/shaping/text_shaper.doc/line_metrics.md")]
    #[must_use]
    fn line_metrics(&self, font_size: f32) -> LineMetrics {
        LineMetrics::idealized(font_size)
    }

    #[doc = include_str!("../../docs/api/shaping/text_shaper.doc/line_metrics_for.md")]
    #[must_use]
    fn line_metrics_for(&self, _font: FontId, font_size: f32) -> LineMetrics {
        self.line_metrics(font_size)
    }
}
