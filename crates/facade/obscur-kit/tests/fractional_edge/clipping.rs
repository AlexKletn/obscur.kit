use super::*;

#[test]
fn polygon_clip_restricts_paint_and_hit_testing_of_transformed_children() {
    let el = |id, class: &str| {
        ViewNode::element(ComponentId::from_raw(id))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
    };
    let view =
        View::single(el(1, "parent").with_children(ViewFragment::from_nodes(vec![el(2, "child")])));
    let size = Size::new(24.0, 24.0);
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), size);
    let sheet = compile(
        ".parent { width: 20px; height: 20px; transform: translateX(2px); \
                    clip-path: polygon(0 0, 100% 0, 0 100%) } \
         .child { width: 20px; height: 20px; background-color: red }",
    );
    let _ = runtime.render_now(&view, &[sheet]);
    let root = runtime.tree().root().unwrap();
    let parent = runtime.tree().get(root).unwrap().children()[0];
    let child = runtime.tree().get(parent).unwrap().children()[0];
    let inside = runtime.hit_test(Point::new(4.0, 2.0));
    let outside = runtime.hit_test(Point::new(18.0, 14.0));
    assert_eq!(inside.target().unwrap().node(), child);
    assert!(!outside.nodes().contains(&child));
    assert!(!outside.nodes().contains(&parent));

    let submission = FrameSubmission::new(RenderFrame::new(runtime.snapshot(), size));
    let mut cpu = CpuRenderer::new(RendererId::from_raw(2));
    cpu.render(&submission, &RenderTarget::bitmap(BitmapTarget::new(size)))
        .unwrap();
    let pixels = cpu.pixmap().unwrap();
    assert_eq!(pixels.pixel(4, 2).unwrap().red(), 255);
    assert_eq!(pixels.pixel(18, 14).unwrap().alpha(), 0);
}

#[test]
fn clip_path_restricts_the_outline() {
    let size = Size::new(24.0, 24.0);
    let submission = scene(
        ".dot { width: 20px; height: 20px; outline: 2px solid red; outline-offset: -2px; clip-path: inset(0 50% 0 0) }",
        24.0,
        1.0,
    );
    let mut cpu = CpuRenderer::new(RendererId::from_raw(2));
    cpu.render(&submission, &RenderTarget::bitmap(BitmapTarget::new(size)))
        .unwrap();
    let pixels = cpu.pixmap().unwrap();
    assert_eq!(pixels.pixel(4, 0).unwrap().red(), 255);
    assert_eq!(pixels.pixel(15, 0).unwrap().alpha(), 0);
}

#[test]
fn clip_path_applies_after_filter_blur() {
    let size = Size::new(24.0, 24.0);
    let submission = scene(
        concat!(
            ".dot { width: 20px; height: 20px; background: red; ",
            "filter: blur(3px); clip-path: inset(0 50% 0 0) }",
        ),
        24.0,
        1.0,
    );
    let mut cpu = CpuRenderer::new(RendererId::from_raw(2));
    cpu.render(&submission, &RenderTarget::bitmap(BitmapTarget::new(size)))
        .unwrap();
    let pixels = cpu.pixmap().unwrap();
    assert!(pixels.pixel(9, 10).unwrap().alpha() > 245);
    assert_eq!(pixels.pixel(10, 10).unwrap().alpha(), 0);
}

#[test]
#[ignore = "needs a GPU adapter"]
fn layer_transform_moves_content_and_clip_and_restores_the_parent_transform() {
    use obscur_kit::geometry::{AffineTransform, Rect};
    use obscur_kit::scene::clip::Clip;
    use obscur_kit::scene::command::{DisplayList, PaintCommand};
    use obscur_kit::scene::layer::SceneLayer;
    use obscur_kit::scene::scene::{SceneGeneration, SceneSnapshot};

    let size = Size::new(24.0, 24.0);
    let plain = scene(
        ".dot { width: 8px; height: 8px; background: red }",
        24.0,
        1.0,
    );
    let layer = SceneLayer::new()
        .with_transform(AffineTransform::translate(8.0, 0.0))
        .with_clip(Clip::rect(Rect::new(Point::ORIGIN, Size::new(4.0, 8.0))));
    let draw = plain
        .frame()
        .scene()
        .display_list()
        .commands()
        .iter()
        .find(|command| matches!(command, PaintCommand::DrawPath { .. }))
        .unwrap();
    let commands = vec![
        PaintCommand::PushTransform(AffineTransform::translate(0.0, 4.0)),
        PaintCommand::PushLayer(layer),
        draw.clone(),
        PaintCommand::PopLayer,
        draw.clone(),
        PaintCommand::PopTransform,
    ];
    let snapshot = SceneSnapshot::new(
        DisplayList::from_commands(commands),
        SceneGeneration::from_raw(1),
    );
    let submission = FrameSubmission::new(RenderFrame::new(snapshot, size));
    let mut cpu = CpuRenderer::new(RendererId::from_raw(2));
    cpu.render(&submission, &RenderTarget::bitmap(BitmapTarget::new(size)))
        .unwrap();
    let (device, queue) = headless_gpu();
    let mut gpu = VelloRenderer::new(RendererId::from_raw(3))
        .with_device(VelloDevice::new(device, queue).unwrap());
    let (_, _, pixels) = gpu.render_to_image(&submission, 24, 24).unwrap();
    for buffer in [cpu.pixmap().unwrap().data(), &pixels] {
        for (x, y, alpha) in [(2, 6, 255), (10, 6, 255), (14, 6, 0), (10, 2, 0)] {
            assert_eq!(buffer[(y * 24 + x) * 4 + 3], alpha, "pixel ({x}, {y})");
        }
    }
}

#[test]
#[ignore = "needs a GPU adapter"]
fn vello_preserves_clip_when_the_nested_filter_falls_back() {
    const EXTENT: u32 = 24;
    let submission = scene(
        concat!(
            ".dot { width: 20px; height: 20px; background: red; ",
            "filter: blur(3px); clip-path: inset(0 50% 0 0) }",
        ),
        24.0,
        1.0,
    );
    let (device, queue) = headless_gpu();
    let device = VelloDevice::new(device, queue).expect("Vello device");
    let mut gpu = VelloRenderer::new(RendererId::from_raw(3)).with_device(device);
    let (_, _, pixels) = gpu
        .render_to_image(&submission, EXTENT, EXTENT)
        .expect("GPU render");
    let pixel = |x| (10 * EXTENT as usize + x) * 4;
    assert_eq!(&pixels[pixel(9)..pixel(9) + 4], &[255, 0, 0, 255]);
    assert_eq!(&pixels[pixel(10)..pixel(10) + 4], &[0; 4]);
}

#[test]
#[ignore = "needs a GPU adapter"]
fn polygon_clip_winding_agrees_between_cpu_and_vello() {
    let (device, queue) = headless_gpu();
    let mut gpu = VelloRenderer::new(RendererId::from_raw(3))
        .with_device(VelloDevice::new(device, queue).unwrap());
    let size = Size::new(16.0, 16.0);
    let mut cpu = CpuRenderer::new(RendererId::from_raw(2));
    for (rule, alpha) in [("nonzero", 255), ("evenodd", 0)] {
        let css = format!(
            ".dot {{ width: 16px; height: 16px; background: red; \
                  clip-path: polygon({rule}, 0 0, 100% 0, 0 100%, 0 0, 100% 0, 0 100%) }}"
        );
        let submission = scene(&css, size.width, 1.0);
        cpu.render(&submission, &RenderTarget::bitmap(BitmapTarget::new(size)))
            .unwrap();
        let (_, _, pixels) = gpu.render_to_image(&submission, 16, 16).unwrap();
        assert_eq!(cpu.pixmap().unwrap().pixel(4, 4).unwrap().alpha(), alpha);
        assert_eq!(pixels[(4 * 16 + 4) * 4 + 3], alpha);
    }
}

#[test]
fn clip_path_reference_boxes_agree_in_paint_and_hit_testing() {
    let size = Size::new(20.0, 20.0);
    let view = dot_view();
    for (value, visible, hidden) in [
        ("padding-box", (3, 3), (1, 1)),
        ("content-box", (7, 7), (15, 15)),
        ("fill-box", (7, 7), (3, 3)),
        ("content-box polygon(0 0, 100% 0, 0 100%)", (7, 7), (15, 15)),
        ("polygon(0 0, 100% 0, 0 100%) fill-box", (7, 7), (3, 3)),
        ("stroke-box inset(10%)", (3, 3), (1, 1)),
        ("inset(10%) view-box", (3, 3), (1, 1)),
    ] {
        let sheet = compile(&format!(
            ".dot {{ box-sizing: border-box; width: 20px; height: 20px; \
                 border: 2px solid transparent; padding: 4px; background: red; \
                 clip-path: {value} }}"
        ));
        let mut rt = UiRuntime::new(RuntimeId::from_raw(1), size);
        let _ = rt.render_now(&view, &[sheet]);
        let root = rt.tree().root().unwrap();
        let node = rt.tree().get(root).unwrap().children()[0];
        let frame = FrameSubmission::new(RenderFrame::new(rt.snapshot(), size));
        let mut cpu = CpuRenderer::new(RendererId::from_raw(2));
        cpu.render(&frame, &RenderTarget::bitmap(BitmapTarget::new(size)))
            .unwrap();
        let pixels = cpu.pixmap().unwrap();
        for ((x, y), inside) in [(hidden, false), (visible, true)] {
            let hit = rt.hit_test(Point::new(x as f32 + 0.5, y as f32 + 0.5));
            assert_eq!(hit.nodes().contains(&node), inside, "{value} at ({x}, {y})");
            assert_eq!(
                pixels.pixel(x, y).unwrap().alpha(),
                if inside { 255 } else { 0 },
                "{value} at ({x}, {y})"
            );
        }
    }
}

#[test]
fn basic_clip_shapes_agree_in_paint_and_hit_testing() {
    let size = Size::new(24.0, 16.0);
    let view = dot_view();
    for (shape, inside, outside) in [
        ("inset(25% 4px round 6px / 3px)", (12, 5), (4, 4)),
        ("inset(-4px 4px 25% 25%)", (7, 1), (5, 1)),
        ("var(--clip-shape)", (12, 5), (4, 4)),
        (
            "inset(var(--clip-edge) 4px round 6px / 3px)",
            (12, 5),
            (4, 4),
        ),
        (
            "inset(var(--missing-edge, 25%) 4px round 6px / 3px)",
            (12, 5),
            (4, 4),
        ),
        ("rect(4px auto 12px 25% round 4px)", (22, 8), (6, 4)),
        ("xywh(25% 25% 50% 50% round 6px / 4px)", (10, 6), (6, 4)),
        ("circle(25% at 8px 8px)", (12, 7), (14, 7)),
        ("ellipse(25% 50% at center)", (12, 1), (5, 8)),
        (
            "ellipse(closest-side farthest-side at 6px 4px)",
            (6, 14),
            (11, 14),
        ),
        ("circle(at 5px 8px)", (1, 8), (11, 8)),
        ("inset(calc(25% + 2px) 0 0)", (12, 7), (12, 4)),
        ("rect(0 min(50%, 16px) auto 0)", (10, 6), (13, 6)),
        ("xywh(0 0 clamp(2px, 50%, 16px) 100%)", (10, 6), (13, 6)),
        ("inset(4px round max(4px, 20%))", (10, 5), (4, 4)),
        ("circle(calc(25% + 2px))", (18, 7), (20, 7)),
        (
            "circle(3px at calc(50% + 4px) min(50%, 6px))",
            (16, 6),
            (12, 6),
        ),
        (
            "ellipse(3px 2px at right calc(25% + 2px) bottom max(4px, 50%))",
            (16, 8),
            (20, 8),
        ),
        ("polygon(0 0, calc(50% + 2px) 0, 0 100%)", (9, 2), (14, 2)),
    ] {
        let sheet = compile(&format!(
            ".dot {{ width: 24px; height: 16px; background: red; \
                --clip-shape: inset(25% 4px round 6px / 3px); --clip-edge: 25%; \
                clip-path: {shape} }}"
        ));
        let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), size);
        runtime.set_custom_value_parser(std::rc::Rc::new(
            obscur_kit::css::compiler::CustomValueCache::new(),
        ));
        let _ = runtime.render_now(&view, &[sheet]);
        let root = runtime.tree().root().unwrap();
        let node = runtime.tree().get(root).unwrap().children()[0];
        let frame = FrameSubmission::new(RenderFrame::new(runtime.snapshot(), size));
        let mut cpu = CpuRenderer::new(RendererId::from_raw(2));
        cpu.render(&frame, &RenderTarget::bitmap(BitmapTarget::new(size)))
            .unwrap();
        let pixels = cpu.pixmap().unwrap();
        for ((x, y), visible) in [(inside, true), (outside, false)] {
            let hit = runtime.hit_test(Point::new(x as f32 + 0.5, y as f32 + 0.5));
            assert_eq!(
                hit.nodes().contains(&node),
                visible,
                "{shape} at ({x}, {y})"
            );
            assert_eq!(
                pixels.pixel(x, y).unwrap().alpha(),
                if visible { 255 } else { 0 },
                "{shape} at ({x}, {y})"
            );
        }
    }
}

#[test]
fn overconstrained_insets_do_not_paint_or_receive_hits() {
    let size = Size::new(20.0, 20.0);
    let view = dot_view();
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), size);
    let sheet = compile(
        ".dot { width: 20px; height: 20px; background: red; clip-path: inset(75% 0 50% 0) }",
    );
    let _ = runtime.render_now(&view, &[sheet]);
    let root = runtime.tree().root().unwrap();
    let node = runtime.tree().get(root).unwrap().children()[0];
    assert!(
        !runtime
            .hit_test(Point::new(10.0, 12.0))
            .nodes()
            .contains(&node)
    );
    let frame = FrameSubmission::new(RenderFrame::new(runtime.snapshot(), size));
    let mut cpu = CpuRenderer::new(RendererId::from_raw(2));
    cpu.render(&frame, &RenderTarget::bitmap(BitmapTarget::new(size)))
        .unwrap();
    assert!(
        cpu.pixmap()
            .unwrap()
            .data()
            .iter()
            .all(|&channel| channel == 0)
    );
}

#[test]
fn svg_clip_path_translates_and_preserves_fill_winding() {
    let size = Size::new(24.0, 24.0);
    let view = dot_view();
    for (rule, center_visible) in [("nonzero", true), ("evenodd", false)] {
        let sheet = compile(&format!(
            ".dot {{ box-sizing: border-box; width: 24px; height: 24px; padding: 4px; \
                background: red; clip-path: path({rule}, 'M0 0 h16 v16 h-16 z M4 4 h8 v8 h-8 z') content-box }}"
        ));
        let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), size);
        let _ = runtime.render_now(&view, &[sheet]);
        let root = runtime.tree().root().unwrap();
        let node = runtime.tree().get(root).unwrap().children()[0];
        let frame = FrameSubmission::new(RenderFrame::new(runtime.snapshot(), size));
        let mut cpu = CpuRenderer::new(RendererId::from_raw(2));
        cpu.render(&frame, &RenderTarget::bitmap(BitmapTarget::new(size)))
            .unwrap();
        let pixels = cpu.pixmap().unwrap();
        for (x, y, visible) in [(2, 2, false), (6, 6, true), (12, 12, center_visible)] {
            let hit = runtime.hit_test(Point::new(x as f32 + 0.5, y as f32 + 0.5));
            assert_eq!(hit.nodes().contains(&node), visible, "{rule} at ({x}, {y})");
            assert_eq!(
                pixels.pixel(x, y).unwrap().alpha(),
                if visible { 255 } else { 0 },
                "{rule} at ({x}, {y})"
            );
        }
    }
}
