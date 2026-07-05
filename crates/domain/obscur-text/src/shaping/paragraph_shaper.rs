//! The multi-font paragraph orchestrator composing itemize → resolve → shape
//! (spec §5.10).

use crate::font::{
    FontAttributes, FontFace, FontFallback, FontId, FontResolveRequest, FontResolver,
};
use crate::shaping::{
    GlyphRun, ShapedGlyph, ShapedRun, ShapingRequest, TextDirection, TextItemizer, TextShaper,
};
use crate::span::TextRange;

/// The orchestrator that shapes a whole paragraph across multiple fonts
/// (spec §5.10 font fallback).
///
/// A single [`TextShaper`] only shapes one uniformly-styled, single-font run;
/// real paragraphs mix scripts and therefore fonts. `ParagraphShaper` composes
/// three ports to bridge that gap: it [`itemize`](TextItemizer::itemize)s the
/// text into uniform runs, [`resolve`](FontResolver::resolve)s a covering font
/// for each, and [`shape`](TextShaper::shape)s each run with the resolved face,
/// yielding one [`ShapedRun`] per item in logical order.
///
/// Each sub-run is shaped against a *slice* of the paragraph, so its glyph
/// clusters and source range come back relative to that slice (0-based).
/// `ParagraphShaper` rewrites them into **full-text byte coordinates** — every
/// glyph's cluster is offset by the item's start and the run's source is set to
/// the item's range — so callers can map any glyph straight back to the original
/// string.
pub struct ParagraphShaper {
    itemizer: Box<dyn TextItemizer>,
    resolver: Box<dyn FontResolver>,
    shaper: Box<dyn TextShaper>,
}

impl ParagraphShaper {
    /// Creates a shaper composing the given `itemizer`, `resolver`, and `shaper`
    /// ports.
    #[must_use]
    pub fn new(
        itemizer: Box<dyn TextItemizer>,
        resolver: Box<dyn FontResolver>,
        shaper: Box<dyn TextShaper>,
    ) -> Self {
        Self {
            itemizer,
            resolver,
            shaper,
        }
    }

    /// Shapes `text` end to end, returning one [`ShapedRun`] per itemized run in
    /// logical (reading) order (spec §5.10).
    ///
    /// For each itemized run the resolver is asked for a covering font from
    /// `families` matching `attributes`; when it returns `None` the run falls
    /// back to [`FontId::PRIMARY`] paired with `default_face`. Every run is
    /// shaped at `font_size` with the item's own direction (itself seeded by
    /// `base_direction`), and its glyph clusters and source range are normalized
    /// to full-text byte coordinates. Empty `text` yields a single empty
    /// `ShapedRun` (the itemizer emits one empty item), never a panic.
    #[must_use]
    pub fn shape_paragraph(
        &self,
        text: &str,
        families: &FontFallback,
        attributes: &FontAttributes,
        default_face: &FontFace,
        font_size: f32,
        base_direction: TextDirection,
    ) -> Vec<ShapedRun> {
        let items = self.itemizer.itemize(text, base_direction);
        let mut runs = Vec::with_capacity(items.len());
        for item in items {
            let range = item.range();
            let sub = &text[range.start..range.end];

            let request = FontResolveRequest::new(families, attributes, sub, item.script());
            let (font, face) = match self.resolver.resolve(&request) {
                Some(resolved) => (resolved.font(), resolved.into_face()),
                None => (FontId::PRIMARY, default_face.clone()),
            };

            let shape_request =
                ShapingRequest::new(sub, face, font_size).with_direction(item.direction());
            let run = self.shaper.shape(&shape_request);

            runs.push(ShapedRun::new(font, rebase_run(&run, range)));
        }
        runs
    }
}

/// Rewrites a run shaped against a slice into full-text coordinates: shifts each
/// glyph's cluster by `range.start` and sets the run source to `range`.
fn rebase_run(run: &GlyphRun, range: TextRange) -> GlyphRun {
    let glyphs: Vec<ShapedGlyph> = run
        .glyphs()
        .iter()
        .map(|glyph| {
            ShapedGlyph::new(
                glyph.glyph_id(),
                glyph.cluster() + range.start,
                glyph.x_advance(),
            )
            .with_offset(glyph.x_offset(), glyph.y_offset())
        })
        .collect();
    GlyphRun::new(glyphs, run.direction(), run.font_size(), range)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{FontFamily, FontMetrics, FontResolver, FontSource, MockFontResolver};
    use crate::shaping::{MockTextShaper, ScriptTag, SingleRunItemizer, TextItem};

    fn face(name: &str) -> FontFace {
        FontFace::new(
            FontFamily::named(name),
            FontAttributes::default(),
            FontSource::system(name),
            FontMetrics::new(1000, 800.0, -200.0, 0.0),
        )
    }

    /// A stub itemizer that cuts a fixed two-script string "abВГ" into a Latin
    /// run `[0..2)` and a Cyrillic run `[2..6)` (each Cyrillic char is 2 bytes).
    struct TwoScriptItemizer;

    impl TextItemizer for TwoScriptItemizer {
        fn itemize(&self, _text: &str, _base_direction: TextDirection) -> Vec<TextItem> {
            vec![
                TextItem::new(
                    TextRange::new(0, 2),
                    ScriptTag::LATIN,
                    TextDirection::LeftToRight,
                    0,
                ),
                TextItem::new(
                    TextRange::new(2, 6),
                    ScriptTag::CYRILLIC,
                    TextDirection::LeftToRight,
                    0,
                ),
            ]
        }
    }

    fn attrs() -> FontAttributes {
        FontAttributes::default()
    }

    fn chain() -> FontFallback {
        FontFallback::new(FontFamily::named("Inter"))
    }

    #[test]
    fn splits_paragraph_into_two_font_tagged_runs_in_order() {
        let mut resolver = MockFontResolver::new();
        resolver.register(ScriptTag::LATIN, FontId::new(0), face("Latin"));
        resolver.register(ScriptTag::CYRILLIC, FontId::new(1), face("Cyrillic"));

        let shaper = ParagraphShaper::new(
            Box::new(TwoScriptItemizer),
            Box::new(resolver),
            Box::new(MockTextShaper::new()),
        );

        let default = face("Fallback");
        let runs = shaper.shape_paragraph(
            "abВГ",
            &chain(),
            &attrs(),
            &default,
            16.0,
            TextDirection::LeftToRight,
        );

        assert_eq!(runs.len(), 2);
        // Fonts in logical order: Latin (0) then Cyrillic (1).
        assert_eq!(runs[0].font(), FontId::new(0));
        assert_eq!(runs[1].font(), FontId::new(1));

        // Latin run: clusters 0,1 and source 0..2.
        let latin = runs[0].run();
        assert_eq!(latin.source(), TextRange::new(0, 2));
        assert_eq!(latin.glyph_at(0).unwrap().cluster(), 0);
        assert_eq!(latin.glyph_at(1).unwrap().cluster(), 1);

        // Cyrillic run: clusters normalized to FULL-TEXT bytes — first is 2, not
        // 0 (each Cyrillic char is two bytes, so 2 then 4), source 2..6.
        let cyrillic = runs[1].run();
        assert_eq!(cyrillic.source(), TextRange::new(2, 6));
        assert_eq!(cyrillic.glyph_at(0).unwrap().cluster(), 2);
        assert_eq!(cyrillic.glyph_at(1).unwrap().cluster(), 4);
    }

    #[test]
    fn single_run_itemizer_reproduces_single_font_behavior() {
        // Empty resolver → every run falls back to PRIMARY + default_face.
        let shaper = ParagraphShaper::new(
            Box::new(SingleRunItemizer::new()),
            Box::new(MockFontResolver::new()),
            Box::new(MockTextShaper::new()),
        );

        let default = face("Fallback");
        let runs = shaper.shape_paragraph(
            "hello",
            &chain(),
            &attrs(),
            &default,
            16.0,
            TextDirection::LeftToRight,
        );

        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].font(), FontId::PRIMARY);
        let run = runs[0].run();
        assert_eq!(run.source(), TextRange::new(0, 5));
        assert_eq!(run.len(), 5);
        // Clusters unchanged for a whole-text run (item starts at 0).
        assert_eq!(run.glyph_at(0).unwrap().cluster(), 0);
        assert_eq!(run.glyph_at(4).unwrap().cluster(), 4);
    }

    #[test]
    fn empty_text_yields_one_empty_run_without_panic() {
        let shaper = ParagraphShaper::new(
            Box::new(SingleRunItemizer::new()),
            Box::new(MockFontResolver::new()),
            Box::new(MockTextShaper::new()),
        );
        let default = face("Fallback");
        let runs = shaper.shape_paragraph(
            "",
            &chain(),
            &attrs(),
            &default,
            16.0,
            TextDirection::LeftToRight,
        );
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].font(), FontId::PRIMARY);
        assert!(runs[0].run().is_empty());
        assert_eq!(runs[0].run().source(), TextRange::empty(0));
    }

    #[test]
    fn unregistered_script_falls_back_to_default_face() {
        // Resolver knows only Latin; the Cyrillic run must fall back to PRIMARY +
        // default_face rather than fail.
        let mut resolver = MockFontResolver::new();
        resolver.register(ScriptTag::LATIN, FontId::new(3), face("Latin"));

        let shaper = ParagraphShaper::new(
            Box::new(TwoScriptItemizer),
            Box::new(resolver),
            Box::new(MockTextShaper::new()),
        );

        let default = face("Fallback");
        let runs = shaper.shape_paragraph(
            "abВГ",
            &chain(),
            &attrs(),
            &default,
            16.0,
            TextDirection::LeftToRight,
        );

        assert_eq!(runs.len(), 2);
        // Latin resolved to font 3.
        assert_eq!(runs[0].font(), FontId::new(3));
        // Cyrillic unregistered → PRIMARY + default_face, still normalized.
        assert_eq!(runs[1].font(), FontId::PRIMARY);
        assert_eq!(runs[1].run().source(), TextRange::new(2, 6));
        assert_eq!(runs[1].run().glyph_at(0).unwrap().cluster(), 2);
    }

    #[test]
    fn is_boxable_via_ports() {
        // Compile-time check the ports compose through trait objects.
        let resolver: Box<dyn FontResolver> = Box::new(MockFontResolver::new());
        let _shaper = ParagraphShaper::new(
            Box::new(SingleRunItemizer::new()),
            resolver,
            Box::new(MockTextShaper::new()),
        );
    }
}
