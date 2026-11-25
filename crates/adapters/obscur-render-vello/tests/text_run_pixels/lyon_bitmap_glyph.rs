//! Check sbix color, alpha, placement, and synthesised slant on a real GPU.

use obscur_geometry::{AffineTransform, Point};
use obscur_scene::color::Color;
use obscur_scene::paint::Brush;
use obscur_scene::text::{GlyphRun, PositionedGlyph};

use super::{NO_ADAPTER, NoFallback, PaintCommand, VelloDevice, VelloRenderer, WIDTH, raster};

#[test]
#[ignore = "needs a GPU adapter"]
fn lyon_draws_bitmap_glyphs_without_fallback() {
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
    for renderer in [&mut classic, &mut lyon] {
        renderer.register_font(0, include_bytes!("../fixtures/bitmap-glyph.ttf").to_vec());
    }

    let commands = |oblique, x| {
        vec![
            PaintCommand::PushTransform(AffineTransform::translate(x, 0.0)),
            PaintCommand::DrawText {
                origin: Point::new(20.0, 40.0),
                run: GlyphRun::new(
                    vec![PositionedGlyph::new(1, 0, 0.0, 0.0, 36.0)],
                    36.0,
                    "\u{e000}",
                )
                .with_synthesis(oblique, false),
                brush: Brush::solid(Color::BLACK),
            },
            PaintCommand::PopTransform,
        ]
    };
    let pixel = |pixels: &[u8], x, y| -> [u8; 4] {
        let start = (y * WIDTH as usize + x) * 4;
        pixels[start..start + 4].try_into().unwrap()
    };
    for oblique in [false, true] {
        let actual = raster(&mut lyon, commands(oblique, 0.0));
        let reference = raster(&mut classic, commands(oblique, 0.0));
        assert_eq!(pixel(&actual, 21, 29), [120, 40, 10, 128]);
        assert_eq!(pixel(&actual, 32, 29), [20, 80, 240, 255]);
        assert_eq!(pixel(&actual, 10, 29), [0; 4]);
        for (x, y) in [(21, 29), (32, 29), (10, 29)] {
            assert_eq!(pixel(&actual, x, y)[3], pixel(&reference, x, y)[3]);
        }
        let shifted = raster(&mut lyon, commands(oblique, 5.0));
        assert_eq!(pixel(&shifted, 26, 29), pixel(&actual, 21, 29));
        assert_eq!(pixel(&shifted, 37, 29), pixel(&actual, 32, 29));
    }
    let upright = raster(&mut lyon, commands(false, 0.0));
    let slanted = raster(&mut lyon, commands(true, 0.0));
    assert_ne!(slanted, upright);
}
