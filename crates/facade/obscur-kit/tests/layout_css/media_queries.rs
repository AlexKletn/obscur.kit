use super::*;

#[doc = include_str!("../layout_css.doc/media_orientation_tracks_viewport_changes.md")]
#[test]
fn media_orientation_tracks_viewport_changes() {
    let sheet = compile(
        ".row { height: 10px }
         @media (orientation: landscape) { .row { height: 20px } }
         @media (orientation: portrait) { .row { height: 30px } }",
    );
    let view = two_child_view();
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 200.0));
    for (size, expected) in [
        (Size::new(300.0, 200.0), 20.0),
        (Size::new(200.0, 200.0), 30.0),
        (Size::new(200.0, 300.0), 30.0),
        (Size::new(300.0, 200.0), 20.0),
    ] {
        runtime.set_viewport(size);
        runtime.render_now(&view, std::slice::from_ref(&sheet));
        assert_eq!(geom(&runtime, container_node(&runtime)).3, expected);
    }
}

#[doc = include_str!("../layout_css.doc/media_resolution_tracks_fractional_surface_scale.md")]
#[test]
fn media_resolution_tracks_fractional_surface_scale() {
    let sheet = compile(
        ".row { height: 10px }
         @media (144dpi <= resolution < 2dppx) { .row { height: 20px } }",
    );
    let view = two_child_view();
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 200.0));
    runtime.set_device_scale(1.25);
    runtime.render_now(&view, std::slice::from_ref(&sheet));
    let node = container_node(&runtime);
    assert_eq!(geom(&runtime, node).3, 10.0);

    runtime.set_device_scale(1.75);
    assert!(
        runtime
            .run_due_frame(&view, std::slice::from_ref(&sheet))
            .is_some()
    );
    assert_eq!(geom(&runtime, node).3, 20.0);

    runtime.set_device_scale(2.0);
    assert!(
        runtime
            .run_due_frame(&view, std::slice::from_ref(&sheet))
            .is_some()
    );
    assert_eq!(geom(&runtime, node).3, 10.0);

    runtime.set_surface(Size::new(375.0, 250.0), 1.25);
    assert!(runtime.run_due_frame(&view, &[sheet]).is_some());
    assert_eq!(runtime.viewport(), Size::new(300.0, 200.0));
    assert_eq!(geom(&runtime, node).3, 10.0);
}

#[doc = include_str!("../layout_css.doc/media_input_tracks_primary_and_additional_devices.md")]
#[test]
fn media_input_tracks_primary_and_additional_devices() {
    use obscur_kit::AppControl;
    use obscur_kit::bootstrap::CssPipeline;
    use obscur_kit::style::matching::{MediaInput, PointerAccuracy};

    let sheet = compile(
        ".row { width: 100px; height: 10px }
         @media (pointer: coarse) and (hover: none) { .row { height: 20px } }
         @media (any-pointer: fine) and (any-pointer: coarse) and (any-hover) {
             .row { width: 200px }
         }
         @media (any-pointer: none) { .row { width: 50px } }",
    );
    let view = two_child_view();
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 200.0));
    let control = AppControl::new();
    let mut css = CssPipeline::new();
    runtime.render_now(&view, std::slice::from_ref(&sheet));
    let node = container_node(&runtime);
    for (input, expected) in [
        (Some(MediaInput::default()), Size::new(50.0, 10.0)),
        (
            Some(MediaInput {
                primary: Some(PointerAccuracy::Coarse),
                ..MediaInput::default()
            }),
            Size::new(100.0, 20.0),
        ),
        (
            Some(MediaInput {
                primary: Some(PointerAccuracy::Coarse),
                additional_fine: true,
                additional_hover: true,
                ..MediaInput::default()
            }),
            Size::new(200.0, 20.0),
        ),
        (None, Size::new(100.0, 10.0)),
    ] {
        control.set_media_input(input);
        control.apply(&mut runtime, &mut css);
        assert!(
            runtime
                .run_due_frame(&view, std::slice::from_ref(&sheet))
                .is_some()
        );
        assert_eq!(runtime.layout_boxes()[&node].size(), expected);
    }

    runtime.set_detected_media_input(Some(MediaInput {
        primary: Some(PointerAccuracy::Coarse),
        ..MediaInput::default()
    }));
    assert_eq!(
        runtime.media_input(),
        None,
        "the unknown override still wins"
    );
    control.clear_media_input_override();
    control.apply(&mut runtime, &mut css);
    assert!(runtime.run_due_frame(&view, &[sheet]).is_some());
    assert_eq!(runtime.layout_boxes()[&node].size(), Size::new(100.0, 20.0));
}

#[doc = include_str!("../layout_css.doc/media_boolean_groups_restyle_on_viewport_changes.md")]
#[test]
fn media_boolean_groups_restyle_on_viewport_changes() {
    let sheet = compile(
        ".row { height: 10px }
         @media (orientation: portrait) or ((width > 350px) and (not (height < 180px))) {
             .row { height: 20px }
         }",
    );
    let view = two_child_view();
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 200.0));
    for (size, expected) in [
        (Size::new(300.0, 200.0), 10.0),
        (Size::new(200.0, 300.0), 20.0),
        (Size::new(400.0, 200.0), 20.0),
        (Size::new(400.0, 160.0), 10.0),
    ] {
        runtime.set_viewport(size);
        runtime.render_now(&view, std::slice::from_ref(&sheet));
        assert_eq!(geom(&runtime, container_node(&runtime)).3, expected);
    }
}

#[doc = include_str!("../layout_css.doc/malformed_media_branches_keep_valid_alternatives.md")]
#[test]
fn malformed_media_branches_keep_valid_alternatives() {
    let source = CssSource::anonymous(
        ".row { height: 10px }
         @media (: 1px), (width > 200px), [bad, screen] { .row { height: 20px } }
         @media (: 1px) { .row { height: 99px } }",
    );
    let parsed = StylesheetParser::new(&source).parse();
    assert_eq!(parsed.errors.len(), 3);
    let sheet = StylesheetCompiler::new().compile(&parsed).matchable;
    let view = two_child_view();
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(100.0, 200.0));
    for (width, expected) in [(100.0, 10.0), (300.0, 20.0)] {
        runtime.set_viewport(Size::new(width, 200.0));
        runtime.render_now(&view, std::slice::from_ref(&sheet));
        assert_eq!(geom(&runtime, container_node(&runtime)).3, expected);
    }
}
