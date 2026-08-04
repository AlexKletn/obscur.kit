#![doc = include_str!("slider_moves.doc/_module.md")]
#![forbid(unsafe_code)]

use std::cell::Cell;
use std::rc::Rc;

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::{Point, Size};
use obscur_kit::identity::{InternedSymbol, RuntimeId};
use obscur_kit::interaction::command::{KeyEvent, LogicalKey, NamedKey, PhysicalKey};
use obscur_kit::interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{EVENT_INPUT, View, ViewEventBinding, ViewNode};
use obscur_kit::widgets::slider::Slider;

const ON_INPUT: InternedSymbol = InternedSymbol::from_raw(0x7F01);

/// 116 wide, so 100 usable between the painter's two 8px insets.
const CSS: &str = "slider{width:116px;height:24px}";

fn compile(css: &str) -> Stylesheet {
    let parsed = StylesheetParser::new(&CssSource::anonymous(css)).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("slider_moves.doc/view_of.md")]
fn view_of(slider: &Slider) -> View {
    let ViewNode::Element {
        component,
        props,
        children,
        ..
    } = slider.view().root().nodes()[0].clone()
    else {
        panic!("a slider is an element");
    };
    let mut node = ViewNode::element(component).with_children(children);
    for prop in props {
        node = node.with_prop(prop);
    }
    View::single(node.with_event(ViewEventBinding::new(EVENT_INPUT, ON_INPUT)))
}

#[doc = include_str!("slider_moves.doc/staged.md")]
fn staged(slider: &Slider) -> (UiRuntime, Rc<Cell<(f32, f32)>>) {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 100.0));
    let _ = rt.render_now(&view_of(slider), std::slice::from_ref(&compile(CSS)));
    let seen = Rc::new(Cell::new((f32::NAN, f32::NAN)));
    let sink = Rc::clone(&seen);
    rt.on_value_input(ON_INPUT, move |low, high| sink.set((low, high)));
    (rt, seen)
}

#[doc = include_str!("slider_moves.doc/settle.md")]
fn settle(rt: &mut UiRuntime, base: &Slider, ranged: bool, seen: &Rc<Cell<(f32, f32)>>) {
    // What an application does: re-render with the value the control reported.
    // A slider is CONTROLLED — the runtime never writes into the tree — so a
    // test that skipped this would be driving a slider whose value never moves,
    // and every gesture after the first would read the original.
    let (low, high) = seen.get();
    if !low.is_finite() {
        return;
    }
    let scale = base.scale();
    let mut next = if ranged {
        Slider::ranging(low, high)
    } else {
        Slider::new(low)
    }
    .over(scale.min, scale.max);
    if let Some(step) = scale.step {
        next = next.stepped_by(step);
    }
    let _ = rt.render_now(&view_of(&next), std::slice::from_ref(&compile(CSS)));
}

fn press(rt: &mut UiRuntime, x: f32) {
    rt.handle_pointer(
        PointerEvent::new(
            PointerId::MOUSE,
            PointerDevice::Mouse,
            PointerEventKind::Down,
            Point::new(x, 12.0),
        )
        .as_trusted(),
    );
}

fn drag(rt: &mut UiRuntime, x: f32) {
    rt.handle_pointer(
        PointerEvent::new(
            PointerId::MOUSE,
            PointerDevice::Mouse,
            PointerEventKind::Move,
            Point::new(x, 12.0),
        )
        .as_trusted(),
    );
}

fn arrow(rt: &mut UiRuntime, key: NamedKey) {
    rt.handle_key(
        KeyEvent::press(PhysicalKey::ArrowRight, Some(LogicalKey::named(key))).as_trusted(),
    );
}

#[doc = include_str!("slider_moves.doc/a_press_on_the_track_jumps_the_thumb.md")]
#[test]
fn a_press_on_the_track_jumps_the_thumb() {
    // The behaviour a press on a track has to have, and the first thing that
    // proves the whole road works: pointer, hit test, the enclosing control, the
    // scale, the value, the report.
    let (mut rt, seen) = staged(&Slider::new(0.0).over(0.0, 100.0));
    press(&mut rt, 58.0);
    let (low, _) = seen.get();
    assert!((low - 50.0).abs() < 0.5, "half the track: {low}");
}

#[doc = include_str!("slider_moves.doc/a_drag_keeps_steering_after_it_leaves_the_track.md")]
#[test]
fn a_drag_keeps_steering_after_it_leaves_the_track() {
    // Capture: a fast drag runs off the control and must keep the thumb, which
    // is what every slider anywhere does.
    let base = Slider::new(0.0).over(0.0, 100.0);
    let (mut rt, seen) = staged(&base);
    press(&mut rt, 58.0);
    drag(&mut rt, 900.0);
    assert!((seen.get().0 - 100.0).abs() < 0.01, "pinned to the maximum");
    settle(&mut rt, &base, false, &seen);
    drag(&mut rt, -900.0);
    assert!((seen.get().0).abs() < 0.01, "and back to the minimum");
}

#[doc = include_str!("slider_moves.doc/a_stepped_drag_reports_only_the_values_it_offers.md")]
#[test]
fn a_stepped_drag_reports_only_the_values_it_offers() {
    let (mut rt, seen) = staged(&Slider::new(0.0).over(0.0, 100.0).stepped_by(25.0));
    press(&mut rt, 8.0 + 60.0);
    assert_eq!(seen.get().0, 50.0, "snapped, not 60");
}

#[doc = include_str!("slider_moves.doc/arrows_move_by_the_step.md")]
#[test]
fn arrows_move_by_the_step() {
    let base = Slider::new(50.0).over(0.0, 100.0).stepped_by(10.0);
    let (mut rt, seen) = staged(&base);
    // Focus it the way a user would. The press lands on the value it already
    // holds, so it reports nothing and the arrows below start from 50.
    press(&mut rt, 58.0);
    assert!(
        !seen.get().0.is_finite(),
        "an unmoved press reports nothing"
    );

    arrow(&mut rt, NamedKey::ArrowRight);
    assert_eq!(seen.get().0, 60.0);
    settle(&mut rt, &base, false, &seen);
    arrow(&mut rt, NamedKey::Home);
    assert_eq!(seen.get().0, 0.0, "Home goes to the minimum");
    settle(&mut rt, &base, false, &seen);
    arrow(&mut rt, NamedKey::End);
    assert_eq!(seen.get().0, 100.0, "End to the maximum");
}

#[doc = include_str!("slider_moves.doc/a_ranged_drag_grabs_the_nearer_thumb_and_the_ends_never_cross.md")]
#[test]
fn a_ranged_drag_grabs_the_nearer_thumb_and_the_ends_never_cross() {
    let (mut rt, seen) = staged(&Slider::ranging(20.0, 80.0).over(0.0, 100.0));
    // Nearer the high thumb, then dragged past the low one.
    press(&mut rt, 8.0 + 75.0);
    drag(&mut rt, 8.0 + 5.0);
    let (low, high) = seen.get();
    assert!(low <= high, "the ends did not cross: {low}..{high}");
    assert!((low - 20.0).abs() < 0.5, "the low end stayed put: {low}");
    assert!(
        (high - 20.0).abs() < 0.5,
        "and the high stopped on it: {high}"
    );
}

#[doc = include_str!("slider_moves.doc/a_press_that_is_refused_moves_nothing.md")]
#[test]
fn a_press_that_is_refused_moves_nothing() {
    // The slider takes the press AFTER the raw `pointerdown`, so refusing that
    // event stops it — which is what makes a slider inside a custom gesture
    // possible at all.
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 100.0));
    let slider = Slider::new(0.0).over(0.0, 100.0);
    let ViewNode::Element {
        component,
        props,
        children,
        ..
    } = slider.view().root().nodes()[0].clone()
    else {
        panic!("a slider is an element");
    };
    let mut node = ViewNode::element(component).with_children(children);
    for prop in props {
        node = node.with_prop(prop);
    }
    let view = View::single(
        node.with_event(ViewEventBinding::new(EVENT_INPUT, ON_INPUT))
            .with_event(ViewEventBinding::new(
                obscur_kit::view::view::EVENT_POINTER_DOWN,
                InternedSymbol::from_raw(0x7F02),
            )),
    );
    let _ = rt.render_now(&view, std::slice::from_ref(&compile(CSS)));
    let seen = Rc::new(Cell::new(false));
    let sink = Rc::clone(&seen);
    rt.on_value_input(ON_INPUT, move |_, _| sink.set(true));
    rt.on_command(
        InternedSymbol::from_raw(0x7F02),
        |e: &mut obscur_kit::interaction::event::UiEvent| e.prevent_default(),
    );

    press(&mut rt, 58.0);
    assert!(!seen.get(), "a refused press moved no thumb");
}
