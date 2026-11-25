#![doc = include_str!("text_run_pixels.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_geometry::{Point, Size};
use obscur_render::RendererId;
use obscur_render::frame::{FrameSubmission, RenderFrame};
use obscur_render_vello::renderer::{VelloDevice, VelloRenderer};
use obscur_scene::color::Color;
use obscur_scene::command::{DisplayList, PaintCommand};
use obscur_scene::paint::Brush;
use obscur_scene::scene::{SceneGeneration, SceneSnapshot};
use obscur_scene::text::{GlyphRun, PositionedGlyph};

#[doc = include_str!("text_run_pixels.doc/FACE.md")]
const FACE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../assets/fonts/NotoSans-Regular.ttf"
));

#[doc = include_str!("text_run_pixels.doc/WIDTH.md")]
const WIDTH: u32 = 160;
#[doc = include_str!("text_run_pixels.doc/HEIGHT.md")]
const HEIGHT: u32 = 64;

#[doc = include_str!("text_run_pixels.doc/ORIGIN.md")]
const ORIGIN: Point = Point::new(16.0, 46.0);

#[doc = include_str!("text_run_pixels.doc/Difference.md")]
#[derive(Debug, Default)]
struct Difference {
    bytes: usize,
    visible_pixels: usize,
    worst_alpha: u8,
}

#[doc = include_str!("text_run_pixels.doc/NO_ADAPTER.md")]
const NO_ADAPTER: &str = "no GPU adapter on this machine, and these tests were \
    asked for by name. Every claim in this file is a claim about pixels a GPU \
    produced, so there is nothing they could have verified without one and passing \
    here would be a lie. Either run them where an adapter exists, or leave them \
    ignored — the default — where they are counted as ignored and not as passed.";

#[doc = include_str!("text_run_pixels.doc/renderer.md")]
fn renderer() -> VelloRenderer {
    let instance = wgpu::Instance::default();
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect(NO_ADAPTER);
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .expect("the adapter this machine reported refused to open a device");
    let device = VelloDevice::new(device, queue).expect("a Vello device on that adapter");
    let mut renderer = VelloRenderer::new(RendererId::from_raw(1)).with_device(device);
    renderer.register_font(0, FACE.to_vec());
    renderer
}

#[doc = include_str!("text_run_pixels.doc/text.md")]
fn text(glyphs: &[(u32, f32)], font_size: f32) -> PaintCommand {
    PaintCommand::DrawText {
        origin: ORIGIN,
        run: GlyphRun::new(
            glyphs
                .iter()
                .map(|(id, x)| PositionedGlyph::new(*id, 0, *x, 0.0, 10.0))
                .collect(),
            font_size,
            "x",
        ),
        brush: Brush::solid(Color::WHITE),
    }
}

#[doc = include_str!("text_run_pixels.doc/raster.md")]
fn raster(renderer: &mut VelloRenderer, commands: Vec<PaintCommand>) -> Vec<u8> {
    let snapshot = SceneSnapshot::new(
        DisplayList::from_commands(commands),
        SceneGeneration::from_raw(1),
    );
    #[allow(clippy::cast_precision_loss)]
    let submission = FrameSubmission::new(RenderFrame::new(
        snapshot,
        Size::new(WIDTH as f32, HEIGHT as f32),
    ));
    let (_, _, bytes) = renderer
        .render_to_image(&submission, WIDTH, HEIGHT)
        .expect("the offscreen render succeeds");
    bytes
}

#[doc = include_str!("text_run_pixels.doc/apart_and_together.md")]
fn apart_and_together(
    renderer: &mut VelloRenderer,
    glyphs: &[(u32, f32)],
    font_size: f32,
) -> Difference {
    let apart: Vec<PaintCommand> = glyphs
        .iter()
        .map(|glyph| text(std::slice::from_ref(glyph), font_size))
        .collect();
    let together = vec![text(glyphs, font_size)];
    difference(&raster(renderer, apart), &raster(renderer, together))
}

#[doc = include_str!("text_run_pixels.doc/difference.md")]
fn difference(a: &[u8], b: &[u8]) -> Difference {
    assert_eq!(a.len(), b.len(), "two rasters of the same target");
    let mut found = Difference::default();
    for (left, right) in a.chunks_exact(4).zip(b.chunks_exact(4)) {
        if left == right {
            continue;
        }
        found.bytes += left.iter().zip(right).filter(|(l, r)| l != r).count();
        if left[3] == 0 && right[3] == 0 {
            continue;
        }
        found.visible_pixels += 1;
        found.worst_alpha = found.worst_alpha.max(left[3].abs_diff(right[3]));
    }
    found
}

#[doc = include_str!("text_run_pixels.doc/the_renderer_repeats_itself_exactly.md")]
#[test]
#[ignore = "needs a GPU adapter; run with `cargo test -p obscur-render-vello --test text_run_pixels -- --ignored`"]
fn the_renderer_repeats_itself_exactly() {
    let mut renderer = renderer();
    let commands = vec![text(&[(36, 0.0)], 15.0), text(&[(37, 30.0)], 15.0)];
    let mut previous = raster(&mut renderer, commands.clone());
    let mut unstable_bytes = 0;
    for _ in 1..6 {
        let next = raster(&mut renderer, commands.clone());
        let found = difference(&previous, &next);
        assert_eq!(
            found.visible_pixels, 0,
            "the same list twice disagreed about a pixel that reaches a screen"
        );
        unstable_bytes = unstable_bytes.max(found.bytes);
        previous = next;
    }
    eprintln!(
        "control: no visible pixel moves across six renders; \
         up to {unstable_bytes} bytes do, all inside fully transparent pixels"
    );
}

#[doc = include_str!("text_run_pixels.doc/legends_far_enough_apart_survive_being_drawn_together.md")]
#[test]
#[ignore = "needs a GPU adapter; run with `cargo test -p obscur-render-vello --test text_run_pixels -- --ignored`"]
fn legends_far_enough_apart_survive_being_drawn_together() {
    let mut renderer = renderer();
    let found = apart_and_together(&mut renderer, &[(36, 0.0), (37, 30.0), (38, 60.0)], 15.0);
    eprintln!(
        "far apart: {} visible pixels differ, {} bytes differ in all",
        found.visible_pixels, found.bytes
    );
    assert_eq!(
        found.visible_pixels, 0,
        "well-separated glyphs must not change when folded into one run \
         (differing bytes {}, all in fully transparent pixels)",
        found.bytes
    );
}

#[doc = include_str!("text_run_pixels.doc/legends_whose_boxes_touch_do_not.md")]
#[test]
#[ignore = "needs a GPU adapter; run with `cargo test -p obscur-render-vello --test text_run_pixels -- --ignored`"]
fn legends_whose_boxes_touch_do_not() {
    let mut renderer = renderer();
    let found = apart_and_together(&mut renderer, &[(36, 0.0), (37, 8.3)], 15.0);
    assert!(
        found.visible_pixels > 0,
        "folding two touching 15 px legends into one glyph run changed no \
         visible pixel here. That is the premise the one-run-per-command rule \
         in `draw_text` rests on; re-examine it before deleting either."
    );
    eprintln!(
        "touching: {} visible pixels differ, worst alpha delta {}/255",
        found.visible_pixels, found.worst_alpha
    );
}

#[doc = include_str!("text_run_pixels.doc/overlapping_glyphs_differ_by_far_more_than_rounding.md")]
#[test]
#[ignore = "needs a GPU adapter; run with `cargo test -p obscur-render-vello --test text_run_pixels -- --ignored`"]
fn overlapping_glyphs_differ_by_far_more_than_rounding() {
    let mut renderer = renderer();
    let found = apart_and_together(&mut renderer, &[(50, 0.0), (50, 1.0)], 40.0);
    assert!(
        found.worst_alpha >= 16,
        "two overlapping 40 px glyphs folded into one non-zero fill differed \
         by only {}/255 in alpha over {} pixels; the union-versus-composite \
         difference this file exists to record has changed shape",
        found.worst_alpha,
        found.visible_pixels
    );
    eprintln!(
        "overlapping: {} visible pixels differ, worst alpha delta {}/255",
        found.visible_pixels, found.worst_alpha
    );
}

#[cfg(feature = "hybrid")]
#[doc = include_str!("text_run_pixels.doc/hybrid_renders_premultiplied_pixels_and_clears_the_reused_target.md")]
#[test]
#[ignore = "needs a GPU adapter"]
fn hybrid_renders_premultiplied_pixels_and_clears_the_reused_target() {
    use obscur_geometry::{CornerRadii, Rect};
    use obscur_scene::clip::Clip;
    use obscur_scene::paint::Paint;
    use obscur_scene::path::Path;

    let instance = wgpu::Instance::default();
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect(NO_ADAPTER);
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .expect("the adapter refused to open a device");
    let mut renderer = VelloRenderer::default().with_hybrid_device(device, queue);
    renderer.register_font(0, FACE.to_vec());

    let clip = Rect::new(Point::new(0.0, 0.0), Size::new(8.0, 8.0));
    let fill = Rect::new(Point::new(0.0, 0.0), Size::new(20.0, 20.0));
    let pixels = raster(
        &mut renderer,
        vec![
            PaintCommand::PushClip(Clip::rect(clip)),
            PaintCommand::DrawPath {
                path: Path::rect(fill),
                paint: Paint::solid_fill(Color::rgba(1.0, 0.0, 0.0, 0.5)),
            },
            PaintCommand::PopClip,
            PaintCommand::DrawShadow {
                rect: Rect::new(Point::new(80.0, 16.0), Size::new(24.0, 24.0)),
                radii: CornerRadii::uniform(4.0),
                color: Color::rgba(0.0, 0.0, 1.0, 0.5),
                blur: 8.0,
                spread: 0.0,
                offset: (0.0, 0.0),
                inset: false,
            },
            text(&[(36, 0.0)], 15.0),
        ],
    );

    let pixel = |x: usize, y: usize| {
        let i = (y * WIDTH as usize + x) * 4;
        &pixels[i..i + 4]
    };
    assert_eq!(pixel(4, 4), &[128, 0, 0, 128]);
    assert_eq!(pixel(10, 4), &[0; 4]);
    assert!(pixel(82, 28)[2] > 0);
    assert!(pixels.chunks_exact(4).any(|rgba| rgba[1] > 0));
    assert!(
        raster(&mut renderer, Vec::new())
            .iter()
            .all(|byte| *byte == 0)
    );
}

#[cfg(feature = "lyon")]
struct NoFallback;

#[cfg(feature = "lyon")]
impl obscur_diagnostics::DiagnosticSink for NoFallback {
    fn emit(&self, diagnostic: &obscur_diagnostics::Diagnostic) {
        assert_ne!(
            diagnostic.code.as_str(),
            "render.lyon.fallback",
            "{}",
            diagnostic.message
        );
    }
}

#[cfg(feature = "lyon")]
#[doc = include_str!("text_run_pixels.doc/lyon_updates_paint_and_reuses_geometry_after_images.md")]
#[test]
#[ignore = "needs a GPU adapter"]
fn lyon_updates_paint_and_reuses_geometry_after_images() {
    use obscur_geometry::{CornerRadii, Rect};
    use obscur_scene::clip::Clip;
    use obscur_scene::color::AlphaMode;
    use obscur_scene::paint::{Paint, SceneImage, Stroke};
    use obscur_scene::path::Path;

    let instance = wgpu::Instance::default();
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect(NO_ADAPTER);
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .expect("open GPU device");
    let mut renderer = VelloRenderer::default().with_lyon_device(device, queue);
    renderer.register_font(0, FACE.to_vec());
    renderer.set_diagnostic_sink(std::sync::Arc::new(NoFallback));

    let clipped = Rect::new(Point::new(0.0, 0.0), Size::new(8.0, 8.0));
    let fill = Rect::new(Point::new(0.0, 0.0), Size::new(20.0, 20.0));
    let mut dashed = Path::new();
    dashed
        .move_to(Point::new(80.0, 56.0))
        .line_to(Point::new(112.0, 56.0));
    let commands = |color| {
        vec![
            PaintCommand::PushClip(Clip::rect(clipped)),
            PaintCommand::DrawPath {
                path: Path::rect(fill),
                paint: Paint::solid_fill(color),
            },
            PaintCommand::PopClip,
            PaintCommand::DrawShadow {
                rect: Rect::new(Point::new(80.0, 16.0), Size::new(24.0, 24.0)),
                radii: CornerRadii::uniform(4.0),
                color: Color::rgba(0.0, 0.0, 1.0, 0.5),
                blur: 8.0,
                spread: 0.0,
                offset: (0.0, 0.0),
                inset: false,
            },
            text(&[(36, 0.0)], 15.0),
            PaintCommand::DrawPath {
                path: dashed.clone(),
                paint: Paint::stroke(
                    Stroke::new(Brush::solid(Color::WHITE), 2.0).with_dash(vec![4.0, 4.0], 0.0),
                ),
            },
        ]
    };
    let pixel_offset = |x, y| (y * WIDTH as usize + x) * 4;
    let middle = pixel_offset(4, 4);
    let outside = pixel_offset(12, 4);

    let red_fill = commands(Color::rgba(1.0, 0.0, 0.0, 0.5));
    let red = raster(&mut renderer, red_fill.clone());
    assert_eq!(&red[middle..middle + 4], &[128, 0, 0, 128]);
    assert_eq!(&red[outside..outside + 4], &[0; 4]);
    assert!(red[pixel_offset(77, 28) + 2] > 0);
    assert_eq!(red[pixel_offset(82, 56) + 3], 255);
    assert_eq!(red[pixel_offset(86, 56) + 3], 0);
    assert!(red.chunks_exact(4).any(|pixel| pixel[1] > 0));

    let green = raster(&mut renderer, commands(Color::rgba(0.0, 1.0, 0.0, 0.5)));
    assert_eq!(&green[middle..middle + 4], &[0, 128, 0, 128]);

    let moved = |x| {
        vec![
            PaintCommand::PushTransform(obscur_geometry::AffineTransform::translate(x, 0.0)),
            PaintCommand::PushClip(Clip::rect(clipped)),
            PaintCommand::DrawPath {
                path: Path::rect(fill),
                paint: Paint::solid_fill(Color::WHITE),
            },
            PaintCommand::PopClip,
            PaintCommand::PopTransform,
        ]
    };
    let first = raster(&mut renderer, moved(10.0));
    let second = raster(&mut renderer, moved(20.0));
    let pixel_alpha = |bytes: &[u8], x| bytes[pixel_offset(x, 4) + 3];
    assert_eq!(pixel_alpha(&first, 14), 255);
    assert_eq!(pixel_alpha(&second, 14), 0);
    assert_eq!(pixel_alpha(&second, 24), 255);
    assert_eq!(pixel_alpha(&second, 30), 0);

    let image = SceneImage::new(1, 1, vec![255, 255, 0, 255], AlphaMode::Straight)
        .expect("one valid RGBA pixel");
    let image_frame = raster(
        &mut renderer,
        vec![PaintCommand::DrawImage {
            image,
            tile: fill,
            region: fill,
            repeat_x: false,
            repeat_y: false,
        }],
    );
    assert_eq!(&image_frame[middle..middle + 4], &[255, 255, 0, 255]);
    assert_eq!(raster(&mut renderer, red_fill), red);
}

#[cfg(feature = "lyon")]
#[doc = include_str!("text_run_pixels.doc/lyon_composites_nested_opacity_groups.md")]
#[test]
#[ignore = "needs a GPU adapter"]
fn lyon_composites_nested_opacity_groups() {
    use obscur_geometry::Rect;
    use obscur_scene::clip::Clip;
    use obscur_scene::layer::SceneLayer;
    use obscur_scene::paint::Paint;
    use obscur_scene::path::Path;

    let instance = wgpu::Instance::default();
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect(NO_ADAPTER);
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .expect("open GPU device");
    let mut renderer = VelloRenderer::default().with_lyon_device(device, queue);
    renderer.set_diagnostic_sink(std::sync::Arc::new(NoFallback));
    let rect = |x, y, w, h| Rect::new(Point::new(x, y), Size::new(w, h));
    let fill = |x, y, w, h, color| PaintCommand::DrawPath {
        path: Path::rect(rect(x, y, w, h)),
        paint: Paint::solid_fill(color),
    };
    let red = Color::rgba(1.0, 0.0, 0.0, 1.0);
    let blue = Color::rgba(0.0, 0.0, 1.0, 1.0);
    let bytes = raster(
        &mut renderer,
        vec![
            PaintCommand::PushClip(Clip::rect(rect(0.0, 0.0, 34.0, 35.0))),
            PaintCommand::PushLayer(SceneLayer::new().with_opacity(0.5)),
            fill(0.0, 0.0, 20.0, 20.0, red),
            fill(10.0, 0.0, 35.0, 20.0, blue),
            PaintCommand::PushLayer(SceneLayer::new().with_opacity(0.5)),
            PaintCommand::PushClip(Clip::rect(rect(2.0, 20.0, 21.0, 20.0))),
            fill(0.0, 20.0, 25.0, 20.0, Color::WHITE),
            fill(10.0, 22.0, 10.0, 12.0, Color::WHITE),
            PaintCommand::PopClip,
            PaintCommand::PopLayer,
            PaintCommand::PopLayer,
            PaintCommand::PopClip,
            fill(40.0, 0.0, 10.0, 10.0, red),
            PaintCommand::PushLayer(SceneLayer::new().with_opacity(0.5)),
            PaintCommand::PushClip(Clip::rect(rect(62.0, 0.0, 5.0, 10.0))),
            fill(60.0, 0.0, 10.0, 10.0, blue),
            PaintCommand::PopClip,
            PaintCommand::PopLayer,
        ],
    );
    let pixel = |x, y| {
        let start = (y * WIDTH as usize + x) * 4;
        &bytes[start..start + 4]
    };
    assert_eq!(pixel(5, 5), [128, 0, 0, 128]);
    assert_eq!(
        pixel(15, 5),
        [0, 0, 128, 128],
        "opacity applies after overlap"
    );
    assert_eq!(pixel(15, 25), [64; 4], "nested opacity multiplies");
    assert_eq!(
        pixel(1, 25),
        [0; 4],
        "the single fill respects its own clip"
    );
    assert_eq!(
        pixel(15, 36),
        [0; 4],
        "ancestor clip constrains the nested layer"
    );
    assert_eq!(
        pixel(38, 5),
        [0; 4],
        "ancestor clip constrains the outer layer"
    );
    assert_eq!(
        pixel(43, 5),
        [255, 0, 0, 255],
        "clip and opacity end with their groups"
    );

    let single_fill = |opacity| {
        vec![
            PaintCommand::PushTransform(obscur_geometry::AffineTransform::translate(10.0, 0.0)),
            PaintCommand::PushLayer(SceneLayer::new().with_opacity(opacity)),
            PaintCommand::PushClip(Clip::rect(rect(2.0, 0.0, 6.0, 8.0))),
            fill(0.0, 0.0, 10.0, 10.0, red),
            PaintCommand::PopClip,
            PaintCommand::PopLayer,
            PaintCommand::PopTransform,
        ]
    };
    let offset = (4 * WIDTH as usize + 14) * 4;
    for (opacity, alpha) in [(1.0, 255), (0.5, 128), (0.25, 64), (0.0, 0)] {
        let bytes = raster(&mut renderer, single_fill(opacity));
        assert_eq!(&bytes[offset..offset + 4], &[alpha, 0, 0, alpha]);
        assert_eq!(
            bytes[offset - 4 * 4 + 3],
            0,
            "clip still applies when opacity changes"
        );
    }
}

#[cfg(feature = "lyon")]
#[doc = include_str!("text_run_pixels.doc/lyon_renders_gradients_without_fallback.md")]
#[test]
#[ignore = "needs a GPU adapter"]
fn lyon_renders_gradients_without_fallback() {
    use obscur_geometry::{AffineTransform, Rect};
    use obscur_scene::clip::Clip;
    use obscur_scene::layer::SceneLayer;
    use obscur_scene::paint::{
        ConicGradient, ExtendMode, GradientStop, LinearGradient, Paint, RadialGradient,
    };
    use obscur_scene::path::Path;

    let instance = wgpu::Instance::default();
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect(NO_ADAPTER);
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .expect("open GPU device");
    let mut lyon = VelloRenderer::default().with_lyon_device(device, queue);
    lyon.set_diagnostic_sink(std::sync::Arc::new(NoFallback));

    let stops = || {
        vec![
            GradientStop::new(0.0, Color::BLACK),
            GradientStop::new(1.0, Color::WHITE),
        ]
    };
    let rect = |x, y, w, h| Rect::new(Point::new(x, y), Size::new(w, h));
    let fill = |bounds, brush| PaintCommand::DrawPath {
        path: Path::rect(bounds),
        paint: Paint::fill(brush),
    };
    let mut commands = vec![
        PaintCommand::PushTransform(AffineTransform::translate(10.0, 0.0)),
        PaintCommand::PushLayer(SceneLayer::new().with_opacity(0.5)),
        PaintCommand::PushClip(Clip::rect(rect(0.0, 0.0, 35.0, 40.0))),
        fill(
            rect(0.0, 0.0, 40.0, 40.0),
            Brush::Linear(
                LinearGradient::new(Point::ORIGIN, Point::new(16.0, 0.0), stops())
                    .with_extend(ExtendMode::Reflect),
            ),
        ),
        PaintCommand::PopClip,
        PaintCommand::PopLayer,
        PaintCommand::PopTransform,
        fill(
            rect(50.0, 0.0, 48.0, 52.0),
            Brush::Radial(
                RadialGradient::new_elliptical(Point::new(74.5, 24.5), 20.0, 12.0, stops())
                    .with_extend(ExtendMode::Repeat),
            ),
        ),
        fill(
            rect(104.0, 0.0, 54.0, 52.0),
            Brush::Conic(ConicGradient::new(Point::new(130.5, 24.5), 0.0, stops())),
        ),
    ];
    commands.push(fill(
        rect(0.0, 44.0, 10.0, 12.0),
        Brush::Linear(LinearGradient::new(
            Point::new(0.5, 50.5),
            Point::new(8.5, 50.5),
            vec![
                GradientStop::new(0.0, Color::rgba(0.0, 1.0, 0.0, 0.0)),
                GradientStop::new(1.0, Color::rgba(1.0, 0.0, 0.0, 1.0)),
            ],
        )),
    ));
    let actual = raster(&mut lyon, commands);
    let premultiplied = (50 * WIDTH as usize + 4) * 4;
    assert!(actual[premultiplied].abs_diff(128) <= 1);
    assert_eq!(&actual[premultiplied + 1..premultiplied + 4], &[0, 0, 128]);

    assert_eq!(actual[(15 * WIDTH as usize + 44) * 4 + 3], 128);
    assert_eq!(actual[(15 * WIDTH as usize + 46) * 4 + 3], 0);
    // Oklab interpolation from black to white at t=10.5/16 has L=t.
    // Its encoded sRGB channel is about 144, then group opacity halves it.
    assert!(actual[(15 * WIDTH as usize + 20) * 4].abs_diff(72) <= 2);
    assert!(actual[(15 * WIDTH as usize + 31) * 4].abs_diff(72) <= 2);

    for (x, y, lightness) in [
        (84, 24, 0.5),  // Halfway along the radial x radius.
        (74, 42, 0.5),  // 1.5 radial y radii wrap to 0.5.
        (140, 24, 0.0), // The conic seam starts on the positive x axis.
        (130, 32, 0.25),
        (120, 24, 0.5),
        (130, 12, 0.75),
    ] {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let expected = (obscur_scene::color::convert::oklab_to_srgb(lightness, 0.0, 0.0)[0] * 255.0)
            .round() as u8;
        let index = (y * WIDTH as usize + x) * 4;
        for channel in &actual[index..index + 3] {
            assert!(
                channel.abs_diff(expected) <= 2,
                "pixel ({x}, {y}): expected {expected}, got {channel}"
            );
        }
        assert_eq!(actual[index + 3], 255);
    }
}

#[cfg(feature = "lyon")]
#[doc = include_str!("text_run_pixels.doc/lyon_draws_tiled_images_with_alpha_and_clips.md")]
#[test]
#[ignore = "needs a GPU adapter"]
fn lyon_draws_tiled_images_with_alpha_and_clips() {
    use obscur_geometry::{AffineTransform, Rect};
    use obscur_scene::clip::Clip;
    use obscur_scene::color::AlphaMode;
    use obscur_scene::layer::SceneLayer;
    use obscur_scene::paint::SceneImage;

    let instance = wgpu::Instance::default();
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect(NO_ADAPTER);
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .expect("open GPU device");
    let mut renderer = VelloRenderer::default().with_lyon_device(device, queue);
    renderer.set_diagnostic_sink(std::sync::Arc::new(NoFallback));

    let rect = |x, y, w, h| Rect::new(Point::new(x, y), Size::new(w, h));
    let image = SceneImage::new(
        2,
        1,
        vec![255, 0, 0, 255, 0, 255, 0, 0],
        AlphaMode::Straight,
    )
    .unwrap();
    let group = vec![
        PaintCommand::PushTransform(AffineTransform::translate(2.0, 0.0)),
        PaintCommand::PushLayer(SceneLayer::new().with_opacity(0.5)),
        PaintCommand::PushClip(Clip::rect(rect(0.0, 0.0, 34.0, 15.0))),
        PaintCommand::DrawImage {
            image,
            tile: rect(0.5, 2.0, 20.0, 8.0),
            region: rect(0.0, 0.0, 50.0, 30.0),
            repeat_x: true,
            repeat_y: false,
        },
        PaintCommand::PopClip,
        PaintCommand::PopLayer,
        PaintCommand::PopTransform,
    ];
    let bounds = rect(48.0, 2.0, 16.0, 16.0);
    let alternate = PaintCommand::DrawImage {
        image: SceneImage::new(1, 1, vec![0, 0, 64, 128], AlphaMode::Premultiplied).unwrap(),
        tile: bounds,
        region: rect(46.0, 2.0, 22.0, 36.0),
        repeat_x: false,
        repeat_y: true,
    };
    let bytes = raster(
        &mut renderer,
        group.iter().cloned().chain([alternate.clone()]).collect(),
    );
    let reordered = raster(
        &mut renderer,
        std::iter::once(alternate).chain(group).collect(),
    );
    assert_eq!(
        bytes, reordered,
        "texture bindings follow the mesh's image order"
    );
    let pixel = |x, y| {
        let offset = (y * WIDTH as usize + x) * 4;
        &bytes[offset..offset + 4]
    };
    assert_eq!(
        pixel(12, 5),
        [64, 0, 0, 64],
        "transparent RGB must not leak through filtering"
    );
    assert_eq!(pixel(32, 5), pixel(12, 5), "tiles repeat along x");
    assert_eq!(
        pixel(37, 5),
        [0; 4],
        "the translated clip bounds repeated images"
    );
    assert_eq!(pixel(12, 14), [0; 4], "the image does not repeat along y");
    assert_eq!(pixel(52, 25), pixel(52, 5), "tiles repeat along y");
    assert_eq!(
        pixel(47, 25),
        [0; 4],
        "vertical repeat does not extend in x"
    );
    assert_eq!(
        pixel(52, 5),
        [0, 0, 64, 128],
        "premultiplied input stays premultiplied"
    );
}

#[cfg(feature = "lyon")]
#[doc = include_str!("text_run_pixels.doc/lyon_applies_color_filters_to_composited_groups.md")]
#[test]
#[ignore = "needs a GPU adapter"]
fn lyon_applies_color_filters_to_composited_groups() {
    use obscur_geometry::Rect;
    use obscur_scene::clip::Clip;
    use obscur_scene::layer::{LayerEffect, SceneLayer};
    use obscur_scene::paint::Paint;
    use obscur_scene::path::Path;

    let instance = wgpu::Instance::default();
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect(NO_ADAPTER);
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .expect("open GPU device");
    let mut renderer = VelloRenderer::default().with_lyon_device(device, queue);
    renderer.set_diagnostic_sink(std::sync::Arc::new(NoFallback));

    let rect = |x, y, w, h| Rect::new(Point::new(x, y), Size::new(w, h));
    let fill = |bounds, color| PaintCommand::DrawPath {
        path: Path::rect(bounds),
        paint: Paint::solid_fill(color),
    };
    let commands = |scale, bias| {
        let mut matrix = [0.0; 20];
        for channel in 0..3 {
            matrix[channel * 5] = scale;
            matrix[16 + channel] = bias;
        }
        matrix[15] = 0.5;
        let mut layer = SceneLayer::new()
            .with_clip(Clip::rect(rect(1.0, 1.0, 32.0, 22.0)))
            .with_opacity(0.5);
        layer.add_effect(LayerEffect::color_matrix(matrix));
        vec![
            PaintCommand::PushLayer(layer),
            fill(rect(0.0, 0.0, 22.0, 20.0), Color::rgba(1.0, 0.0, 0.0, 1.0)),
            fill(rect(10.0, 0.0, 25.0, 20.0), Color::rgba(0.0, 0.0, 1.0, 0.5)),
            PaintCommand::PopLayer,
            fill(rect(40.0, 0.0, 10.0, 10.0), Color::WHITE),
        ]
    };
    let inverted = raster(&mut renderer, commands(-1.0, 1.0));
    let normal = raster(&mut renderer, commands(1.0, 0.0));
    let offset = |x, y| (y * WIDTH as usize + x) * 4;
    let pixel = |x, y| {
        let start = offset(x, y);
        &inverted[start..start + 4]
    };
    assert_eq!(pixel(3, 5), [0, 64, 64, 64]);
    assert_eq!(pixel(15, 5), [32, 64, 32, 64], "filter runs after overlap");
    assert_eq!(
        pixel(28, 5),
        [32, 32, 0, 32],
        "RGB filtering uses straight color"
    );
    assert_eq!(
        pixel(28, 21),
        [0; 4],
        "the RGB bias leaves transparent pixels clear"
    );
    assert_eq!(pixel(34, 5), [0; 4], "the layer retains its clip");
    assert_eq!(pixel(44, 5), [255; 4], "the filter is scoped to the group");
    let start = offset(3, 5);
    assert_eq!(&normal[start..start + 4], &[64, 0, 0, 64]);
}

#[cfg(feature = "lyon")]
#[doc = include_str!("text_run_pixels.doc/lyon_blurs_filtered_groups_without_fallback.md")]
#[test]
#[ignore = "needs a GPU adapter"]
fn lyon_blurs_filtered_groups_without_fallback() {
    use obscur_geometry::Rect;
    use obscur_scene::clip::Clip;
    use obscur_scene::layer::{LayerEffect, SceneLayer};
    use obscur_scene::paint::Paint;
    use obscur_scene::path::Path;

    let instance = wgpu::Instance::default();
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect(NO_ADAPTER);
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .expect("open GPU device");
    let mut renderer = VelloRenderer::default().with_lyon_device(device, queue);
    renderer.set_diagnostic_sink(std::sync::Arc::new(NoFallback));

    let rect = |x, y, w, h| Rect::new(Point::new(x, y), Size::new(w, h));
    let commands = |radius| {
        let mut layer = SceneLayer::new().with_clip(Clip::rect(rect(20.0, 16.0, 12.0, 12.0)));
        layer.add_effect(LayerEffect::blur(radius));
        let mut matrix = [0.0; 20];
        // Route red into blue, keeping the source alpha.
        matrix[8] = 1.0;
        matrix[15] = 1.0;
        layer.add_effect(LayerEffect::color_matrix(matrix));
        vec![
            PaintCommand::PushClip(Clip::rect(rect(16.0, 8.0, 30.0, 32.0))),
            PaintCommand::PushLayer(SceneLayer::new().with_opacity(0.5)),
            PaintCommand::PushLayer(layer),
            PaintCommand::DrawPath {
                path: Path::rect(rect(18.0, 14.0, 15.0, 16.0)),
                paint: Paint::solid_fill(Color::rgba(1.0, 0.0, 0.0, 1.0)),
            },
            PaintCommand::PopLayer,
            PaintCommand::PopLayer,
            PaintCommand::PopClip,
        ]
    };
    let sharp = raster(&mut renderer, commands(0.0));
    let blurred = raster(&mut renderer, commands(3.0));
    let center = (22 * WIDTH as usize + 26) * 4;
    let outside_layer = (22 * WIDTH as usize + 17) * 4;
    let outside_ancestor = (22 * WIDTH as usize + 15) * 4;
    assert_eq!(&sharp[center..center + 4], &[0, 0, 128, 128]);
    assert!(blurred[center + 3] > 100 && blurred[center + 3] < 128);
    assert_eq!(&sharp[outside_layer..outside_layer + 4], &[0; 4]);
    assert!(
        blurred[outside_layer + 3] > 0,
        "the blur extends beyond the source clip"
    );
    assert_eq!(blurred[outside_layer], 0);
    assert_eq!(blurred[outside_layer + 2], blurred[outside_layer + 3]);
    assert_eq!(&blurred[outside_ancestor..outside_ancestor + 4], &[0; 4]);
    assert_eq!(blurred, raster(&mut renderer, commands(3.0)));
}

#[cfg(feature = "lyon")]
#[doc = include_str!("text_run_pixels.doc/lyon_blurs_the_accumulated_backdrop.md")]
#[test]
#[ignore = "needs a GPU adapter"]
fn lyon_blurs_the_accumulated_backdrop() {
    use obscur_geometry::{AffineTransform, CornerRadii, Rect};
    use obscur_scene::clip::Clip;
    use obscur_scene::layer::SceneLayer;
    use obscur_scene::paint::Paint;
    use obscur_scene::path::Path;

    let instance = wgpu::Instance::default();
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect(NO_ADAPTER);
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .expect("open GPU device");
    let mut renderer = VelloRenderer::default().with_lyon_device(device, queue);
    renderer.set_diagnostic_sink(std::sync::Arc::new(NoFallback));

    let rect = |x, y, w, h| Rect::new(Point::new(x, y), Size::new(w, h));
    let fill = |bounds, color| PaintCommand::DrawPath {
        path: Path::rect(bounds),
        paint: Paint::solid_fill(color),
    };
    let red = Color::rgba(1.0, 0.0, 0.0, 1.0);
    let commands = vec![
        fill(rect(0.0, 0.0, 24.0, 64.0), Color::BLACK),
        fill(rect(24.0, 0.0, 16.0, 64.0), Color::WHITE),
        fill(rect(40.0, 0.0, 16.0, 64.0), Color::BLACK),
        PaintCommand::PushClip(Clip::rect(rect(16.0, 0.0, 24.0, 45.0))),
        PaintCommand::PushTransform(AffineTransform::translate(2.0, 0.0)),
        PaintCommand::BackdropFilter {
            bounds: rect(20.0, 10.0, 24.0, 24.0),
            radii: CornerRadii::uniform(8.0),
            blur: 3.0,
        },
        PaintCommand::PopTransform,
        fill(rect(31.0, 20.0, 3.0, 6.0), red),
        PaintCommand::PopClip,
        PaintCommand::PushLayer(SceneLayer::new().with_opacity(0.5)),
        fill(rect(62.0, 0.0, 75.0, 64.0), Color::rgba(1.0, 0.0, 0.0, 0.5)),
        PaintCommand::BackdropFilter {
            bounds: rect(80.0, 12.0, 30.0, 32.0),
            radii: CornerRadii::uniform(0.0),
            blur: 3.0,
        },
        PaintCommand::PopLayer,
    ];
    let pixels = raster(&mut renderer, commands.clone());
    assert_eq!(pixels, raster(&mut renderer, commands));
    let pixel = |x, y| {
        let start = (y * WIDTH as usize + x) * 4;
        &pixels[start..start + 4]
    };
    assert!(
        pixel(23, 22)[0] > 50,
        "blur samples the white stripe on the right"
    );
    assert!(
        pixel(24, 22)[0] < 250,
        "blur samples the black stripe on the left"
    );
    assert_eq!(
        pixel(22, 10),
        [0, 0, 0, 255],
        "rounded corner remains sharp"
    );
    assert_eq!(
        pixel(23, 9),
        [0, 0, 0, 255],
        "outside the panel remains sharp"
    );
    assert!(
        pixel(39, 22)[0] < 250,
        "blur samples across the ancestor clip edge"
    );
    assert_eq!(
        pixel(40, 22),
        [0, 0, 0, 255],
        "the ancestor clips the blurred output"
    );
    assert_eq!(
        pixel(32, 22),
        [255, 0, 0, 255],
        "foreground is drawn after the blur"
    );
    assert_eq!(
        pixel(90, 24),
        [64, 0, 0, 64],
        "a translucent backdrop retains its alpha inside an isolated group"
    );
}

#[cfg(feature = "lyon")]
#[doc = include_str!("text_run_pixels.doc/lyon_renders_inset_shadows_without_fallback.md")]
#[test]
#[ignore = "needs a GPU adapter"]
fn lyon_renders_inset_shadows_without_fallback() {
    use obscur_geometry::{CornerRadii, Rect};

    let instance = wgpu::Instance::default();
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect(NO_ADAPTER);
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .expect("open GPU device");
    let mut classic = VelloRenderer::default()
        .with_device(VelloDevice::new(device.clone(), queue.clone()).expect("classic device"));
    let mut lyon = VelloRenderer::default().with_lyon_device(device, queue);
    lyon.set_diagnostic_sink(std::sync::Arc::new(NoFallback));

    let commands = |blur, spread, offset| {
        vec![PaintCommand::DrawShadow {
            rect: Rect::new(Point::new(12.0, 8.0), Size::new(40.0, 40.0)),
            radii: CornerRadii::uniform(6.0),
            color: Color::BLACK,
            blur,
            spread,
            offset,
            inset: true,
        }]
    };
    let alpha = |pixels: &[u8], x, y| pixels[(y * WIDTH as usize + x) * 4 + 3];
    for (spread, offset, shadow_x, clear_x) in
        [(4.0, (-3.0, 0.0), 49, 25), (-4.0, (10.0, 0.0), 15, 26)]
    {
        let sharp = commands(0.0, spread, offset);
        let actual = raster(&mut lyon, sharp.clone());
        let reference = raster(&mut classic, sharp);
        for pixels in [&actual, &reference] {
            assert_eq!(alpha(pixels, shadow_x, 26), 255);
            assert_eq!(alpha(pixels, clear_x, 26), 0);
            assert_eq!(alpha(pixels, 54, 26), 0);
        }
    }
    let soft = commands(8.0, 4.0, (-3.0, 0.0));
    let actual = raster(&mut lyon, soft.clone());
    let reference = raster(&mut classic, soft);
    for pixels in [&actual, &reference] {
        let edge = alpha(pixels, 44, 26);
        assert!(
            (30..220).contains(&edge),
            "the hole edge is feathered: {edge}"
        );
        assert!(alpha(pixels, 49, 26) > edge);
        assert!(alpha(pixels, 37, 26) < edge);
        assert_eq!(
            alpha(pixels, 12, 8),
            0,
            "the rounded outer corner clips the shadow"
        );
        assert_eq!(alpha(pixels, 54, 26), 0);
    }

    let collapsed = raster(&mut lyon, commands(8.0, 24.0, (0.0, 0.0)));
    assert_eq!(alpha(&collapsed, 32, 28), 255);
    assert_eq!(alpha(&collapsed, 12, 8), 0);
}

#[cfg(feature = "lyon")]
#[path = "text_run_pixels/lyon_synthetic_bold.rs"]
mod lyon_synthetic_bold;

#[cfg(feature = "lyon")]
#[path = "text_run_pixels/lyon_blending.rs"]
mod lyon_blending;

#[cfg(feature = "lyon")]
#[path = "text_run_pixels/lyon_bitmap_glyph.rs"]
mod lyon_bitmap_glyph;

#[cfg(feature = "lyon")]
#[path = "text_run_pixels/lyon_color_glyph.rs"]
mod lyon_color_glyph;

#[path = "text_run_pixels/image_mask.rs"]
mod image_mask;
#[cfg(feature = "lyon")]
#[path = "text_run_pixels/lyon_mesh_upload.rs"]
mod lyon_mesh_upload;
