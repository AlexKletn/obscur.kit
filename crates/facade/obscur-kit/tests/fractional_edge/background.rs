use super::*;

#[test]
fn a_translucent_gradient_composites_over_the_background_color() {
    let size = Size::new(16.0, 16.0);
    let submission = scene(
        ".dot { width: 16px; height: 16px; background-color: blue; \
                  background-image: linear-gradient(rgba(255, 0, 0, 0.5), rgba(255, 0, 0, 0.5)) }",
        size.width,
        1.0,
    );
    let mut renderer = CpuRenderer::new(RendererId::from_raw(2));
    renderer
        .render(&submission, &RenderTarget::bitmap(BitmapTarget::new(size)))
        .expect("CPU render");
    let pixel = renderer.pixmap().unwrap().pixel(8, 8).unwrap();
    assert_eq!(pixel.alpha(), 255);
    assert_eq!(pixel.green(), 0);
    assert!(pixel.red().abs_diff(128) <= 2, "{pixel:?}");
    assert!(pixel.blue().abs_diff(128) <= 2, "{pixel:?}");
}

#[test]
fn a_none_background_layer_leaves_the_other_layers_visible() {
    let size = Size::new(16.0, 16.0);
    for (image, expected_red) in [
        ("none, linear-gradient(red, red), none", 255),
        ("linear-gradient(red, red); background-image: none", 0),
    ] {
        let css = format!(
            ".dot {{ width: 16px; height: 16px; background-color: blue; background-image: {image} }}"
        );
        let mut renderer = CpuRenderer::new(RendererId::from_raw(2));
        renderer
            .render(
                &scene(&css, size.width, 1.0),
                &RenderTarget::bitmap(BitmapTarget::new(size)),
            )
            .expect("CPU render");
        let pixel = renderer.pixmap().unwrap().pixel(8, 8).unwrap();
        assert_eq!(pixel.red(), expected_red, "{image}");
        assert_eq!(pixel.blue(), 255 - expected_red, "{image}");
    }
}

#[test]
fn gradient_backgrounds_use_their_layer_tile_and_repeat() {
    let size = Size::new(24.0, 24.0);
    let submission = scene(
        ".dot { width: 24px; height: 24px; background-color: lime; \
            background-image: none, linear-gradient(to right, red, blue); \
            background-size: auto, 8px 4px; background-position: 0 0, 4px 6px; \
            background-repeat: no-repeat, repeat-x }",
        size.width,
        1.0,
    );
    let mut cpu = CpuRenderer::new(RendererId::from_raw(2));
    cpu.render(&submission, &RenderTarget::bitmap(BitmapTarget::new(size)))
        .expect("CPU render");
    let pixels = cpu.pixmap().unwrap();
    let start = pixels.pixel(5, 7).unwrap();
    let end = pixels.pixel(10, 7).unwrap();
    assert!(start.red() > start.blue());
    assert!(end.blue() > end.red());
    assert_eq!(start, pixels.pixel(13, 7).unwrap());
    assert_eq!(pixels.pixel(5, 5).unwrap().green(), 255);
    assert_eq!(pixels.pixel(5, 10).unwrap().green(), 255);
}

#[test]
fn gradient_background_space_leaves_gaps_and_round_resizes_tiles() {
    let size = Size::new(24.0, 24.0);
    for (mode, expected_green) in [("space", 255), ("round", 0)] {
        let css = format!(
            ".dot {{ width: 24px; height: 24px; background-color: lime; \
                background-image: linear-gradient(red, red); background-size: 9px 4px; \
                background-repeat: {mode} no-repeat }}"
        );
        let mut cpu = CpuRenderer::new(RendererId::from_raw(2));
        cpu.render(
            &scene(&css, size.width, 1.0),
            &RenderTarget::bitmap(BitmapTarget::new(size)),
        )
        .expect("CPU render");
        let pixels = cpu.pixmap().unwrap();
        assert_eq!(pixels.pixel(0, 1).unwrap().red(), 255, "{mode}");
        assert_eq!(pixels.pixel(23, 1).unwrap().red(), 255, "{mode}");
        assert_eq!(
            pixels.pixel(11, 1).unwrap().green(),
            expected_green,
            "{mode}"
        );
        assert_eq!(pixels.pixel(11, 5).unwrap().green(), 255, "{mode}");
    }
}

fn background_blend_scene() -> FrameSubmission {
    let element = |id, class: &str| {
        ViewNode::element(ComponentId::from_raw(id))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
    };
    let view = View::from_fragment(ViewFragment::from_nodes(vec![
        element(31, "underlay").with_children(ViewFragment::from_nodes(vec![
            element(32, "opaque"),
            element(33, "translucent"),
            element(34, "hue"),
        ])),
    ]));
    scene_for_view(
        &view,
        ".underlay { width: 24px; height: 24px; background-color: lime } \
         .opaque, .translucent, .hue { width: 16px; height: 8px } \
         .opaque { background-color: blue; \
            background-image: none, linear-gradient(red, red); \
            background-blend-mode: normal, multiply } \
         .translucent { background-image: linear-gradient(#ff000080, #ff000080); \
            background-blend-mode: multiply } \
         .hue { background-color: blue; background-image: linear-gradient(red, red); \
            background-blend-mode: hue }",
        24.0,
        1.0,
    )
}

fn assert_background_blend_pixels(pixels: &[u8]) {
    let pixel = |x: usize, y: usize| {
        let offset = (y * 24 + x) * 4;
        &pixels[offset..offset + 4]
    };
    assert_eq!(pixel(4, 4), [0, 0, 0, 255]);
    let translucent = pixel(4, 12);
    assert!(
        translucent
            .iter()
            .zip([128, 127, 0, 255])
            .all(|(&got, expected)| got.abs_diff(expected) <= 1),
        "{translucent:?}"
    );
    assert_eq!(pixel(20, 12), [0, 255, 0, 255]);
    let hue = pixel(4, 20);
    assert!(hue[0].abs_diff(94) <= 2, "{hue:?}");
    assert_eq!(&hue[1..], [0, 0, 255]);
}

#[test]
fn background_blend_modes_stay_inside_the_background_group() {
    let size = Size::new(24.0, 24.0);
    let mut cpu = CpuRenderer::new(RendererId::from_raw(2));
    cpu.render(
        &background_blend_scene(),
        &RenderTarget::bitmap(BitmapTarget::new(size)),
    )
    .expect("CPU render");
    assert_background_blend_pixels(cpu.pixmap().unwrap().data());
}

#[test]
#[ignore = "needs a GPU adapter"]
fn vello_background_blend_modes_stay_inside_the_background_group() {
    let (device, queue) = headless_gpu();
    let device = VelloDevice::new(device, queue).expect("Vello device");
    let mut gpu = VelloRenderer::new(RendererId::from_raw(3)).with_device(device);
    let (_, _, pixels) = gpu
        .render_to_image(&background_blend_scene(), 24, 24)
        .expect("GPU render");
    assert_background_blend_pixels(&pixels);
}

#[test]
fn background_shorthand_resets_preceding_blend_and_placement() {
    let size = Size::new(16.0, 16.0);
    let submission = scene(
        ".dot { width: 16px; height: 16px; background-blend-mode: multiply; \
            background-size: 4px 4px; background-repeat: no-repeat; \
            background: linear-gradient(red, red) blue }",
        size.width,
        1.0,
    );
    let mut renderer = CpuRenderer::new(RendererId::from_raw(2));
    renderer
        .render(&submission, &RenderTarget::bitmap(BitmapTarget::new(size)))
        .expect("CPU render");
    let pixel = renderer.pixmap().unwrap().pixel(8, 8).unwrap();
    assert_eq!(
        [pixel.red(), pixel.green(), pixel.blue(), pixel.alpha()],
        [255, 0, 0, 255]
    );
}

#[test]
fn background_shorthand_places_each_gradient_layer() {
    let size = Size::new(16.0, 16.0);
    let submission = scene(
        ".dot { width: 16px; height: 16px; \
            background: linear-gradient(red, red) left top / 4px 4px no-repeat, \
                        blue right bottom / 6px 6px linear-gradient(lime, lime) no-repeat }",
        size.width,
        1.0,
    );
    let mut renderer = CpuRenderer::new(RendererId::from_raw(2));
    renderer
        .render(&submission, &RenderTarget::bitmap(BitmapTarget::new(size)))
        .expect("CPU render");
    let pixmap = renderer.pixmap().unwrap();
    assert_eq!(pixmap.pixel(2, 2).unwrap().red(), 255);
    assert_eq!(pixmap.pixel(14, 14).unwrap().green(), 255);
    assert_eq!(pixmap.pixel(8, 8).unwrap().blue(), 255);
}

#[test]
fn background_position_resolves_offsets_from_far_edges() {
    let size = Size::new(16.0, 16.0);
    let submission = scene(
        ".dot { width: 16px; height: 16px; font-size: 10px; \
            background: linear-gradient(red, red) right 0.2em bottom 25% / 4px 4px no-repeat, \
                        blue linear-gradient(lime, lime) bottom 2px left / 4px 4px no-repeat }",
        size.width,
        1.0,
    );
    let mut renderer = CpuRenderer::new(RendererId::from_raw(2));
    renderer
        .render(&submission, &RenderTarget::bitmap(BitmapTarget::new(size)))
        .expect("CPU render");
    let pixmap = renderer.pixmap().unwrap();
    assert_eq!(pixmap.pixel(10, 9).unwrap().red(), 255);
    assert_eq!(pixmap.pixel(13, 12).unwrap().red(), 255);
    assert_eq!(pixmap.pixel(9, 9).unwrap().blue(), 255);
    assert_eq!(pixmap.pixel(10, 8).unwrap().blue(), 255);
    assert_eq!(pixmap.pixel(0, 10).unwrap().green(), 255);
    assert_eq!(pixmap.pixel(0, 9).unwrap().blue(), 255);
}

#[test]
fn background_clip_uses_each_layer_box_and_the_bottom_box_for_color() {
    let size = Size::new(20.0, 20.0);
    let submission = scene(
        ".dot { box-sizing: border-box; width: 20px; height: 20px; \
            border: 2px solid transparent; padding: 3px; background-color: blue; \
            background-image: linear-gradient(red, red), none; \
            background-size: 4px 4px; background-repeat: no-repeat; \
            background-origin: border-box; background-clip: padding-box, content-box }",
        size.width,
        1.0,
    );
    let mut renderer = CpuRenderer::new(RendererId::from_raw(2));
    renderer
        .render(&submission, &RenderTarget::bitmap(BitmapTarget::new(size)))
        .expect("CPU render");
    let pixels = renderer.pixmap().unwrap();
    assert_eq!(pixels.pixel(1, 3).unwrap().alpha(), 0);
    assert_eq!(pixels.pixel(3, 3).unwrap().red(), 255);
    assert_eq!(pixels.pixel(4, 4).unwrap().alpha(), 0);
    assert_eq!(pixels.pixel(5, 5).unwrap().blue(), 255);
    assert_eq!(pixels.pixel(15, 15).unwrap().alpha(), 0);
}

#[test]
fn background_origin_and_clip_use_separate_shorthand_boxes() {
    let size = Size::new(20.0, 20.0);
    let submission = scene(
        ".dot { box-sizing: border-box; width: 20px; height: 20px; \
            border: 2px solid transparent; padding: 3px; \
            background: linear-gradient(red, red) -2px -2px / 4px 4px no-repeat \
                        content-box border-box }",
        size.width,
        1.0,
    );
    let mut renderer = CpuRenderer::new(RendererId::from_raw(2));
    renderer
        .render(&submission, &RenderTarget::bitmap(BitmapTarget::new(size)))
        .expect("CPU render");
    let pixels = renderer.pixmap().unwrap();
    assert_eq!(pixels.pixel(2, 3).unwrap().alpha(), 0);
    assert_eq!(pixels.pixel(3, 3).unwrap().red(), 255);
    assert_eq!(pixels.pixel(6, 6).unwrap().red(), 255);
    assert_eq!(pixels.pixel(7, 6).unwrap().alpha(), 0);
}

#[test]
fn background_attachment_tracks_the_requested_scroll_space() {
    let size = Size::new(24.0, 24.0);
    let el = |id, class: &str| {
        ViewNode::element(ComponentId::from_raw(id))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
    };
    let view = View::single(
        el(1, "scroller").with_children(ViewFragment::from_nodes(vec![el(2, "content")])),
    );
    for (selector, attachment, transformed, stays_visible) in [
        (".scroller", "scroll", false, true),
        (".scroller", "local", false, false),
        (".content", "scroll", false, false),
        (".content", "fixed", false, true),
        (".content", "fixed", true, false),
    ] {
        let transform = if transformed {
            "transform: translateX(1px);"
        } else {
            ""
        };
        let sheet = compile(&format!(
            ".scroller {{ width: 20px; height: 16px; overflow: hidden; {transform} }} \
             .content {{ width: 20px; height: 48px }} \
             {selector} {{ background: linear-gradient(red, red) {attachment} \
                 left top / 8px 4px no-repeat }}"
        ));
        let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), size);
        let _ = runtime.render_now(&view, std::slice::from_ref(&sheet));
        let root = runtime.tree().root().unwrap();
        let scroller = runtime.tree().get(root).unwrap().children()[0];
        runtime.set_scroll_top(scroller, 8.0);
        let _ = runtime.render_now(&view, &[sheet]);
        assert_eq!(runtime.scroll_top(scroller), 8.0);
        let submission = FrameSubmission::new(RenderFrame::new(runtime.snapshot(), size));
        let mut cpu = CpuRenderer::new(RendererId::from_raw(2));
        cpu.render(&submission, &RenderTarget::bitmap(BitmapTarget::new(size)))
            .expect("CPU render");
        let pixel = cpu.pixmap().unwrap().pixel(3, 1).unwrap();
        assert_eq!(
            pixel.red(),
            if stays_visible { 255 } else { 0 },
            "{selector} {attachment} {transformed}"
        );
    }
}

#[test]
fn local_content_background_clip_scrolls_inside_the_padding_port() {
    let size = Size::new(24.0, 24.0);
    let content = ViewNode::element(ComponentId::from_raw(2))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("content".into())));
    let scroller = ViewNode::element(ComponentId::from_raw(1))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("scroller".into())))
        .with_children(ViewFragment::from_nodes(vec![content]));
    let view = View::single(scroller);
    let sheet = compile(
        ".scroller { box-sizing: border-box; width: 24px; height: 24px; overflow: hidden; \
            border: 2px solid transparent; padding: 4px; \
            background: linear-gradient(red, red) local content-box } \
         .content { height: 48px }",
    );
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), size);
    let _ = runtime.render_now(&view, std::slice::from_ref(&sheet));
    let root = runtime.tree().root().unwrap();
    let scroller = runtime.tree().get(root).unwrap().children()[0];
    let mut cpu = CpuRenderer::new(RendererId::from_raw(2));
    for offset in [0.0, 8.0] {
        runtime.set_scroll_top(scroller, offset);
        let _ = runtime.render_now(&view, std::slice::from_ref(&sheet));
        let submission = FrameSubmission::new(RenderFrame::new(runtime.snapshot(), size));
        cpu.render(&submission, &RenderTarget::bitmap(BitmapTarget::new(size)))
            .expect("CPU render");
        let pixels = cpu.pixmap().unwrap();
        assert_eq!(
            pixels.pixel(7, 3).unwrap().red(),
            if offset > 0.0 { 255 } else { 0 }
        );
        assert_eq!(pixels.pixel(7, 1).unwrap().alpha(), 0);
        assert_eq!(pixels.pixel(3, 8).unwrap().alpha(), 0);
    }
}
