#![doc = include_str!("fractional_edge.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};
use obscur_render::frame::{FrameSubmission, RenderFrame};
use obscur_render::target::{BitmapTarget, RenderTarget};
use obscur_render::{Renderer, RendererId};
use obscur_render_cpu::renderer::CpuRenderer;
use obscur_render_vello::renderer::{VelloDevice, VelloRenderer};

#[doc = include_str!("fractional_edge.doc/LOGICAL.md")]
const LOGICAL: f32 = 56.0;

#[doc = include_str!("fractional_edge.doc/SCALE.md")]
const SCALE: f32 = 1.6;

#[doc = include_str!("fractional_edge.doc/CIRCLE_CSS.md")]
const CIRCLE_CSS: &str = ".dot{width:56px;height:56px;border-radius:28px;background-color:#ff0000}";

#[doc = include_str!("fractional_edge.doc/compile.md")]
fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("fractional_edge.doc/dot_view.md")]
fn dot_view() -> View {
    let node = ViewNode::element(ComponentId::from_raw(30))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("dot".into())));
    View::from_fragment(ViewFragment::from_nodes(vec![node]))
}

#[doc = include_str!("fractional_edge.doc/buffer_extent.md")]
fn buffer_extent(logical: f32, scale: f32) -> f32 {
    (logical * scale).round()
}

#[doc = include_str!("fractional_edge.doc/scene.md")]
fn scene(css: &str, physical: f32, scale: f32) -> FrameSubmission {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(physical, physical));
    rt.set_surface(Size::new(physical, physical), scale);
    let sheet = compile(css);
    let result = rt.render_now(&dot_view(), std::slice::from_ref(&sheet));
    assert!(result.restyled(), "the authored circle resolved a style");
    FrameSubmission::new(RenderFrame::new(
        rt.snapshot(),
        Size::new(physical, physical),
    ))
}

#[doc = include_str!("fractional_edge.doc/Coverage.md")]
struct Coverage {
    #[doc = include_str!("fractional_edge.doc/width.md")]
    width: u32,
    #[doc = include_str!("fractional_edge.doc/height.md")]
    height: u32,
    #[doc = include_str!("fractional_edge.doc/opaque.md")]
    opaque: Vec<bool>,
}

impl Coverage {
    #[doc = include_str!("fractional_edge.doc/from_pixmap.md")]
    fn from_pixmap(pm: &tiny_skia::Pixmap) -> Self {
        let opaque = pm.pixels().iter().map(|p| p.alpha() > 0).collect();
        Self {
            width: pm.width(),
            height: pm.height(),
            opaque,
        }
    }

    #[doc = include_str!("fractional_edge.doc/from_rgba.md")]
    fn from_rgba(width: u32, height: u32, pixels: &[u8]) -> Self {
        let opaque = pixels.chunks_exact(4).map(|px| px[3] > 0).collect();
        Self {
            width,
            height,
            opaque,
        }
    }

    #[doc = include_str!("fractional_edge.doc/row_span.md")]
    fn row_span(&self, y: u32) -> Option<(u32, u32)> {
        let row = &self.opaque[(y * self.width) as usize..((y + 1) * self.width) as usize];
        let first = row.iter().position(|on| *on)?;
        let last = row.iter().rposition(|on| *on).unwrap_or(first);
        let index = |i: usize| u32::try_from(i).expect("a column index of a rendered row");
        Some((index(first), index(last)))
    }

    #[doc = include_str!("fractional_edge.doc/column_span.md")]
    fn column_span(&self, x: u32) -> Option<(u32, u32)> {
        let on = |y: u32| self.opaque[(y * self.width + x) as usize];
        let first = (0..self.height).find(|y| on(*y))?;
        let last = (0..self.height).rev().find(|y| on(*y)).unwrap();
        Some((first, last))
    }
}

#[doc = include_str!("fractional_edge.doc/ideal_half.md")]
fn ideal_half(y: u32, centre: f32, radius: f32) -> (f32, f32) {
    #[allow(clippy::cast_precision_loss)]
    let (top, bottom) = (y as f32, y as f32 + 1.0);
    let near = if centre >= top && centre <= bottom {
        0.0
    } else {
        (centre - top).abs().min((centre - bottom).abs())
    };
    let far = (centre - top).abs().max((centre - bottom).abs());
    let half = |d: f32| (radius * radius - d * d).max(0.0).sqrt();
    (half(near), half(far))
}

#[doc = include_str!("fractional_edge.doc/deviation.md")]
fn deviation(cover: &Coverage) -> (f32, f32) {
    let radius = LOGICAL * SCALE / 2.0;
    let centre = radius;
    let (mut overshoot, mut undershoot) = (f32::MIN, f32::MIN);
    for y in 0..cover.height {
        let (wide, narrow) = ideal_half(y, centre, radius);
        let Some((first, last)) = cover.row_span(y) else {
            undershoot = undershoot.max(narrow * 2.0);
            continue;
        };
        #[allow(clippy::cast_precision_loss)]
        let (left, right) = (first as f32, last as f32 + 1.0);
        overshoot = overshoot.max((right - (centre + wide)).max((centre - wide) - left));
        undershoot = undershoot.max((left - (centre - narrow)).max((centre + narrow) - right));
    }
    (overshoot, undershoot)
}

#[doc = include_str!("fractional_edge.doc/assert_is_a_circle.md")]
fn assert_is_a_circle(cover: &Coverage, backend: &str) {
    let (overshoot, undershoot) = deviation(cover);
    // One pixel of overshoot is the rasterizer's own granularity: a pixel counts
    // as covered when the shape touches any part of it.
    assert!(
        overshoot < 1.0,
        "[{backend}] painted {overshoot} px beyond where the circle reaches"
    );
    assert!(
        undershoot < 0.25,
        "[{backend}] left {undershoot} px of the circle unpainted"
    );
}

#[doc = include_str!("fractional_edge.doc/assert_touches_every_edge.md")]
fn assert_touches_every_edge(cover: &Coverage, backend: &str) {
    let mid_row = cover.height / 2;
    let mid_col = cover.width / 2;
    assert_eq!(
        cover.row_span(mid_row),
        Some((0, cover.width - 1)),
        "[{backend}] the widest row of a circle drawn tangent to the surface \
         must reach both extreme columns of a {}px buffer",
        cover.width
    );
    assert_eq!(
        cover.column_span(mid_col),
        Some((0, cover.height - 1)),
        "[{backend}] the tallest column must reach both extreme rows of a {}px \
         buffer",
        cover.height
    );
}

#[doc = include_str!("fractional_edge.doc/cpu_coverage.md")]
fn cpu_coverage(css: &str, physical: f32, scale: f32) -> Coverage {
    let mut renderer = CpuRenderer::new(RendererId::from_raw(2));
    let target = RenderTarget::bitmap(BitmapTarget::new(Size::new(physical, physical)));
    renderer
        .render(&scene(css, physical, scale), &target)
        .expect("the CPU backend rasterizes the scene");
    Coverage::from_pixmap(renderer.pixmap().expect("a rasterized frame"))
}

#[doc = include_str!("fractional_edge.doc/a_circle_tangent_to_a_fractionally_scaled_surface_keeps_its_extreme_pixels.md")]
#[test]
fn a_circle_tangent_to_a_fractionally_scaled_surface_keeps_its_extreme_pixels() {
    let physical = buffer_extent(LOGICAL, SCALE);
    assert_eq!(physical, 90.0, "the layer runner's own buffer arithmetic");
    let cover = cpu_coverage(CIRCLE_CSS, physical, SCALE);
    assert_touches_every_edge(&cover, "cpu");
    assert_is_a_circle(&cover, "cpu");
}

#[doc = include_str!("fractional_edge.doc/the_extreme_column_survives_every_fractional_scale.md")]
#[test]
fn the_extreme_column_survives_every_fractional_scale() {
    for scale in [1.0f32, 1.1, 1.2, 1.25, 1.4, 1.5, 1.6, 1.75, 2.0] {
        let physical = buffer_extent(LOGICAL, scale);
        let cover = cpu_coverage(CIRCLE_CSS, physical, scale);
        let mid = cover.height / 2;
        assert_eq!(
            cover.row_span(mid),
            Some((0, cover.width - 1)),
            "scale {scale}: a circle filling a {LOGICAL}px surface lost an \
             extreme column of its {physical}px buffer"
        );
    }
}

#[doc = include_str!("fractional_edge.doc/the_check_above_can_see_a_lost_extreme_column.md")]
#[test]
fn the_check_above_can_see_a_lost_extreme_column() {
    let physical = buffer_extent(LOGICAL, SCALE);
    let mut renderer = CpuRenderer::new(RendererId::from_raw(4));
    // The scene is composed for the full 90px buffer; only the destination is a
    // column and a row short, which is exactly the hypothesis under test.
    let short = RenderTarget::bitmap(BitmapTarget::new(Size::new(physical - 1.0, physical - 1.0)));
    renderer
        .render(&scene(CIRCLE_CSS, physical, SCALE), &short)
        .expect("the CPU backend rasterizes the scene");
    let cover = Coverage::from_pixmap(renderer.pixmap().expect("a rasterized frame"));
    let (_, undershoot) = deviation(&cover);
    assert!(
        undershoot > 0.25,
        "a buffer one column short must read as {undershoot} px of missing \
         circle, or the assertions above prove nothing"
    );
}

#[doc = include_str!("fractional_edge.doc/headless_gpu.md")]
fn headless_gpu() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        force_fallback_adapter: false,
        compatible_surface: None,
    }))
    .ok()?;
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).ok()
}

#[doc = include_str!("fractional_edge.doc/the_gpu_backend_keeps_the_same_extreme_pixels.md")]
#[test]
#[ignore = "needs a GPU adapter"]
fn the_gpu_backend_keeps_the_same_extreme_pixels() {
    let Some((device, queue)) = headless_gpu() else {
        eprintln!("[gpu] no adapter in this environment; nothing was checked");
        return;
    };
    let vello = VelloDevice::new(device, queue).expect("a Vello device");
    let mut renderer = VelloRenderer::new(RendererId::from_raw(3)).with_device(vello);
    let physical = buffer_extent(LOGICAL, SCALE);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let extent = physical as u32;
    let (width, height, pixels) = renderer
        .render_to_image(&scene(CIRCLE_CSS, physical, SCALE), extent, extent)
        .expect("the GPU backend rasterizes the scene");
    let cover = Coverage::from_rgba(width, height, &pixels);
    assert_touches_every_edge(&cover, "vello");
    assert_is_a_circle(&cover, "vello");
}
