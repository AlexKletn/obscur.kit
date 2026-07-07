//! **Proof: the built-in UA default stylesheet gives bare controls a base look,
//! and an author sheet overrides it.**
//!
//! Renders three BARE built-in controls — a [`Button`], an [`Input`], and a
//! [`Checkbox`] — twice through the software [`CpuRenderer`], stitching the two
//! frames into one `target/ua_default_proof.png`:
//!
//! - **Top row — UA default only.** The runtime has the toolkit's UA default
//!   stylesheet installed at the lowest cascade origin
//!   ([`CascadeOrigin::SystemDefault`]) via [`ua_stylesheet`], and NO author CSS
//!   for the controls (only a neutral layout harness that arranges them). The
//!   controls show their neutral base look: border, padding, radius, background.
//! - **Bottom row — same, plus a one-line author override.** An author sheet at
//!   `GlobalApp` adds `button { background: #cc0000 }`. Because the UA sheet sits
//!   below every author sheet, the button turns red while the untouched input and
//!   checkbox keep the UA look — proving the UA sheet is a true, overridable base.
//!
//! It leans only on properties the CPU renderer paints faithfully (border,
//! padding, radius, background, color, font-size).
//!
//! Run it:
//! `cargo run -p obscur-kit --example ua_default_proof \
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
use obscur_kit::widgets::input::Input;

/// The logical extent one variant is laid out and rasterized under.
const VIEWPORT: Size = Size::new(360.0, 220.0);

/// A demo command symbol the bare controls fire (never dispatched here).
const CMD: InternedSymbol = InternedSymbol::from_raw(0x0001);

/// The layout harness (GlobalApp): arranges the three controls in a padded column
/// on a white page. It deliberately styles ONLY the `.page` container — never the
/// control types — so the controls' appearance comes entirely from the UA sheet.
const HARNESS_CSS: &str = "\
.page { display: flex; flex-direction: column; gap: 16px; padding: 24px; background: #ffffff; }";

/// The harness plus a one-line author override of the button background.
const OVERRIDE_CSS: &str = "\
.page { display: flex; flex-direction: column; gap: 16px; padding: 24px; background: #ffffff; }\
button { background: #cc0000; color: #ffffff; border-color: #7a0000; }";

fn main() {
    println!("== obscur-kit UA default stylesheet proof (headless CPU) ==\n");

    // The UA default sheet — the toolkit's overridable base look, at the lowest
    // cascade origin. Installed as the runtime's always-first default sheet.
    let ua = obscur_kit::ua_stylesheet();
    println!(
        "[ua] compiled default sheet: {} rule(s) at {:?}",
        ua.rules().len(),
        ua.origin(),
    );

    // Top: UA base only (harness arranges, no control override).
    let top = render_variant(&ua, HARNESS_CSS, "UA default only");
    // Bottom: UA base + author override of `button`.
    let bottom = render_variant(&ua, OVERRIDE_CSS, "UA + author `button { background:#c00 }`");

    // Stitch the two variants into one image (top over bottom, with a divider).
    let combined = stitch(&top, &bottom);

    let png = combined.encode_png().expect("encode png");
    let out = output_path("ua_default_proof.png");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create target dir");
    }
    std::fs::write(&out, &png).expect("write png");
    println!(
        "\n[png] wrote {} × {} → {} ({} bytes) — top: UA base look; bottom: author override wins",
        combined.width(),
        combined.height(),
        out.display(),
        png.len(),
    );
}

/// Renders one variant: boots a runtime with the UA sheet installed as its default
/// (always-first) sheet, matches the author `css` on top, and rasterizes the bare
/// controls into a pixmap.
fn render_variant(ua: &Stylesheet, css: &str, label: &str) -> tiny_skia::Pixmap {
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    // Install the UA default sheet — matched below every author sheet.
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
        "[frame] {label}: {} paint command(s)",
        snapshot.display_list().commands().len(),
    );

    let frame = RenderFrame::new(snapshot, VIEWPORT);
    let submission = FrameSubmission::new(frame);
    let target = RenderTarget::bitmap(BitmapTarget::new(VIEWPORT));
    renderer.render(&submission, &target).expect("cpu render succeeds");
    renderer.pixmap().expect("a rasterized frame").clone()
}

/// The view: a `.page` container holding three bare built-in controls, each
/// emitting its own stable CSS type (`button` / `input` / `checkbox`) that the UA
/// sheet keys on.
fn controls_view() -> View {
    let button = widget_root(Button::new("Save", CMD).view());
    let input = widget_root(Input::new(CMD).placeholder("Search…").view());
    let checkbox = widget_root(Checkbox::new("Enabled", CMD).checked(true).view());

    let page = ViewNode::element(ComponentId::from_raw(1))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("page".into())))
        .with_children(ViewFragment::from_nodes(vec![button, input, checkbox]));
    View::single(page)
}

/// Extracts a widget's root [`ViewNode`] from its single-node [`View`].
fn widget_root(view: View) -> ViewNode {
    view.root().nodes()[0].clone()
}

/// Vertically stitches `top` over `bottom` into one pixmap with a thin divider.
fn stitch(top: &tiny_skia::Pixmap, bottom: &tiny_skia::Pixmap) -> tiny_skia::Pixmap {
    let gap = 2u32;
    let width = top.width().max(bottom.width());
    let height = top.height() + gap + bottom.height();
    let mut out = tiny_skia::Pixmap::new(width, height).expect("combined pixmap");
    // Light-gray backdrop so the divider band reads.
    out.fill(tiny_skia::Color::from_rgba8(0xdd, 0xda, 0xd3, 0xff));
    let paint = tiny_skia::PixmapPaint::default();
    out.draw_pixmap(0, 0, top.as_ref(), &paint, tiny_skia::Transform::identity(), None);
    out.draw_pixmap(
        0,
        (top.height() + gap) as i32,
        bottom.as_ref(),
        &paint,
        tiny_skia::Transform::identity(),
        None,
    );
    out
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
