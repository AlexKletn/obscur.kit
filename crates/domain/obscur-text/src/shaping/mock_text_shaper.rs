use crate::font::FontFace;
use crate::span::TextRange;

use super::{GlyphRun, ShapedGlyph, ShapingRequest, TextShaper};

#[doc = include_str!("../../docs/api/shaping/text_shaper.doc/MockTextShaper.md")]
#[derive(Debug, Default, Clone)]
pub struct MockTextShaper;

impl MockTextShaper {
    /// The fraction of the font size used as every glyph's advance — a fixed
    /// monospace em-ratio (`advance = font_size * ADVANCE_RATIO`). `0.6` is a
    /// typical monospace character width and keeps the mock's metrics simple and
    /// deterministic.
    pub const ADVANCE_RATIO: f32 = 0.6;

    /// Creates a new mock shaper.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl TextShaper for MockTextShaper {
    fn shape(&self, request: &ShapingRequest) -> GlyphRun {
        let text = request.text();
        let advance = request.font_size() * Self::ADVANCE_RATIO;
        // One glyph per scalar; each keeps its source byte offset as its cluster.
        let mut glyphs: Vec<ShapedGlyph> = text
            .char_indices()
            .map(|(byte_index, ch)| ShapedGlyph::new(u32::from(ch), byte_index, advance))
            .collect();
        // Real shapers return an RTL run's glyphs in visual (pen, left-to-right)
        // order, so the logical-last scalar's glyph comes first. Reversing keeps
        // each glyph's cluster (source byte offset) intact — they just appear in
        // reversed order. LTR is already visual == logical.
        if request.direction().is_rtl() {
            glyphs.reverse();
        }
        GlyphRun::new(
            glyphs,
            request.direction(),
            request.font_size(),
            TextRange::new(0, text.len()),
        )
    }

    fn can_shape(&self, _face: &FontFace) -> bool {
        // The mock is font-agnostic: it shapes any face.
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{FontAttributes, FontFamily, FontMetrics, FontSource, GenericFamily};
    use crate::shaping::TextDirection;

    fn mono_face() -> FontFace {
        FontFace::new(
            FontFamily::generic(GenericFamily::Monospace),
            FontAttributes::default(),
            FontSource::system("mono"),
            FontMetrics::new(1000, 800.0, -200.0, 0.0),
        )
    }

    #[test]
    fn text_shaper_is_object_safe() {
        let shaper: Box<dyn TextShaper> = Box::new(MockTextShaper::new());
        assert!(shaper.can_shape(&mono_face()));
    }

    #[test]
    fn shaping_is_deterministic() {
        let shaper = MockTextShaper::new();
        let request = ShapingRequest::new("hello", mono_face(), 16.0);
        assert_eq!(shaper.shape(&request), shaper.shape(&request));
    }

    #[test]
    fn one_glyph_per_scalar_with_expected_advance() {
        let shaper = MockTextShaper::new();
        let request = ShapingRequest::new("abc", mono_face(), 10.0);
        let run = shaper.shape(&request);
        assert_eq!(run.len(), 3);
        // advance == font_size * ADVANCE_RATIO * glyph_count.
        assert_eq!(run.advance(), 10.0 * 0.6 * 3.0);
        assert_eq!(run.glyph_at(0).unwrap().glyph_id(), u32::from('a'));
        assert_eq!(run.glyph_at(0).unwrap().x_advance(), 6.0);
    }

    #[test]
    fn clusters_are_byte_offsets_for_multibyte_utf8() {
        // "aé": 'a' at byte 0, 'é' (U+00E9) at byte 1 spanning two bytes, so the
        // source range covers all three bytes.
        let shaper = MockTextShaper::new();
        let request = ShapingRequest::new("aé", mono_face(), 12.0);
        let run = shaper.shape(&request);
        assert_eq!(run.len(), 2);
        assert_eq!(run.glyph_at(0).unwrap().cluster(), 0);
        assert_eq!(run.glyph_at(1).unwrap().cluster(), 1);
        assert_eq!(run.glyph_at(1).unwrap().glyph_id(), u32::from('é'));
        assert_eq!(run.source(), TextRange::new(0, 3));
    }

    #[test]
    fn ltr_emits_glyphs_in_logical_order() {
        // LTR: visual order == logical order, clusters ascending.
        let shaper = MockTextShaper::new();
        let request = ShapingRequest::new("abc", mono_face(), 10.0);
        let run = shaper.shape(&request);
        let ids: Vec<u32> = run.glyphs().iter().map(ShapedGlyph::glyph_id).collect();
        let clusters: Vec<usize> = run.glyphs().iter().map(ShapedGlyph::cluster).collect();
        assert_eq!(ids, vec![u32::from('a'), u32::from('b'), u32::from('c')]);
        assert_eq!(clusters, vec![0, 1, 2]);
        // Advances are uniform and the run direction/source are unchanged.
        assert!(run.glyphs().iter().all(|g| g.x_advance() == 6.0));
        assert_eq!(run.direction(), TextDirection::LeftToRight);
        assert_eq!(run.source(), TextRange::new(0, 3));
    }

    #[test]
    fn rtl_emits_glyphs_in_visual_reversed_order() {
        // RTL: the run's glyphs come out in visual (pen) order, so the
        // logical-last scalar ('c') is first. Clusters are the SOURCE byte
        // offsets, preserved but reversed; advances stay uniform.
        let shaper = MockTextShaper::new();
        let request = ShapingRequest::new("abc", mono_face(), 10.0)
            .with_direction(TextDirection::RightToLeft);
        let run = shaper.shape(&request);
        let ids: Vec<u32> = run.glyphs().iter().map(ShapedGlyph::glyph_id).collect();
        let clusters: Vec<usize> = run.glyphs().iter().map(ShapedGlyph::cluster).collect();
        assert_eq!(ids, vec![u32::from('c'), u32::from('b'), u32::from('a')]);
        assert_eq!(clusters, vec![2, 1, 0]);
        assert!(run.glyphs().iter().all(|g| g.x_advance() == 6.0));
        // Direction and full-text source range are unchanged by reversal.
        assert_eq!(run.direction(), TextDirection::RightToLeft);
        assert_eq!(run.source(), TextRange::new(0, 3));
    }

    #[test]
    fn rtl_shaping_is_deterministic() {
        let shaper = MockTextShaper::new();
        let request = ShapingRequest::new("abc", mono_face(), 10.0)
            .with_direction(TextDirection::RightToLeft);
        assert_eq!(shaper.shape(&request), shaper.shape(&request));
    }

    #[test]
    fn empty_rtl_text_yields_empty_run() {
        let shaper = MockTextShaper::new();
        let request =
            ShapingRequest::new("", mono_face(), 16.0).with_direction(TextDirection::RightToLeft);
        let run = shaper.shape(&request);
        assert!(run.is_empty());
        assert_eq!(run.source(), TextRange::empty(0));
    }

    #[test]
    fn direction_and_font_size_flow_through() {
        let shaper = MockTextShaper::new();
        let request =
            ShapingRequest::new("x", mono_face(), 24.0).with_direction(TextDirection::RightToLeft);
        let run = shaper.shape(&request);
        assert_eq!(run.direction(), TextDirection::RightToLeft);
        assert_eq!(run.font_size(), 24.0);
    }

    #[test]
    fn empty_text_yields_empty_run() {
        let shaper = MockTextShaper::new();
        let run = shaper.shape(&ShapingRequest::new("", mono_face(), 16.0));
        assert!(run.is_empty());
        assert_eq!(run.advance(), 0.0);
        assert_eq!(run.source(), TextRange::empty(0));
    }

    #[test]
    fn default_line_metrics_are_idealized_ratios() {
        // The trait default (used by the mock) gives ascent 0.8·size, descent
        // 0.2·size, zero line-gap — so line_height == font_size exactly.
        let shaper = MockTextShaper::new();
        let metrics = shaper.line_metrics(16.0);
        assert_eq!(metrics.ascent(), 12.8); // 0.8 * 16
        assert_eq!(metrics.descent(), 16.0 * 0.2);
        assert_eq!(metrics.line_gap(), 0.0);
        assert_eq!(metrics.line_height(), 16.0);
        assert_eq!(metrics.baseline(), 12.8);
    }
}
