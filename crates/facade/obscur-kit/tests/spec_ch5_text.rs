#![doc = include_str!("spec_ch5_text.doc/_module.md")]
#![forbid(unsafe_code)]

#[path = "spec_ch5_support/mod.rs"]
mod support;

use obscur_kit::text::font::{
    FontAttributes, FontFace, FontFallback, FontFamily, FontId, FontMetrics, FontResolveRequest,
    FontResolver, FontSource, ResolvedFont,
};
use obscur_kit::text::layout::{ParagraphLayout, TextAlignment, TextOverflow, WhiteSpace};
use obscur_kit::text::selection::SelectionGeometry;
use obscur_kit::text::shaping::{
    MockTextShaper, ParagraphShaper, ShapedRun, TextDirection, TextItemizer,
};

// ---------------------------------------------------------------------------
// A coverage-driven font resolver: the fallback model css-fonts-4 §5.3 describes.
// ---------------------------------------------------------------------------

#[doc = include_str!("spec_ch5_text.doc/Coverage.md")]
struct Coverage {
    face: FontFace,
    id: FontId,
    covers: Vec<char>,
}

#[doc = include_str!("spec_ch5_text.doc/CoverageResolver.md")]
struct CoverageResolver {
    fonts: Vec<Coverage>,
}

impl FontResolver for CoverageResolver {
    fn resolve(&self, request: &FontResolveRequest) -> Option<ResolvedFont> {
        let wanted: Vec<char> = request.text().chars().collect();
        // Whole-sequence coverage, in chain order.
        for font in &self.fonts {
            if wanted.iter().all(|c| font.covers.contains(c)) {
                return Some(ResolvedFont::new(font.id, font.face.clone()));
            }
        }
        // Canonical equivalence: e + U+0301 is canonically equivalent to U+00E9.
        if wanted == ['e', '\u{0301}'] {
            for font in &self.fonts {
                if font.covers.contains(&'\u{00E9}') {
                    return Some(ResolvedFont::new(font.id, font.face.clone()));
                }
            }
        }
        None
    }
}

#[doc = include_str!("spec_ch5_text.doc/face.md")]
fn face(name: &str) -> FontFace {
    FontFace::new(
        FontFamily::named(name),
        FontAttributes::default(),
        FontSource::system(name),
        FontMetrics::new(1000, 800.0, -200.0, 0.0),
    )
}

#[doc = include_str!("spec_ch5_text.doc/itemizer.md")]
#[cfg(feature = "text-unicode")]
fn itemizer() -> Box<dyn TextItemizer> {
    Box::new(obscur_kit::text_unicode::UnicodeTextItemizer::new())
}

#[cfg(not(feature = "text-unicode"))]
fn itemizer() -> Box<dyn TextItemizer> {
    Box::new(obscur_kit::text::shaping::SingleRunItemizer)
}

#[doc = include_str!("spec_ch5_text.doc/shape_with.md")]
fn shape_with(text: &str, fonts: Vec<Coverage>, families: &[&str]) -> Vec<ShapedRun> {
    let chain =
        FontFallback::from_families(families.iter().map(|n| FontFamily::named(*n)).collect());
    let default_face = face("default");
    let shaper = ParagraphShaper::new(
        itemizer(),
        Box::new(CoverageResolver { fonts }),
        Box::new(MockTextShaper::new()),
    );
    shaper.shape_paragraph(
        text,
        &chain,
        &FontAttributes::default(),
        &default_face,
        16.0,
        TextDirection::LeftToRight,
    )
}

#[doc = include_str!("spec_ch5_text.doc/tx_01_a_cluster_falls_back_as_one_unit.md")]
#[test]
fn tx_01_a_cluster_falls_back_as_one_unit() {
    // FontA has 'e' but not the combining acute; FontB has both.
    let runs = shape_with(
        "e\u{0301}",
        vec![
            Coverage {
                face: face("FontA"),
                id: FontId::new(1),
                covers: vec!['e'],
            },
            Coverage {
                face: face("FontB"),
                id: FontId::new(2),
                covers: vec!['e', '\u{0301}'],
            },
        ],
        &["FontA", "FontB"],
    );

    assert_eq!(
        runs.len(),
        1,
        "the cluster stays in ONE run; per-codepoint fallback yields 2. \
         Runs: {:?}",
        runs.iter()
            .map(|r| (r.font(), r.run().len()))
            .collect::<Vec<_>>()
    );
    assert_eq!(runs[0].font(), FontId::new(2), "the run's face is FontB");
    assert_eq!(runs[0].run().len(), 2, "base + mark, both from FontB");
    assert!(
        runs[0].run().glyphs().iter().all(|g| g.glyph_id() != 0),
        "zero .notdef glyphs"
    );
}

#[doc = include_str!("spec_ch5_text.doc/tx_02_a_canonically_equivalent_cluster_becomes_one_glyph.md")]
#[test]
fn tx_02_a_canonically_equivalent_cluster_becomes_one_glyph() {
    // FontC has 'e' and 'é' but NOT the combining acute.
    let fonts = || {
        vec![Coverage {
            face: face("FontC"),
            id: FontId::new(3),
            covers: vec!['e', '\u{00E9}'],
        }]
    };

    let runs = shape_with("e\u{0301}", fonts(), &["FontC"]);
    assert_eq!(runs.len(), 1, "one run for the whole cluster");
    assert_eq!(runs[0].font(), FontId::new(3), "the run's face is FontC");
    assert_eq!(
        runs[0].run().len(),
        1,
        "the cluster maps to the single precomposed glyph, not 'e' + .notdef"
    );
    assert_eq!(
        runs[0].run().glyphs()[0].glyph_id(),
        u32::from('\u{00E9}'),
        "the glyph is the one mapped from U+00E9"
    );

    let precomposed = shape_with("\u{00E9}", fonts(), &["FontC"]);
    let advance_of = |rs: &[ShapedRun]| rs.iter().map(|r| r.run().advance()).sum::<f32>();
    assert!(
        (advance_of(&runs) - advance_of(&precomposed)).abs() < 1e-4,
        "the cluster's advance equals the advance of U+00E9 alone: {} vs {}",
        advance_of(&runs),
        advance_of(&precomposed)
    );
}

#[doc = include_str!("spec_ch5_text.doc/tx_03_collapsed_spaces_have_zero_advance_and_keep_their_offsets.md")]
#[test]
fn tx_03_collapsed_spaces_have_zero_advance_and_keep_their_offsets() {
    let collapsed = plain_layout("a   b");
    let reference = plain_layout("a b");

    assert!(
        (line_advance(&collapsed) - line_advance(&reference)).abs() < 1e-3,
        "'a   b' measures the same as 'a b': {} vs {}",
        line_advance(&collapsed),
        line_advance(&reference)
    );

    let x = |offset: usize| caret_x(&collapsed, offset);
    assert!(
        x(1) < x(2),
        "the FIRST space advances: x(1)={} x(2)={}",
        x(1),
        x(2)
    );
    assert!(
        (x(2) - x(3)).abs() < 1e-4 && (x(3) - x(4)).abs() < 1e-4,
        "the second and third spaces contribute zero advance: \
         x(2)={} x(3)={} x(4)={}",
        x(2),
        x(3),
        x(4)
    );
    assert!(x(5) > x(4), "'b' advances: x(4)={} x(5)={}", x(4), x(5));

    // Six valid caret offsets exist, 0..=5 — the source string is not rewritten.
    let clusters: Vec<usize> = collapsed
        .lines()
        .iter()
        .flat_map(|l| l.segments())
        .flat_map(|s| s.run().glyphs())
        .map(|g| g.cluster())
        .collect();
    assert_eq!(
        clusters,
        vec![0, 1, 2, 3, 4],
        "every source character keeps its own cluster; deleting the collapsed \
         characters from the buffer shifts every downstream offset"
    );
}

#[doc = include_str!("spec_ch5_text.doc/tx_04_an_unbreakable_word_overflows_rather_than_breaking.md")]
#[test]
fn tx_04_an_unbreakable_word_overflows_rather_than_breaking() {
    let text = "a".repeat(20);

    // overflow-wrap: normal — no soft wrap opportunity inside the word.
    let normal = wrapped(&text, 40.0);
    assert_eq!(
        normal.line_count(),
        1,
        "with no break opportunity the word takes exactly one line, not {}",
        normal.line_count()
    );
    assert!(
        line_advance(&normal) > 40.0,
        "the line overflows its 40px container rather than being broken or \
         clipped away, got {}",
        line_advance(&normal)
    );

    // overflow-wrap: anywhere — an emergency break is permitted.
    let anywhere = wrapped_anywhere(&text, 40.0);
    assert!(
        anywhere.line_count() >= 2,
        "`overflow-wrap: anywhere` breaks the word, got {} line(s)",
        anywhere.line_count()
    );
    for (i, line) in anywhere.lines().iter().enumerate() {
        assert!(
            line.width() <= 40.0 + 1e-3,
            "line {i} fits the 40px container, got {}",
            line.width()
        );
    }
    // The line ranges cover 0..20 with no gaps and no overlaps.
    let mut next = 0usize;
    for (i, line) in anywhere.lines().iter().enumerate() {
        assert_eq!(
            line.range().start,
            next,
            "line {i} resumes where {i}-1 ended"
        );
        next = line.range().end;
    }
    assert_eq!(next, 20, "the lines cover the whole 20-character word");
}

#[doc = include_str!("spec_ch5_text.doc/tx_05_bidi_selection_is_disjoint_and_runs_reorder.md")]
#[test]
fn tx_05_bidi_selection_is_disjoint_and_runs_reorder() {
    let text = "abc\u{05D0}\u{05D1}\u{05D2}def";
    let layout = plain_layout(text);

    // Selection of the logical range [2,5): 'c' and the first two Hebrew letters.
    let geometry = SelectionGeometry::from_paragraph(&layout, 2, 5);
    assert_eq!(
        geometry.len(),
        2,
        "a bidi selection is 2 disjoint rectangles, got {:?}",
        geometry.rects()
    );

    // The unselected U+05D2 sits visually between the two selected pieces; no
    // rectangle may cover it.
    let d2_x = caret_x(&layout, 3 + 2); // byte offset of U+05D2's cluster
    for rect in geometry.rects() {
        let covers = rect.origin.x < d2_x && d2_x < rect.origin.x + rect.size.width;
        assert!(
            !covers,
            "no selection rect may cover the UNSELECTED U+05D2 at x={d2_x}: {rect:?}"
        );
    }

    // Visual left-to-right glyph order: a b c U+05D2 U+05D1 U+05D0 d e f.
    let visual: Vec<char> = layout
        .lines()
        .iter()
        .flat_map(|l| l.segments())
        .flat_map(|s| s.run().glyphs())
        .filter_map(|g| char::from_u32(g.glyph_id()))
        .collect();
    assert_eq!(
        visual,
        vec![
            'a', 'b', 'c', '\u{05D2}', '\u{05D1}', '\u{05D0}', 'd', 'e', 'f'
        ],
        "the Hebrew run reverses as a unit; the Latin runs do not"
    );
}

// ---------------------------------------------------------------------------
// Measurement helpers. These MEASURE; they carry no expectations.
// ---------------------------------------------------------------------------

#[doc = include_str!("spec_ch5_text.doc/universal_shaper.md")]
fn universal_shaper() -> ParagraphShaper {
    ParagraphShaper::new(
        itemizer(),
        Box::new(CoverageResolver { fonts: Vec::new() }),
        Box::new(MockTextShaper::new()),
    )
}

#[doc = include_str!("spec_ch5_text.doc/plain_layout.md")]
fn plain_layout(text: &str) -> ParagraphLayout {
    wrapped(text, f32::INFINITY)
}

#[doc = include_str!("spec_ch5_text.doc/wrapped.md")]
fn wrapped(text: &str, max_width: f32) -> ParagraphLayout {
    let shaper = universal_shaper();
    let runs = shaper.shape_paragraph(
        text,
        &FontFallback::new(FontFamily::named("Test")),
        &FontAttributes::default(),
        &face("Test"),
        16.0,
        TextDirection::LeftToRight,
    );
    ParagraphLayout::wrap(
        &runs,
        text,
        max_width,
        16.0,
        12.8,
        TextAlignment::Start,
        TextOverflow::Visible,
        WhiteSpace::Normal,
    )
}

#[doc = include_str!("spec_ch5_text.doc/wrapped_anywhere.md")]
fn wrapped_anywhere(text: &str, max_width: f32) -> ParagraphLayout {
    use obscur_kit::text::layout::Hyphenator;

    #[doc = include_str!("spec_ch5_text.doc/Anywhere.md")]
    struct Anywhere;
    impl Hyphenator for Anywhere {
        fn hyphenate(&self, word: &str) -> Vec<usize> {
            word.char_indices().map(|(i, _)| i).skip(1).collect()
        }
    }

    let shaper = universal_shaper();
    let runs = shaper.shape_paragraph(
        text,
        &FontFallback::new(FontFamily::named("Test")),
        &FontAttributes::default(),
        &face("Test"),
        16.0,
        TextDirection::LeftToRight,
    );
    ParagraphLayout::wrap_hyphenated(
        &runs,
        text,
        max_width,
        16.0,
        12.8,
        TextAlignment::Start,
        TextOverflow::Visible,
        WhiteSpace::Normal,
        &Anywhere,
        None,
        None,
    )
}

#[doc = include_str!("spec_ch5_text.doc/line_advance.md")]
fn line_advance(layout: &ParagraphLayout) -> f32 {
    layout
        .lines()
        .iter()
        .map(obscur_kit::text::layout::ParagraphLine::width)
        .fold(0.0_f32, f32::max)
}

#[doc = include_str!("spec_ch5_text.doc/caret_x.md")]
fn caret_x(layout: &ParagraphLayout, byte_offset: usize) -> f32 {
    let Some(line) = layout.lines().first() else {
        return 0.0;
    };
    let mut pen = line.x_offset();
    for segment in line.segments() {
        for glyph in segment.run().glyphs() {
            if byte_offset <= glyph.cluster() {
                return pen;
            }
            pen += glyph.x_advance();
        }
    }
    pen
}
