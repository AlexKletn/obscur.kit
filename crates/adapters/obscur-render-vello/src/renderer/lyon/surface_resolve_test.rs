//! Verify direct resolution through the same view format used on windows.

use obscur_geometry::{Point, Rect, Size};
use obscur_scene::clip::Clip;
use obscur_scene::color::Color;
use obscur_scene::command::{DisplayList, PaintCommand};
use obscur_scene::layer::SceneLayer;
use obscur_scene::paint::{BlendMode, Paint};
use obscur_scene::path::Path;

use crate::renderer::premultiply_blit::PremultiplyBlitter;

use super::*;

const WIDTH: u32 = 64;
const HEIGHT: u32 = 8;
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

#[test]
#[ignore = "needs a GPU adapter"]
fn bgra_surface_resolve_matches_blit_with_translucent_group_and_clip() {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        required_features: crate::renderer::VelloDevice::diagnostic_features(&adapter),
        ..Default::default()
    }))
    .unwrap();
    let surface = || {
        VelloFrame::from_texture(
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some("obscur-test-surface"),
                size: size(),
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: FORMAT.add_srgb_suffix(),
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[FORMAT],
            }),
            FORMAT,
        )
    };

    let fill = |left, width, color| PaintCommand::DrawPath {
        path: Path::rect(Rect::new(Point::new(left, 0.0), Size::new(width, 8.0))),
        paint: Paint::solid_fill(color),
    };
    let list = Arc::new(DisplayList::from_commands(vec![
        PaintCommand::PushClip(Clip::rect(Rect::new(
            Point::new(2.25, 0.0),
            Size::new(43.0, 8.0),
        ))),
        PaintCommand::PushLayer(SceneLayer::new().with_opacity(0.5)),
        fill(0.0, 32.0, Color::rgba(1.0, 0.0, 0.25, 0.75)),
        fill(16.0, 48.0, Color::rgba(0.0, 0.25, 1.0, 1.0)),
        PaintCommand::PopLayer,
        PaintCommand::PopClip,
        fill(52.0, 8.0, Color::rgba(0.0, 1.0, 0.0, 1.0)),
    ]));
    let fonts = SceneConverter::default();
    let mut lyon = LyonRenderer::default();
    let direct = surface();
    let reference = surface();
    let offscreen = VelloFrame::new_offscreen(&device, WIDTH, HEIGHT);
    let render = |lyon: &mut LyonRenderer, target| {
        lyon.render(&device, &queue, target, &list, &fonts).unwrap();
    };
    render(&mut lyon, &offscreen);
    render(&mut lyon, &direct);

    let mut encoder = device.create_command_encoder(&Default::default());
    PremultiplyBlitter::with_alpha_conversion(&device, FORMAT, false).copy(
        &device,
        &mut encoder,
        offscreen.view(),
        reference.view(),
    );
    queue.submit([encoder.finish()]);
    let expected = read(&device, &queue, &reference);
    assert_eq!(&expected[..4], &[0; 4]);
    assert!((90..=100).contains(&expected[3 + 4 * 10]));
    assert_eq!(&expected[4 * 56..4 * 57], &[0, 255, 0, 255]);
    assert_eq!(read(&device, &queue, &direct), expected);
    render(&mut lyon, &offscreen);
    render(&mut lyon, &direct);
    assert_eq!(read(&device, &queue, &direct), expected);
}

fn size() -> wgpu::Extent3d {
    wgpu::Extent3d {
        width: WIDTH,
        height: HEIGHT,
        depth_or_array_layers: 1,
    }
}

fn read(device: &wgpu::Device, queue: &wgpu::Queue, frame: &VelloFrame) -> Vec<u8> {
    let (width, height) = frame.size();
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("obscur-test-surface-readback"),
        size: u64::from(width) * u64::from(height) * 4,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        frame.texture().as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: None,
            },
        },
        frame.texture().size(),
    );
    queue.submit([encoder.finish()]);
    buffer.map_async(wgpu::MapMode::Read, .., |result| result.unwrap());
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    let bytes = buffer.get_mapped_range(..).to_vec();
    buffer.unmap();
    bytes
}

#[test]
#[ignore = "needs a GPU adapter"]
fn cropped_opacity_group_matches_full_size_with_a_fractional_clip() {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        required_features: crate::renderer::VelloDevice::diagnostic_features(&adapter),
        ..Default::default()
    }))
    .unwrap();
    let size = (1024, 768);
    let frame = VelloFrame::new_offscreen(&device, size.0, size.1);
    let fonts = SceneConverter::default();
    let rectangle = |x, y, w, h| Rect::new(Point::new(x, y), Size::new(w, h));
    let paint = |rect, color| PaintCommand::DrawPath {
        path: Path::rect(rect),
        paint: Paint::solid_fill(color),
    };
    for width in [163.5, 923.5] {
        let list = Arc::new(DisplayList::from_commands(vec![
            paint(
                rectangle(0.0, 0.0, 1024.0, 768.0),
                Color::rgba(0.0, 0.0, 0.25, 1.0),
            ),
            PaintCommand::PushLayer(SceneLayer::new().with_opacity(0.47)),
            PaintCommand::PushClip(Clip::rect(rectangle(67.25, 35.75, width, 690.0))),
            paint(
                rectangle(59.5, 21.5, width, 702.5),
                Color::rgba(1.0, 0.0, 0.0, 0.85),
            ),
            paint(
                rectangle(97.0, 81.75, width - 52.0, 101.0),
                Color::rgba(0.0, 1.0, 0.0, 0.75),
            ),
            PaintCommand::PopClip,
            PaintCommand::PopLayer,
            paint(rectangle(2.0, 2.0, 7.0, 7.0), Color::WHITE),
        ]));
        let mut lyon = LyonRenderer::default();
        let render = |lyon: &mut LyonRenderer, crop| {
            lyon.render_with_group_options(&device, &queue, &frame, &list, &fonts, crop, false)
                .unwrap();
        };
        render(&mut lyon, false);
        let expected = read(&device, &queue, &frame);
        let candidate = GroupCrop::for_draws(&lyon.mesh.draws, size);
        assert_eq!(candidate.is_some(), width < 200.0);
        render(&mut lyon, true);
        let actual = read(&device, &queue, &frame);
        assert!(
            actual == expected,
            "cropped output differs from full-size output"
        );
        if let Some(crop) = candidate {
            assert_eq!(crop.extent, (192, 736));
            if std::env::var_os("OBSCUR_GPU_GROUP_STATS").as_deref() == Some("1".as_ref()) {
                for crop in [false, true, true, false] {
                    println!("[crop-probe] enabled={crop}");
                    for _ in 0..12 {
                        render(&mut lyon, crop);
                        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
                    }
                }
            }
        }
    }
}

#[test]
#[ignore = "needs a GPU adapter"]
fn scheduled_groups_preserve_nested_clips_and_paint_order() {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        required_features: crate::renderer::VelloDevice::diagnostic_features(&adapter),
        ..Default::default()
    }))
    .unwrap();
    let frame = VelloFrame::new_offscreen(&device, WIDTH, HEIGHT);
    let fonts = SceneConverter::default();
    let rectangle = |left, width| Rect::new(Point::new(left, 0.0), Size::new(width, 8.0));
    let fill = |left, width, color| PaintCommand::DrawPath {
        path: Path::rect(rectangle(left, width)),
        paint: Paint::solid_fill(color),
    };
    let layer = |opacity| PaintCommand::PushLayer(SceneLayer::new().with_opacity(opacity));
    let list = Arc::new(DisplayList::from_commands(vec![
        fill(0.0, 64.0, Color::BLACK),
        PaintCommand::PushClip(Clip::rect(rectangle(5.25, 48.5))),
        layer(0.7),
        fill(0.0, 60.0, Color::rgba(1.0, 0.0, 0.0, 0.75)),
        PaintCommand::PushLayer(
            SceneLayer::new()
                .with_opacity(0.6)
                .with_clip(Clip::rect(rectangle(18.25, 25.5))),
        ),
        fill(10.0, 40.0, Color::WHITE),
        fill(22.0, 32.0, Color::rgba(0.0, 1.0, 0.0, 1.0)),
        PaintCommand::PopLayer,
        fill(47.0, 10.0, Color::WHITE),
        PaintCommand::PopLayer,
        PaintCommand::PopClip,
        fill(0.0, 22.0, Color::rgba(0.0, 0.0, 1.0, 0.5)),
        layer(0.8),
        fill(14.0, 38.0, Color::rgba(0.0, 0.0, 1.0, 0.6)),
        fill(34.0, 22.0, Color::rgba(0.0, 1.0, 0.0, 0.4)),
        PaintCommand::PopLayer,
        fill(57.0, 7.0, Color::WHITE),
    ]));
    let mut lyon = LyonRenderer::default();
    let render = |lyon: &mut LyonRenderer, schedule| {
        lyon.render_with_group_options(&device, &queue, &frame, &list, &fonts, false, schedule)
            .unwrap();
        read(&device, &queue, &frame)
    };
    let reference = render(&mut lyon, false);
    let schedule =
        GroupSchedule::for_draws(&lyon.mesh.draws, frame.size(), lyon.group_clip_options)
            .expect("eligible scene");
    assert_eq!(schedule.roots, [0, 2]);
    assert_eq!(schedule.groups[0].children, [1]);
    assert_eq!(schedule.groups[1].popped_clips.len(), 1);
    assert!(
        render(&mut lyon, true) == reference,
        "scheduled pixels differ"
    );
    assert!(
        render(&mut lyon, true) == reference,
        "cached targets changed pixels"
    );

    let mut commands = list.commands().to_vec();
    let sibling = commands
        .iter()
        .rposition(|command| matches!(command, PaintCommand::PushLayer(_)))
        .unwrap();
    let child = commands
        .iter()
        .enumerate()
        .filter(|(_, command)| matches!(command, PaintCommand::PushLayer(_)))
        .nth(1)
        .unwrap()
        .0;
    let clipped_child = PaintCommand::PushLayer(
        SceneLayer::new()
            .with_opacity(0.35)
            .with_clip(Clip::rect(rectangle(18.25, 25.5))),
    );
    for (index, command) in [
        (sibling, layer(0.45)),
        (sibling + 1, fill(14.0, 38.0, Color::WHITE)),
        (child, clipped_child),
        (child + 1, fill(10.0, 40.0, Color::BLACK)),
    ] {
        commands[index] = command;
        let next = Arc::new(DisplayList::from_commands(commands.clone()));
        let mut baseline = LyonRenderer::default();
        baseline
            .render_with_group_options(&device, &queue, &frame, &next, &fonts, false, false)
            .unwrap();
        let expected = read(&device, &queue, &frame);
        assert!(expected != reference);
        lyon.render_with_group_options(&device, &queue, &frame, &next, &fonts, false, true)
            .unwrap();
        assert!(read(&device, &queue, &frame) == expected);
    }
}

#[test]
#[ignore = "needs a GPU adapter"]
fn translated_group_cache_matches_rasterization_in_both_directions() {
    use obscur_geometry::AffineTransform;

    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        required_features: crate::renderer::VelloDevice::diagnostic_features(&adapter),
        ..Default::default()
    }))
    .unwrap();
    let frame = VelloFrame::new_offscreen(&device, 192, 80);
    let fonts = SceneConverter::default();
    let rect = |x, y, w, h| Rect::new(Point::new(x, y), Size::new(w, h));
    let fill = |rect, color| PaintCommand::DrawPath {
        path: Path::rect(rect),
        paint: Paint::solid_fill(color),
    };
    let layer = || PaintCommand::PushLayer(SceneLayer::new().with_opacity(0.65));
    let mut retained = LyonRenderer::default();
    let mut reference = LyonRenderer::default();
    for (x, y) in [
        (6.0, 7.0),
        (18.0, 9.0),
        (-4.0, -3.0),
        (-16.0, -8.0),
        (6.5, 7.0),
        (7.0, 7.0),
    ] {
        let list = Arc::new(DisplayList::from_commands(vec![
            fill(rect(0.0, 0.0, 192.0, 80.0), Color::BLACK),
            PaintCommand::PushTransform(AffineTransform::translate(x, y)),
            layer(),
            PaintCommand::PushClip(Clip::rect(rect(12.25, 9.25, 26.5, 30.5))),
            fill(rect(9.5, 9.5, 22.0, 20.25), Color::rgba(0.5, 0.2, 0.0, 0.7)),
            fill(rect(22.75, 18.5, 17.25, 18.0), Color::WHITE),
            PaintCommand::PopClip,
            PaintCommand::PopLayer,
            PaintCommand::PopTransform,
            layer(),
            fill(
                rect(73.0, 10.0, 30.0, 24.0),
                Color::rgba(0.0, 0.0, 1.0, 0.6),
            ),
            fill(rect(82.0, 18.0, 18.0, 21.0), Color::WHITE),
            PaintCommand::PopLayer,
        ]));
        reference
            .render_with_group_options(&device, &queue, &frame, &list, &fonts, false, false)
            .unwrap();
        let expected = read(&device, &queue, &frame);
        retained
            .render_with_group_options(&device, &queue, &frame, &list, &fonts, false, true)
            .unwrap();
        assert!(
            read(&device, &queue, &frame) == expected,
            "pixels differ at ({x}, {y})"
        );
    }
}

#[test]
#[ignore = "needs a GPU adapter"]
fn retained_group_applies_changed_ancestor_clip_at_composition() {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        required_features: crate::renderer::VelloDevice::diagnostic_features(&adapter),
        ..Default::default()
    }))
    .unwrap();
    let frame = VelloFrame::new_offscreen(&device, 192, 80);
    let fonts = SceneConverter::default();
    let rect = |x, y, w, h| Rect::new(Point::new(x, y), Size::new(w, h));
    let fill = |rect, color| PaintCommand::DrawPath {
        path: Path::rect(rect),
        paint: Paint::solid_fill(color),
    };
    let layer = || PaintCommand::PushLayer(SceneLayer::new().with_opacity(0.65));
    let mut retained = LyonRenderer::default();
    let mut reference = LyonRenderer::default();
    let mut previous = None;
    for width in [26.25, 45.5, 19.75, 26.25] {
        let list = Arc::new(DisplayList::from_commands(vec![
            fill(rect(0.0, 0.0, 192.0, 80.0), Color::BLACK),
            PaintCommand::PushClip(Clip::rect(rect(4.25, 3.75, 72.0, 58.0))),
            PaintCommand::PushClip(Clip::rect(rect(12.25, 7.5, width, 50.25))),
            layer(),
            PaintCommand::PushClip(Clip::rect(rect(6.75, 8.25, 64.0, 46.5))),
            fill(
                rect(9.5, 9.5, 60.25, 44.25),
                Color::rgba(0.5, 0.2, 0.0, 0.7),
            ),
            fill(rect(22.75, 18.5, 32.25, 18.0), Color::WHITE),
            PaintCommand::PopClip,
            PaintCommand::PopLayer,
            PaintCommand::PopClip,
            fill(rect(9.5, 41.0, 64.0, 5.0), Color::rgba(0.0, 0.0, 1.0, 0.6)),
            PaintCommand::PopClip,
            layer(),
            fill(
                rect(73.0, 10.0, 30.0, 24.0),
                Color::rgba(0.0, 0.0, 1.0, 0.6),
            ),
            fill(rect(82.0, 18.0, 18.0, 21.0), Color::WHITE),
            PaintCommand::PopLayer,
        ]));
        reference
            .render_with_group_options(&device, &queue, &frame, &list, &fonts, false, false)
            .unwrap();
        let expected = read(&device, &queue, &frame);
        assert_ne!(previous.as_ref(), Some(&expected));
        retained
            .render_with_group_options(&device, &queue, &frame, &list, &fonts, false, true)
            .unwrap();
        assert!(
            read(&device, &queue, &frame) == expected,
            "pixels differ at width {width}"
        );
        previous = Some(expected);
    }
}

#[test]
#[ignore = "needs a GPU adapter and OBSCUR_LYON_CACHE_OUTER_CLIPS=1"]
fn scheduled_scissor_preserves_fractional_and_empty_clips() {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
    let frame = VelloFrame::new_offscreen(&device, WIDTH, HEIGHT);
    let fonts = SceneConverter::default();
    let rectangle = |left, width| Rect::new(Point::new(left, 0.0), Size::new(width, 8.0));
    let fill = |left, width| PaintCommand::DrawPath {
        path: Path::rect(rectangle(left, width)),
        paint: Paint::solid_fill(Color::WHITE),
    };
    let layer = || PaintCommand::PushLayer(SceneLayer::new().with_opacity(0.7));
    let mut renderer = LyonRenderer {
        group_clip_options: super::GroupClipOptions {
            defer_outer_clip: true,
            scissor: true,
        },
        ..LyonRenderer::default()
    };

    for left in [5.25, 56.0] {
        let list = Arc::new(DisplayList::from_commands(vec![
            PaintCommand::PushClip(Clip::rect(rectangle(left, 7.5))),
            layer(),
            fill(0.0, 24.0),
            fill(7.0, 16.0),
            PaintCommand::PopLayer,
            PaintCommand::PopClip,
            layer(),
            fill(30.0, 7.0),
            fill(33.0, 8.0),
            PaintCommand::PopLayer,
        ]));
        let render = |renderer: &mut LyonRenderer, scheduled| {
            renderer
                .render_with_group_options(&device, &queue, &frame, &list, &fonts, false, scheduled)
                .unwrap();
            read(&device, &queue, &frame)
        };
        let reference = render(&mut renderer, false);
        let pixels = render(&mut renderer, true);
        let schedule = GroupSchedule::for_draws(
            &renderer.mesh.draws,
            frame.size(),
            renderer.group_clip_options,
        )
        .unwrap();
        assert!(schedule.groups[0].composite_clip().is_some());
        let scissors = schedule.clip_scissors(&renderer.mesh, frame.size());
        assert!(scissors[0][2] < schedule.groups[0].bounds[2]);
        assert_eq!(scissors[0][2] == 0, left == 56.0);
        assert_eq!(scissors[1], schedule.groups[1].bounds);
        assert_eq!(pixels, reference);
    }
}

#[test]
#[ignore = "needs a GPU adapter"]
fn isolation_keeps_multiply_off_the_outer_backdrop() {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
    eprintln!("isolation adapter: {:?}", adapter.get_info());
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        required_features: crate::renderer::VelloDevice::diagnostic_features(&adapter),
        ..Default::default()
    }))
    .unwrap();
    let frame = VelloFrame::new_offscreen(&device, WIDTH, HEIGHT);
    let fonts = SceneConverter::default();
    let mut renderer = LyonRenderer::default();
    let fill = |color| PaintCommand::DrawPath {
        path: Path::rect(Rect::new(
            Point::ORIGIN,
            Size::new(WIDTH as f32, HEIGHT as f32),
        )),
        paint: Paint::solid_fill(color),
    };

    for (isolated, expected) in [
        (false, [0, 0, 0, 255]),
        (true, [0, 0, 255, 255]),
        (false, [0, 0, 0, 255]),
    ] {
        let list = Arc::new(DisplayList::from_commands(vec![
            fill(Color::rgba(1.0, 0.0, 0.0, 1.0)),
            PaintCommand::PushLayer(SceneLayer::new().with_isolation(isolated)),
            PaintCommand::PushLayer(SceneLayer::new().with_blend_mode(BlendMode::Multiply)),
            fill(Color::rgba(0.0, 0.0, 1.0, 1.0)),
            PaintCommand::PopLayer,
            PaintCommand::PopLayer,
        ]));
        renderer
            .render(&device, &queue, &frame, &list, &fonts)
            .unwrap();
        let pixels = read(&device, &queue, &frame);
        let center = ((HEIGHT / 2 * WIDTH + WIDTH / 2) * 4) as usize;
        assert_eq!(pixels[center..center + 4], expected, "isolated={isolated}");
    }
}
