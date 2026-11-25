//! Verify image-backed mask coverage and isolation across GPU backends.

use obscur_capability::SupportLevel;
use obscur_geometry::{AffineTransform, Point, Rect, Size};
use obscur_render::{Renderer, SceneEffect};
use obscur_scene::clip::{Clip, Mask};
use obscur_scene::color::{AlphaMode, Color};
use obscur_scene::layer::SceneLayer;
use obscur_scene::paint::{Paint, SceneImage};
use obscur_scene::path::Path;

use super::{NO_ADAPTER, PaintCommand, VelloDevice, VelloRenderer, WIDTH, raster};

#[cfg(feature = "lyon")]
#[test]
#[ignore = "needs a GPU adapter"]
fn lyon_masks_clips_and_layers_with_image_coverage() {
    let instance = wgpu::Instance::default();
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect(NO_ADAPTER);
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .expect("open GPU device");
    let mut lyon = VelloRenderer::default()
        .with_device(VelloDevice::lyon(device, queue).without_lyon_fallback());

    assert_image_masks(&mut lyon, 0);
}

#[test]
#[ignore = "needs a GPU adapter"]
fn classic_vello_masks_clips_and_layers_with_image_coverage() {
    let instance = wgpu::Instance::default();
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect(NO_ADAPTER);
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .expect("open GPU device");
    let mut classic =
        VelloRenderer::default().with_device(VelloDevice::new(device, queue).unwrap());
    assert_image_masks(&mut classic, 1);
}

#[cfg(feature = "hybrid")]
#[test]
#[ignore = "needs a GPU adapter"]
fn hybrid_routes_image_mask_frames_to_classic_vello() {
    let instance = wgpu::Instance::default();
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect(NO_ADAPTER);
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .expect("open GPU device");
    let mut hybrid = VelloRenderer::default().with_device(VelloDevice::hybrid(device, queue));
    #[derive(Default)]
    struct Diagnostics(std::sync::Mutex<Vec<String>>);

    impl obscur_diagnostics::DiagnosticSink for Diagnostics {
        fn emit(&self, diagnostic: &obscur_diagnostics::Diagnostic) {
            self.0.lock().unwrap().push(diagnostic.message.clone());
        }
    }

    let diagnostics = std::sync::Arc::new(Diagnostics::default());
    hybrid.set_diagnostic_sink(diagnostics.clone());
    assert_image_masks(&mut hybrid, 1);
    let messages = diagnostics.0.lock().unwrap();
    assert_eq!(
        messages.as_slice(),
        ["degraded effect: image-mask (Vello Hybrid renders image-mask frames with classic Vello)"]
    );
}

fn assert_image_masks(renderer: &mut VelloRenderer, tolerance: u8) {
    assert_eq!(
        renderer.supports_effect(SceneEffect::ImageMask),
        SupportLevel::Limited
    );
    assert_eq!(
        renderer.supports_effect(SceneEffect::Mask),
        SupportLevel::Unavailable
    );

    let rect = |x, y, w, h| Rect::new(Point::new(x, y), Size::new(w, h));
    let fill = |bounds, color| PaintCommand::DrawPath {
        path: Path::rect(bounds),
        paint: Paint::solid_fill(color),
    };
    let bounds = rect(16.0, 8.0, 33.0, 30.0);
    let red = Color::rgba(1.0, 0.0, 0.0, 1.0);
    let alpha = SceneImage::new(
        2,
        1,
        vec![255, 255, 255, 128, 0, 0, 0, 0],
        AlphaMode::Straight,
    )
    .unwrap();
    let pixels = raster(
        renderer,
        vec![
            fill(rect(0.0, 0.0, 75.0, 42.0), Color::BLACK),
            PaintCommand::PushClip(Clip::rect(rect(10.0, 10.0, 42.0, 15.0))),
            PaintCommand::PushTransform(AffineTransform::translate(4.0, 0.0)),
            PaintCommand::PushClip(Clip::image_mask(Mask::alpha(), alpha, bounds)),
            PaintCommand::PopTransform,
            fill(rect(10.0, 10.0, 45.0, 30.0), red),
            fill(rect(18.0, 10.0, 20.0, 30.0), red),
            PaintCommand::PopClip,
            PaintCommand::PopClip,
        ],
    );
    let pixel = |pixels: &[u8], x: usize, y: usize| -> [u8; 4] {
        let start = (y * WIDTH as usize + x) * 4;
        pixels[start..start + 4].try_into().unwrap()
    };
    assert_eq!(pixel(&pixels, 18, 16), [0, 0, 0, 255]);
    assert_eq!(pixel(&pixels, 24, 16), [128, 0, 0, 255]);
    assert_eq!(pixel(&pixels, 49, 16), [0, 0, 0, 255]);
    assert_eq!(pixel(&pixels, 24, 27), [0, 0, 0, 255]);

    let luminance = SceneImage::new(
        2,
        1,
        vec![0, 0, 0, 255, 255, 255, 255, 255],
        AlphaMode::Straight,
    )
    .unwrap();
    let layer = SceneLayer::new()
        .with_opacity(0.5)
        .with_clip(Clip::image_mask(Mask::luminance(), luminance, bounds));
    let pixels = raster(
        renderer,
        vec![
            PaintCommand::PushLayer(layer),
            fill(rect(0.0, 0.0, 75.0, 42.0), red),
            PaintCommand::PopLayer,
        ],
    );
    assert_eq!(pixel(&pixels, 15, 16), [0; 4]);
    assert_eq!(pixel(&pixels, 20, 16), [0; 4]);
    let midpoint = pixel(&pixels, 32, 16);
    assert!(
        midpoint
            .iter()
            .zip([64, 0, 0, 64])
            .all(|(&got, want)| got.abs_diff(want) <= tolerance),
        "{midpoint:?}"
    );
    assert_eq!(pixel(&pixels, 46, 16), [128, 0, 0, 128]);
    assert_eq!(pixel(&pixels, 52, 16), [0; 4]);
}
