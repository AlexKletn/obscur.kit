#![doc = include_str!("overflow_axis_longhands.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::{Point, Size};
use obscur_kit::identity::{ComponentId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

fn el(id: u64, class: &str, children: Vec<ViewNode>) -> ViewNode {
    ViewNode::element(ComponentId::from_raw(id))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
        .with_children(ViewFragment::from_nodes(children))
}

#[doc = include_str!("overflow_axis_longhands.doc/view.md")]
fn view() -> View {
    let rows: Vec<ViewNode> = (0..40).map(|i| el(0xB100 + i, "row", vec![])).collect();
    View::single(el(0xB000, "list", rows))
}

#[doc = include_str!("overflow_axis_longhands.doc/scrolled_by_a_wheel.md")]
fn scrolled_by_a_wheel(css: &str) -> (f32, bool) {
    let sheet = compile(css);
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(200.0, 100.0));
    let _ = rt.render_now(&view(), std::slice::from_ref(&sheet));

    let root = rt.tree().root().expect("a root");
    let list = rt.tree().get(root).expect("live").children()[0];
    let taken = rt.handle_scroll(40.0, Point::new(50.0, 10.0));
    (rt.scroll_top(list), taken == Some(list))
}

const ROWS: &str = " .row { width: 200px; height: 20px }";

#[test]
fn overflow_y_alone_scrolls_the_box() {
    // The declaration an author writes to scroll one direction. It used to name
    // no property at all — `overflow` was the only overflow id — so the whole
    // declaration was recorded as unsupported and the box never became a scroll
    // container: `handle_scroll` answered `None` over a list that visibly
    // overflowed.
    let (top, took_the_wheel) = scrolled_by_a_wheel(&format!(
        ".list {{ width: 200px; height: 100px; overflow-y: auto }}{ROWS}"
    ));
    assert!(
        took_the_wheel,
        "`overflow-y: auto` must make the box the wheel's target"
    );
    assert!(top > 0.0, "and the box must actually have moved");
}

#[test]
fn overflow_x_alone_carries_the_block_axis_with_it() {
    // css-overflow-3 §3: one axis stated as `hidden` computes an unstated
    // `visible` sibling to `auto`, which is why a box that only hides its
    // horizontal overflow still scrolls vertically everywhere else.
    let (top, took_the_wheel) = scrolled_by_a_wheel(&format!(
        ".list {{ width: 200px; height: 100px; overflow-x: hidden }}{ROWS}"
    ));
    assert!(
        took_the_wheel,
        "`overflow-x: hidden` leaves the block axis `auto`, which scrolls"
    );
    assert!(top > 0.0);
}

#[test]
fn the_shorthand_and_the_longhands_agree_and_cascade() {
    // The shorthand is now an expansion over the two longhands, so it must still
    // scroll on its own …
    let (shorthand_top, shorthand_took) = scrolled_by_a_wheel(&format!(
        ".list {{ width: 200px; height: 100px; overflow: auto }}{ROWS}"
    ));
    assert!(shorthand_took);
    assert!(shorthand_top > 0.0);

    // … and a longhand after it in the same block must win on that axis alone,
    // which is the whole point of storing the pair rather than the shorthand.
    let (top, took_the_wheel) = scrolled_by_a_wheel(&format!(
        ".list {{ width: 200px; height: 100px; overflow: hidden; overflow-y: auto }}{ROWS}"
    ));
    assert!(
        took_the_wheel,
        "the later `overflow-y: auto` must override the shorthand's block axis"
    );
    assert_eq!(top, shorthand_top);

    // The reverse order loses: the shorthand sets both axes, so it overrides the
    // earlier longhand and the box is `hidden` — a scroll container, but not one
    // the wheel may move.
    let (_, took_the_wheel) = scrolled_by_a_wheel(&format!(
        ".list {{ width: 200px; height: 100px; overflow-y: auto; overflow: hidden }}{ROWS}"
    ));
    assert!(
        !took_the_wheel,
        "a shorthand after a longhand resets both axes"
    );
}

#[test]
fn horizontal_scroll_moves_paint_and_hit_testing_on_the_same_axis() {
    use obscur_kit::identity::InternedSymbol;
    use obscur_kit::interaction::pointer::{
        PointerDevice, PointerEvent, PointerEventKind, PointerId, WheelEvent, WheelPhase,
    };
    use obscur_kit::scene::command::PaintCommand;
    use obscur_kit::view::view::{EVENT_SCROLL_GESTURE, ViewEventBinding};

    let list = el(1, "list", vec![el(2, "strip", vec![el(3, "item", vec![])])]);
    let view = View::single(list.with_event(ViewEventBinding::new(
        EVENT_SCROLL_GESTURE,
        InternedSymbol::named("observe-scroll"),
    )));
    let sheet = compile(
        ".list { width: 100px; height: 60px; overflow-x: auto; overflow-y: hidden }
         .strip { width: 240px; height: 60px; position: relative }
         .item { position: absolute; left: 160px; top: 0px; width: 20px; height: 30px }",
    );
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 100.0));
    rt.render_now(&view, &[sheet]);
    let root = rt.tree().root().unwrap();
    let list = rt.tree().get(root).unwrap().children()[0];
    let strip = rt.tree().get(list).unwrap().children()[0];
    let item = rt.tree().get(strip).unwrap().children()[0];

    let pointer = |device, kind, x| {
        PointerEvent::new(PointerId::from_raw(1), device, kind, Point::new(x, 12.0))
    };
    assert_eq!(
        rt.handle_wheel(WheelEvent::new(Point::new(20.0, 12.0), 0.0).with_delta_x(90.0)),
        Some(list),
    );
    assert_eq!(rt.scroll_left(list), 90.0);
    assert_eq!(rt.scroll_top(list), 0.0);
    assert_eq!(rt.layout_boxes()[&item].origin().x, 70.0);
    let frame = rt.run_due_frame(&view, &[]).unwrap();
    assert!(
        frame
            .snapshot()
            .display_list()
            .commands()
            .iter()
            .any(|command| {
                matches!(command, PaintCommand::PushTransform(transform)
            if transform.apply(Point::ORIGIN) == Point::new(-90.0, 0.0))
            })
    );
    assert_eq!(
        rt.handle_pointer(pointer(PointerDevice::Mouse, PointerEventKind::Move, 75.0))
            .target(),
        Some(item),
    );

    let wheel = |phase, x, delta| {
        WheelEvent::new(Point::new(x, 12.0), 0.0)
            .with_phase(phase)
            .with_delta_x(delta)
            .as_trusted()
    };
    rt.handle_wheel(wheel(WheelPhase::Started, 75.0, 0.0));
    rt.handle_wheel(wheel(WheelPhase::Moved, 250.0, 12.0));
    assert_eq!(
        rt.scroll_left(list),
        102.0,
        "the start point selects the scroller"
    );
    rt.handle_wheel(wheel(WheelPhase::Moved, 250.0, 8.0));
    rt.handle_wheel(wheel(WheelPhase::Ended, 250.0, 0.0));
    assert_eq!(
        rt.scroll_left(list),
        110.0,
        "the winner retains the sequence"
    );
    rt.set_scroll_left(list, 90.0);

    rt.handle_pointer(pointer(PointerDevice::Touch, PointerEventKind::Down, 75.0).as_trusted());
    rt.handle_pointer(pointer(PointerDevice::Touch, PointerEventKind::Move, 45.0).as_trusted());
    assert!(rt.scroll_left(list) > 90.0);
    rt.set_scroll_left(list, 1_000.0);
    assert_eq!(rt.scroll_left(list), 140.0);
}

#[test]
fn nested_scrollers_fold_each_axis_and_route_wheel_to_the_matching_container() {
    use obscur_kit::interaction::pointer::WheelEvent;

    let view = View::single(el(
        1,
        "outer",
        vec![
            el(2, "spacer", vec![]),
            el(3, "inner", vec![el(4, "strip", vec![])]),
            el(5, "spacer", vec![]),
        ],
    ));
    let sheet = compile(
        ".outer { width: 100px; height: 80px; overflow-y: auto; overflow-x: hidden }
         .spacer { height: 100px }
         .inner { width: 100px; height: 50px; overflow-x: auto; overflow-y: hidden }
         .strip { width: 240px; height: 50px }",
    );
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 180.0));
    rt.render_now(&view, &[sheet]);
    let root = rt.tree().root().unwrap();
    let outer = rt.tree().get(root).unwrap().children()[0];
    let inner = rt.tree().get(outer).unwrap().children()[1];
    let strip = rt.tree().get(inner).unwrap().children()[0];

    rt.set_scroll_top(outer, 80.0);
    let position = Point::new(20.0, 30.0);
    assert_eq!(
        rt.handle_wheel(WheelEvent::new(position, 0.0).with_delta_x(40.0)),
        Some(inner),
    );
    assert_eq!(rt.handle_scroll(20.0, position), Some(outer));
    assert_eq!(rt.scroll_left(inner), 40.0);
    assert_eq!(rt.scroll_top(outer), 100.0);
    assert_eq!(rt.layout_boxes()[&strip].origin(), Point::new(-40.0, 0.0));
}

#[test]
fn rtl_scroll_starts_at_the_right_and_reveals_content_to_the_left() {
    use obscur_kit::identity::NodeId;
    use obscur_kit::interaction::pointer::{
        PointerDevice, PointerEvent, PointerEventKind, PointerId, WheelEvent,
    };

    use obscur_kit::scene::command::PaintCommand;

    let view = View::single(el(
        1,
        "list",
        vec![el(2, "strip", vec![el(3, "item", vec![])])],
    ));
    let sheet = compile(
        ".list { position: absolute; left: 50px; width: 100px; height: 60px;
                 direction: rtl; overflow-x: auto; overflow-y: hidden }
         .strip { width: 240px; height: 60px; position: relative }
         .item { position: absolute; left: 0; top: 0; width: 20px; height: 30px }",
    );
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 100.0));
    rt.render_now(&view, &[sheet]);
    let root = rt.tree().root().unwrap();
    let first_child =
        |runtime: &UiRuntime, node: NodeId| runtime.tree().get(node).unwrap().children()[0];
    let list = first_child(&rt, root);
    let strip = first_child(&rt, list);
    let item = first_child(&rt, strip);
    let thumb_x = |runtime: &UiRuntime| {
        runtime
            .inspect_snapshot()
            .nodes
            .iter()
            .find(|part| {
                part.parent == Some(list) && part.local_name.as_deref() == Some("scrollbar-thumb")
            })
            .unwrap()
            .bounds
            .unwrap()
            .origin
            .x
    };

    assert_eq!(rt.scroll_left(list), 0.0);
    assert_eq!(rt.layout_boxes()[&item].origin().x, -90.0);
    let start_thumb = thumb_x(&rt);
    assert!(start_thumb > 100.0);
    let wheel = WheelEvent::new(Point::new(70.0, 12.0), 0.0).with_delta_x(-90.0);
    assert_eq!(rt.handle_wheel(wheel), Some(list));
    assert_eq!(rt.scroll_left(list), -90.0);
    assert!(thumb_x(&rt) < start_thumb);
    let frame = rt.run_due_frame(&view, &[]).unwrap();
    assert!(
        frame
            .snapshot()
            .display_list()
            .commands()
            .iter()
            .any(|command| {
                matches!(command, PaintCommand::PushTransform(transform)
            if transform.apply(Point::ORIGIN) == Point::new(90.0, 0.0))
            })
    );

    assert!(rt.scroll_into_view(item));
    assert_eq!(rt.scroll_left(list), -140.0);
    assert_eq!(rt.layout_boxes()[&item].origin().x, 50.0);
    assert_eq!(thumb_x(&rt), 50.0);
    let pointer = PointerEvent::new(
        PointerId::MOUSE,
        PointerDevice::Mouse,
        PointerEventKind::Move,
        Point::new(55.0, 12.0),
    );
    assert_eq!(rt.handle_pointer(pointer).target(), Some(item));

    rt.set_scroll_left(list, -500.0);
    assert_eq!(rt.scroll_left(list), -140.0);
    rt.set_scroll_left(list, 500.0);
    assert_eq!(rt.scroll_left(list), 0.0);
    assert_eq!(thumb_x(&rt), start_thumb);

    for (kind, x) in [
        (PointerEventKind::Down, 65.0),
        (PointerEventKind::Move, 95.0),
    ] {
        let event = PointerEvent::new(
            PointerId::touch(7),
            PointerDevice::Touch,
            kind,
            Point::new(x, 12.0),
        )
        .as_trusted();
        rt.handle_pointer(event);
    }
    assert!(rt.scroll_left(list) < 0.0);
}

#[test]
fn horizontal_thumb_drag_keeps_its_grab_point_and_capture_in_ltr_and_rtl() {
    use obscur_kit::geometry::Rect;
    use obscur_kit::interaction::pointer::{
        PointerButton, PointerButtons, PointerDevice, PointerEvent, PointerEventKind, PointerId,
    };

    for (rtl, device, pointer) in [
        (false, PointerDevice::Mouse, PointerId::MOUSE),
        (true, PointerDevice::Touch, PointerId::touch(1)),
    ] {
        let direction = if rtl { "rtl" } else { "ltr" };
        let view = View::single(el(1, "list", vec![el(2, "strip", vec![])]));
        let sheet = compile(&format!(
            ".list {{ position: absolute; left: 50px; width: 100px; height: 60px;
                      direction: {direction}; overflow-x: auto; overflow-y: hidden }}
             .strip {{ width: 240px; height: 60px }}"
        ));
        let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 200.0));
        rt.render_now(&view, &[sheet]);
        let root = rt.tree().root().unwrap();
        let list = rt.tree().get(root).unwrap().children()[0];
        let part = |runtime: &UiRuntime, name: &str| -> Rect {
            runtime
                .inspect_snapshot()
                .nodes
                .iter()
                .find(|p| p.parent == Some(list) && p.local_name.as_deref() == Some(name))
                .unwrap()
                .bounds
                .unwrap()
        };
        let track = part(&rt, "scrollbar-track");
        let thumb = part(&rt, "scrollbar-thumb");
        let travel = track.size.width - thumb.size.width;
        let sign = if rtl { -1.0 } else { 1.0 };
        let x = thumb.origin.x + 8.0;
        let y = thumb.origin.y + 2.0;
        let event = |kind, x, y| {
            let mut buttons = PointerButtons::new();
            if matches!(
                kind,
                PointerEventKind::Down | PointerEventKind::Move | PointerEventKind::Leave
            ) {
                buttons.press(PointerButton::Primary);
            }
            PointerEvent::new(pointer, device, kind, Point::new(x, y))
                .with_buttons(buttons)
                .with_button(PointerButton::Primary)
                .as_trusted()
        };

        let down = rt.handle_pointer(event(PointerEventKind::Down, x, y));
        assert_eq!(down.target(), Some(list));
        assert_eq!(rt.pointer_capture(pointer), Some(list));
        assert!(!down.focus_changed());
        assert_eq!(rt.scroll_left(list), 0.0);

        rt.handle_pointer(event(PointerEventKind::Leave, x, 120.0));
        assert_eq!(rt.pointer_capture(pointer), Some(list));
        rt.handle_pointer(event(
            PointerEventKind::Move,
            x + sign * travel / 2.0,
            120.0,
        ));
        assert!((rt.scroll_left(list) - sign * 70.0).abs() < 0.001);
        let updated = part(&rt, "scrollbar-thumb");
        assert!((updated.origin.x + 8.0 - (x + sign * travel / 2.0)).abs() < 0.001);

        let up = rt.handle_pointer(event(PointerEventKind::Up, x + sign * 500.0, 120.0));
        assert_eq!(rt.scroll_left(list), sign * 140.0);
        assert_eq!(rt.pointer_capture(pointer), None);
        assert_eq!(up.command(), None);

        let thumb = part(&rt, "scrollbar-thumb");
        rt.handle_pointer(event(PointerEventKind::Down, thumb.origin.x + 4.0, y));
        assert_eq!(rt.pointer_capture(pointer), Some(list));
        rt.handle_pointer(event(PointerEventKind::Cancel, 80.0, 120.0));
        assert_eq!(rt.pointer_capture(pointer), None);
        assert_eq!(rt.scroll_left(list), sign * 140.0);
    }
}

#[test]
fn a_prevented_thumb_press_leaves_scroll_and_capture_untouched() {
    use obscur_kit::identity::InternedSymbol;
    use obscur_kit::interaction::event::UiEvent;
    use obscur_kit::interaction::pointer::{
        PointerButton, PointerDevice, PointerEvent, PointerEventKind, PointerId,
    };
    use obscur_kit::view::view::{EVENT_POINTER_DOWN, ViewEventBinding};

    let command = InternedSymbol::named("block-thumb-drag");
    let view = View::single(
        el(1, "list", vec![el(2, "strip", vec![])])
            .with_event(ViewEventBinding::new(EVENT_POINTER_DOWN, command)),
    );
    let sheet = compile(
        ".list { width: 100px; height: 60px; overflow-x: auto; overflow-y: hidden }
         .strip { width: 240px; height: 60px }",
    );
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 100.0));
    rt.on_command(command, |event: &mut UiEvent| event.prevent_default());
    rt.render_now(&view, &[sheet]);
    let root = rt.tree().root().unwrap();
    let list = rt.tree().get(root).unwrap().children()[0];

    let down = PointerEvent::new(
        PointerId::MOUSE,
        PointerDevice::Mouse,
        PointerEventKind::Down,
        Point::new(8.0, 54.0),
    )
    .with_button(PointerButton::Primary)
    .as_trusted();
    assert_eq!(rt.handle_pointer(down).target(), Some(list));
    assert_eq!(rt.pointer_capture(PointerId::MOUSE), None);
    assert_eq!(rt.scroll_left(list), 0.0);
}
