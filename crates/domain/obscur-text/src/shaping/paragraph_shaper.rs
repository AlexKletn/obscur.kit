#![doc = include_str!("paragraph_shaper.doc/_module.md")]

use std::cell::RefCell;
use std::collections::HashMap;

use unicode_normalization::UnicodeNormalization;

use crate::font::{
    FontAttributes, FontFace, FontFallback, FontId, FontResolveRequest, FontResolver,
};
use crate::shaping::ScriptTag;
use crate::shaping::paragraph_key::ParagraphKey;
use crate::shaping::{
    GlyphRun, LineMetrics, ShapedGlyph, ShapedRun, ShapingRequest, TextDirection, TextItemizer,
    TextShaper,
};
use crate::span::TextRange;

#[doc = include_str!("paragraph_shaper.doc/ParagraphShaper.md")]
pub struct ParagraphShaper {
    itemizer: Box<dyn TextItemizer>,
    resolver: Box<dyn FontResolver>,
    shaper: Box<dyn TextShaper>,
    #[doc = include_str!("paragraph_shaper.doc/language.md")]
    language: Option<Box<str>>,
    #[doc = include_str!("paragraph_shaper.doc/cache.md")]
    cache: RefCell<HashMap<ParagraphKey, Vec<ShapedRun>>>,
    #[doc = include_str!("paragraph_shaper.doc/capacity.md")]
    capacity: usize,
}

impl ParagraphShaper {
    #[doc = include_str!("paragraph_shaper.doc/MAX_ENTRIES.md")]
    pub const MAX_ENTRIES: usize = 512;

    #[doc = include_str!("paragraph_shaper.doc/new.md")]
    #[must_use]
    pub fn new(
        itemizer: Box<dyn TextItemizer>,
        resolver: Box<dyn FontResolver>,
        shaper: Box<dyn TextShaper>,
    ) -> Self {
        Self::with_capacity(itemizer, resolver, shaper, Self::MAX_ENTRIES)
    }

    #[doc = include_str!("paragraph_shaper.doc/with_capacity.md")]
    #[must_use]
    pub fn with_capacity(
        itemizer: Box<dyn TextItemizer>,
        resolver: Box<dyn FontResolver>,
        shaper: Box<dyn TextShaper>,
        capacity: usize,
    ) -> Self {
        Self {
            itemizer,
            resolver,
            shaper,
            language: None,
            cache: RefCell::new(HashMap::new()),
            capacity: capacity.max(1),
        }
    }

    #[doc = include_str!("paragraph_shaper.doc/set_shaper.md")]
    pub fn set_shaper(&mut self, shaper: Box<dyn TextShaper>) {
        self.shaper = shaper;
        self.clear_cache();
    }

    #[doc = include_str!("paragraph_shaper.doc/set_itemizer.md")]
    pub fn set_itemizer(&mut self, itemizer: Box<dyn TextItemizer>) {
        self.itemizer = itemizer;
        self.clear_cache();
    }

    #[doc = include_str!("paragraph_shaper.doc/set_resolver.md")]
    pub fn set_resolver(&mut self, resolver: Box<dyn FontResolver>) {
        self.resolver = resolver;
        self.clear_cache();
    }

    #[doc = include_str!("paragraph_shaper.doc/set_language.md")]
    pub fn set_language(&mut self, language: Option<Box<str>>) {
        self.language = language;
        self.clear_cache();
    }

    #[doc = include_str!("paragraph_shaper.doc/language_2.md")]
    #[must_use]
    pub fn language(&self) -> Option<&str> {
        self.language.as_deref()
    }

    #[doc = include_str!("paragraph_shaper.doc/clear_cache.md")]
    fn clear_cache(&mut self) {
        self.cache.get_mut().clear();
    }

    #[doc = include_str!("paragraph_shaper.doc/cache_len.md")]
    #[must_use]
    pub fn cache_len(&self) -> usize {
        self.cache.borrow().len()
    }

    #[doc = include_str!("paragraph_shaper.doc/line_metrics.md")]
    #[must_use]
    pub fn line_metrics(&self, font_size: f32) -> LineMetrics {
        self.shaper.line_metrics(font_size)
    }

    #[doc = include_str!("paragraph_shaper.doc/line_metrics_for.md")]
    #[must_use]
    pub fn line_metrics_for(&self, font: FontId, font_size: f32) -> LineMetrics {
        self.shaper.line_metrics_for(font, font_size)
    }

    #[doc = include_str!("paragraph_shaper.doc/shape_paragraph.md")]
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
        let key = ParagraphKey::new(
            text,
            families,
            *attributes,
            default_face,
            font_size,
            base_direction,
        );
        if let Some(runs) = self.cache.borrow().get(&key) {
            return runs.clone();
        }

        let runs = self.shape_paragraph_uncached(
            text,
            families,
            attributes,
            default_face,
            font_size,
            base_direction,
        );

        let mut cache = self.cache.borrow_mut();
        // Flush wholesale on overflow: a simple, deterministic bound with no
        // eviction-order dependency (callers only ever observe the returned
        // value, never iteration order).
        if cache.len() >= self.capacity && !cache.contains_key(&key) {
            cache.clear();
        }
        cache.insert(key, runs.clone());
        runs
    }

    #[doc = include_str!("paragraph_shaper.doc/shape_paragraph_uncached.md")]
    // `attributes` stays by reference to mirror the public `shape_paragraph`
    // signature this helper is factored out of and the convention used across
    // this pass.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn shape_paragraph_uncached(
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
            let (font, face, synth_oblique, synth_bold) = match self.resolver.resolve(&request) {
                Some(resolved) => {
                    let oblique = resolved.synth_oblique();
                    let bold = resolved.synth_bold();
                    (resolved.font(), resolved.into_face(), oblique, bold)
                }
                // A default-face fallback carries no synthesis: nothing matched to
                // fake a style against.
                None => (FontId::PRIMARY, default_face.clone(), false, false),
            };

            // css-fonts-4 §5.3 canonical equivalence: when the itemized run is a
            // multi-codepoint sequence canonically equivalent to a single
            // precomposed character that the resolved font supports, shape that
            // single character so the whole cluster maps to its one glyph — rather
            // than the base plus a (possibly `.notdef`) mark. The `range` used for
            // rebasing stays the ORIGINAL sub-run bytes, so the composed glyph's
            // cluster still points at the sequence's start in the source text.
            let composed = self.canonical_compose(sub, families, attributes, item.script(), font);
            let shape_text: &str = composed.as_deref().unwrap_or(sub);

            let mut shape_request =
                ShapingRequest::new(shape_text, face, font_size).with_direction(item.direction());
            if let Some(language) = &self.language {
                shape_request = shape_request.with_language(language.clone());
            }
            let run = self.shaper.shape(&shape_request);

            runs.push(
                ShapedRun::new(font, rebase_run(&run, range))
                    .with_level(item.level())
                    .with_synthesis(synth_oblique, synth_bold),
            );
        }
        runs
    }

    #[doc = include_str!("paragraph_shaper.doc/canonical_compose.md")]
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn canonical_compose(
        &self,
        sub: &str,
        families: &FontFallback,
        attributes: &FontAttributes,
        script: ScriptTag,
        font: FontId,
    ) -> Option<String> {
        // Only a multi-codepoint sequence can be canonically equivalent to a
        // single character; a single scalar (or empty) never composes.
        sub.chars().nth(1)?;
        let nfc: String = sub.nfc().collect();
        // The composition must collapse to exactly one scalar, and must actually
        // change the text (an already-composed or non-composable run is left alone).
        let mut chars = nfc.chars();
        let c = chars.next()?;
        if chars.next().is_some() || nfc == sub {
            return None;
        }
        // css §5.3: compose only when the font supports the precomposed character.
        // Resolving `c` against the same chain and requiring the SAME font keeps a
        // directly-supported sequence (resolved via a different or whole-sequence
        // path) on its decomposed glyphs.
        let mut buf = [0u8; 4];
        let request =
            FontResolveRequest::new(families, attributes, c.encode_utf8(&mut buf), script);
        match self.resolver.resolve(&request) {
            Some(resolved) if resolved.font() == font => Some(nfc),
            _ => None,
        }
    }
}

#[doc = include_str!("paragraph_shaper.doc/rebase_run.md")]
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

    /// A stub itemizer emitting a single right-to-left item at odd bidi level 1
    /// spanning the whole text, to check the shaper tags the run's level.
    struct RtlItemizer;

    impl TextItemizer for RtlItemizer {
        fn itemize(&self, text: &str, _base_direction: TextDirection) -> Vec<TextItem> {
            vec![TextItem::new(
                TextRange::new(0, text.len()),
                ScriptTag::ARABIC,
                TextDirection::RightToLeft,
                1,
            )]
        }
    }

    /// The language hint of every request a [`RecordingShaper`] was handed, kept
    /// outside it because the orchestrator owns the shaper once installed.
    type Recorded = std::rc::Rc<RefCell<Vec<Option<String>>>>;

    /// A shaper that records the language hint of every request it is handed, so
    /// a test can assert what actually reached the port rather than what was
    /// configured.
    struct RecordingShaper(Recorded);

    impl TextShaper for RecordingShaper {
        fn shape(&self, request: &ShapingRequest) -> GlyphRun {
            self.0
                .borrow_mut()
                .push(request.language().map(ToOwned::to_owned));
            MockTextShaper::new().shape(request)
        }

        fn can_shape(&self, face: &FontFace) -> bool {
            MockTextShaper::new().can_shape(face)
        }
    }

    /// A two-run (`abВГ`) paragraph shaper over a [`RecordingShaper`].
    fn recording_shaper() -> (ParagraphShaper, Recorded) {
        let mut resolver = MockFontResolver::new();
        resolver.register(ScriptTag::LATIN, FontId::new(0), face("Latin"));
        resolver.register(ScriptTag::CYRILLIC, FontId::new(1), face("Cyrillic"));
        let recorded = Recorded::default();
        let shaper = ParagraphShaper::new(
            Box::new(TwoScriptItemizer),
            Box::new(resolver),
            Box::new(RecordingShaper(Recorded::clone(&recorded))),
        );
        (shaper, recorded)
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
    fn setters_swap_ports_and_line_metrics_delegates() {
        // Start single-run (empty resolver → one PRIMARY run), then swap in the
        // two-script itemizer + a resolver so the same text splits into two fonts.
        let mut shaper = ParagraphShaper::new(
            Box::new(SingleRunItemizer::new()),
            Box::new(MockFontResolver::new()),
            Box::new(MockTextShaper::new()),
        );
        let default = face("Fallback");
        assert_eq!(
            shaper
                .shape_paragraph(
                    "abВГ",
                    &chain(),
                    &attrs(),
                    &default,
                    16.0,
                    TextDirection::LeftToRight
                )
                .len(),
            1
        );

        shaper.set_itemizer(Box::new(TwoScriptItemizer));
        let mut resolver = MockFontResolver::new();
        resolver.register(ScriptTag::LATIN, FontId::new(0), face("Latin"));
        resolver.register(ScriptTag::CYRILLIC, FontId::new(1), face("Cyrillic"));
        shaper.set_resolver(Box::new(resolver));

        let runs = shaper.shape_paragraph(
            "abВГ",
            &chain(),
            &attrs(),
            &default,
            16.0,
            TextDirection::LeftToRight,
        );
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].font(), FontId::new(0));
        assert_eq!(runs[1].font(), FontId::new(1));

        // Swapping the inner shaper is observable and line metrics delegate to it.
        shaper.set_shaper(Box::new(MockTextShaper::new()));
        assert_eq!(
            shaper.line_metrics(16.0),
            MockTextShaper::new().line_metrics(16.0)
        );
    }

    #[test]
    fn tags_run_with_item_bidi_level() {
        // A two-script (LTR then LTR) run tags both at even level 0.
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
        assert_eq!(runs[0].level(), 0);
        assert_eq!(runs[1].level(), 0);

        // An RTL item at odd level 1 tags the resulting run's level odd.
        let rtl_shaper = ParagraphShaper::new(
            Box::new(RtlItemizer),
            Box::new(MockFontResolver::new()),
            Box::new(MockTextShaper::new()),
        );
        let rtl_runs = rtl_shaper.shape_paragraph(
            "hello",
            &chain(),
            &attrs(),
            &default,
            16.0,
            TextDirection::LeftToRight,
        );
        assert_eq!(rtl_runs.len(), 1);
        assert_eq!(rtl_runs[0].level(), 1);
        assert!(
            rtl_runs[0].level() % 2 == 1,
            "RTL item must carry an odd level"
        );
    }

    /// A resolver that always resolves to font 0 with both synthesis flags set,
    /// to prove the shaper carries synthesis from the [`ResolvedFont`] onto every
    /// [`ShapedRun`].
    struct SynthResolver;

    impl FontResolver for SynthResolver {
        fn resolve(
            &self,
            _request: &crate::font::FontResolveRequest,
        ) -> Option<crate::font::ResolvedFont> {
            Some(
                crate::font::ResolvedFont::new(FontId::new(0), face("Synth"))
                    .with_synthesis(true, true),
            )
        }
    }

    #[test]
    fn propagates_synthesis_from_resolved_font_onto_runs() {
        let shaper = ParagraphShaper::new(
            Box::new(TwoScriptItemizer),
            Box::new(SynthResolver),
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
        for run in &runs {
            assert!(run.synth_oblique(), "oblique flag must reach the run");
            assert!(run.synth_bold(), "bold flag must reach the run");
        }

        // A default-face fallback (empty resolver) synthesizes nothing.
        let plain = ParagraphShaper::new(
            Box::new(TwoScriptItemizer),
            Box::new(MockFontResolver::new()),
            Box::new(MockTextShaper::new()),
        );
        let runs = plain.shape_paragraph(
            "abВГ",
            &chain(),
            &attrs(),
            &default,
            16.0,
            TextDirection::LeftToRight,
        );
        for run in &runs {
            assert!(!run.synth_oblique());
            assert!(!run.synth_bold());
        }
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

    use crate::font::FontStretch;
    use std::cell::Cell;
    use std::rc::Rc;

    /// An itemizer that counts every [`itemize`](TextItemizer::itemize) call in a
    /// shared cell (readable after it is boxed into the shaper) and delegates the
    /// actual itemization to a [`SingleRunItemizer`]. Proves how often the cache
    /// reaches the underlying pipeline.
    struct CountingItemizer {
        calls: Rc<Cell<usize>>,
        inner: SingleRunItemizer,
    }

    impl CountingItemizer {
        fn new(calls: Rc<Cell<usize>>) -> Self {
            Self {
                calls,
                inner: SingleRunItemizer::new(),
            }
        }
    }

    impl TextItemizer for CountingItemizer {
        fn itemize(&self, text: &str, base_direction: TextDirection) -> Vec<TextItem> {
            self.calls.set(self.calls.get() + 1);
            self.inner.itemize(text, base_direction)
        }
    }

    /// Builds a single-font shaper whose itemizer counts calls into `calls`.
    fn counting_shaper(calls: Rc<Cell<usize>>) -> ParagraphShaper {
        ParagraphShaper::new(
            Box::new(CountingItemizer::new(calls)),
            Box::new(MockFontResolver::new()),
            Box::new(MockTextShaper::new()),
        )
    }

    #[test]
    fn identical_calls_itemize_once() {
        let calls = Rc::new(Cell::new(0));
        let shaper = counting_shaper(calls.clone());
        let default = face("Fallback");
        let a = shaper.shape_paragraph(
            "hello",
            &chain(),
            &attrs(),
            &default,
            16.0,
            TextDirection::LeftToRight,
        );
        let b = shaper.shape_paragraph(
            "hello",
            &chain(),
            &attrs(),
            &default,
            16.0,
            TextDirection::LeftToRight,
        );
        assert_eq!(calls.get(), 1, "second identical call must hit the cache");
        assert_eq!(a, b);
        assert_eq!(shaper.cache_len(), 1);
    }

    #[test]
    fn each_key_dimension_misses_independently() {
        // Vary exactly one keyed input at a time from a shared baseline; each
        // variant must reach the itemizer, proving no false collision on any of
        // text, size, direction, families, attributes, or default_face.
        let calls = Rc::new(Cell::new(0));
        let shaper = counting_shaper(calls.clone());
        let default = face("Fallback");
        let other_family = FontFallback::new(FontFamily::named("Roboto"));
        let other_attrs = attrs().with_stretch(FontStretch::Condensed);
        let other_face = face("OtherFallback");

        // Baseline, then one changed dimension per call.
        let _ = shaper.shape_paragraph(
            "hi",
            &chain(),
            &attrs(),
            &default,
            16.0,
            TextDirection::LeftToRight,
        );
        let _ = shaper.shape_paragraph(
            "bye",
            &chain(),
            &attrs(),
            &default,
            16.0,
            TextDirection::LeftToRight,
        ); // text
        let _ = shaper.shape_paragraph(
            "hi",
            &chain(),
            &attrs(),
            &default,
            18.0,
            TextDirection::LeftToRight,
        ); // size
        let _ = shaper.shape_paragraph(
            "hi",
            &chain(),
            &attrs(),
            &default,
            16.0,
            TextDirection::RightToLeft,
        ); // direction
        let _ = shaper.shape_paragraph(
            "hi",
            &other_family,
            &attrs(),
            &default,
            16.0,
            TextDirection::LeftToRight,
        ); // families
        let _ = shaper.shape_paragraph(
            "hi",
            &chain(),
            &other_attrs,
            &default,
            16.0,
            TextDirection::LeftToRight,
        ); // attributes
        let _ = shaper.shape_paragraph(
            "hi",
            &chain(),
            &attrs(),
            &other_face,
            16.0,
            TextDirection::LeftToRight,
        ); // default_face
        assert_eq!(calls.get(), 7, "each distinct key must reach the itemizer");
        assert_eq!(shaper.cache_len(), 7);

        // Re-issuing the baseline is a hit, not an eighth itemize.
        let _ = shaper.shape_paragraph(
            "hi",
            &chain(),
            &attrs(),
            &default,
            16.0,
            TextDirection::LeftToRight,
        );
        assert_eq!(calls.get(), 7, "the baseline repeat must hit the cache");
    }

    #[test]
    fn cached_output_equals_uncached() {
        // A cached shaper and a fresh uncached shaper (its cache never consulted
        // because every input differs) must agree run-for-run.
        let cached = ParagraphShaper::new(
            Box::new(TwoScriptItemizer),
            Box::new(two_font_resolver()),
            Box::new(MockTextShaper::new()),
        );
        let uncached = ParagraphShaper::new(
            Box::new(TwoScriptItemizer),
            Box::new(two_font_resolver()),
            Box::new(MockTextShaper::new()),
        );
        let default = face("Fallback");
        let direct = uncached.shape_paragraph(
            "abВГ",
            &chain(),
            &attrs(),
            &default,
            16.0,
            TextDirection::LeftToRight,
        );
        let via_cache = cached.shape_paragraph(
            "abВГ",
            &chain(),
            &attrs(),
            &default,
            16.0,
            TextDirection::LeftToRight,
        );
        let via_cache_again = cached.shape_paragraph(
            "abВГ",
            &chain(),
            &attrs(),
            &default,
            16.0,
            TextDirection::LeftToRight,
        );
        assert_eq!(direct, via_cache);
        assert_eq!(direct, via_cache_again);
    }

    #[test]
    fn capacity_bound_flushes_and_stays_correct() {
        let cap = 4;
        let shaper = ParagraphShaper::with_capacity(
            Box::new(SingleRunItemizer::new()),
            Box::new(MockFontResolver::new()),
            Box::new(MockTextShaper::new()),
            cap,
        );
        let default = face("Fallback");
        for i in 0..50 {
            let text = format!("t{i}");
            let runs = shaper.shape_paragraph(
                &text,
                &chain(),
                &attrs(),
                &default,
                12.0,
                TextDirection::LeftToRight,
            );
            // Correct despite the bound.
            let uncached = ParagraphShaper::new(
                Box::new(SingleRunItemizer::new()),
                Box::new(MockFontResolver::new()),
                Box::new(MockTextShaper::new()),
            );
            assert_eq!(
                runs,
                uncached.shape_paragraph(
                    &text,
                    &chain(),
                    &attrs(),
                    &default,
                    12.0,
                    TextDirection::LeftToRight
                )
            );
            assert!(
                shaper.cache_len() <= cap,
                "cache exceeded capacity: {}",
                shaper.cache_len()
            );
        }
    }

    #[test]
    fn swapping_a_port_flushes_the_cache() {
        let mut shaper = ParagraphShaper::new(
            Box::new(SingleRunItemizer::new()),
            Box::new(MockFontResolver::new()),
            Box::new(MockTextShaper::new()),
        );
        let default = face("Fallback");
        let before = shaper.shape_paragraph(
            "abВГ",
            &chain(),
            &attrs(),
            &default,
            16.0,
            TextDirection::LeftToRight,
        );
        assert_eq!(before.len(), 1);
        assert_eq!(shaper.cache_len(), 1);

        // Swapping in the two-script itemizer + resolver must not serve the stale
        // single-run entry: the same text now splits into two runs.
        shaper.set_itemizer(Box::new(TwoScriptItemizer));
        assert_eq!(shaper.cache_len(), 0, "port swap must flush the cache");
        shaper.set_resolver(Box::new(two_font_resolver()));
        let after = shaper.shape_paragraph(
            "abВГ",
            &chain(),
            &attrs(),
            &default,
            16.0,
            TextDirection::LeftToRight,
        );
        assert_eq!(after.len(), 2);
    }

    /// A resolver mapping Latin→font 0 and Cyrillic→font 1 (mirrors the two-font
    /// layout tests).
    fn two_font_resolver() -> MockFontResolver {
        let mut resolver = MockFontResolver::new();
        resolver.register(ScriptTag::LATIN, FontId::new(0), face("Latin"));
        resolver.register(ScriptTag::CYRILLIC, FontId::new(1), face("Cyrillic"));
        resolver
    }

    // ---- the language hint (spec §5.10, §7.2 text shaping integration) ----

    #[test]
    fn without_a_language_no_request_carries_one() {
        // The state of the world before `set_language` existed: `ShapingRequest`
        // has always had the field and nothing ever filled it.
        let (shaper, recorded) = recording_shaper();
        assert_eq!(shaper.language(), None);
        let _ = shaper.shape_paragraph(
            "abВГ",
            &chain(),
            &attrs(),
            &face("Fallback"),
            16.0,
            TextDirection::LeftToRight,
        );
        assert_eq!(&*recorded.borrow(), &[None, None]);
    }

    #[test]
    fn the_installed_language_reaches_every_shaping_request() {
        let (mut shaper, recorded) = recording_shaper();
        shaper.set_language(Some("ru".into()));
        assert_eq!(shaper.language(), Some("ru"));
        let _ = shaper.shape_paragraph(
            "abВГ",
            &chain(),
            &attrs(),
            &face("Fallback"),
            16.0,
            TextDirection::LeftToRight,
        );
        // Both itemized runs, not just the first: the hint is the paragraph's.
        assert_eq!(
            &*recorded.borrow(),
            &[Some("ru".to_owned()), Some("ru".to_owned())]
        );
    }

    #[test]
    fn changing_the_language_flushes_the_cache_and_reshapes() {
        // Without the flush the second call would hit the memoized runs and the
        // new hint would never reach the shaper — the language would appear to be
        // installed and be silently inert.
        let (mut shaper, recorded) = recording_shaper();
        let default = face("Fallback");
        let shape = |shaper: &ParagraphShaper| {
            let _ = shaper.shape_paragraph(
                "abВГ",
                &chain(),
                &attrs(),
                &default,
                16.0,
                TextDirection::LeftToRight,
            );
        };

        shaper.set_language(Some("ru".into()));
        shape(&shaper);
        assert_eq!(shaper.cache_len(), 1);

        shaper.set_language(Some("sr".into()));
        assert_eq!(
            shaper.cache_len(),
            0,
            "a language swap must flush the cache"
        );
        shape(&shaper);

        assert_eq!(
            &*recorded.borrow(),
            &[
                Some("ru".to_owned()),
                Some("ru".to_owned()),
                Some("sr".to_owned()),
                Some("sr".to_owned()),
            ]
        );
    }

    #[test]
    fn clearing_the_language_unsets_the_hint() {
        let (mut shaper, recorded) = recording_shaper();
        shaper.set_language(Some("ru".into()));
        shaper.set_language(None);
        assert_eq!(shaper.language(), None);
        let _ = shaper.shape_paragraph(
            "abВГ",
            &chain(),
            &attrs(),
            &face("Fallback"),
            16.0,
            TextDirection::LeftToRight,
        );
        assert_eq!(&*recorded.borrow(), &[None, None]);
    }
}
