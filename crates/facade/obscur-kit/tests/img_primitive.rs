//! End-to-end verification that the **`Image` primitive's `src`** reaches paint
//! through the *real* live pipeline: an `Image::new("logo.png")` node carries its
//! source through reconciliation (as the retained node's `image_source`), the
//! runtime loads it with a sandbox-rooted `FsResourceLoader`, decodes it with the
//! `image`-crate `ImageDecoder`, caches it as a `SceneImage`, and the coordinator
//! paints it as a `DrawImage` over the element's box as the element's CONTENT,
//! sized by `object-fit` (spec §2.6, §5.15 load, §5.16 decode, §5.2 paint).
//!
//! This closes the gap that the `Image` primitive built a node whose `src` never
//! became painted pixels: the primitive now RENDERS.
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
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewProperty, ViewValue};

/// The fixed viewport every test renders under, so geometry is deterministic.
const VIEWPORT: f32 = 64.0;

/// A real 2×2 opaque-red PNG (RGB), embedded so the test needs no image-encoding
/// dependency: the fs loader reads these exact bytes off disk and the `image`-crate
/// decoder turns them into RGBA8.
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
    let dir = std::env::temp_dir().join(format!("obscur-kit-img-prim-{}-{n}", std::process::id()));
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

/// An `Image` primitive referencing `logo.png`, classed `.img` (so CSS can size
/// it) — the primitive lowered to its element node plus a class attribute.
fn img_view() -> View {
    let node = Image::new("logo.png")
        .into_view_node()
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("img".into())));
    View::from_fragment(ViewFragment::from_nodes(vec![node]))
}

#[test]
fn image_primitive_src_reaches_paint_and_the_cpu_backend() {
    let dir = temp_asset_dir();

    // The `Image` primitive, sized 40×30 via CSS. Default `object-fit: fill`
    // stretches the decoded 2×2 image over the whole box.
    let sheet = compile(".img { width: 40px; height: 30px }");

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    // The FACADE injects the real fs loader (sandbox-rooted at the temp dir) + the
    // real image decoder as domain ports — application depends on neither adapter.
    KitBootstrap::new().install_image_resources(&mut runtime, &dir);

    let result = runtime.render_now(&img_view(), std::slice::from_ref(&sheet));

    // 1. The scene carries a DrawImage over the element box: the primitive renders.
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
        "the Image primitive paints one DrawImage"
    );
    let PaintCommand::DrawImage {
        tile,
        region,
        image,
        repeat_x,
        repeat_y,
    } = draw_images[0]
    else {
        unreachable!("filtered to DrawImage");
    };
    // Default `object-fit: fill` → the tile is the whole 40×30 box (stretched).
    assert!(
        (region.size.width - 40.0).abs() < 1e-3 && (region.size.height - 30.0).abs() < 1e-3,
        "region is the 40x30 element box, got {region:?}"
    );
    assert!(
        (tile.size.width - 40.0).abs() < 1e-3 && (tile.size.height - 30.0).abs() < 1e-3,
        "object-fit fill stretches the tile to the box, got {tile:?}"
    );
    assert!(!repeat_x && !repeat_y, "an element image never tiles");
    assert_eq!(image.width(), 2, "the decoded intrinsic width is 2px");
    assert_eq!(image.height(), 2, "the decoded intrinsic height is 2px");

    // 2. The CPU backend rasterizes the image's red pixels inside the box.
    let mut renderer = CpuRenderer::new(RendererId::from_raw(2));
    let frame = RenderFrame::new(snapshot, Size::new(VIEWPORT, VIEWPORT));
    let submission = FrameSubmission::new(frame);
    let target = RenderTarget::bitmap(BitmapTarget::new(Size::new(VIEWPORT, VIEWPORT)));
    renderer.render(&submission, &target).expect("cpu render");
    let pixmap = renderer.pixmap().expect("a rasterized frame");

    let px = pixmap.pixel(20, 15).expect("pixel inside the box");
    assert!(
        px.red() > 200,
        "center pixel is red (image), got r={}",
        px.red()
    );
    assert!(
        px.alpha() > 200,
        "center pixel is opaque, got a={}",
        px.alpha()
    );
}

#[test]
fn image_primitive_object_fit_none_centers_via_css() {
    let dir = temp_asset_dir();

    // `object-fit: none` draws the intrinsic 2×2 image, centered in the 40×30 box
    // at ((40−2)/2, (30−2)/2) = (19, 14). This also exercises the CSS `object-fit`
    // parse path.
    let sheet = compile(".img { width: 40px; height: 30px; object-fit: none }");

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    KitBootstrap::new().install_image_resources(&mut runtime, &dir);

    let result = runtime.render_now(&img_view(), std::slice::from_ref(&sheet));
    let snapshot = result.snapshot().clone();
    let PaintCommand::DrawImage { tile, .. } = snapshot
        .display_list()
        .commands()
        .iter()
        .find(|c| matches!(c, PaintCommand::DrawImage { .. }))
        .expect("one DrawImage")
    else {
        unreachable!("filtered to DrawImage");
    };
    assert!(
        (tile.size.width - 2.0).abs() < 1e-3 && (tile.size.height - 2.0).abs() < 1e-3,
        "object-fit none draws the intrinsic 2x2 tile, got {tile:?}"
    );
    assert!(
        (tile.origin.x - 19.0).abs() < 1e-3 && (tile.origin.y - 14.0).abs() < 1e-3,
        "the intrinsic tile is centered at (19, 14), got {tile:?}"
    );
}

#[test]
fn missing_image_src_paints_no_image_and_does_not_panic() {
    // The sandbox root exists but the referenced file does not: the load fails, the
    // image caches nothing, and the frame paints no DrawImage (honest skip).
    let dir = temp_asset_dir();
    let sheet = compile(".img { width: 40px; height: 30px }");

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    KitBootstrap::new().install_image_resources(&mut runtime, &dir);

    let node = Image::new("absent.png")
        .into_view_node()
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("img".into())));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![node]));

    let result = runtime.render_now(&view, std::slice::from_ref(&sheet));
    let has_image = result
        .snapshot()
        .display_list()
        .commands()
        .iter()
        .any(|c| matches!(c, PaintCommand::DrawImage { .. }));
    assert!(!has_image, "a missing asset paints no image");
    assert!(result.is_balanced());
}
