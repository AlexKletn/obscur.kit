#![doc = include_str!("overflow_hidden_scroll.doc/_module.md")]
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

#[doc = include_str!("overflow_hidden_scroll.doc/view.md")]
fn view() -> View {
    let rows: Vec<ViewNode> = (0..40)
        .map(|i| el(0xA100 + i, "row", vec![el(0xA200 + i, "wide", vec![])]))
        .collect();
    View::single(el(0xA000, "list", rows))
}

const CSS: &str = ".list { width: 200px; height: 100px; overflow: scroll }\
 .row { width: 200px; height: 20px; overflow: hidden }\
 .wide { width: 200px; height: 200px }";

#[test]
fn the_wheel_passes_through_a_hidden_box_to_the_scroller_behind_it() {
    let sheet = compile(CSS);
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(200.0, 100.0));
    let _ = rt.render_now(&view(), std::slice::from_ref(&sheet));

    // A point over the first row — which clips, so it is a scroll container in the
    // CSS sense, but must not take the wheel.
    let target = rt.handle_scroll(40.0, Point::new(50.0, 10.0));

    let root = rt.tree().root().expect("a root");
    let list = rt.tree().get(root).expect("live").children()[0];
    assert_eq!(
        target,
        Some(list),
        "the wheel over a `hidden` row must scroll the `scroll` list behind it"
    );
    assert!(
        rt.scroll_top(list) > 0.0,
        "and the list must actually have moved"
    );
}

#[test]
fn a_hidden_box_still_clips() {
    // The other half: refusing the wheel must not make `hidden` stop clipping.
    let sheet = compile(CSS);
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(200.0, 100.0));
    let _ = rt.render_now(&view(), std::slice::from_ref(&sheet));

    let root = rt.tree().root().expect("a root");
    let list = rt.tree().get(root).expect("live").children()[0];
    let row = rt.tree().get(list).expect("live").children()[0];
    let wide = rt.tree().get(row).expect("live").children()[0];

    let row_box = rt.layout_boxes().get(&row).expect("arranged");
    let wide_box = rt.layout_boxes().get(&wide).expect("arranged");
    assert!(
        wide_box.size().height > row_box.size().height,
        "the child really does overflow its clipping parent vertically — which is \
         what would make the row a scroll candidate at all"
    );
}

#[doc = include_str!("overflow_hidden_scroll.doc/command_count.md")]
fn command_count(rt: &UiRuntime) -> usize {
    rt.snapshot().display_list().commands().len()
}

#[test]
fn a_hidden_box_paints_no_scrollbar() {
    // The visual half of the same rule. `hidden` clips and can be scrolled
    // programmatically, so it keeps its clip and its content translation — but a
    // scrollbar advertises a scroll the user cannot perform, and CSS gives one only
    // to `scroll` (always) and `auto` (when overflowing).
    let sheet = compile(CSS);
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(200.0, 100.0));
    let _ = rt.render_now(&view(), std::slice::from_ref(&sheet));
    let with_clipping_rows = command_count(&rt);

    // The same tree with `clip` instead of `hidden`. The two clip identically; `clip`
    // is explicitly NOT a scroll container, so it never had a scrollbar. Any
    // difference in the painted commands is therefore the scrollbar and nothing else.
    let no_clip = compile(
        ".list { width: 200px; height: 100px; overflow: scroll }\
         .row { width: 200px; height: 20px; overflow: clip }\
         .wide { width: 200px; height: 200px }",
    );
    let mut plain = UiRuntime::new(RuntimeId::from_raw(2), Size::new(200.0, 100.0));
    let _ = plain.render_now(&view(), std::slice::from_ref(&no_clip));

    assert_eq!(
        with_clipping_rows,
        command_count(&plain),
        "clipping rows must not add scrollbar rects — only the `scroll` list has one"
    );
}

#[test]
fn scroll_metrics_report_both_axes_after_an_absolute_scroll() {
    let sheet = compile(
        ".list { width: 100px; height: 80px; overflow: scroll }
         .content { width: 300px; height: 200px }",
    );
    let view = View::single(el(1, "list", vec![el(2, "content", vec![])]));
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(500.0, 400.0));
    let _ = rt.render_now(&view, &[sheet]);

    let root = rt.tree().root().unwrap();
    let list = rt.tree().get(root).unwrap().children()[0];
    let content = rt.tree().get(list).unwrap().children()[0];
    assert!(rt.scroll_metrics(content).is_none());
    assert!(!rt.scroll_to(content, Point::new(10.0, 10.0)));

    assert!(rt.scroll_to(list, Point::new(490.0, 45.0)));
    let metrics = rt.scroll_metrics(list).unwrap();
    assert_eq!(metrics.viewport, Size::new(100.0, 80.0));
    assert_eq!(metrics.content_extent, Size::new(300.0, 200.0));
    assert_eq!(metrics.offset, Point::new(200.0, 45.0));
    let saved = rt.capture_scroll_state(list).unwrap();

    assert!(!rt.scroll_to(list, Point::new(0.0, f32::NAN)));
    assert_eq!(rt.scroll_metrics(list), Some(metrics));
    assert!(rt.scroll_to(list, Point::ORIGIN));
    assert!(rt.restore_scroll_state(list, saved));
    assert_eq!(rt.scroll_metrics(list), Some(metrics));
}

#[test]
fn smooth_scroll_keeps_a_frame_pending_until_completion_and_allows_interruption() {
    let sheet = compile(
        ".list { width: 100px; height: 80px; overflow: scroll }
         .content { width: 300px; height: 200px }",
    );
    let sheets = [sheet];
    let view = View::single(el(1, "list", vec![el(2, "content", vec![])]));
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(500.0, 400.0));
    let _ = rt.render_now(&view, &sheets);
    let root = rt.tree().root().unwrap();
    let list = rt.tree().get(root).unwrap().children()[0];

    assert!(rt.scroll_to_smooth(list, Point::new(200.0, 120.0), 0.5));
    assert!(rt.run_due_frame(&view, &sheets).is_some());
    assert!(rt.has_pending_frame());
    assert_eq!(rt.scroll_metrics(list).unwrap().offset, Point::ORIGIN);

    rt.advance_motion(0.25);
    assert!(rt.run_due_frame(&view, &sheets).is_some());
    assert!(rt.has_pending_frame());
    assert_eq!(
        rt.scroll_metrics(list).unwrap().offset,
        Point::new(175.0, 105.0)
    );

    rt.advance_motion(0.25);
    assert!(rt.run_due_frame(&view, &sheets).is_some());
    assert!(!rt.has_pending_frame());
    assert_eq!(
        rt.scroll_metrics(list).unwrap().offset,
        Point::new(200.0, 120.0)
    );

    assert!(rt.scroll_to_smooth(list, Point::ORIGIN, 0.5));
    rt.advance_motion(0.25);
    assert!(rt.scroll_to(list, Point::new(40.0, 30.0)));
    rt.advance_motion(0.5);
    assert_eq!(
        rt.scroll_metrics(list).unwrap().offset,
        Point::new(40.0, 30.0)
    );
}

#[test]
fn wheel_forwards_only_the_unconsumed_delta_to_the_outer_scroller() {
    let sheet = compile(
        ".outer { width: 100px; height: 100px; overflow: scroll }
         .inner { width: 100px; height: 60px; overflow: scroll }
         .content { height: 100px }
         .spacer { height: 140px }",
    );
    let inner = el(2, "inner", vec![el(3, "content", vec![])]);
    let view = View::single(el(1, "outer", vec![inner, el(4, "spacer", vec![])]));
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(500.0, 400.0));
    let _ = rt.render_now(&view, &[sheet]);
    let root = rt.tree().root().unwrap();
    let outer = rt.tree().get(root).unwrap().children()[0];
    let inner = rt.tree().get(outer).unwrap().children()[0];

    rt.set_scroll_top(inner, 30.0);
    let point = Point::new(10.0, 10.0);
    assert_eq!(rt.handle_scroll(25.0, point), Some(inner));
    assert_eq!(rt.scroll_top(inner), 40.0);
    let hit_before = rt.hit_test(point).target().unwrap().node();
    assert_eq!(rt.handle_scroll(f32::NAN, point), None);
    assert_eq!(rt.scroll_top(inner), 40.0);
    assert_eq!(rt.hit_test(point).target().unwrap().node(), hit_before);
    assert_eq!(rt.scroll_top(outer), 15.0);
    assert_eq!(rt.handle_scroll(10.0, point), Some(outer));
    assert_eq!(rt.scroll_top(outer), 25.0);
}

#[test]
fn touch_drag_and_kinetic_scroll_share_the_nested_chain() {
    use obscur_kit::interaction::pointer::{
        PointerDevice, PointerEvent, PointerEventKind, PointerId,
    };

    let sheet = compile(
        ".outer { width: 100px; height: 100px; overflow: scroll }
         .inner { width: 100px; height: 60px; overflow: scroll }
         .content { height: 100px }
         .spacer { height: 440px }",
    );
    let sheets = [sheet];
    let inner = el(2, "inner", vec![el(3, "content", vec![])]);
    let view = View::single(el(1, "outer", vec![inner, el(4, "spacer", vec![])]));
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(500.0, 600.0));
    let _ = rt.render_now(&view, &sheets);
    let root = rt.tree().root().unwrap();
    let outer = rt.tree().get(root).unwrap().children()[0];
    let inner = rt.tree().get(outer).unwrap().children()[0];

    rt.set_scroll_top(inner, 30.0);
    let touch = |kind, y, time| {
        PointerEvent::new(
            PointerId::touch(1),
            PointerDevice::Touch,
            kind,
            Point::new(10.0, y),
        )
        .with_timestamp(time)
        .as_trusted()
    };
    rt.handle_pointer(touch(PointerEventKind::Down, 50.0, 1_000));
    rt.handle_pointer(touch(PointerEventKind::Move, 25.0, 21_000));
    assert_eq!(rt.scroll_top(inner), 40.0);
    assert!(rt.scroll_top(outer) > 0.0);
    rt.handle_pointer(touch(PointerEventKind::Move, 15.0, 31_000));
    let released_at = rt.scroll_top(outer);
    rt.handle_pointer(touch(PointerEventKind::Up, 15.0, 32_000));
    assert!(rt.run_due_frame(&view, &sheets).is_some());
    assert!(rt.has_pending_frame());
    rt.advance_motion(0.016);
    assert_eq!(rt.scroll_top(inner), 40.0);
    assert!(rt.scroll_top(outer) > released_at);
}

#[test]
fn overscroll_behavior_stops_the_chain_independently_per_axis() {
    use obscur_kit::interaction::pointer::WheelEvent;

    for (policy, expected) in [
        ("overscroll-behavior: auto contain", Point::new(15.0, 0.0)),
        ("overscroll-behavior: none", Point::ORIGIN),
        (
            "overscroll-behavior: contain; overscroll-behavior-y: auto",
            Point::new(0.0, 15.0),
        ),
    ] {
        let sheet = compile(&format!(
            ".outer {{ width: 100px; height: 100px; overflow: scroll }}
             .inner {{ width: 60px; height: 60px; overflow: scroll; {policy} }}
             .content {{ width: 100px; height: 100px }}
             .spacer {{ width: 200px; height: 240px }}"
        ));
        let inner = el(2, "inner", vec![el(3, "content", vec![])]);
        let view = View::single(el(1, "outer", vec![inner, el(4, "spacer", vec![])]));
        let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(500.0, 400.0));
        let _ = rt.render_now(&view, &[sheet]);
        let root = rt.tree().root().unwrap();
        let outer = rt.tree().get(root).unwrap().children()[0];
        let inner = rt.tree().get(outer).unwrap().children()[0];

        assert!(rt.scroll_to(inner, Point::new(30.0, 30.0)));
        let wheel = WheelEvent::new(Point::new(10.0, 10.0), 25.0).with_delta_x(25.0);
        assert_eq!(rt.handle_wheel(wheel), Some(inner));
        assert_eq!(
            rt.scroll_metrics(inner).unwrap().offset,
            Point::new(40.0, 40.0)
        );
        assert_eq!(
            rt.scroll_metrics(outer).unwrap().offset,
            expected,
            "{policy}"
        );
    }
}

#[test]
fn scroll_anchor_preserves_a_retained_rows_viewport_position_after_reflow() {
    let before = compile(
        ".list { height: 100px; width: 120px; overflow: auto }
         .leading { height: 100px } .anchor { height: 40px } .trailing { height: 300px }",
    );
    let after = compile(
        ".list { height: 100px; width: 120px; overflow: auto }
         .leading { height: 150px } .anchor { height: 40px } .trailing { height: 300px }",
    );
    let view = View::single(el(
        1,
        "list",
        vec![
            el(2, "leading", vec![]),
            el(3, "anchor", vec![]),
            el(4, "trailing", vec![]),
        ],
    ));
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 400.0));
    let _ = rt.render_now(&view, &[before]);
    let root = rt.tree().root().unwrap();
    let list = rt.tree().get(root).unwrap().children()[0];
    let row = rt.tree().get(list).unwrap().children()[1];
    assert!(rt.scroll_to(list, Point::new(0.0, 90.0)));
    let anchor = rt.capture_scroll_anchor(list, row).unwrap();
    assert!(rt.capture_scroll_anchor(list, root).is_none());

    let _ = rt.render_now(&view, &[after]);
    assert!(rt.restore_scroll_anchor(anchor));
    assert_eq!(rt.scroll_top(list), 140.0);
    assert_eq!(
        rt.hit_test(Point::new(20.0, 15.0)).target().unwrap().node(),
        row
    );

    let reduced = View::single(el(1, "list", vec![el(2, "leading", vec![])]));
    let sheet = compile(".list { height: 100px; overflow: auto } .leading { height: 150px }");
    let _ = rt.render_now(&reduced, &[sheet]);
    assert!(!rt.restore_scroll_anchor(anchor));
}

#[test]
fn explicit_scroll_snap_uses_the_nearest_reachable_candidate() {
    let sheet = compile(
        ".list { width: 100px; height: 80px; overflow: scroll }
         .content { width: 300px; height: 200px }",
    );
    let view = View::single(el(1, "list", vec![el(2, "content", vec![])]));
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(500.0, 400.0));
    let _ = rt.render_now(&view, &[sheet]);
    let root = rt.tree().root().unwrap();
    let list = rt.tree().get(root).unwrap().children()[0];

    assert!(rt.scroll_to(list, Point::new(180.0, 85.0)));
    assert!(rt.snap_scroll_to_nearest(
        list,
        &[
            Point::new(110.0, 85.0),
            Point::new(500.0, 85.0),
            Point::new(f32::NAN, 85.0)
        ],
        0.0,
    ));
    assert_eq!(
        rt.scroll_metrics(list).unwrap().offset,
        Point::new(200.0, 85.0)
    );
    assert!(!rt.snap_scroll_to_nearest(list, &[], 0.0));
}

#[test]
fn scroll_velocity_accumulates_input_between_motion_ticks() {
    let sheet = compile(
        ".list { width: 100px; height: 80px; overflow: scroll }
         .content { width: 300px; height: 400px }",
    );
    let view = View::single(el(1, "list", vec![el(2, "content", vec![])]));
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(500.0, 400.0));
    let _ = rt.render_now(&view, &[sheet]);
    let root = rt.tree().root().unwrap();
    let list = rt.tree().get(root).unwrap().children()[0];
    assert_eq!(rt.scroll_velocity(list), Some(Point::ORIGIN));
    assert_eq!(rt.scroll_velocity(root), None);

    assert_eq!(rt.handle_scroll(10.0, Point::new(10.0, 10.0)), Some(list));
    assert_eq!(rt.handle_scroll(20.0, Point::new(10.0, 10.0)), Some(list));
    rt.advance_motion(0.25);
    assert_eq!(rt.scroll_velocity(list), Some(Point::new(0.0, 120.0)));
    rt.advance_motion(0.25);
    assert_eq!(rt.scroll_velocity(list), Some(Point::ORIGIN));

    assert!(rt.scroll_to_smooth(list, Point::new(200.0, 30.0), 0.5));
    rt.advance_motion(0.25);
    assert_eq!(rt.scroll_velocity(list), Some(Point::new(700.0, 0.0)));
    assert!(rt.scroll_to(list, Point::ORIGIN));
    assert_eq!(rt.scroll_velocity(list), Some(Point::ORIGIN));
}

#[test]
fn a_tracked_scroll_anchor_compensates_in_the_reflow_frame() {
    let stylesheet = |height| {
        compile(&format!(
            ".list {{ height: 100px; width: 120px; overflow: auto }}
             .leading {{ height: {height}px }} .anchor {{ height: 40px }}
             .trailing {{ height: 300px }}"
        ))
    };
    let view = View::single(el(
        1,
        "list",
        vec![
            el(2, "leading", vec![]),
            el(3, "anchor", vec![]),
            el(4, "trailing", vec![]),
        ],
    ));
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 400.0));
    let _ = rt.render_now(&view, &[stylesheet(100)]);
    let root = rt.tree().root().unwrap();
    let list = rt.tree().get(root).unwrap().children()[0];
    let row = rt.tree().get(list).unwrap().children()[1];
    assert!(rt.scroll_to(list, Point::new(0.0, 90.0)));
    assert!(rt.set_scroll_anchor(list, Some(row)));

    let _ = rt.render_now(&view, &[stylesheet(150)]);
    assert_eq!(rt.scroll_top(list), 140.0);
    assert_eq!(
        rt.hit_test(Point::new(20.0, 15.0)).target().unwrap().node(),
        row
    );

    assert!(rt.scroll_to(list, Point::new(0.0, 125.0)));
    let _ = rt.render_now(&view, &[stylesheet(170)]);
    assert_eq!(rt.scroll_top(list), 145.0);

    assert!(rt.set_scroll_anchor(list, None));
    let _ = rt.render_now(&view, &[stylesheet(210)]);
    assert_eq!(rt.scroll_top(list), 145.0);
}

#[test]
fn configured_snap_waits_for_wheel_input_to_settle_and_respects_programmatic_scroll() {
    let sheet = compile(
        ".list { width: 100px; height: 80px; overflow: scroll }
         .content { width: 100px; height: 400px }",
    );
    let sheets = [sheet];
    let view = View::single(el(1, "list", vec![el(2, "content", vec![])]));
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(500.0, 400.0));
    let _ = rt.render_now(&view, &sheets);
    let root = rt.tree().root().unwrap();
    let list = rt.tree().get(root).unwrap().children()[0];
    let point = Point::new(10.0, 10.0);
    assert!(rt.set_scroll_snap_points(list, &[Point::ORIGIN, Point::new(0.0, 100.0)]));

    assert_eq!(rt.handle_scroll(60.0, point), Some(list));
    assert!(rt.run_due_frame(&view, &sheets).is_some());
    assert!(rt.has_pending_frame());
    rt.advance_motion(0.08);
    assert_eq!(rt.scroll_top(list), 60.0);
    assert_eq!(rt.handle_scroll(10.0, point), Some(list));
    rt.advance_motion(0.08);
    assert_eq!(rt.scroll_top(list), 70.0);
    rt.advance_motion(0.05);
    rt.advance_motion(0.2);
    assert_eq!(rt.scroll_top(list), 100.0);

    rt.handle_scroll(-30.0, point);
    assert!(rt.scroll_to(list, Point::new(0.0, 65.0)));
    rt.advance_motion(0.5);
    rt.advance_motion(0.5);
    assert_eq!(rt.scroll_top(list), 65.0);
}

#[doc = include_str!("overflow_hidden_scroll.doc/scroll_offset_resets_when_the_container_disappears.md")]
#[test]
fn scroll_offset_resets_when_the_container_disappears() {
    let sheets = [compile(
        ".list { width: 100px; height: 50px }
         .scroll { overflow: auto }
         .hide { overflow: auto; display: none }
         .clip { overflow: hidden }
         .content { height: 200px }
         .short { height: 20px }",
    )];
    let view = |class: &str, content| {
        View::single(el(
            1,
            &format!("list {class}"),
            vec![el(2, content, vec![])],
        ))
    };
    let normal = view("scroll", "content");
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(200.0, 200.0));
    let _ = rt.render_now(&normal, &sheets);
    let root = rt.tree().root().unwrap();
    let node = rt.tree().get(root).unwrap().children()[0];

    rt.set_scroll_top(node, 120.0);
    let _ = rt.render_now(&view("clip", "content"), &sheets);
    assert_eq!(rt.scroll_top(node), 120.0);
    let _ = rt.render_now(&view("scroll", "short"), &sheets);
    assert_eq!(rt.scroll_top(node), 0.0);
    let _ = rt.render_now(&normal, &sheets);
    assert_eq!(rt.scroll_top(node), 120.0);

    for class in ["hide", "plain"] {
        rt.set_scroll_top(node, 120.0);
        let _ = rt.render_now(&view(class, "content"), &sheets);
        assert!(rt.tree().contains(node));
        assert_eq!(rt.scroll_top(node), 0.0);
        let _ = rt.render_now(&normal, &sheets);
        assert_eq!(rt.tree().get(root).unwrap().children()[0], node);
        assert_eq!(rt.scroll_top(node), 0.0);
    }
}
