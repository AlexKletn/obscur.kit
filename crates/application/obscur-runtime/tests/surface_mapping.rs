#![doc = include_str!("surface_mapping.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_geometry::{AffineTransform, Size};
use obscur_interaction::pointer::PointerEventKind;
use obscur_scene::command::PaintCommand;
use obscur_style::cascade::CascadeOrigin;
use obscur_style::matching::Stylesheet;
use obscur_testkit::fixture::{ClassRule, InteractionPanel, TestFixture, mouse};

/// A sheet where `panel` actually paints (a background), so a wrapped display
/// list has real content between the brackets — not just the brackets.
fn painted_panel_sheet() -> Stylesheet {
    Stylesheet::new(
        CascadeOrigin::GlobalApp,
        vec![
            ClassRule::new("panel")
                .sized(100.0, 100.0)
                .background(0.1, 0.2, 0.3)
                .build(),
        ],
    )
}

// ---- Device scale (HiDPI): root scale transform + pointer coord mapping ----

#[test]
fn device_scale_wraps_the_display_list_in_a_root_scale_transform() {
    let sheet = painted_panel_sheet();
    let sheets = std::slice::from_ref(&sheet);

    // Scale 2.0: the composed list is bracketed by a single root
    // PushTransform(scale(2, 2)) … PopTransform, and stays balanced.
    let mut scaled = TestFixture::runtime();
    scaled.set_device_scale(2.0);
    assert_eq!(scaled.device_scale(), 2.0);
    let frame = scaled.render_now(&InteractionPanel::view(), sheets);
    let list = frame.snapshot().display_list();
    assert!(list.is_balanced(), "the wrapped list stays balanced");
    assert!(list.len() >= 3, "at least the two brackets plus content");
    assert_eq!(
        list.commands().first(),
        Some(&PaintCommand::PushTransform(AffineTransform::scale(
            2.0, 2.0
        ))),
        "the list opens with the root device-scale transform",
    );
    assert_eq!(
        list.commands().last(),
        Some(&PaintCommand::PopTransform),
        "and closes with its matching pop",
    );

    // Scale 1.0 (default): no root transform is emitted — byte-identical to
    // the pre-device-scale path.
    let mut plain = TestFixture::runtime();
    let plain_frame = plain.render_now(&InteractionPanel::view(), sheets);
    let plain_list = plain_frame.snapshot().display_list();
    assert!(
        !plain_list
            .commands()
            .iter()
            .any(|c| matches!(c, PaintCommand::PushTransform(_))),
        "no root transform at scale 1.0",
    );

    // The scaled list is exactly the plain list wrapped in the two brackets.
    let mut expected = vec![PaintCommand::PushTransform(AffineTransform::scale(
        2.0, 2.0,
    ))];
    expected.extend_from_slice(plain_list.commands());
    expected.push(PaintCommand::PopTransform);
    assert_eq!(list.commands(), expected.as_slice());
}

#[test]
fn device_scale_one_is_byte_identical_to_before() {
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);

    // Default (implicit 1.0) vs an explicit set to 1.0: identical display
    // lists, and an identical pointer hit-test.
    let mut default_rt = TestFixture::runtime();
    let default_list = default_rt
        .render_now(&InteractionPanel::view(), sheets)
        .snapshot()
        .display_list()
        .commands()
        .to_vec();
    let d_target = default_rt
        .handle_pointer(mouse(PointerEventKind::Move, 50.0, 75.0))
        .target();

    let mut explicit_rt = TestFixture::runtime();
    explicit_rt.set_device_scale(1.0);
    let explicit_list = explicit_rt
        .render_now(&InteractionPanel::view(), sheets)
        .snapshot()
        .display_list()
        .commands()
        .to_vec();
    let e_target = explicit_rt
        .handle_pointer(mouse(PointerEventKind::Move, 50.0, 75.0))
        .target();

    assert_eq!(default_list, explicit_list, "scale 1.0 changes no commands");
    assert_eq!(d_target, e_target, "scale 1.0 changes no hit-test");
}

#[test]
fn device_scale_divides_incoming_pointer_coordinates() {
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);

    // Baseline at scale 1: physical (50, 75) lands in `b` (y ∈ [50, 100)).
    let mut base = TestFixture::runtime();
    let _ = base.render_now(&InteractionPanel::view(), sheets);
    let (_panel, a, b) = InteractionPanel::nodes(&base);
    assert_eq!(
        base.handle_pointer(mouse(PointerEventKind::Move, 50.0, 75.0))
            .target(),
        Some(b),
    );

    // At scale 2, physical (100, 150) maps to logical (50, 75) → same `b`.
    let mut scaled = TestFixture::runtime();
    scaled.set_device_scale(2.0);
    let _ = scaled.render_now(&InteractionPanel::view(), sheets);
    assert_eq!(
        scaled
            .handle_pointer(mouse(PointerEventKind::Move, 100.0, 150.0))
            .target(),
        Some(b),
        "physical (100,150) / 2 = logical (50,75) hits the same node as scale-1 (50,75)",
    );

    // And physical (50, 75) at scale 2 maps to logical (25, 37.5) → `a`
    // (y ∈ [0, 50)), proving the coordinates were actually divided.
    assert_eq!(
        scaled
            .handle_pointer(mouse(PointerEventKind::Move, 50.0, 75.0))
            .target(),
        Some(a),
    );
}

#[test]
fn set_device_scale_clamps_to_a_sane_minimum() {
    let mut runtime = TestFixture::runtime();
    runtime.set_device_scale(0.0);
    assert_eq!(runtime.device_scale(), 0.1, "non-positive clamps to 0.1");
    runtime.set_device_scale(-4.0);
    assert_eq!(runtime.device_scale(), 0.1, "negative clamps to 0.1");
    runtime.set_device_scale(f32::NAN);
    assert_eq!(runtime.device_scale(), 0.1, "non-finite clamps to 0.1");
    runtime.set_device_scale(2.5);
    assert_eq!(runtime.device_scale(), 2.5, "a sane value passes through");
}

#[test]
fn set_surface_maps_physical_extent_to_a_logical_viewport() {
    let mut runtime = TestFixture::runtime();
    // A 1000×600 physical surface at 2× density → a 500×300 logical viewport.
    runtime.set_surface(Size::new(1000.0, 600.0), 2.0);
    assert_eq!(runtime.device_scale(), 2.0);
    assert_eq!(runtime.viewport(), Size::new(500.0, 300.0));
    // At 1× the viewport equals the physical extent (no scaling).
    runtime.set_surface(Size::new(890.0, 1140.0), 1.0);
    assert_eq!(runtime.viewport(), Size::new(890.0, 1140.0));
    // The viewport divides by the *clamped* scale, so the two never disagree.
    runtime.set_surface(Size::new(100.0, 100.0), 0.0);
    assert_eq!(runtime.device_scale(), 0.1);
    assert_eq!(runtime.viewport(), Size::new(1000.0, 1000.0));
}

// ---- Content scroll offset: root translate + pointer coord mapping ----

#[test]
fn content_offset_translates_the_display_list_root() {
    let sheet = painted_panel_sheet();
    let sheets = std::slice::from_ref(&sheet);

    // Offset (0, 100) at scale 1.0: the list is bracketed by a single root
    // PushTransform of the pure translate(0, -100) … PopTransform.
    let mut scrolled = TestFixture::runtime();
    scrolled.set_content_offset(0.0, 100.0);
    assert_eq!(scrolled.content_offset(), (0.0, 100.0));
    let frame = scrolled.render_now(&InteractionPanel::view(), sheets);
    let list = frame.snapshot().display_list();
    assert!(list.is_balanced(), "the wrapped list stays balanced");
    assert_eq!(
        list.commands().first(),
        Some(&PaintCommand::PushTransform(AffineTransform::translate(
            0.0, -100.0
        ))),
        "offset 100 translates content up by 100 (scale·translate reduces to translate at s=1)",
    );
    assert_eq!(list.commands().last(), Some(&PaintCommand::PopTransform));

    // Offset (0, 0) at scale 1.0: no root transform emitted (byte-identical).
    let mut plain = TestFixture::runtime();
    let plain_frame = plain.render_now(&InteractionPanel::view(), sheets);
    let plain_list = plain_frame.snapshot().display_list();
    assert!(
        !plain_list
            .commands()
            .iter()
            .any(|c| matches!(c, PaintCommand::PushTransform(_))),
        "no root transform at offset (0,0) + scale 1.0",
    );
    let mut expected = vec![PaintCommand::PushTransform(AffineTransform::translate(
        0.0, -100.0,
    ))];
    expected.extend_from_slice(plain_list.commands());
    expected.push(PaintCommand::PopTransform);
    assert_eq!(list.commands(), expected.as_slice());
}

#[test]
fn content_offset_composes_with_device_scale() {
    let sheet = painted_panel_sheet();
    let sheets = std::slice::from_ref(&sheet);

    // scale 2 + offset (0, 100) → root = scale(2)·translate(0, -100).
    let mut rt = TestFixture::runtime();
    rt.set_device_scale(2.0);
    rt.set_content_offset(0.0, 100.0);
    let frame = rt.render_now(&InteractionPanel::view(), sheets);
    let list = frame.snapshot().display_list();
    let expected_root =
        AffineTransform::scale(2.0, 2.0).multiply(AffineTransform::translate(0.0, -100.0));
    assert_eq!(
        list.commands().first(),
        Some(&PaintCommand::PushTransform(expected_root)),
    );
    assert!(list.is_balanced());
}

#[test]
fn content_offset_maps_incoming_pointer_coordinates() {
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);

    // Baseline at offset 0: physical (50, 25) lands in `a` (y ∈ [0, 50)).
    let mut base = TestFixture::runtime();
    let _ = base.render_now(&InteractionPanel::view(), sheets);
    let (_panel, a, b) = InteractionPanel::nodes(&base);
    assert_eq!(
        base.handle_pointer(mouse(PointerEventKind::Move, 50.0, 25.0))
            .target(),
        Some(a),
    );

    // With offset (0, 100): physical (50, 25) maps to content y = 25/1 + 100
    // = 125 → out of the 100-tall panel → no target; and physical (50, -75)
    // maps to content y = -75 + 100 = 25 → `a`. Prove the offset is added.
    let mut scrolled = TestFixture::runtime();
    scrolled.set_content_offset(0.0, 100.0);
    let _ = scrolled.render_now(&InteractionPanel::view(), sheets);
    assert_eq!(
        scrolled
            .handle_pointer(mouse(PointerEventKind::Move, 50.0, -75.0))
            .target(),
        Some(a),
        "physical y -75 + offset 100 = content y 25 hits `a`",
    );

    // physical (50, -25) → content y = 75 → `b` (y ∈ [50, 100)).
    assert_eq!(
        scrolled
            .handle_pointer(mouse(PointerEventKind::Move, 50.0, -25.0))
            .target(),
        Some(b),
    );
}

#[test]
fn content_offset_maps_pointer_with_device_scale() {
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);

    // scale 2 + offset (0, 100): content y = physical / 2 + 100.
    let mut rt = TestFixture::runtime();
    rt.set_device_scale(2.0);
    rt.set_content_offset(0.0, 100.0);
    let _ = rt.render_now(&InteractionPanel::view(), sheets);
    let (_panel, _a, b) = InteractionPanel::nodes(&rt);
    // physical y -50 → -25 + 100 = content y 75 → `b`.
    assert_eq!(
        rt.handle_pointer(mouse(PointerEventKind::Move, 100.0, -50.0))
            .target(),
        Some(b),
        "physical (100,-50): x=100/2=50, y=-50/2+100=75 → `b`",
    );
}

#[test]
fn content_offset_zero_is_byte_identical_and_pointer_unchanged() {
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);

    let mut default_rt = TestFixture::runtime();
    let default_list = default_rt
        .render_now(&InteractionPanel::view(), sheets)
        .snapshot()
        .display_list()
        .commands()
        .to_vec();
    let d_target = default_rt
        .handle_pointer(mouse(PointerEventKind::Move, 50.0, 75.0))
        .target();

    let mut explicit_rt = TestFixture::runtime();
    explicit_rt.set_content_offset(0.0, 0.0);
    assert_eq!(explicit_rt.content_offset(), (0.0, 0.0));
    let explicit_list = explicit_rt
        .render_now(&InteractionPanel::view(), sheets)
        .snapshot()
        .display_list()
        .commands()
        .to_vec();
    let e_target = explicit_rt
        .handle_pointer(mouse(PointerEventKind::Move, 50.0, 75.0))
        .target();

    assert_eq!(
        default_list, explicit_list,
        "offset (0,0) changes no commands"
    );
    assert_eq!(d_target, e_target, "offset (0,0) changes no hit-test");
}

#[test]
fn set_content_offset_ignores_non_finite_components() {
    let mut runtime = TestFixture::runtime();
    runtime.set_content_offset(10.0, 20.0);
    runtime.set_content_offset(f32::NAN, 40.0);
    assert_eq!(
        runtime.content_offset(),
        (10.0, 40.0),
        "NaN x keeps prior x"
    );
    runtime.set_content_offset(50.0, f32::INFINITY);
    assert_eq!(
        runtime.content_offset(),
        (50.0, 40.0),
        "inf y keeps prior y"
    );
}
