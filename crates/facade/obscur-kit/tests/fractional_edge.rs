#![doc = include_str!("fractional_edge.doc/_module.md")]
#![forbid(unsafe_code)]

#[path = "fractional_edge/background.rs"]
mod background;
#[path = "fractional_edge/clipping.rs"]
mod clipping;

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::{Point, Size};
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
    scene_for_view(&dot_view(), css, physical, scale)
}

fn scene_for_view(view: &View, css: &str, physical: f32, scale: f32) -> FrameSubmission {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(physical, physical));
    rt.set_surface(Size::new(physical, physical), scale);
    let sheet = compile(css);
    let result = rt.render_now(view, std::slice::from_ref(&sheet));
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
fn headless_gpu() -> (wgpu::Device, wgpu::Queue) {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        force_fallback_adapter: false,
        compatible_surface: None,
    }))
    .expect(
        "no GPU adapter on this machine, and this test was asked for by name. It \
         checks pixels a GPU produced, so there is nothing it could have verified \
         without one and passing here would be a lie. Either run it where an adapter \
         exists, or leave it ignored — the default — where it is counted as ignored \
         and not as passed.",
    );
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
        .expect("the adapter this machine reported refused to open a device")
}

#[doc = include_str!("fractional_edge.doc/the_gpu_backend_keeps_the_same_extreme_pixels.md")]
#[test]
#[ignore = "needs a GPU adapter; run with `cargo test -p obscur-kit --test fractional_edge -- --ignored`"]
fn the_gpu_backend_keeps_the_same_extreme_pixels() {
    let (device, queue) = headless_gpu();
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

#[doc = include_str!("fractional_edge.doc/gpu_readback_matches_cpu_premultiplied_alpha.md")]
#[test]
#[ignore = "needs a GPU adapter"]
fn gpu_readback_matches_cpu_premultiplied_alpha() {
    const EXTENT: u32 = 37;
    let size = Size::new(37.0, 37.0);
    let submission = scene(
        ".dot { width: 30px; height: 30px; background-color: rgba(51, 102, 153, 0.5) }",
        size.width,
        1.0,
    );
    let mut cpu = CpuRenderer::new(RendererId::from_raw(2));
    cpu.render(&submission, &RenderTarget::bitmap(BitmapTarget::new(size)))
        .expect("CPU render");
    let cpu = cpu.pixmap().unwrap().data();
    let (device, queue) = headless_gpu();
    let devices = std::iter::once((
        "Vello",
        VelloDevice::new(device.clone(), queue.clone()).expect("Vello device"),
    ));
    #[cfg(feature = "render-lyon")]
    let devices = devices.chain(std::iter::once((
        "Lyon",
        VelloDevice::lyon(device.clone(), queue.clone()).without_lyon_fallback(),
    )));
    #[cfg(feature = "render-vello-hybrid")]
    let devices = devices.chain(std::iter::once((
        "Vello hybrid",
        VelloDevice::hybrid(device, queue),
    )));
    let center = (15 * EXTENT as usize + 15) * 4;
    let expected = &cpu[center..center + 4];
    assert_eq!(expected, &[26, 51, 77, 128]);
    for (backend, device) in devices {
        let mut gpu = VelloRenderer::new(RendererId::from_raw(3)).with_device(device);
        let (width, height, gpu) = gpu
            .render_to_image(&submission, EXTENT, EXTENT)
            .expect("GPU render");
        assert_eq!((width, height), (EXTENT, EXTENT));
        assert_eq!(gpu.len(), cpu.len());
        let actual = &gpu[center..center + 4];
        for (&got, &want) in actual.iter().zip(expected) {
            assert!(
                got.abs_diff(want) <= 1,
                "{backend} {actual:?}, CPU {expected:?}"
            );
        }
        assert_eq!(&gpu[gpu.len() - 4..], &[0, 0, 0, 0], "{backend}");
    }
}

#[doc = include_str!("fractional_edge.doc/cpu_and_vello_share_the_oklab_gradient_midpoint.md")]
#[test]
#[ignore = "needs a GPU adapter"]
fn cpu_and_vello_share_the_oklab_gradient_midpoint() {
    const EXTENT: u32 = 65;
    let size = Size::new(65.0, 65.0);
    let submission = scene(
        ".dot { width: 65px; height: 65px; background-image: linear-gradient(to right, red, green) }",
        size.width,
        1.0,
    );
    let mut cpu = CpuRenderer::new(RendererId::from_raw(2));
    cpu.render(&submission, &RenderTarget::bitmap(BitmapTarget::new(size)))
        .expect("CPU render");
    let (device, queue) = headless_gpu();
    let vello = VelloDevice::new(device, queue).expect("Vello device");
    let mut gpu = VelloRenderer::new(RendererId::from_raw(3)).with_device(vello);
    let (_, _, gpu) = gpu
        .render_to_image(&submission, EXTENT, EXTENT)
        .expect("GPU render");
    let mid = (32 * EXTENT as usize + 32) * 4;
    let cpu = &cpu.pixmap().unwrap().data()[mid..mid + 4];
    let gpu = &gpu[mid..mid + 4];
    for (actual, expected) in [(cpu, [166_u8, 105, 0, 255]), (gpu, [166, 105, 0, 255])] {
        assert!(
            actual
                .iter()
                .zip(expected)
                .all(|(&got, want)| got.abs_diff(want) <= 3),
            "pixel {actual:?} differs from Oklab midpoint {expected:?}"
        );
    }
}

#[test]
#[ignore = "needs a GPU adapter"]
fn cpu_and_vello_invert_the_same_translucent_css_color() {
    const EXTENT: u32 = 16;
    let size = Size::new(16.0, 16.0);
    let submission = scene(
        ".dot { width: 8px; height: 8px; background-color: rgba(255, 0, 0, 0.5); filter: invert(1) }",
        size.width,
        1.0,
    );
    let mut cpu = CpuRenderer::new(RendererId::from_raw(2));
    cpu.render(&submission, &RenderTarget::bitmap(BitmapTarget::new(size)))
        .expect("CPU render");
    let (device, queue) = headless_gpu();
    let device = VelloDevice::new(device, queue).expect("Vello device");
    let mut gpu = VelloRenderer::new(RendererId::from_raw(3)).with_device(device);
    let (_, _, gpu) = gpu
        .render_to_image(&submission, EXTENT, EXTENT)
        .expect("GPU render");

    let inside = (4 * EXTENT as usize + 4) * 4;
    let outside = (12 * EXTENT as usize + 12) * 4;
    for (backend, pixels) in [
        ("CPU", cpu.pixmap().unwrap().data()),
        ("Vello", gpu.as_slice()),
    ] {
        let color = &pixels[inside..inside + 4];
        assert!(
            color
                .iter()
                .zip([0, 128, 128, 128])
                .all(|(&got, want)| got.abs_diff(want) <= 1),
            "{backend}: {color:?}"
        );
        assert_eq!(&pixels[outside..outside + 4], &[0; 4]);
    }
}

#[test]
#[ignore = "needs a GPU adapter"]
fn vello_keeps_transform_and_clip_after_a_backdrop_boundary() {
    use obscur_kit::geometry::{AffineTransform, CornerRadii, Rect};
    use obscur_kit::scene::{
        clip::Clip,
        command::{DisplayList, PaintCommand},
        scene::{SceneGeneration, SceneSnapshot},
    };

    let size = Size::new(24.0, 24.0);
    let plain = scene(
        ".dot { width: 8px; height: 8px; background: red }",
        24.0,
        1.0,
    );
    let draw = plain
        .frame()
        .scene()
        .display_list()
        .commands()
        .iter()
        .find(|command| matches!(command, PaintCommand::DrawPath { .. }))
        .unwrap();
    let bounds = Rect::new(Point::ORIGIN, Size::new(4.0, 8.0));
    let commands = vec![
        PaintCommand::PushTransform(AffineTransform::translate(8.0, 4.0)),
        PaintCommand::PushClip(Clip::rect(bounds)),
        PaintCommand::BackdropFilter {
            bounds,
            radii: CornerRadii::default(),
            blur: 2.0,
        },
        draw.clone(),
        PaintCommand::PopClip,
        PaintCommand::PopTransform,
    ];
    let snapshot = SceneSnapshot::new(
        DisplayList::from_commands(commands),
        SceneGeneration::from_raw(1),
    );
    let submission = FrameSubmission::new(RenderFrame::new(snapshot, size));
    let (device, queue) = headless_gpu();
    let mut gpu = VelloRenderer::new(RendererId::from_raw(3))
        .with_device(VelloDevice::new(device, queue).unwrap());
    let (_, _, pixels) = gpu.render_to_image(&submission, 24, 24).unwrap();
    for (x, y, alpha) in [(9, 6, 255), (3, 6, 0), (13, 6, 0), (9, 2, 0)] {
        assert_eq!(pixels[(y * 24 + x) * 4 + 3], alpha, "pixel ({x}, {y})");
    }

    let bounds = Rect::new(Point::ORIGIN, Size::new(16.0, 16.0));
    let clip = Rect::new(Point::new(8.0, 4.0), Size::new(4.0, 8.0));
    let commands = vec![
        draw.clone(),
        PaintCommand::PushClip(Clip::rect(clip)),
        PaintCommand::BackdropFilter {
            bounds,
            radii: CornerRadii::default(),
            blur: 2.0,
        },
        PaintCommand::PopClip,
    ];
    let snapshot = SceneSnapshot::new(
        DisplayList::from_commands(commands),
        SceneGeneration::from_raw(2),
    );
    let submission = FrameSubmission::new(RenderFrame::new(snapshot, size));
    let (_, _, pixels) = gpu.render_to_image(&submission, 24, 24).unwrap();
    let alpha = |x: usize, y: usize| pixels[(y * 24 + x) * 4 + 3];
    assert!(
        alpha(8, 6) > 0,
        "the blur crosses the source's edge inside the clip"
    );
    assert_eq!(alpha(8, 2), 0, "the blur stops at the clip");
    assert_eq!(alpha(7, 6), 255, "the original survives outside the clip");

    use obscur_kit::scene::{color::Color, layer::SceneLayer, paint::Paint, path::Path};
    let mut triangle = Path::new();
    triangle.move_to(Point::new(4.0, 0.0));
    triangle.line_to(Point::new(16.0, 0.0));
    triangle.line_to(Point::new(4.0, 16.0));
    triangle.close();
    let commands = vec![
        PaintCommand::DrawPath {
            path: Path::rect(Rect::new(Point::ORIGIN, Size::new(8.0, 16.0))),
            paint: Paint::solid_fill(Color::rgba(1.0, 0.0, 0.0, 0.5)),
        },
        PaintCommand::PushLayer(
            SceneLayer::new()
                .with_clip(Clip::path(triangle))
                .with_transform(AffineTransform::translate(0.0, 1.0)),
        ),
        PaintCommand::BackdropFilter {
            bounds,
            radii: CornerRadii::elliptical(2.0, 6.0),
            blur: 2.0,
        },
        PaintCommand::PopLayer,
    ];
    let snapshot = SceneSnapshot::new(
        DisplayList::from_commands(commands),
        SceneGeneration::from_raw(3),
    );
    let submission = FrameSubmission::new(RenderFrame::new(snapshot, size));
    let (_, _, pixels) = gpu.render_to_image(&submission, 24, 24).unwrap();
    let alpha = |x: usize, y: usize| pixels[(y * 24 + x) * 4 + 3];
    assert!(
        (1..128).contains(&alpha(7, 4)),
        "blur replaces source alpha inside the clip"
    );
    assert!(alpha(8, 4) > 0);
    assert_eq!(
        alpha(8, 13),
        0,
        "the clip cuts off the blur inside its bounding box"
    );
    assert_eq!(alpha(2, 4), 128);
}

#[test]
#[ignore = "needs a GPU adapter"]
fn vello_applies_the_filter_layers_path_clip() {
    use obscur_kit::geometry::Rect;
    use obscur_kit::scene::{
        clip::Clip,
        color::Color,
        command::{DisplayList, PaintCommand},
        layer::{LayerEffect, SceneLayer},
        paint::Paint,
        path::Path,
        scene::{SceneGeneration, SceneSnapshot},
    };

    let mut triangle = Path::new();
    triangle.move_to(Point::new(2.0, 2.0));
    triangle.line_to(Point::new(18.0, 2.0));
    triangle.line_to(Point::new(2.0, 18.0));
    triangle.close();
    let mut layer = SceneLayer::new().with_clip(Clip::path(triangle));
    let mut matrix = [0.0; 20];
    matrix[8] = 1.0;
    matrix[15] = 1.0;
    layer.add_effect(LayerEffect::color_matrix(matrix));
    let size = Size::new(24.0, 24.0);
    let list = DisplayList::from_commands(vec![
        PaintCommand::PushLayer(layer),
        PaintCommand::DrawPath {
            path: Path::rect(Rect::new(Point::ORIGIN, size)),
            paint: Paint::solid_fill(Color::rgba(1.0, 0.0, 0.0, 1.0)),
        },
        PaintCommand::PopLayer,
    ]);
    let snapshot = SceneSnapshot::new(list, SceneGeneration::from_raw(1));
    let submission = FrameSubmission::new(RenderFrame::new(snapshot, size));
    let (device, queue) = headless_gpu();
    let mut gpu = VelloRenderer::new(RendererId::from_raw(3))
        .with_device(VelloDevice::new(device, queue).unwrap());
    let (_, _, pixels) = gpu.render_to_image(&submission, 24, 24).unwrap();
    let pixel = |x: usize, y: usize| {
        let start = (y * 24 + x) * 4;
        &pixels[start..start + 4]
    };
    assert_eq!(pixel(5, 5), &[0, 0, 255, 255]);
    assert_eq!(pixel(13, 13), &[0; 4]);

    let mut layer = SceneLayer::new().with_clip(Clip::rect(Rect::new(Point::ORIGIN, size)));
    layer.add_effect(LayerEffect::blur(0.0));
    let list = DisplayList::from_commands(vec![
        PaintCommand::PushLayer(layer),
        PaintCommand::DrawPath {
            path: Path::rect(Rect::new(Point::ORIGIN, size)),
            paint: Paint::solid_fill(Color::rgba(1.0, 0.0, 0.0, 0.5)),
        },
        PaintCommand::PopLayer,
    ]);
    let snapshot = SceneSnapshot::new(list, SceneGeneration::from_raw(2));
    let submission = FrameSubmission::new(RenderFrame::new(snapshot, size));
    let (_, _, pixels) = gpu.render_to_image(&submission, 24, 24).unwrap();
    let offset = (5 * 24 + 5) * 4;
    assert_eq!(&pixels[offset..offset + 4], &[128, 0, 0, 128]);
}

#[test]
#[ignore = "needs a GPU adapter"]
fn vello_composites_a_filter_before_later_siblings() {
    let view = View::from_fragment(ViewFragment::from_nodes(
        ["filtered", "later"]
            .into_iter()
            .enumerate()
            .map(|(index, class)| {
                ViewNode::element(ComponentId::from_raw(index as u64 + 1))
                    .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
            })
            .collect(),
    ));
    let (device, queue) = headless_gpu();
    let mut gpu = VelloRenderer::new(RendererId::from_raw(3))
        .with_device(VelloDevice::new(device, queue).unwrap());
    for (background, foreground, before, overlap) in [
        (
            "red",
            "background: blue",
            [0, 255, 255, 255],
            [0, 0, 255, 255],
        ),
        (
            "rgba(255, 0, 0, 0.5)",
            "background: yellow; mix-blend-mode: multiply",
            [0, 128, 128, 128],
            [127, 255, 0, 255],
        ),
    ] {
        let submission = scene_for_view(
            &view,
            &format!(
                ".filtered {{ position: absolute; left: 0; top: 0; width: 20px; height: 20px; background: {background}; filter: invert(1); z-index: 1 }} \
                 .later {{ position: absolute; left: 10px; top: 0; width: 14px; height: 20px; {foreground}; z-index: 2 }}"
            ),
            24.0,
            1.0,
        );
        let (_, _, pixels) = gpu.render_to_image(&submission, 24, 24).unwrap();
        let pixel = |x: usize| &pixels[(8 * 24 + x) * 4..(8 * 24 + x + 1) * 4];
        assert_eq!(pixel(5), &before);
        assert_eq!(pixel(15), &overlap);
    }
}

#[test]
#[ignore = "needs a GPU adapter"]
fn cpu_and_vello_blur_preserve_a_uniform_translucent_color() {
    const EXTENT: u32 = 32;
    let submission = scene(
        ".dot { width: 24px; height: 24px; background-color: rgba(64, 128, 192, 0.5); filter: blur(1px) }",
        32.0,
        1.0,
    );
    let mut cpu = CpuRenderer::new(RendererId::from_raw(2));
    cpu.render(
        &submission,
        &RenderTarget::bitmap(BitmapTarget::new(Size::new(32.0, 32.0))),
    )
    .expect("CPU render");
    let (device, queue) = headless_gpu();
    let device = VelloDevice::new(device, queue).expect("Vello device");
    let mut gpu = VelloRenderer::new(RendererId::from_raw(3)).with_device(device);
    let (_, _, pixels) = gpu
        .render_to_image(&submission, EXTENT, EXTENT)
        .expect("GPU render");
    let center = (12 * EXTENT as usize + 12) * 4;
    let edge = (12 * EXTENT as usize + 24) * 4;
    for (backend, pixels) in [
        ("CPU", cpu.pixmap().unwrap().data()),
        ("Vello", pixels.as_slice()),
    ] {
        let color = &pixels[center..center + 4];
        assert!(
            color
                .iter()
                .zip([32, 64, 96, 128])
                .all(|(&got, want)| got.abs_diff(want) <= 1),
            "{backend}: {color:?}"
        );
        assert!(pixels[edge + 3] > 0 && pixels[edge + 3] < 128, "{backend}");
    }
}

#[doc = include_str!("fractional_edge.doc/cpu_and_vello_rasterize_the_same_radial_ellipse.md")]
#[test]
#[ignore = "needs a GPU adapter"]
fn cpu_and_vello_rasterize_the_same_radial_ellipse() {
    const EXTENT: u32 = 65;
    let size = Size::new(65.0, 65.0);
    let mut cpu = CpuRenderer::new(RendererId::from_raw(2));
    let (device, queue) = headless_gpu();
    let vello = VelloDevice::new(device, queue).expect("Vello device");
    let mut gpu = VelloRenderer::new(RendererId::from_raw(3)).with_device(vello);

    use obscur_kit::geometry::{Point, Rect};
    use obscur_kit::scene::color::Color;
    use obscur_kit::scene::command::{DisplayList, PaintCommand};
    use obscur_kit::scene::paint::{Brush, ExtendMode, GradientStop, Paint, RadialGradient};
    use obscur_kit::scene::path::Path;
    use obscur_kit::scene::scene::{SceneGeneration, SceneSnapshot};

    for (mode, outside) in [
        (ExtendMode::Pad, [0, 128, 0, 255]),
        (ExtendMode::Repeat, [166, 105, 0, 255]),
        (ExtendMode::Reflect, [166, 105, 0, 255]),
    ] {
        let gradient = RadialGradient::new_elliptical(
            Point::new(32.5, 32.5),
            20.0,
            10.0,
            vec![
                GradientStop::new(0.0, Color::rgb(1.0, 0.0, 0.0)),
                GradientStop::new(1.0, Color::rgb(0.0, 128.0 / 255.0, 0.0)),
            ],
        )
        .with_extend(mode);
        let scene = SceneSnapshot::new(
            DisplayList::from_commands(vec![PaintCommand::DrawPath {
                path: Path::rect(Rect::new(Point::ORIGIN, size)),
                paint: Paint::Fill(Brush::Radial(gradient)),
            }]),
            SceneGeneration::from_raw(1),
        );
        let css = match mode {
            ExtendMode::Pad => Some("radial-gradient(ellipse 20px 10px, red, green)"),
            ExtendMode::Repeat => {
                Some("repeating-radial-gradient(ellipse 40px 20px, red 10px, green 75%)")
            }
            ExtendMode::Reflect => None,
        };
        let submission = if let Some(css) = css {
            self::scene(
                &format!(".dot {{ width: 65px; height: 65px; background-image: {css} }}"),
                size.width,
                1.0,
            )
        } else {
            FrameSubmission::new(RenderFrame::new(scene, size))
        };
        cpu.render(&submission, &RenderTarget::bitmap(BitmapTarget::new(size)))
            .expect("CPU render");
        let (_, _, gpu) = gpu
            .render_to_image(&submission, EXTENT, EXTENT)
            .expect("GPU render");
        let cpu = cpu.pixmap().unwrap().data();

        let samples = if mode == ExtendMode::Repeat {
            [(32, 32, outside), (52, 32, outside), (32, 42, outside)]
        } else {
            [
                (42, 32, [166_u8, 105, 0, 255]),
                (32, 37, [166, 105, 0, 255]),
                (62, 32, outside),
            ]
        };
        for (x, y, expected) in samples {
            let offset = (y * EXTENT as usize + x) * 4;
            // A subpixel shift is more visible on the 10 px axis. Allow
            // for the GPU's sample position and interpolation precision.
            for (backend, pixels, tolerance) in [("CPU", cpu, 3), ("Vello", gpu.as_slice(), 12)] {
                let actual = &pixels[offset..offset + 4];
                assert!(
                    actual
                        .iter()
                        .zip(expected)
                        .all(|(&got, want)| got.abs_diff(want) <= tolerance),
                    "{backend} {mode:?} at ({x}, {y}): {actual:?} differs from {expected:?}",
                );
            }
        }
    }

    let collapsed = self::scene(
        ".dot { width: 65px; height: 65px;
          background-image: repeating-radial-gradient(circle, red 10px, green 10px) }",
        size.width,
        1.0,
    );
    cpu.render(&collapsed, &RenderTarget::bitmap(BitmapTarget::new(size)))
        .expect("CPU render");
    let center = cpu.pixmap().unwrap().pixel(32, 32).unwrap();
    assert_eq!(
        [center.red(), center.green(), center.blue(), center.alpha()],
        [128, 64, 0, 255]
    );
}

#[doc = include_str!("fractional_edge.doc/linear_repeat_keeps_the_first_stops_phase.md")]
#[test]
#[ignore = "needs a GPU adapter"]
fn linear_repeat_keeps_the_first_stops_phase() {
    const EXTENT: u32 = 80;
    let size = Size::new(80.0, 80.0);
    let mut cpu = CpuRenderer::new(RendererId::from_raw(2));
    let (device, queue) = headless_gpu();
    let vello = VelloDevice::new(device, queue).expect("Vello device");
    let mut gpu = VelloRenderer::new(RendererId::from_raw(3)).with_device(vello);
    for stops in ["red 10px, green 62.5%", "red -30px, green 12.5%"] {
        let submission = scene(
            &format!(
                ".dot {{ width: 80px; height: 80px;
                  background-image: repeating-linear-gradient(to right, {stops}) }}"
            ),
            size.width,
            1.0,
        );
        cpu.render(&submission, &RenderTarget::bitmap(BitmapTarget::new(size)))
            .expect("CPU render");
        let (_, _, gpu) = gpu
            .render_to_image(&submission, EXTENT, EXTENT)
            .expect("GPU render");
        for x in [29, 69] {
            let offset = (32 * EXTENT as usize + x) * 4;
            let expected = [166_u8, 105, 0, 255];
            for pixels in [cpu.pixmap().unwrap().data(), gpu.as_slice()] {
                let actual = &pixels[offset..offset + 4];
                assert!(
                    actual
                        .iter()
                        .zip(expected)
                        .all(|(&got, want)| got.abs_diff(want) <= 9),
                    "{stops} at x={x}: {actual:?} differs from {expected:?}",
                );
            }
        }
    }
}
