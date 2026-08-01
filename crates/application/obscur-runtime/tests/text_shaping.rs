#![doc = include_str!("text_shaping.doc/_module.md")]
#![forbid(unsafe_code)]

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use obscur_identity::ComponentId;
use obscur_localization::locale::LanguageTag;
use obscur_runtime::UiRuntime;
use obscur_runtime::frame::FrameResult;
use obscur_scene::command::PaintCommand;
use obscur_scene::text::PositionedGlyph;
use obscur_testkit::fixture::TestFixture;
use obscur_text::font::{
    FontAttributes, FontFace, FontFamily, FontId, FontMetrics, FontSource, GenericFamily,
    MockFontResolver,
};
use obscur_text::shaping::{
    GlyphRun, MockTextShaper, ScriptTag, ShapedGlyph, ShapingRequest, SingleRunItemizer, TextItem,
    TextItemizer, TextShaper,
};
use obscur_text::span::TextRange;
use obscur_view::view::{View, ViewFragment, ViewNode};

/// A view holding one text node with `content`, under one element.
fn text_view(component: u64, content: &str) -> View {
    View::from_fragment(ViewFragment::from_nodes(vec![
        ViewNode::element(ComponentId::from_raw(component))
            .with_children(ViewFragment::from_nodes(vec![ViewNode::text(content)])),
    ]))
}

/// Collects the glyph ids of every `DrawText` run in a frame's display list.
fn drawtext_glyph_ids(result: &FrameResult) -> Vec<u32> {
    result
        .snapshot()
        .display_list()
        .commands()
        .iter()
        .filter_map(|c| match c {
            PaintCommand::DrawText { run, .. } => Some(run),
            _ => None,
        })
        .flat_map(|run| run.glyphs().iter().map(PositionedGlyph::glyph_id))
        .collect()
}

/// Collects the `(font, glyph_ids)` of every `DrawText` run in a frame, in
/// paint order.
fn drawtext_font_runs(result: &FrameResult) -> Vec<(u32, Vec<u32>)> {
    result
        .snapshot()
        .display_list()
        .commands()
        .iter()
        .filter_map(|c| match c {
            PaintCommand::DrawText { run, .. } => Some((
                run.font(),
                run.glyphs().iter().map(PositionedGlyph::glyph_id).collect(),
            )),
            _ => None,
        })
        .collect()
}

/// A stub [`TextShaper`] emitting a fixed glyph id per source char, so an
/// installed shaper is observable in the composed `DrawText` run.
struct FixedIdShaper(u32);

impl TextShaper for FixedIdShaper {
    fn shape(&self, request: &ShapingRequest) -> GlyphRun {
        let glyphs = request
            .text()
            .char_indices()
            .map(|(byte, _)| ShapedGlyph::new(self.0, byte, 8.0))
            .collect();
        GlyphRun::new(
            glyphs,
            request.direction(),
            request.font_size(),
            TextRange::new(0, request.text().len()),
        )
    }

    fn can_shape(&self, _face: &FontFace) -> bool {
        true
    }
}

#[test]
fn set_text_shaper_changes_the_emitted_glyph_run() {
    // A view with a single text node "AB": the pipeline lays it out and the
    // scene coordinator emits a DrawText run for it.
    let view = text_view(10, "AB");

    // Default (mock) shaper: glyph ids are the Unicode scalars of "AB".
    let mut runtime = TestFixture::runtime();
    let mock = runtime.render_now(&view, &[]);
    let mock_ids = drawtext_glyph_ids(&mock);
    assert_eq!(mock_ids, vec![u32::from('A'), u32::from('B')]);

    // Install a stub shaper through the public runtime seam and re-render: the
    // composed run now reflects it, proving the shaper injection plumbing.
    runtime.set_text_shaper(Box::new(FixedIdShaper(7)));
    let stub = runtime.render_now(&view, &[]);
    let stub_ids = drawtext_glyph_ids(&stub);
    assert_eq!(stub_ids, vec![7, 7]);
    assert_ne!(mock_ids, stub_ids);
}

/// A shaper that tallies every `shape` call through a shared cell, so a test
/// can read the count after the shaper is moved behind the runtime seam.
struct TallyShaper(Rc<Cell<usize>>);

impl TextShaper for TallyShaper {
    fn shape(&self, request: &ShapingRequest) -> GlyphRun {
        self.0.set(self.0.get() + 1);
        MockTextShaper::new().shape(request)
    }

    fn can_shape(&self, _face: &FontFace) -> bool {
        true
    }
}

#[test]
fn shaping_cache_dedups_measure_arrange_and_compose_within_a_frame() {
    // A single text node needs shaping in the layout measure pass, the layout
    // arrange pass, AND the scene compose pass. The runtime wraps the installed
    // shaper in a CachingShaper, so the underlying engine must be invoked
    // exactly ONCE for that one unique run despite all three needing it.
    let view = text_view(11, "cached");

    let calls = Rc::new(Cell::new(0));
    let mut runtime = TestFixture::runtime();
    runtime.set_text_shaper(Box::new(TallyShaper(Rc::clone(&calls))));

    let frame = runtime.render_now(&view, &[]);
    // The text still shaped correctly (mock: glyph ids are the scalars).
    assert_eq!(
        drawtext_glyph_ids(&frame),
        "cached".chars().map(u32::from).collect::<Vec<_>>()
    );
    // ...but the inner engine ran once, not once per measure/arrange/compose.
    assert_eq!(
        calls.get(),
        1,
        "measure + arrange + compose must share one cached shaping call"
    );
}

/// A stub itemizer cutting the fixed two-script string "abВГ" into a Latin run
/// `[0..2)` and a Cyrillic run `[2..6)` (each Cyrillic char is 2 bytes).
struct TwoScriptItemizer;

impl TextItemizer for TwoScriptItemizer {
    fn itemize(&self, _text: &str, _base: obscur_text::shaping::TextDirection) -> Vec<TextItem> {
        use obscur_text::shaping::TextDirection;
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

#[test]
fn multi_font_text_renders_font_tagged_segments_end_to_end() {
    // A single two-script text node "abВГ". Installing a script itemizer plus a
    // covering resolver through the public runtime seams makes the whole
    // measure → arrange → compose path route through font fallback, so the
    // frame paints two font-tagged DrawText runs on one line.
    let view = text_view(12, "abВГ");

    let mut runtime = TestFixture::runtime();
    runtime.set_text_itemizer(Box::new(TwoScriptItemizer));
    let mut resolver = MockFontResolver::new();
    let face = FontFace::new(
        FontFamily::generic(GenericFamily::Monospace),
        FontAttributes::default(),
        FontSource::system("mock"),
        FontMetrics::new(1000, 800.0, -200.0, 0.0),
    );
    resolver.register(ScriptTag::LATIN, FontId::new(0), face.clone());
    resolver.register(ScriptTag::CYRILLIC, FontId::new(1), face);
    runtime.set_font_resolver(Box::new(resolver));

    let frame = runtime.render_now(&view, &[]);
    let runs = drawtext_font_runs(&frame);
    // Two font-tagged runs: Latin (font 0, "ab") then Cyrillic (font 1, "ВГ").
    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0].0, 0);
    assert_eq!(runs[0].1, vec![u32::from('a'), u32::from('b')]);
    assert_eq!(runs[1].0, 1);
    assert_eq!(runs[1].1, vec![u32::from('В'), u32::from('Г')]);
}

/// An itemizer that counts every `itemize` call into a shared cell (readable
/// after it is boxed into the runtime) and delegates to a single-run
/// itemizer. Proves how often a frame reaches the underlying pipeline.
struct CountingItemizer(Rc<Cell<usize>>);

impl TextItemizer for CountingItemizer {
    fn itemize(&self, text: &str, base: obscur_text::shaping::TextDirection) -> Vec<TextItem> {
        self.0.set(self.0.get() + 1);
        SingleRunItemizer::new().itemize(text, base)
    }
}

#[test]
fn one_frame_itemizes_a_text_node_once_across_measure_arrange_compose() {
    // Requirement (d): a text node is shaped in the layout pass (measure +
    // arrange) AND again in the scene pass (compose), every frame. The
    // paragraph cache — shared because both passes borrow the same
    // `ParagraphShaper` — must collapse all of that to a single itemization.
    let view = text_view(14, "hello");

    let calls = Rc::new(Cell::new(0));
    let mut runtime = TestFixture::runtime();
    runtime.set_text_itemizer(Box::new(CountingItemizer(calls.clone())));

    let _ = runtime.render_now(&view, &[]);
    assert_eq!(
        calls.get(),
        1,
        "measure + arrange + compose must share one cached itemization"
    );
}

#[test]
fn default_single_font_text_stays_one_primary_run() {
    // Requirement (a): with no ports installed the default single-run path
    // renders a text node exactly as before — one DrawText run on the primary
    // face (font 0) whose glyph ids are the Unicode scalars.
    let view = text_view(13, "hi");
    let mut runtime = TestFixture::runtime();
    let runs = drawtext_font_runs(&runtime.render_now(&view, &[]));
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].0, 0);
    assert_eq!(runs[0].1, vec![u32::from('h'), u32::from('i')]);
}

// ---- The language the shaper is told (§7.2) --------------------------------

/// The language hint of every shaping request the runtime issued.
type Recorded = Rc<RefCell<Vec<Option<String>>>>;

/// A shaper recording what actually reached the port. The assertion has to
/// be made here: `set_locale` configures the orchestrator, and only the
/// request the shaper is handed proves the configuration survived the
/// measure and compose passes and the shaping caches between them.
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

/// A runtime whose text shaper records language hints.
fn recording_runtime() -> (UiRuntime, Recorded) {
    let mut runtime = TestFixture::runtime();
    let recorded = Recorded::default();
    runtime.set_text_shaper(Box::new(RecordingShaper(Rc::clone(&recorded))));
    (runtime, recorded)
}

/// A view holding one Russian text node.
fn russian_text() -> View {
    View::from_fragment(ViewFragment::from_nodes(vec![ViewNode::text("файлы")]))
}

fn tag(s: &str) -> LanguageTag {
    LanguageTag::parse(s).expect("valid tag")
}

#[test]
fn the_default_locale_is_und_not_en() {
    // The Anglocentric default §7.2 exists to refuse.
    let runtime = TestFixture::runtime();
    assert!(runtime.locale().is_root());
    assert_eq!(runtime.locale().to_string(), "und");
}

#[test]
fn without_set_locale_no_shaping_request_carries_a_language() {
    // The state this change found: `ShapingRequest::language` exists, the
    // HarfRust adapter reads it, and no production path had ever set it.
    let (mut runtime, recorded) = recording_runtime();
    runtime.render_now(&russian_text(), &[]);
    assert!(
        !recorded.borrow().is_empty(),
        "the frame must have shaped something for this test to mean anything"
    );
    assert!(
        recorded.borrow().iter().all(Option::is_none),
        "expected no language hints, got {:?}",
        recorded.borrow()
    );
}

#[test]
fn set_locale_puts_the_language_on_every_shaping_request() {
    let (mut runtime, recorded) = recording_runtime();
    runtime.set_locale(&tag("ru-RU"));
    runtime.render_now(&russian_text(), &[]);

    assert!(!recorded.borrow().is_empty(), "nothing was shaped");
    assert!(
        recorded
            .borrow()
            .iter()
            .all(|language| language.as_deref() == Some("ru")),
        "expected every request to carry `ru`, got {:?}",
        recorded.borrow()
    );
}

#[test]
fn the_shaper_gets_the_language_subtag_not_the_whole_tag() {
    // `ru-RU` shapes as `ru`: OpenType has no opinion about a region, and
    // handing HarfRust `ru-RU` would be a different (and wrong) hint.
    let (mut runtime, recorded) = recording_runtime();
    runtime.set_locale(&tag("ru-RU"));
    runtime.render_now(&russian_text(), &[]);
    assert_eq!(recorded.borrow().first(), Some(&Some("ru".to_owned())));
    assert_eq!(runtime.locale().to_string(), "ru-RU");
}

/// A shaper whose advances depend on the language hint, standing in for the
/// language-sensitive substitution a real shaper performs (`locl` can swap
/// a glyph for one of a different width). The mock is language-blind, so
/// without this no test in this workspace could tell whether a stale
/// *measured size* survived a locale switch.
struct LanguageWidthShaper;

impl LanguageWidthShaper {
    /// Serbian glyphs are twice as wide as anyone else's, here.
    fn scale(language: Option<&str>) -> f32 {
        if language == Some("sr") { 2.0 } else { 1.0 }
    }
}

impl TextShaper for LanguageWidthShaper {
    fn shape(&self, request: &ShapingRequest) -> GlyphRun {
        let run = MockTextShaper::new().shape(request);
        let scale = Self::scale(request.language());
        let glyphs: Vec<_> = run
            .glyphs()
            .iter()
            .map(|glyph| {
                ShapedGlyph::new(glyph.glyph_id(), glyph.cluster(), glyph.x_advance() * scale)
                    .with_offset(glyph.x_offset(), glyph.y_offset())
            })
            .collect();
        GlyphRun::new(glyphs, run.direction(), run.font_size(), run.source())
    }

    fn can_shape(&self, face: &FontFace) -> bool {
        MockTextShaper::new().can_shape(face)
    }
}

/// The width of the frame's only text box.
fn text_box_width(runtime: &UiRuntime) -> f32 {
    let root = runtime.tree().root().expect("a root");
    let text = runtime.tree().get(root).expect("live").child_nodes()[0];
    runtime
        .layout_boxes()
        .get(&text)
        .expect("the text node was laid out")
        .size()
        .width
}

#[test]
fn a_locale_switch_remeasures_rather_than_serving_a_stale_measured_size() {
    // `set_language` flushes the *paragraph* cache, but the runtime's
    // measure cache retains `measure(node, constraints) -> Size` across
    // frames independently. A language that changes advances therefore
    // changes measured sizes, and that is a layout input no computed-style
    // diff can see — so `set_locale` must drop it too.
    let mut runtime = TestFixture::runtime();
    runtime.set_text_shaper(Box::new(LanguageWidthShaper));

    runtime.set_locale(&tag("ru"));
    runtime.render_now(&russian_text(), &[]);
    let ru_width = text_box_width(&runtime);

    runtime.set_locale(&tag("sr"));
    runtime.render_now(&russian_text(), &[]);
    let sr_width = text_box_width(&runtime);

    assert!(ru_width > 0.0, "the text measured to nothing");
    assert!(
        (sr_width - ru_width * 2.0).abs() < 0.01,
        "expected the `sr` box to be twice the `ru` box, got {ru_width} then {sr_width}"
    );
}

#[test]
fn switching_locale_reshapes_rather_than_serving_the_cached_runs() {
    // Two caches sit between `set_locale` and the shaper — the paragraph
    // cache and the measure cache — and either would happily serve the
    // previous language's runs. This is the assertion that they do not.
    let (mut runtime, recorded) = recording_runtime();
    runtime.set_locale(&tag("ru"));
    runtime.render_now(&russian_text(), &[]);
    recorded.borrow_mut().clear();

    runtime.set_locale(&tag("sr"));
    runtime.render_now(&russian_text(), &[]);

    assert!(
        !recorded.borrow().is_empty(),
        "the swap served a stale cache"
    );
    assert!(
        recorded
            .borrow()
            .iter()
            .all(|language| language.as_deref() == Some("sr")),
        "expected `sr` after the switch, got {:?}",
        recorded.borrow()
    );
}
