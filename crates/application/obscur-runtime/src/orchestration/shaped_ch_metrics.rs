#![doc = include_str!("../../docs/api/orchestration/shaped_ch_metrics.doc/_module.md")]

use obscur_style::computed::{ChMetrics, ComputedStyle};
use obscur_text::shaping::ParagraphShaper;

use super::font_families::families_for;
use super::length_resolution::font_size_px;
use super::text_attributes::attributes_for;
use super::text_leaf_measure::measure_ch_width;

#[doc = include_str!("../../docs/api/orchestration/shaped_ch_metrics.doc/ShapedChMetrics.md")]
#[derive(Clone, Copy)]
pub struct ShapedChMetrics<'a> {
    /// The shaper every other text measurement in the frame goes through.
    paragraph: &'a ParagraphShaper,
}

impl<'a> ShapedChMetrics<'a> {
    /// Wraps `paragraph` as the resolver's `ch` measurement port.
    #[must_use]
    pub const fn new(paragraph: &'a ParagraphShaper) -> Self {
        Self { paragraph }
    }
}

impl ChMetrics for ShapedChMetrics<'_> {
    #[doc = include_str!("../../docs/api/orchestration/shaped_ch_metrics.doc/zero_advance.md")]
    fn zero_advance(&self, font: &ComputedStyle) -> Option<f32> {
        measure_ch_width(
            self.paragraph,
            &families_for(Some(font)),
            &attributes_for(Some(font)),
            font_size_px(font),
        )
    }
}
