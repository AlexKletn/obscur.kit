#![doc = include_str!("text_stack_install.doc/_module.md")]
#![cfg(all(feature = "text-harfrust", feature = "text-unicode"))]
#![forbid(unsafe_code)]

use obscur_kit::bootstrap::{FontFaceBytes, FontSet, KitBootstrap};
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, NodeId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::scene::text::PositionedGlyph;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::text::font::{FontAttributes, FontFamily, GenericFamily};
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("text_stack_install.doc/NOTO_SANS.md")]
const NOTO_SANS: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../assets/fonts/NotoSans-Regular.ttf"
));
const NOTO_SANS_MONO: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../assets/fonts/NotoSansMono-Regular.ttf"
));

#[doc = include_str!("text_stack_install.doc/SANS_LINE_HEIGHT_16.md")]
const SANS_LINE_HEIGHT_16: f32 = 21.792;
#[doc = include_str!("text_stack_install.doc/IDEALIZED_LINE_HEIGHT_16.md")]
const IDEALIZED_LINE_HEIGHT_16: f32 = 16.0;
#[doc = include_str!("text_stack_install.doc/MONO_CH_16.md")]
const MONO_CH_16: f32 = 9.6;
#[doc = include_str!("text_stack_install.doc/SANS_CH_16.md")]
const SANS_CH_16: f32 = 9.152;
#[doc = include_str!("text_stack_install.doc/MOCK_CH_16.md")]
const MOCK_CH_16: f32 = 9.6;
#[doc = include_str!("text_stack_install.doc/CH_SUBSTITUTE_16.md")]
const CH_SUBSTITUTE_16: f32 = 8.0;
#[doc = include_str!("text_stack_install.doc/GID_A.md")]
const GID_A: u32 = 36;
const GID_CYRILLIC_BE: u32 = 425;

#[doc = include_str!("text_stack_install.doc/vendored_fonts.md")]
fn vendored_fonts() -> FontSet {
    FontSet::new()
        .with(
            FontFamily::generic(GenericFamily::SansSerif),
            FontAttributes::new(),
            NOTO_SANS,
        )
        .with(
            FontFamily::generic(GenericFamily::Monospace),
            FontAttributes::new(),
            NOTO_SANS_MONO,
        )
}

fn compile(css: &str) -> Stylesheet {
    use obscur_kit::css::compiler::StylesheetCompiler;
    use obscur_kit::css::parser::StylesheetParser;
    use obscur_kit::css::source::CssSource;

    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("text_stack_install.doc/render.md")]
fn render(css: &str, text: &str, install: bool) -> (UiRuntime, NodeId) {
    let sheet = compile(css);
    let component = ComponentId::from_raw(0x00C0_0001);
    let item = ViewNode::element(component)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("item".into())))
        .with_children(ViewFragment::from_nodes(vec![ViewNode::text(text)]));
    let page = ViewNode::element(component)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("page".into())))
        .with_children(ViewFragment::from_nodes(vec![item]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![page]));

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(400.0, 400.0));
    if install {
        let installed = KitBootstrap::new().install_text_stack(&mut runtime, &vendored_fonts());
        assert!(!installed.is_empty(), "the vendored set installed nothing");
    }
    runtime.render_now(&view, std::slice::from_ref(&sheet));

    let root = runtime.tree().root().expect("a root");
    let page_id = runtime.tree().get(root).expect("live").child_nodes()[0];
    let item_id = runtime.tree().get(page_id).expect("live").child_nodes()[0];
    (runtime, item_id)
}

#[doc = include_str!("text_stack_install.doc/item_height.md")]
fn item_height(runtime: &UiRuntime, item: NodeId) -> f32 {
    runtime
        .layout_boxes()
        .get(&item)
        .expect("the item is laid out")
        .size()
        .height
}

#[doc = include_str!("text_stack_install.doc/item_width.md")]
fn item_width(runtime: &UiRuntime, item: NodeId) -> f32 {
    runtime
        .layout_boxes()
        .get(&item)
        .expect("the item is laid out")
        .size()
        .width
}

#[doc = include_str!("text_stack_install.doc/glyph_ids.md")]
fn glyph_ids(runtime: &UiRuntime) -> Vec<u32> {
    runtime
        .snapshot()
        .display_list()
        .commands()
        .iter()
        .find_map(|c| match c {
            PaintCommand::DrawText { run, .. } => {
                Some(run.glyphs().iter().map(PositionedGlyph::glyph_id).collect())
            }
            _ => None,
        })
        .unwrap_or_default()
}

#[doc = include_str!("text_stack_install.doc/text_runs.md")]
fn text_runs(runtime: &UiRuntime) -> Vec<(u32, Vec<u32>)> {
    runtime
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

#[doc = include_str!("text_stack_install.doc/install_text_stack_puts_the_real_faces_metrics_in_the_line_box.md")]
#[test]
fn install_text_stack_puts_the_real_faces_metrics_in_the_line_box() {
    let css = ".page { width: 400px; height: 400px; font-size: 16px } .item { }";

    let (before, item) = render(css, "hello", false);
    let default_height = item_height(&before, item);
    assert!(
        (default_height - IDEALIZED_LINE_HEIGHT_16).abs() < 1e-2,
        "the out-of-the-box stack sizes by idealized ratios: expected \
         {IDEALIZED_LINE_HEIGHT_16}, got {default_height}"
    );

    let (after, item) = render(css, "hello", true);
    let installed_height = item_height(&after, item);
    assert!(
        (installed_height - SANS_LINE_HEIGHT_16).abs() < 1e-2,
        "one install call must size the line box from Noto Sans's own hhea \
         metrics: expected {SANS_LINE_HEIGHT_16}, got {installed_height}"
    );
}

#[doc = include_str!("text_stack_install.doc/install_text_stack_emits_the_faces_real_glyph_indices.md")]
#[test]
fn install_text_stack_emits_the_faces_real_glyph_indices() {
    let css = ".page { width: 400px; height: 400px; font-size: 16px } \
               .item { font-family: sans-serif }";

    let (before, _) = render(css, "A", false);
    assert_eq!(
        glyph_ids(&before),
        vec!['A' as u32],
        "the mock shaper emits the Unicode scalar"
    );

    let (after, _) = render(css, "A", true);
    assert_eq!(
        glyph_ids(&after),
        vec![GID_A],
        "the installed HarfRust shaper must emit Noto Sans's own gid for 'A'"
    );
}

#[doc = include_str!("text_stack_install.doc/install_text_stack_resolves_cyrillic_through_the_registrys_coverage.md")]
#[test]
fn install_text_stack_resolves_cyrillic_through_the_registrys_coverage() {
    let css = ".page { width: 400px; height: 400px; font-size: 16px } \
               .item { font-family: sans-serif }";
    let (runtime, _) = render(css, "Б", true);
    assert_eq!(
        glyph_ids(&runtime),
        vec![GID_CYRILLIC_BE],
        "Noto Sans covers Cyrillic; the resolver must find it"
    );
}

#[doc = include_str!("text_stack_install.doc/install_text_stack_registers_each_generic_under_its_own_family.md")]
#[test]
fn install_text_stack_registers_each_generic_under_its_own_family() {
    let css = ".page { width: 400px; height: 400px; font-size: 16px } \
               .item { font-family: monospace }";
    let (runtime, _) = render(css, "iW", true);

    let runs = text_runs(&runtime);
    assert_eq!(runs.len(), 1, "one run expected, got {runs:?}");
    assert_eq!(
        runs[0].0, 1,
        "monospace is the second face installed, so it paints under handle 1"
    );
}

#[doc = include_str!("text_stack_install.doc/install_text_stack_gives_the_runtime_bidi.md")]
#[test]
fn install_text_stack_gives_the_runtime_bidi() {
    // `a`=68, `b`=69, `c`=70, `.`=17 in Noto Sans; the mock emits scalars instead.
    const GID_A_LOWER: u32 = 68;
    const GID_B_LOWER: u32 = 69;
    const GID_C_LOWER: u32 = 70;
    const GID_PERIOD: u32 = 17;

    let css = ".page { width: 400px; height: 400px; font-size: 16px } \
               .item { font-family: sans-serif; direction: rtl }";

    let before = text_runs(&render(css, "abc.", false).0);
    assert_eq!(
        before,
        vec![(0, vec!['.' as u32, 'c' as u32, 'b' as u32, 'a' as u32])],
        "out of the box: one undifferentiated run, reversed wholesale to `.cba` — \
         the word itself is mangled"
    );

    let after = text_runs(&render(css, "abc.", true).0);
    assert_eq!(
        after.len(),
        2,
        "the installed itemizer must resolve two bidi levels, got {after:?}"
    );
    assert_eq!(
        after[0].1,
        vec![GID_PERIOD],
        "rule N2: the neutral `.` takes the RTL paragraph level and reorders to the \
         visual left"
    );
    assert_eq!(
        after[1].1,
        vec![GID_A_LOWER, GID_B_LOWER, GID_C_LOWER],
        "the Latin island stays left-to-right: `abc`, not `cba`"
    );
}

#[doc = include_str!("text_stack_install.doc/ch_resolves_to_the_installed_faces_measured_zero_advance.md")]
#[test]
fn ch_resolves_to_the_installed_faces_measured_zero_advance() {
    const N: f32 = 25.0;
    let css = ".page { width: 400px; height: 400px; font-size: 16px } \
               .item { font-family: sans-serif; width: 25ch }";

    let (before, item) = render(css, "x", false);
    let default_width = item_width(&before, item);
    assert!(
        (default_width - N * MOCK_CH_16).abs() < 1e-2,
        "the mock shaper measures a fixed 0.6em `0`: expected {}, got {default_width}",
        N * MOCK_CH_16
    );

    let (after, item) = render(css, "x", true);
    let installed_width = item_width(&after, item);
    assert!(
        (installed_width - N * SANS_CH_16).abs() < 1e-2,
        "`25ch` in Noto Sans is 25 x 9.152 = {}, got {installed_width}",
        N * SANS_CH_16
    );
    // Both foils, explicitly: the answer is neither the mock's nor the spec's
    // unmeasurable-case substitute.
    assert!(
        (installed_width - N * MOCK_CH_16).abs() > 1.0,
        "the measured answer must differ from the mock's, or this asserts nothing"
    );
    assert!(
        (installed_width - N * CH_SUBSTITUTE_16).abs() > 1.0,
        "the measured answer must differ from the 0.5em substitute"
    );
}

#[doc = include_str!("text_stack_install.doc/a_ch_border_paints_the_installed_sans_faces_zero_advance.md")]
#[test]
fn a_ch_border_paints_the_installed_sans_faces_zero_advance() {
    const N: f32 = 4.0;
    let css = ".page { width: 400px; height: 400px } \
               .item { font-family: sans-serif; font-size: 16px; width: 40px; height: 20px; \
                       border-style: solid; border-width: 4ch; border-color: #ff0000 }";

    let (runtime, item) = render(css, "x", true);
    let bounds = runtime
        .layout_boxes()
        .get(&item)
        .copied()
        .expect("the item is laid out")
        .bounds;
    let painted = painted_edge_extent(runtime.snapshot().display_list().commands(), bounds);

    assert!(
        (painted - N * SANS_CH_16).abs() < 1e-2,
        "`4ch` in Noto Sans is 4 x 9.152 = {}, painted {painted} \
         (0 would mean `ch` never left the computed style)",
        N * SANS_CH_16
    );
    // All three foils, explicitly — the answer is none of them.
    assert!(painted > 0.0, "the old seam painted nothing at all");
    assert!(
        (painted - N * MOCK_CH_16).abs() > 1.0,
        "the measured answer must differ from the mock's {}, or the install asserts nothing",
        N * MOCK_CH_16
    );
    assert!(
        (painted - N * CH_SUBSTITUTE_16).abs() > 1.0,
        "the measured answer must differ from the 0.5em substitute's {}",
        N * CH_SUBSTITUTE_16
    );
}

#[doc = include_str!("text_stack_install.doc/painted_edge_extent.md")]
fn painted_edge_extent(commands: &[PaintCommand], bounds: obscur_kit::geometry::Rect) -> f32 {
    commands
        .iter()
        .filter_map(|c| match c {
            PaintCommand::DrawPath { path, .. } => path.bounds(),
            _ => None,
        })
        .filter(|r| {
            (r.origin.y - bounds.origin.y).abs() < 1e-3 && r.size.width >= bounds.size.width - 1e-3
        })
        .map(|r| r.size.height)
        .fold(0.0f32, f32::max)
}

#[doc = include_str!("text_stack_install.doc/the_mono_faces_ch_is_a_fifth_wider_than_the_specs_substitute.md")]
#[test]
fn the_mono_faces_ch_is_a_fifth_wider_than_the_specs_substitute() {
    let css = ".page { width: 400px; height: 400px; font-size: 16px } \
               .item { font-family: monospace; width: 25ch }";
    let (runtime, item) = render(css, "x", true);
    let width = item_width(&runtime, item);
    assert!(
        (width - 25.0 * MONO_CH_16).abs() < 1e-2,
        "`25ch` in Noto Sans Mono is 25 x 9.6 = {}, got {width}",
        25.0 * MONO_CH_16
    );
    assert!(
        (width - 25.0 * CH_SUBSTITUTE_16).abs() > 1.0,
        "the substitute would be {}, a 20% error",
        25.0 * CH_SUBSTITUTE_16
    );
}

#[doc = include_str!("text_stack_install.doc/installed_fonts_hands_back_dense_handles_in_installed_order.md")]
#[test]
fn installed_fonts_hands_back_dense_handles_in_installed_order() {
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(100.0, 100.0));
    let fonts = vendored_fonts();
    let installed = KitBootstrap::new().install_text_stack(&mut runtime, &fonts);

    assert_eq!(installed.len(), 2);
    assert_eq!(installed.handle(0), Some(0));
    assert_eq!(installed.handle(1), Some(1));
    assert_eq!(installed.handle(2), None);

    let handed: Vec<_> = installed.iter().collect();
    assert_eq!(handed[0].1.as_ref(), NOTO_SANS, "handle 0 must be the sans");
    assert_eq!(
        handed[1].1.as_ref(),
        NOTO_SANS_MONO,
        "handle 1 must be the mono"
    );
}

#[doc = include_str!("text_stack_install.doc/an_empty_font_set_falls_back_to_the_bundled_face_and_paints.md")]
#[test]
fn an_empty_font_set_falls_back_to_the_bundled_face_and_paints() {
    let css = ".page { width: 400px; height: 400px; font-size: 16px } .item { }";
    let sheet = compile(css);
    let component = ComponentId::from_raw(0x00C0_0001);
    let view = View::from_fragment(ViewFragment::from_nodes(vec![
        ViewNode::element(component)
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("page".into())))
            .with_children(ViewFragment::from_nodes(vec![ViewNode::text("hello")])),
    ]));

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(400.0, 400.0));
    let installed = KitBootstrap::new().install_text_stack(&mut runtime, &FontSet::new());
    // The bundled last-resort face is surfaced to the renderer hand-off precisely
    // because the host supplied none: it is the only face that can paint.
    assert_eq!(
        installed.len(),
        1,
        "an empty host set still hands the renderer the bundled fallback face"
    );
    assert_eq!(
        installed.handle(0),
        Some(0),
        "the bundled face takes handle 0"
    );
    assert_eq!(
        installed.iter().next().map(|(_, bytes)| bytes.as_ref()),
        Some(NOTO_SANS),
        "the bundled face is the vendored Noto Sans"
    );

    runtime.render_now(&view, std::slice::from_ref(&sheet));
    let glyphs = glyph_ids(&runtime);
    assert!(
        !glyphs.is_empty(),
        "an empty host set now paints text from the bundled fallback (§5.18 met)"
    );
    assert!(
        glyphs.iter().all(|&g| g != 0),
        "the bundled Noto Sans covers `hello` — no .notdef: {glyphs:?}"
    );
    // Not the mock: the installed HarfRust shaper emits Noto Sans's own gids, not
    // Unicode scalars, so `h`/`e`/`l`/`l`/`o` are not their code points.
    assert!(
        glyphs != "hello".chars().map(|c| c as u32).collect::<Vec<_>>(),
        "the real bundled face shaped these, not the mock: {glyphs:?}"
    );
}

#[doc = include_str!("text_stack_install.doc/a_font_set_is_built_from_bytes_alone.md")]
#[test]
fn a_font_set_is_built_from_bytes_alone() {
    let set = FontSet::new().with_face(FontFaceBytes::new(
        FontFamily::named("Host Provided"),
        FontAttributes::new(),
        NOTO_SANS,
    ));
    assert_eq!(set.len(), 1);
    assert_eq!(set.faces()[0].family().name(), "Host Provided");
    assert_eq!(set.faces()[0].bytes().as_ref(), NOTO_SANS);
}
