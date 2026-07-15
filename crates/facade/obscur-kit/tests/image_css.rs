//! End-to-end verification that **authored `background-image: url(...)` CSS**
//! reaches paint through the *real* live pipeline: a real PNG on disk is loaded by
//! a sandbox-rooted `FsResourceLoader`, decoded by the `image`-crate `ImageDecoder`,
//! cached as a `SceneImage`, and painted as a `DrawImage` over the element's box
//! (spec §5.15 load, §5.16 decode, §5.2 paint) — closing audit finding V6 for
//! images ("loaded bytes never become a usable image").
//!
//! The test authors CSS text, compiles it through the real `obscur-css` frontend,
//! renders one frame on a [`UiRuntime`] wired with the facade's real filesystem
//! loader + image decoder (via [`KitBootstrap::install_image_resources`]), and
//! asserts both that the published [`DisplayList`] carries a
//! [`PaintCommand::DrawImage`] and that the CPU (tiny-skia) backend paints the
//! image's pixels inside the element box (not the transparent background).
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
use obscur_kit::identity::{ComponentId, RuntimeId};
use obscur_kit::render::frame::{FrameSubmission, RenderFrame};
use obscur_kit::render::target::{BitmapTarget, RenderTarget};
use obscur_kit::render::{Renderer, RendererId};
use obscur_kit::render_cpu::renderer::CpuRenderer;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

/// The fixed viewport every test renders under, so geometry is deterministic.
const VIEWPORT: f32 = 64.0;

/// A real 2×2 opaque-red PNG (RGB, generated with `zlib`/`crc32`), embedded so the
/// test needs no image-encoding dependency: the fs loader reads these exact bytes
/// off disk and the `image`-crate decoder turns them into RGBA8.
const RED_2X2_PNG: &[u8] = &[
    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 2, 0, 0, 0, 2, 8, 2, 0,
    0, 0, 253, 212, 154, 115, 0, 0, 0, 16, 73, 68, 65, 84, 120, 218, 99, 248, 207, 192, 0, 68, 12,
    16, 10, 0, 31, 238, 3, 253, 99, 94, 187, 91, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
];

/// Process-unique counter so concurrent tests never collide on a temp name.
static COUNTER: AtomicU32 = AtomicU32::new(0);

/// Creates a fresh temp directory holding `logo.png` (the embedded red PNG), and
/// returns the directory path (the loader's sandbox root).
fn temp_asset_dir() -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("obscur-kit-image-css-{}-{n}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    std::fs::write(dir.join("logo.png"), RED_2X2_PNG).expect("write png");
    dir
}

/// Compiles authored CSS text into a runtime-matchable [`Stylesheet`].
fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

/// A single element classed `.img`, the image-background target.
fn img_view() -> View {
    let node = ViewNode::element(ComponentId::from_raw(40))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("img".into())));
    View::from_fragment(ViewFragment::from_nodes(vec![node]))
}

#[test]
fn url_background_image_reaches_paint_and_the_cpu_backend() {
    let dir = temp_asset_dir();

    // A `.img` element with a `url()` background over a blue color, sized 40x30 at
    // the flow origin. The image must paint OVER the blue.
    let sheet = compile(
        r#".img { background-image: url("logo.png"); background-color: blue;
                 width: 40px; height: 30px }"#,
    );

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    // The FACADE injects the real fs loader (sandbox-rooted at the temp dir) + the
    // real image decoder as domain ports — application depends on neither adapter.
    KitBootstrap::new().install_image_resources(&mut runtime, &dir);

    let result = runtime.render_now(&img_view(), std::slice::from_ref(&sheet));

    // 1. The scene carries a DrawImage over the element box (V6-for-images closed).
    let snapshot = result.snapshot().clone();
    let draw_images: Vec<&PaintCommand> = snapshot
        .display_list()
        .commands()
        .iter()
        .filter(|c| matches!(c, PaintCommand::DrawImage { .. }))
        .collect();
    assert_eq!(
        draw_images.len(),
        1,
        "the loaded+decoded image paints one DrawImage"
    );
    let PaintCommand::DrawImage {
        tile,
        region,
        image,
        ..
    } = draw_images[0]
    else {
        unreachable!("filtered to DrawImage");
    };
    // With no authored size/position/repeat, the CSS initials apply: the region is
    // the 40×30 border box and the tile is the intrinsic 2×2 image at its top-left
    // (repeated to fill — the CSS-correct default, not a full-box stretch).
    assert!(
        (region.size.width - 40.0).abs() < 1e-3 && (region.size.height - 30.0).abs() < 1e-3,
        "region is the 40x30 element box, got {region:?}"
    );
    assert!(
        (tile.size.width - 2.0).abs() < 1e-3 && (tile.size.height - 2.0).abs() < 1e-3,
        "auto size draws the intrinsic 2x2 tile, got {tile:?}"
    );
    assert_eq!(image.width(), 2, "the decoded intrinsic width is 2px");
    assert_eq!(image.height(), 2, "the decoded intrinsic height is 2px");

    // 2. The CPU backend rasterizes the image's red pixels inside the box, on top
    //    of the blue background-color.
    let mut renderer = CpuRenderer::new(RendererId::from_raw(2));
    let frame = RenderFrame::new(snapshot, Size::new(VIEWPORT, VIEWPORT));
    let submission = FrameSubmission::new(frame);
    let target = RenderTarget::bitmap(BitmapTarget::new(Size::new(VIEWPORT, VIEWPORT)));
    renderer.render(&submission, &target).expect("cpu render");
    let pixmap = renderer.pixmap().expect("a rasterized frame");

    // A pixel at the box center is the image's RED, not the blue background — proof
    // the image both loaded and painted over the color.
    let px = pixmap.pixel(20, 15).expect("pixel inside the box");
    assert!(
        px.red() > 200,
        "center pixel is red (image), got r={}",
        px.red()
    );
    assert!(
        px.blue() < 60,
        "center pixel is not blue (background), got b={}",
        px.blue()
    );
    assert!(
        px.alpha() > 200,
        "center pixel is opaque, got a={}",
        px.alpha()
    );
}

#[test]
fn background_size_position_repeat_flow_through_to_paint() {
    let dir = temp_asset_dir();

    // `no-repeat` + `center` + `contain` on a 40×30 box with a 2×2 image:
    // `contain` scales the square image to 30×30 (min fit), `center` places it at
    // x=(40−30)/2=5, y=0, and `no-repeat` clears both axis flags.
    let sheet = compile(
        r#".img { background-image: url("logo.png");
                 background-repeat: no-repeat;
                 background-position: center;
                 background-size: contain;
                 background-color: blue;
                 width: 40px; height: 30px }"#,
    );

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    KitBootstrap::new().install_image_resources(&mut runtime, &dir);

    let result = runtime.render_now(&img_view(), std::slice::from_ref(&sheet));
    let snapshot = result.snapshot().clone();

    let draw_images: Vec<&PaintCommand> = snapshot
        .display_list()
        .commands()
        .iter()
        .filter(|c| matches!(c, PaintCommand::DrawImage { .. }))
        .collect();
    assert_eq!(
        draw_images.len(),
        1,
        "one DrawImage for the single background layer"
    );
    let PaintCommand::DrawImage {
        tile,
        region,
        repeat_x,
        repeat_y,
        ..
    } = draw_images[0]
    else {
        unreachable!("filtered to DrawImage");
    };
    assert!(
        (region.size.width - 40.0).abs() < 1e-3 && (region.size.height - 30.0).abs() < 1e-3,
        "region is the 40x30 border box, got {region:?}"
    );
    // `contain` of a 2×2 image into 40×30 = 30×30 (min scale, aspect preserved).
    assert!(
        (tile.size.width - 30.0).abs() < 1e-3 && (tile.size.height - 30.0).abs() < 1e-3,
        "contain tile is 30x30, got {tile:?}"
    );
    // `center`: x offset (40−30)/2 = 5 from the region's left; y offset 0.
    assert!(
        (tile.origin.x - (region.origin.x + 5.0)).abs() < 1e-3,
        "center x offset is 5, got {tile:?}"
    );
    assert!(
        (tile.origin.y - region.origin.y).abs() < 1e-3,
        "center y offset is 0 (tile fills the height), got {tile:?}"
    );
    assert!(!repeat_x && !repeat_y, "no-repeat clears both axis flags");

    // The CPU backend paints the image's red inside the tile (over the blue).
    let mut renderer = CpuRenderer::new(RendererId::from_raw(2));
    let frame = RenderFrame::new(snapshot, Size::new(VIEWPORT, VIEWPORT));
    let submission = FrameSubmission::new(frame);
    let target = RenderTarget::bitmap(BitmapTarget::new(Size::new(VIEWPORT, VIEWPORT)));
    renderer.render(&submission, &target).expect("cpu render");
    let pixmap = renderer.pixmap().expect("a rasterized frame");
    // A pixel inside the centered 30×30 tile is red (image), not blue.
    let px = pixmap.pixel(20, 15).expect("pixel inside the tile");
    assert!(
        px.red() > 200,
        "tile pixel is red (image), got r={}",
        px.red()
    );
    assert!(
        px.blue() < 60,
        "tile pixel is not blue (background), got b={}",
        px.blue()
    );
}

#[test]
fn missing_url_asset_paints_no_image_and_does_not_panic() {
    // The sandbox root exists but the referenced file does not: the load fails, the
    // image caches nothing, and the frame paints no DrawImage (honest skip).
    let dir = temp_asset_dir();
    let sheet =
        compile(r#".img { background-image: url("absent.png"); width: 40px; height: 30px }"#);

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    KitBootstrap::new().install_image_resources(&mut runtime, &dir);

    let result = runtime.render_now(&img_view(), std::slice::from_ref(&sheet));
    let has_image = result
        .snapshot()
        .display_list()
        .commands()
        .iter()
        .any(|c| matches!(c, PaintCommand::DrawImage { .. }));
    assert!(!has_image, "a missing asset paints no image");
    assert!(result.is_balanced());
}
