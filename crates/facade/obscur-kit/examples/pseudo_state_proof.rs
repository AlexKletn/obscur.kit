//! **Proof: the `:disabled` and `:checked` pseudo-classes bite from widget state.**
//!
//! Renders four BARE built-in controls through the software [`CpuRenderer`] in one
//! `target/pseudo_state_proof.png`:
//!
//! - an **enabled** [`Button`] beside a **disabled** one, and
//! - an **unchecked** [`Checkbox`] beside a **checked** one.
//!
//! The runtime has the toolkit's UA default stylesheet installed at the lowest
//! cascade origin plus a small author sheet whose `button:disabled` and
//! `checkbox:checked` rules restyle the state. Because each widget now emits the
//! reserved `DISABLED`/`CHECKED` structural-state attribute (which survives
//! reconcile and feeds the selector matcher's `PseudoState`), the disabled button
//! and the checked checkbox visibly differ from their enabled/unchecked peers —
//! proving the state pseudo-classes are wired end-to-end.
//!
//! Run it:
//! `cargo run -p obscur-kit --example pseudo_state_proof \
//!   --features "render-cpu text-harfrust text-unicode"`
#![forbid(unsafe_code)]

use std::path::PathBuf;

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, InternedSymbol, RuntimeId};
use obscur_kit::render::frame::{FrameSubmission, RenderFrame};
use obscur_kit::render::target::{BitmapTarget, RenderTarget};
use obscur_kit::render::{Renderer, RendererId};
use obscur_kit::render_cpu::renderer::CpuRenderer;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::cascade::CascadeOrigin;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::text::font::{FontAttributes, FontFamily, FontId, GenericFamily};
use obscur_kit::text_harfrust::{FontRegistry, HarfrustShaper};
use obscur_kit::text_unicode::UnicodeTextItemizer;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};
use obscur_kit::widgets::button::Button;
use obscur_kit::widgets::checkbox::Checkbox;

/// The logical extent the frame is laid out and rasterized under.
const VIEWPORT: Size = Size::new(420.0, 240.0);

/// A demo command symbol the bare controls fire (never dispatched here).
const CMD: InternedSymbol = InternedSymbol::from_raw(0x0001);

/// The author sheet: a neutral layout harness PLUS strong `:disabled`/`:checked`
/// overlays so the state difference is unmistakable in the raster. The overlays
/// only bite because each widget emits the reserved structural-state attribute.
const CSS: &str = "\
.page { display: flex; flex-direction: column; gap: 14px; padding: 24px; background: #ffffff; }\
.row { display: flex; flex-direction: row; gap: 16px; }\
button:disabled { background: #8a8780; color: #ffffff; border-color: #6f6c66; }\
checkbox:checked { color: #ffffff; background: #1f6f3d; border-radius: 4px; padding: 2px 6px; }";

fn main() {
    println!("== obscur-kit :disabled / :checked pseudo-class proof (headless CPU) ==\n");

    let ua = obscur_kit::ua_stylesheet();
    println!(
        "[ua] compiled default sheet: {} rule(s) at {:?}",
        ua.rules().len(),
        ua.origin(),
    );

    let pixmap = render(&ua, CSS);

    let png = pixmap.encode_png().expect("encode png");
    let out = output_path("pseudo_state_proof.png");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create target dir");
    }
    std::fs::write(&out, &png).expect("write png");
    println!(
        "\n[png] wrote {} × {} → {} ({} bytes) — enabled vs disabled button, unchecked vs checked box",
        pixmap.width(),
        pixmap.height(),
        out.display(),
        png.len(),
    );
}

/// Boots a runtime with the UA sheet installed as its default (always-first)
/// sheet, matches the author `css` on top, and rasterizes the controls.
fn render(ua: &Stylesheet, css: &str) -> tiny_skia::Pixmap {
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    runtime.set_default_stylesheet(ua.clone());

    let mut renderer = CpuRenderer::new(RendererId::from_raw(2));
    if let Some(bytes) = load_system_font() {
        wire_text_stack(&mut runtime, &mut renderer, bytes);
    }

    let author = compile(css, CascadeOrigin::GlobalApp);
    let view = controls_view();
    let result = runtime.render_now(&view, std::slice::from_ref(&author));
    let snapshot = result.snapshot().clone();
    println!(
        "[frame] {} paint command(s)",
        snapshot.display_list().commands().len(),
    );

    let frame = RenderFrame::new(snapshot, VIEWPORT);
    let submission = FrameSubmission::new(frame);
    let target = RenderTarget::bitmap(BitmapTarget::new(VIEWPORT));
    renderer
        .render(&submission, &target)
        .expect("cpu render succeeds");
    renderer.pixmap().expect("a rasterized frame").clone()
}

/// The view: a `.page` column with two `.row`s — buttons (enabled | disabled) and
/// checkboxes (unchecked | checked). Each widget emits its own stable CSS type and
/// the reserved structural-state attribute the matcher keys on.
fn controls_view() -> View {
    let enabled = widget_root(Button::new("Enabled", CMD).view());
    let disabled = widget_root(Button::new("Disabled", CMD).disabled(true).view());
    let unchecked = widget_root(Checkbox::new("Unchecked", CMD).view());
    let checked = widget_root(Checkbox::new("Checked", CMD).checked(true).view());

    let buttons = row(vec![enabled, disabled]);
    let boxes = row(vec![unchecked, checked]);

    let page = ViewNode::element(ComponentId::from_raw(1))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("page".into())))
        .with_children(ViewFragment::from_nodes(vec![buttons, boxes]));
    View::single(page)
}

/// A `.row` flex container holding `children`.
fn row(children: Vec<ViewNode>) -> ViewNode {
    ViewNode::element(ComponentId::from_raw(2))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("row".into())))
        .with_children(ViewFragment::from_nodes(children))
}

/// Extracts a widget's root [`ViewNode`] from its single-node [`View`].
fn widget_root(view: View) -> ViewNode {
    view.root().nodes()[0].clone()
}

/// Compiles CSS text into a runtime-matchable [`Stylesheet`] at `origin`.
fn compile(css: &str, origin: CascadeOrigin) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new()
        .with_origin(origin)
        .compile(&parsed)
        .matchable
}

/// The workspace-relative output path `target/<name>`.
fn output_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("target")
        .join(name)
}

/// Wires the text stack and registers `bytes` under `sans-serif` on both the
/// resolver and the CPU renderer so glyph handles line up.
fn wire_text_stack(runtime: &mut UiRuntime, renderer: &mut CpuRenderer, bytes: Vec<u8>) {
    let mut registry = FontRegistry::new();
    let id = registry.register(
        FontFamily::generic(GenericFamily::SansSerif),
        FontAttributes::default(),
        bytes.clone(),
    );
    renderer.register_font(font_handle(id), bytes);
    // The shaper answers line-metric queries by `FontId` alone, so it needs the
    // registry's own per-face metrics; without them the glyphs would be real but
    // every line box sized from idealized ratios.
    let shaper = HarfrustShaper::without_font().with_faces(registry.face_metrics());
    runtime.set_font_resolver(Box::new(registry));
    runtime.set_text_itemizer(Box::new(UnicodeTextItemizer::new()));
    runtime.set_text_shaper(Box::new(shaper));
}

/// The scene font handle (`u32`) for a resolved [`FontId`].
fn font_handle(id: FontId) -> u32 {
    u32::try_from(id.index()).unwrap_or(0)
}

/// Loads a system font's raw bytes, scanning well-known locations (best effort).
fn load_system_font() -> Option<Vec<u8>> {
    const CANDIDATES: &[&str] = &[
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/usr/share/fonts/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/noto/NotoSans-Regular.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        "/usr/share/fonts/liberation/LiberationSans-Regular.ttf",
        "/System/Library/Fonts/Helvetica.ttc",
        "C:\\Windows\\Fonts\\arial.ttf",
    ];
    CANDIDATES.iter().find_map(|p| std::fs::read(p).ok())
}
