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
use obscur_kit::identity::{ComponentId, RuntimeId};
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

#[doc = include_str!("svg_vector.doc/svg_cover_frame.md")]
fn svg_cover_frame() -> obscur_kit::runtime::frame::FrameResult {
    let dir = temp_asset_dir();
    let sheet = compile(
        ".icon { width: 40px; height: 50px; box-sizing: border-box;
                 padding-left: 10px; object-fit: cover; color: red; }",
    );
    let view = icon_view();
    let mut runtime = runtime_at(&dir);
    let _ = runtime.render_now(&view, std::slice::from_ref(&sheet));
    let _ = runtime.poll_resource_tasks();
    let result = runtime.render_now(&view, &[sheet]);
    let _ = std::fs::remove_dir_all(dir);
    result
}

#[test]
fn svg_cover_is_clipped_to_the_content_box() {
    let result = svg_cover_frame();
    let size = Size::new(VIEWPORT, VIEWPORT);
    let mut renderer = CpuRenderer::new(RendererId::from_raw(2));
    let frame = RenderFrame::new(result.snapshot().clone(), size);
    renderer
        .render(
            &FrameSubmission::new(frame),
            &RenderTarget::bitmap(BitmapTarget::new(size)),
        )
        .expect("renders SVG cover");
    let pixmap = renderer.pixmap().unwrap();
    assert!(pixmap.pixel(20, 25).unwrap().red() > 200);
    assert_eq!(
        pixmap.pixel(5, 25).unwrap().alpha(),
        0,
        "left padding is clear"
    );
    assert_eq!(
        pixmap.pixel(45, 25).unwrap().alpha(),
        0,
        "right overflow is clipped"
    );
}

#[doc = include_str!("svg_vector.doc/svg_cover_is_clipped_on_the_gpu.md")]
#[test]
#[ignore = "needs a hardware GPU adapter"]
fn svg_cover_is_clipped_on_the_gpu() {
    use obscur_render_vello::renderer::{VelloDevice, VelloRenderer};

    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&Default::default()))
        .expect("a GPU adapter is required");
    let info = adapter.get_info();
    assert_ne!(info.device_type, wgpu::DeviceType::Cpu);
    eprintln!("GPU: {} ({:?})", info.name, info.backend);
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default()))
        .expect("opens the GPU device");
    let device = VelloDevice::new(device, queue).expect("creates the Vello device");
    let mut renderer = VelloRenderer::new(RendererId::from_raw(3)).with_device(device);
    let result = svg_cover_frame();
    let frame = RenderFrame::new(result.snapshot().clone(), Size::new(VIEWPORT, VIEWPORT));
    let (width, height, pixels) = renderer
        .render_to_image(&FrameSubmission::new(frame), 64, 64)
        .expect("renders SVG cover");
    assert_eq!((width, height), (64, 64));
    let pixel = |x: usize, y: usize| &pixels[(y * 64 + x) * 4..][..4];
    assert!(pixel(20, 25)[0] > 200 && pixel(20, 25)[3] > 200);
    assert_eq!(pixel(5, 25)[3], 0, "left padding is clear");
    assert_eq!(pixel(45, 25)[3], 0, "right overflow is clipped");
}

#[doc = include_str!("svg_vector.doc/svg_preserves_fill_rule_and_paint_opacity.md")]
#[test]
fn svg_preserves_fill_rule_and_paint_opacity() {
    let dir = temp_asset_dir();
    std::fs::write(
        dir.join("icon.svg"),
        br##"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20">
        <path d="M0 0 H16 V20 H0 Z M5 5 H11 V15 H5 Z"
              fill="currentColor" fill-rule="evenodd" fill-opacity=".5"/>
        <rect x="16" width="4" height="20" fill="#00ff00" fill-opacity=".5"/>
        </svg>"##,
    )
    .expect("write svg");
    let sheet = compile(".icon { width: 20px; height: 20px; color: rgba(255, 0, 0, .5) }");
    let mut runtime = runtime_at(&dir);
    let view = icon_view();
    let _ = runtime.render_now(&view, std::slice::from_ref(&sheet));
    let _ = runtime.poll_resource_tasks();
    let result = runtime.render_now(&view, &[sheet]);
    std::fs::remove_dir_all(dir).expect("remove test assets");

    let size = Size::new(VIEWPORT, VIEWPORT);
    let mut renderer = CpuRenderer::new(RendererId::from_raw(2));
    let frame = RenderFrame::new(result.snapshot().clone(), size);
    renderer
        .render(
            &FrameSubmission::new(frame),
            &RenderTarget::bitmap(BitmapTarget::new(size)),
        )
        .expect("renders SVG fills");
    let pixmap = renderer.pixmap().unwrap();
    assert_eq!(pixmap.pixel(8, 10).unwrap().alpha(), 0);
    let current = pixmap.pixel(2, 10).unwrap();
    assert!((63..=65).contains(&current.alpha()), "{current:?}");
    assert_eq!(current.red(), current.alpha());
    let solid = pixmap.pixel(18, 10).unwrap();
    assert!((127..=129).contains(&solid.alpha()), "{solid:?}");
    assert_eq!(solid.green(), solid.alpha());
}

#[doc = include_str!("svg_vector.doc/svg_resource_failures_report_once.md")]
#[test]
fn svg_resource_failures_report_once() {
    use obscur_kit::diagnostics::{Diagnostic, DiagnosticSink};
    use std::{cell::RefCell, rc::Rc};

    struct Recorder(Rc<RefCell<Vec<Diagnostic>>>);

    impl DiagnosticSink for Recorder {
        fn emit(&self, diagnostic: &Diagnostic) {
            self.0.borrow_mut().push(diagnostic.clone());
        }
    }

    let dir = temp_asset_dir();
    std::fs::write(dir.join("broken.svg"), b"invalid SVG").expect("write fixture");

    for (source, loader_installed, expected_code) in [
        ("missing.svg", true, "resource.load_failed"),
        ("broken.svg", true, "resource.load_failed"),
        ("icon.svg", false, "resource.no_loader"),
    ] {
        let mut runtime = if loader_installed {
            runtime_at(&dir)
        } else {
            let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
            KitBootstrap::new().install_image_resources(&mut runtime, &dir);
            runtime
        };
        let diagnostics = Rc::new(RefCell::new(Vec::new()));
        runtime.set_diagnostic_sink(Box::new(Recorder(Rc::clone(&diagnostics))));
        let view = View::from_fragment(ViewFragment::from_nodes(vec![
            Image::new(source).into_view_node(),
        ]));
        for _ in 0..3 {
            let _ = runtime.render_now(&view, &[]);
            let _ = runtime.poll_resource_tasks();
        }

        let diagnostics = diagnostics.borrow();
        assert_eq!(diagnostics.len(), 1, "{source}: {diagnostics:?}");
        let diagnostic = &diagnostics[0];
        assert_eq!(diagnostic.code.as_str(), expected_code);
        assert!(diagnostic.message.contains(source), "{diagnostic:?}");
        if loader_installed {
            assert!(diagnostic.context.resource.is_some());
            assert!(diagnostic.context.node.is_some());
            assert!(diagnostic.context.window.is_some());
        }
    }
    std::fs::remove_dir_all(dir).expect("remove test assets");
}

#[doc = include_str!("svg_vector.doc/svg_background_tiles_stay_vector.md")]
#[test]
fn svg_background_tiles_stay_vector() {
    let dir = temp_asset_dir();
    let view = View::from_fragment(ViewFragment::from_nodes(vec![
        obscur_kit::view::view::ViewNode::element(ComponentId::from_raw(42))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("icon".into()))),
    ]));
    for layers in [
        "url(icon.svg)",
        "url(icon.svg), linear-gradient(blue, blue)",
    ] {
        let sheet = compile(&format!(
            ".icon {{ width: 21px; height: 18px; color: red;
                      background-image: {layers}; background-size: 8px 6px;
                      background-position: 3px 4px; background-repeat: repeat-x }}"
        ));
        let mut runtime = runtime_at(&dir);
        let _ = runtime.render_now(&view, std::slice::from_ref(&sheet));
        assert_eq!(runtime.poll_resource_tasks(), 1);
        let result = runtime.render_now(&view, &[sheet]);
        assert_eq!(painted_fills(&result).len(), 4);
        assert!(
            !result
                .snapshot()
                .display_list()
                .commands()
                .iter()
                .any(|command| matches!(command, PaintCommand::DrawImage { .. }))
        );

        let size = Size::new(VIEWPORT, VIEWPORT);
        let frame = RenderFrame::new(result.snapshot().clone(), size);
        let mut renderer = CpuRenderer::new(RendererId::from_raw(2));
        renderer
            .render(
                &FrameSubmission::new(frame),
                &RenderTarget::bitmap(BitmapTarget::new(size)),
            )
            .expect("renders SVG background");
        let pixels = renderer.pixmap().unwrap();
        for x in [0, 5, 15, 20] {
            assert_eq!(pixels.pixel(x, 6).unwrap().red(), 255, "x={x}");
        }
        assert_eq!(pixels.pixel(5, 3).unwrap().red(), 0);
        assert_eq!(pixels.pixel(5, 10).unwrap().red(), 0);
        assert_eq!(pixels.pixel(21, 6).unwrap().red(), 0);
    }
    std::fs::remove_dir_all(dir).expect("remove test assets");
}
