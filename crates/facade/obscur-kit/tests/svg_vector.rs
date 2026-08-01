#![doc = include_str!("svg_vector.doc/_module.md")]
#![forbid(unsafe_code)]
#![cfg(all(
    feature = "resource-fs",
    feature = "resource-decode",
    feature = "render-cpu"
))]

use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use obscur_kit::bootstrap::KitBootstrap;
use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::RuntimeId;
use obscur_kit::primitives::Image;
use obscur_kit::render::frame::{FrameSubmission, RenderFrame};
use obscur_kit::render::target::{BitmapTarget, RenderTarget};
use obscur_kit::render::{Renderer, RendererId};
use obscur_kit::render_cpu::renderer::CpuRenderer;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::scene::color::Color;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::scene::paint::{Brush, Paint};
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewProperty, ViewValue};

#[doc = include_str!("svg_vector.doc/VIEWPORT.md")]
const VIEWPORT: f32 = 64.0;

#[doc = include_str!("svg_vector.doc/ICON_SVG.md")]
const ICON_SVG: &[u8] =
    br##"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 10 10">
  <rect width="10" height="10" fill="currentColor"/>
</svg>"##;

#[doc = include_str!("svg_vector.doc/COUNTER.md")]
static COUNTER: AtomicU32 = AtomicU32::new(0);

#[doc = include_str!("svg_vector.doc/temp_asset_dir.md")]
fn temp_asset_dir() -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("obscur-kit-svg-{}-{n}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    std::fs::write(dir.join("icon.svg"), ICON_SVG).expect("write svg");
    dir
}

fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("svg_vector.doc/icon_view.md")]
fn icon_view() -> View {
    let node = Image::new("icon.svg")
        .into_view_node()
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("icon".into())));
    View::from_fragment(ViewFragment::from_nodes(vec![node]))
}

#[doc = include_str!("svg_vector.doc/runtime_at.md")]
fn runtime_at(dir: &PathBuf) -> UiRuntime {
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    let boot = KitBootstrap::new();
    boot.install_image_resources(&mut runtime, dir);
    boot.install_vector_resources(&mut runtime);
    runtime
}

#[doc = include_str!("svg_vector.doc/painted_fills.md")]
fn painted_fills(result: &obscur_kit::runtime::frame::FrameResult) -> Vec<Color> {
    result
        .snapshot()
        .display_list()
        .commands()
        .iter()
        .filter_map(|cmd| match cmd {
            PaintCommand::DrawPath {
                paint: Paint::Fill(Brush::Solid(solid)),
                ..
            } => Some(solid.color()),
            _ => None,
        })
        .collect()
}

#[test]
fn an_svg_src_paints_as_vector_paths_and_never_as_an_image() {
    let dir = temp_asset_dir();
    let sheet = compile(".icon { width: 20px; height: 20px; color: #ff0000 }");
    let mut runtime = runtime_at(&dir);

    let _ = runtime.render_now(&icon_view(), std::slice::from_ref(&sheet));
    let _ = runtime.poll_resource_tasks();
    let result = runtime.render_now(&icon_view(), std::slice::from_ref(&sheet));
    let commands = result.snapshot().display_list().commands().to_vec();

    assert!(
        commands
            .iter()
            .any(|c| matches!(c, PaintCommand::DrawPath { .. })),
        "the document's outline reached the scene"
    );
    assert!(
        !commands
            .iter()
            .any(|c| matches!(c, PaintCommand::DrawImage { .. })),
        "§5.17: a vector source must never be rasterized into an image"
    );
    // The `currentColor` fill took the element's computed `color`.
    assert_eq!(
        painted_fills(&result),
        vec![Color::rgba(1.0, 0.0, 0.0, 1.0)]
    );
}

#[test]
fn a_theme_change_recolours_the_same_document_without_reparsing() {
    // THE §5.17 requirement. The document is loaded and parsed on the first frame
    // and cached colour-independently; the second frame changes only the computed
    // `color` and paints a different colour out of that same parsed value.
    let dir = temp_asset_dir();
    let mut runtime = runtime_at(&dir);

    let red = compile(".icon { width: 20px; height: 20px; color: #ff0000 }");
    let _ = runtime.render_now(&icon_view(), std::slice::from_ref(&red));
    let _ = runtime.poll_resource_tasks();
    let first = runtime.render_now(&icon_view(), std::slice::from_ref(&red));
    assert_eq!(painted_fills(&first), vec![Color::rgba(1.0, 0.0, 0.0, 1.0)]);

    // Swap the theme. The file on disk is now DELETED: if the runtime needed to
    // re-parse to follow the colour, this frame could not possibly paint blue.
    std::fs::remove_file(dir.join("icon.svg")).expect("remove the source file");

    let blue = compile(".icon { width: 20px; height: 20px; color: #0000ff }");
    let second = runtime.render_now(&icon_view(), std::slice::from_ref(&blue));
    assert_eq!(
        painted_fills(&second),
        vec![Color::rgba(0.0, 0.0, 1.0, 1.0)],
        "the cached document re-coloured with no access to its source"
    );
}

#[test]
fn the_vector_reaches_the_cpu_backend() {
    let dir = temp_asset_dir();
    let sheet = compile(".icon { width: 20px; height: 20px; color: #ff0000 }");
    let mut runtime = runtime_at(&dir);
    let _ = runtime.render_now(&icon_view(), std::slice::from_ref(&sheet));
    let _ = runtime.poll_resource_tasks();
    let result = runtime.render_now(&icon_view(), std::slice::from_ref(&sheet));

    let mut renderer = CpuRenderer::new(RendererId::from_raw(2));
    let size = Size::new(VIEWPORT, VIEWPORT);
    let frame = RenderFrame::new(result.snapshot().clone(), size);
    let target = RenderTarget::bitmap(BitmapTarget::new(size));
    renderer
        .render(&FrameSubmission::new(frame), &target)
        .expect("the cpu backend renders the vector");
    let pixmap = renderer.pixmap().expect("a rasterized frame");

    // The icon occupies the top-left 20x20; its centre must be the red it was told
    // to be. Nothing here is a bitmap: the backend rasterized the outline itself.
    let px = pixmap.pixel(10, 10).expect("pixel inside the icon");
    assert!(px.red() > 200, "the vector painted red, got r={}", px.red());
    assert!(px.alpha() > 200, "and opaque, got a={}", px.alpha());
}

#[test]
fn without_the_vector_capability_an_svg_paints_nothing_rather_than_rasterizing() {
    // The honest-capability default: a host that installs no vector decoder gets
    // no paint, not a silent rasterization that would break the vector guarantee.
    let dir = temp_asset_dir();
    let sheet = compile(".icon { width: 20px; height: 20px; color: #ff0000 }");
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    KitBootstrap::new().install_image_resources(&mut runtime, &dir);

    let result = runtime.render_now(&icon_view(), std::slice::from_ref(&sheet));
    assert!(painted_fills(&result).is_empty(), "no decoder, no paint");
    assert!(
        !result
            .snapshot()
            .display_list()
            .commands()
            .iter()
            .any(|c| matches!(c, PaintCommand::DrawImage { .. })),
        "and certainly not a rasterized fallback"
    );
}
