//! Compare native COLR layers to classic Vello on a real GPU.

use obscur_geometry::Point;
use obscur_scene::color::Color;
use obscur_scene::color::convert::srgb_encode;
use obscur_scene::paint::Brush;
use obscur_scene::text::{GlyphRun, PositionedGlyph};

use super::{NO_ADAPTER, NoFallback, PaintCommand, VelloDevice, VelloRenderer, WIDTH, raster};

fn encoded_byte(linear: f64, alpha: f64) -> u8 {
    (srgb_encode(linear / alpha) * alpha * 255.0).round() as u8
}

#[test]
#[ignore = "needs a GPU adapter"]
fn lyon_draws_colr_without_fallback() {
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

    let commands = |color, oblique| {
        vec![PaintCommand::DrawText {
            origin: Point::new(16.0, 48.0),
            run: GlyphRun::new(
                vec![PositionedGlyph::new(1, 0, 0.0, 0.0, 40.0)],
                40.0,
                "\u{e000}",
            )
            .with_synthesis(oblique, false),
            brush: Brush::solid(color),
        }]
    };
    let pixel = |pixels: &[u8], x: usize, y: usize| -> [u8; 4] {
        let start = (y * WIDTH as usize + x) * 4;
        pixels[start..start + 4].try_into().unwrap()
    };
    let green = Color::rgba(0.0, 1.0, 0.0, 1.0);
    for renderer in [&mut classic, &mut lyon] {
        renderer.register_font(0, include_bytes!("../fixtures/color-glyph.ttf").to_vec());
    }
    let upright = raster(&mut lyon, commands(green, false));
    for (x, expected) in [
        (22, [255, 0, 0, 255]),
        (34, [127, 0, 128, 255]),
        (43, [0, 0, 128, 128]),
        (50, [0, 255, 0, 255]),
        (60, [0; 4]),
    ] {
        assert_eq!(pixel(&upright, x, 34), expected);
    }
    let slanted = raster(&mut lyon, commands(green, true));
    assert_ne!(upright, slanted);
    for (oblique, pixels) in [(false, &upright), (true, &slanted)] {
        let reference = raster(&mut classic, commands(green, oblique));
        for x in [24, 35, 45, 53, 62] {
            let actual = pixel(pixels, x, 34);
            let mut expected = pixel(&reference, x, 34);
            let alpha = u16::from(expected[3]);
            for channel in &mut expected[..3] {
                *channel = ((u16::from(*channel) * alpha + 127) / 255) as u8;
            }
            assert_eq!(actual, expected, "pixel at x={x}, oblique={oblique}");
        }
    }
    let recolored = raster(&mut lyon, commands(Color::WHITE, false));
    assert_eq!(pixel(&recolored, 50, 34), [255; 4]);
    assert_eq!(pixel(&recolored, 34, 34), pixel(&upright, 34, 34));

    lyon.register_font(0, include_bytes!("../fixtures/color-glyph-v1.ttf").to_vec());
    let actual = raster(&mut lyon, commands(green, false));
    for (x, y, expected) in [
        (18, 34, [0; 4]),
        (24, 34, [255, 0, 0, 255]),
        (34, 34, [187, 0, 188, 255]),
        (44, 34, [0, 0, 128, 128]),
        (44, 42, [0; 4]),
        (51, 34, [0, 128, 0, 128]),
        (55, 34, [0; 4]),
    ] {
        assert_eq!(pixel(&actual, x, y), expected, "COLRv1 at ({x}, {y})");
    }
    lyon.register_font(
        0,
        include_bytes!("../fixtures/color-glyph-linear.ttf").to_vec(),
    );
    for oblique in [false, true] {
        let actual = raster(&mut lyon, commands(green, oblique));
        for x in [22, 30, 38] {
            let sample = pixel(&actual, x, 34);
            let shear = if oblique { (48.0 - 34.5) * 0.25 } else { 0.0 };
            let t = (x as f32 + 0.5 - 16.0 - shear) / 24.0;
            let alpha = 1.0 - f64::from(t) * 0.5;
            let expected = [
                encoded_byte(1.0 - f64::from(t), alpha),
                encoded_byte(f64::from(t) * 0.5, alpha),
                0,
                (255.0 * alpha).round() as u8,
            ];
            assert!(
                sample.iter().zip(expected).all(|(a, b)| a.abs_diff(b) <= 2),
                "COLRv1 gradient at x={x}, oblique={oblique}: {sample:?} vs {expected:?}"
            );
        }
    }
    lyon.register_font(
        0,
        include_bytes!("../fixtures/color-glyph-radial-sweep.ttf").to_vec(),
    );
    let pixels = raster(&mut lyon, commands(green, false));
    // Compute the expected samples in font coordinates at pixel centers.
    let check = |x, y, t: f32| {
        let expected = [
            encoded_byte(1.0 - f64::from(t), 1.0),
            encoded_byte(f64::from(t), 1.0),
            0,
            255,
        ];
        let sample = pixel(&pixels, x, y);
        assert!(
            sample.iter().zip(expected).all(|(a, b)| a.abs_diff(b) <= 3),
            "gradient at ({x}, {y}): {sample:?} vs {expected:?}"
        );
    };
    for (x, y) in [(22, 34), (29, 30)] {
        let dx = (x as f32 + 0.5 - 16.0) / 0.04 - 200.0;
        let dy = (48.0 - y as f32 - 0.5) / 0.04 - 350.0;
        let t = (-200.0 * dx + (160_000.0 * dx * dx + 120_000.0 * dy * dy).sqrt()) / 60_000.0;
        check(x, y, t.clamp(0.0, 1.0));
    }
    // The strip runs along x, leaving points more than two pixels above it clear.
    assert_eq!(pixel(&pixels, 43, 28), [0; 4]);
    let strip_x = (38.5 - 16.0) / 0.04 - 325.0;
    let strip_y = (48.0 - 34.5) / 0.04 - 350.0;
    check(
        38,
        34,
        (strip_x + (2500.0_f32 - strip_y * strip_y).sqrt()) / 300.0,
    );
    for (x, y) in [(53, 29), (49, 39)] {
        let dx = x as f32 + 0.5 - 51.0;
        let dy = 34.0 - y as f32 - 0.5;
        check(x, y, (dy.atan2(dx) / std::f32::consts::TAU).rem_euclid(1.0));
    }
}

#[test]
#[ignore = "needs a GPU adapter"]
fn strict_lyon_reports_unsupported_commands() {
    use obscur_geometry::Size;
    use obscur_render::frame::{FrameSubmission, RenderFrame};
    use obscur_scene::clip::{Clip, Mask};
    use obscur_scene::command::DisplayList;
    use obscur_scene::scene::{SceneGeneration, SceneSnapshot};

    let instance = wgpu::Instance::default();
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect(NO_ADAPTER);
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .expect("open GPU device");
    let mut lyon = VelloRenderer::default()
        .with_device(VelloDevice::lyon(device, queue).without_lyon_fallback());
    let snapshot = SceneSnapshot::new(
        DisplayList::from_commands(vec![
            PaintCommand::PushClip(Clip::mask(Mask::alpha())),
            PaintCommand::PopClip,
        ]),
        SceneGeneration::from_raw(1),
    );
    let submission = FrameSubmission::new(RenderFrame::new(snapshot, Size::new(16.0, 16.0)));
    let error = lyon.render_to_image(&submission, 16, 16).unwrap_err();
    assert!(error.to_string().contains("Lyon: mask clips"), "{error}");
    assert!(
        raster(&mut lyon, Vec::new())
            .iter()
            .all(|channel| *channel == 0)
    );
}

#[test]
#[ignore = "needs a GPU adapter"]
fn lyon_composes_colrv1_inside_the_glyph() {
    use obscur_geometry::{Rect, Size};
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
    let mut lyon = VelloRenderer::default()
        .with_device(VelloDevice::lyon(device, queue).without_lyon_fallback());
    lyon.register_font(
        0,
        include_bytes!("../fixtures/color-glyph-composition.ttf").to_vec(),
    );

    let white = [255; 4];
    let red = [255, 0, 0, 255];
    let green = [0, 255, 0, 255];
    let black = [0, 0, 0, 255];
    let yellow = [255, 255, 0, 255];
    let commands = |mode: u32| {
        vec![
            PaintCommand::PushLayer(SceneLayer::new().with_opacity(0.5)),
            PaintCommand::DrawPath {
                path: Path::rect(Rect::new(Point::new(0.0, 0.0), Size::new(80.0, 60.0))),
                paint: Paint::solid_fill(Color::WHITE),
            },
            PaintCommand::DrawText {
                origin: Point::new(16.0, 48.0),
                run: GlyphRun::new(
                    vec![PositionedGlyph::new(5 + mode, 0, 0.0, 0.0, 40.0)],
                    40.0,
                    "x",
                ),
                brush: Brush::solid(Color::rgba(0.0, 1.0, 0.0, 1.0)),
            },
            PaintCommand::PopLayer,
        ]
    };
    let pixel = |pixels: &[u8], x: usize| -> [u8; 4] {
        let at = (34 * WIDTH as usize + x) * 4;
        pixels[at..at + 4].try_into().unwrap()
    };
    for mode in 0..28 {
        let pixels = raster(&mut lyon, commands(mode));
        assert_eq!(pixel(&pixels, 55), [128; 4], "mode {mode} outside glyph");
        let expected: [[u8; 4]; 3] = match mode {
            0 => [white, white, white], // Clear
            1 => [white, green, green], // Source
            2 => [red, red, white],     // Destination
            3 => [red, green, green],   // Source over
            4 => [red, red, green],     // Destination over
            5 => [white, green, white], // Source in
            6 => [white, red, white],   // Destination in
            7 => [white, white, green], // Source out
            8 => [red, white, white],   // Destination out
            9 => [red, green, white],   // Source atop
            10 => [white, red, green],  // Destination atop
            11 => [red, white, green],  // XOR
            12 | 13 | 16 | 21 | 22 => [red, yellow, green],
            14 | 17 | 18 => [red, red, green],
            15 | 23 => [red, black, green],
            19 => [red, green, green],
            20 | 25 => [red, red, green],
            24 | 26 => [red, [0, 189, 0, 255], green],
            27 => [red, [255, 173, 173, 255], green],
            _ => unreachable!(),
        };
        for (x, expected) in [22, 34, 43].into_iter().zip(expected) {
            let sample = pixel(&pixels, x);
            assert!(
                sample
                    .iter()
                    .zip(expected)
                    .all(|(a, b)| a.abs_diff(b.div_ceil(2)) <= 1),
                "mode {mode} at {x}: {sample:?} vs half of {expected:?}",
            );
        }
    }
}
