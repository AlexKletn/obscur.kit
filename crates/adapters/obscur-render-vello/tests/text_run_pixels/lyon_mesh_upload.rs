//! Compare retained width updates with rendering from fresh buffers.

use obscur_geometry::{Point, Rect, Size};
use obscur_scene::clip::Clip;
use obscur_scene::color::Color;
use obscur_scene::paint::{Brush, GradientStop, LinearGradient, Paint};
use obscur_scene::path::Path;

use super::{NO_ADAPTER, PaintCommand, VelloDevice, VelloRenderer, WIDTH, raster};

#[test]
#[ignore = "needs a GPU adapter"]
fn lyon_reuses_unchanged_mesh_bytes_when_clip_and_shadow_resize() {
    let instance = wgpu::Instance::default();
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect(NO_ADAPTER);
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .expect("open GPU device");
    let renderer = || {
        VelloRenderer::default()
            .with_device(VelloDevice::lyon(device.clone(), queue.clone()).without_lyon_fallback())
    };
    let mut retained = renderer();
    let rect = |x, y, w, h| Rect::new(Point::new(x, y), Size::new(w, h));
    let red = Color::rgba(1.0, 0.0, 0.0, 1.0);
    let commands = |width, shadow| {
        let mut commands = vec![PaintCommand::PushClip(Clip::rect(rect(
            0.0, 0.0, width, 50.0,
        )))];
        for row in 0..12_u16 {
            commands.push(PaintCommand::DrawPath {
                path: Path::rect(rect(0.0, 4.0 * f32::from(row), 124.0, 4.0)),
                paint: Paint::solid_fill(red),
            });
        }
        commands.push(PaintCommand::PopClip);
        commands.push(PaintCommand::DrawPath {
            path: Path::rect(rect(width - shadow, 0.0, shadow, 48.0)),
            paint: Paint::fill(Brush::Linear(LinearGradient::new(
                Point::new(width - shadow, 0.0),
                Point::new(width, 0.0),
                vec![
                    GradientStop::new(0.0, Color::TRANSPARENT),
                    GradientStop::new(1.0, Color::BLACK),
                ],
            ))),
        });
        commands
    };
    for (width, shadow) in [(30.0, 5.0), (96.0, 12.0), (55.0, 8.0), (30.0, 5.0)] {
        let list = commands(width, shadow);
        let actual = raster(&mut retained, list.clone());
        assert_eq!(actual, raster(&mut renderer(), list));
        let alpha_at = |x| actual[(20 * WIDTH as usize + x) * 4 + 3];
        assert_eq!(alpha_at(width as usize - 1), 255);
        assert_eq!(alpha_at(width as usize + 1), 0);
    }
}

#[test]
#[ignore = "needs a GPU adapter"]
fn lyon_partial_redraw_preserves_pixels_between_disjoint_changes() {
    use obscur_geometry::AffineTransform;

    let instance = wgpu::Instance::default();
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect(NO_ADAPTER);
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .expect("open GPU device");
    let renderer = || {
        VelloRenderer::default()
            .with_device(VelloDevice::lyon(device.clone(), queue.clone()).without_lyon_fallback())
    };
    let rect = |x, y, w, h| Rect::new(Point::new(x, y), Size::new(w, h));
    let fill = |rect, color| PaintCommand::DrawPath {
        path: Path::rect(rect),
        paint: Paint::solid_fill(color),
    };
    let commands = |offset| {
        vec![
            fill(rect(0.0, 0.0, 160.0, 64.0), Color::rgba(0.3, 0.4, 0.5, 0.8)),
            PaintCommand::PushTransform(AffineTransform::translate(offset, 0.0)),
            fill(rect(4.0, 4.0, 12.0, 12.0), Color::rgba(1.0, 0.0, 0.0, 0.4)),
            fill(
                rect(132.0, 4.0, 12.0, 12.0),
                Color::rgba(0.0, 1.0, 0.0, 0.4),
            ),
            PaintCommand::PopTransform,
            fill(rect(72.0, 8.0, 16.0, 16.0), Color::WHITE),
        ]
    };
    let mut retained = renderer();
    let mut previous = None;
    for offset in [0.0, 6.25, 1.25, 0.0] {
        let list = commands(offset);
        let pixels = raster(&mut retained, list.clone());
        assert_eq!(pixels, raster(&mut renderer(), list));
        assert_ne!(previous.as_ref(), Some(&pixels));
        previous = Some(pixels);
    }
}
