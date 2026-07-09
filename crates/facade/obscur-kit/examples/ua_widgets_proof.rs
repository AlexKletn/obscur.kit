//! **Proof: the UA default stylesheet draws the wireframe indicator sub-parts.**
//!
//! Lays out BARE built-in form controls (no author CSS for the controls — only a
//! neutral `.page` layout harness) with the toolkit's UA default stylesheet
//! installed at the lowest cascade origin, and rasterizes them through the
//! software [`CpuRenderer`] into `target/ua_widgets_proof.png`:
//!
//! - **Checkbox** — unchecked (empty 20×20 box), checked (filled box + ✓),
//!   indeterminate (filled box + thin bar), and disabled (dashed muted box).
//! - **Radio** — unselected (empty circle), selected (circle + filled dot), and
//!   disabled (dashed muted circle).
//! - **Select** — closed (▼) and open (▲).
//!
//! Every indicator comes entirely from the UA sheet keying on the sub-part
//! classes (`.checkbox-box` / `.radio-dot` / `.select-arrow`) the widgets emit.
//!
//! Run it:
//! `cargo run -p obscur-kit --example ua_widgets_proof \
//!   --features "render-cpu text-harfrust text-unicode"`
#![forbid(unsafe_code)]

use std::path::PathBuf;

use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, InternedSymbol, RuntimeId};
use obscur_kit::render::frame::{FrameSubmission, RenderFrame};
use obscur_kit::render::target::{BitmapTarget, RenderTarget};
use obscur_kit::render::{Renderer, RendererId};
use obscur_kit::render_cpu::renderer::CpuRenderer;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::text::font::{FontAttributes, FontFamily, FontId, GenericFamily};
use obscur_kit::text_harfrust::{FontRegistry, HarfrustShaper};
use obscur_kit::text_unicode::UnicodeTextItemizer;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};
use obscur_kit::widgets::checkbox::Checkbox;
use obscur_kit::widgets::radio::Radio;
use obscur_kit::widgets::select::Select;

/// The logical extent the whole control column is laid out and rasterized under.
const VIEWPORT: Size = Size::new(360.0, 560.0);

/// A demo command symbol the bare controls fire (never dispatched here).
const CMD: InternedSymbol = InternedSymbol::from_raw(0x0001);

/// The layout harness (GlobalApp): arranges the controls in a padded column on a
/// white page. It styles ONLY the `.page` container — never the control types or
/// the indicator sub-parts — so every indicator comes from the UA sheet.
const HARNESS_CSS: &str = "\
.page { display: flex; flex-direction: column; gap: 14px; padding: 24px; background: #ffffff; }";

fn main() {
    println!("== obscur-kit UA widget-indicator proof (headless CPU) ==\n");

    let ua = obscur_kit::ua_stylesheet();
    println!(
        "[ua] compiled default sheet: {} rule(s) at {:?}",
        ua.rules().len(),
        ua.origin(),
    );

    let pixmap = render(&ua);
    let png = pixmap.encode_png().expect("encode png");
    let out = output_path("ua_widgets_proof.png");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create target dir");
    }
    std::fs::write(&out, &png).expect("write png");
    println!(
        "\n[png] wrote {} × {} → {} ({} bytes) — bare checkbox/radio/select with UA indicators",
        pixmap.width(),
        pixmap.height(),
        out.display(),
        png.len(),
    );
}

/// Boots a runtime with the UA sheet as its default (always-first) sheet, matches
/// the neutral `.page` harness on top, and rasterizes the bare controls.
fn render(ua: &Stylesheet) -> tiny_skia::Pixmap {
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    runtime.set_default_stylesheet(ua.clone());

    let mut renderer = CpuRenderer::new(RendererId::from_raw(2));
    if let Some(bytes) = load_system_font() {
        wire_text_stack(&mut runtime, &mut renderer, bytes);
    } else {
        println!("[warn] no system font found — glyph indicators (✓ ▼ ▲) will not render");
    }

    let author = compile(HARNESS_CSS);
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
    renderer.render(&submission, &target).expect("cpu render succeeds");
    renderer.pixmap().expect("a rasterized frame").clone()
}

/// The view: a `.page` column holding the bare form controls in every indicator
/// state — each emitting its own sub-part classes the UA sheet keys on.
fn controls_view() -> View {
    let kids = vec![
        widget_root(Checkbox::new("Checkbox — unchecked", CMD).view()),
        widget_root(Checkbox::new("Checkbox — checked", CMD).checked(true).view()),
        widget_root(Checkbox::indeterminate("Checkbox — mixed", CMD).view()),
        widget_root(Checkbox::new("Checkbox — disabled", CMD).checked(true).disabled(true).view()),
        widget_root(Radio::new("Radio — unselected", CMD).view()),
        widget_root(Radio::new("Radio — selected", CMD).selected(true).view()),
        widget_root(Radio::new("Radio — disabled", CMD).disabled(true).view()),
        widget_root(Select::new(CMD).with_options(["Closed"]).view()),
        widget_root({
            let mut s = Select::new(CMD).with_options(["Open"]);
            s.open();
            s.view()
        }),
    ];

    let page = ViewNode::element(ComponentId::from_raw(1))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("page".into())))
        .with_children(ViewFragment::from_nodes(kids));
    View::single(page)
}

/// Extracts a widget's root [`ViewNode`] from its single-node [`View`].
fn widget_root(view: View) -> ViewNode {
    view.root().nodes()[0].clone()
}

/// Compiles CSS text into a runtime-matchable [`Stylesheet`] at `GlobalApp`.
fn compile(css: &str) -> Stylesheet {
    use obscur_kit::css::compiler::StylesheetCompiler;
    use obscur_kit::css::parser::StylesheetParser;
    use obscur_kit::css::source::CssSource;
    use obscur_kit::style::cascade::CascadeOrigin;

    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new()
        .with_origin(CascadeOrigin::GlobalApp)
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
    runtime.set_font_resolver(Box::new(registry));
    runtime.set_text_itemizer(Box::new(UnicodeTextItemizer::new()));
    runtime.set_text_shaper(Box::new(HarfrustShaper::without_font()));
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
