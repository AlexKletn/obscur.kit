//! Pixel comparisons for Lyon layer blending.

use obscur_geometry::{Point, Rect, Size};
use obscur_scene::clip::Clip;
use obscur_scene::color::Color;
use obscur_scene::layer::{CompositionMode, LayerEffect, SceneLayer};
use obscur_scene::paint::{BlendMode, Paint};
use obscur_scene::path::Path;

use super::{NO_ADAPTER, NoFallback, PaintCommand, VelloDevice, VelloRenderer, WIDTH, raster};

fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::new(Point::new(x, y), Size::new(w, h))
}

fn fill(rect: Rect, color: Color) -> PaintCommand {
    PaintCommand::DrawPath {
        path: Path::rect(rect),
        paint: Paint::solid_fill(color),
    }
}

fn pixel(bytes: &[u8], x: usize, y: usize) -> [u8; 4] {
    let index = (y * WIDTH as usize + x) * 4;
    bytes[index..index + 4].try_into().unwrap()
}

#[test]
#[ignore = "needs a GPU adapter"]
fn lyon_layer_blending_matches_vello_without_fallback() {
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

    let blend_modes = [
        BlendMode::Normal,
        BlendMode::Multiply,
        BlendMode::Screen,
        BlendMode::Overlay,
        BlendMode::Darken,
        BlendMode::Lighten,
        BlendMode::ColorDodge,
        BlendMode::ColorBurn,
        BlendMode::HardLight,
        BlendMode::SoftLight,
        BlendMode::Difference,
        BlendMode::Exclusion,
        BlendMode::Hue,
        BlendMode::Saturation,
        BlendMode::Color,
        BlendMode::Luminosity,
    ];
    let compositions = [
        CompositionMode::SrcOver,
        CompositionMode::Src,
        CompositionMode::DstOver,
        CompositionMode::Clear,
    ];
    let background = Color::rgba(0.32, 0.75, 0.1, 0.6);
    let foreground = Color::rgba(0.88, 0.22, 0.63, 0.7);
    for mode in blend_modes {
        for composition in compositions {
            let layer = SceneLayer::new()
                .with_blend_mode(mode)
                .with_composition(composition)
                .with_opacity(0.75)
                .with_clip(Clip::rect(rect(4.0, 4.0, 40.0, 36.0)));
            let commands = vec![
                PaintCommand::PushLayer(SceneLayer::new().with_opacity(0.9)),
                fill(rect(0.0, 0.0, 32.0, 32.0), background),
                PaintCommand::PushLayer(layer),
                fill(rect(16.0, 12.0, 36.0, 15.0), foreground),
                PaintCommand::PopLayer,
                PaintCommand::PopLayer,
            ];
            let actual = raster(&mut lyon, commands.clone());
            let reference = raster(&mut classic, commands);
            for (x, y) in [(2, 16), (8, 16), (23, 16), (38, 16), (48, 16), (23, 31)] {
                let actual = pixel(&actual, x, y);
                if matches!(composition, CompositionMode::Src | CompositionMode::Clear)
                    && !(4..44).contains(&x)
                {
                    // Destructive composition is scoped to the layer clip.
                    let expected = if x < 32 { [44, 103, 13, 138] } else { [0; 4] };
                    assert_eq!(actual, expected, "{mode:?} {composition:?} outside clip");
                    continue;
                }
                let reference = pixel(&reference, x, y);
                assert!(
                    actual
                        .iter()
                        .zip(reference)
                        .all(|(a, b)| a.abs_diff(b) <= 3),
                    "{mode:?} {composition:?} at ({x}, {y}): Lyon {actual:?}, Vello {reference:?}"
                );
            }
        }
    }

    for composition in [CompositionMode::Src, CompositionMode::Clear] {
        let erase = SceneLayer::new()
            .with_opacity(0.0)
            .with_composition(composition);
        let pixels = raster(
            &mut lyon,
            vec![
                fill(rect(0.0, 0.0, 32.0, 32.0), Color::WHITE),
                PaintCommand::PushClip(Clip::rect(rect(8.0, 8.0, 12.0, 12.0))),
                PaintCommand::PushLayer(erase),
                PaintCommand::PopLayer,
                PaintCommand::PopClip,
            ],
        );
        assert_eq!(pixel(&pixels, 0, 0), [255; 4]);
        assert_eq!(pixel(&pixels, 15, 15), [0; 4]);
        assert_eq!(pixel(&pixels, 25, 25), [255; 4]);
    }

    // Exercise the root target and the route that filters the source before mixing.
    let mut blur_layer = SceneLayer::new().with_blend_mode(BlendMode::Multiply);
    blur_layer.add_effect(LayerEffect::blur(3.0));
    let actual = raster(
        &mut lyon,
        vec![
            fill(rect(0.0, 0.0, 50.0, 50.0), Color::WHITE),
            PaintCommand::PushLayer(blur_layer),
            fill(rect(15.0, 15.0, 20.0, 20.0), Color::BLACK),
            PaintCommand::PopLayer,
        ],
    );
    assert_eq!(pixel(&actual, 2, 25), [255; 4]);
    assert!(pixel(&actual, 25, 25)[0] < 10);
    assert!((1..255).contains(&pixel(&actual, 14, 25)[0]));
}

#[test]
#[ignore = "needs a GPU adapter and scheduled Lyon groups with root caching enabled"]
fn lyon_cached_root_preserves_samples_and_invalidates_changed_content() {
    let instance = wgpu::Instance::default();
    let adapter =
        pollster::block_on(instance.request_adapter(&Default::default())).expect(NO_ADAPTER);
    let (device, queue) =
        pollster::block_on(adapter.request_device(&Default::default())).expect("open GPU device");
    let renderer = || {
        VelloRenderer::default()
            .with_device(VelloDevice::lyon(device.clone(), queue.clone()).without_lyon_fallback())
    };
    let mut cached = renderer();
    let mut reference = renderer();
    for (width, alpha) in [(25.25, 0.6), (39.5, 0.6), (32.75, 0.9)] {
        let commands = vec![
            PaintCommand::PushClip(Clip::rect(rect(5.25, 4.25, 38.5, 33.5))),
            PaintCommand::PushClip(Clip::rect(rect(4.25, 4.25, 40.5, 33.5))),
            fill(
                rect(2.5, 3.25, 41.0, 40.0),
                Color::rgba(0.7, 0.2, 0.4, alpha),
            ),
            PaintCommand::PopClip,
            PaintCommand::PushClip(Clip::rect(rect(4.0, 3.75, width, 33.5))),
            PaintCommand::PushLayer(SceneLayer::new().with_opacity(0.65)),
            fill(
                rect(4.25, 4.25, 40.5, 37.5),
                Color::rgba(0.2, 0.4, 0.8, 0.7),
            ),
            PaintCommand::PopLayer,
            PaintCommand::PopClip,
            PaintCommand::PopClip,
        ];
        // An empty first group keeps this reference on the scheduled path
        // while leaving no root prefix to cache.
        let mut baseline = vec![
            PaintCommand::PushLayer(SceneLayer::new().with_opacity(0.5)),
            PaintCommand::PopLayer,
        ];
        baseline.extend(commands.iter().cloned());
        assert_eq!(
            raster(&mut cached, commands),
            raster(&mut reference, baseline)
        );
    }
}
