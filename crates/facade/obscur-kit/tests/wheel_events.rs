#![doc = include_str!("wheel_events.doc/_module.md")]
#![forbid(unsafe_code)]

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::{Point, Size};
use obscur_kit::identity::{ComponentId, InternedSymbol, NodeId, RuntimeId};
use obscur_kit::interaction::event::UiEvent;
use obscur_kit::interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{
    CLASS, EVENT_POINTER_MOVE, EVENT_WHEEL, View, ViewEventBinding, ViewFragment, ViewNode,
    ViewProperty, ViewValue,
};

const ON_WHEEL: InternedSymbol = InternedSymbol::from_raw(0x7F01);
const ON_MOVE: InternedSymbol = InternedSymbol::from_raw(0x7F02);

const CONTAINER: ComponentId = ComponentId::from_raw(0x00C0_0001);

const CSS: &str = ".list{width:200px;height:100px;overflow:scroll}\
                   .row{width:200px;height:20px}";

/// Where the wheel is aimed in every test below: inside the first row, which is
/// inside the scrolling list.
const OVER_FIRST_ROW: Point = Point::new(50.0, 10.0);

#[doc = include_str!("wheel_events.doc/compile.md")]
fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("wheel_events.doc/Watch.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Watch {
    #[doc = include_str!("wheel_events.doc/Nothing.md")]
    Nothing,
    #[doc = include_str!("wheel_events.doc/Rows.md")]
    Rows,
    #[doc = include_str!("wheel_events.doc/TheList.md")]
    TheList,
}

#[doc = include_str!("wheel_events.doc/list_of_rows.md")]
fn list_of_rows(watch: Watch) -> View {
    let rows: Vec<ViewNode> = (0..40)
        .map(|_| {
            let row = ViewNode::element(CONTAINER)
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("row".into())));
            if watch == Watch::Rows {
                row.with_event(ViewEventBinding::new(EVENT_WHEEL, ON_WHEEL))
            } else {
                row
            }
        })
        .collect();
    let list = ViewNode::element(CONTAINER)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("list".into())))
        .with_children(ViewFragment::from_nodes(rows));
    View::single(if watch == Watch::TheList {
        list.with_event(ViewEventBinding::new(EVENT_WHEEL, ON_WHEEL))
    } else {
        list
    })
}

#[doc = include_str!("wheel_events.doc/staged.md")]
fn staged(view: &View) -> UiRuntime {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(200.0, 100.0));
    let sheet = compile(CSS);
    let _ = rt.render_now(view, std::slice::from_ref(&sheet));
    rt
}

#[doc = include_str!("wheel_events.doc/list_node.md")]
fn list_node(rt: &UiRuntime) -> NodeId {
    let root = rt.tree().root().expect("a root after a frame");
    rt.tree().get(root).expect("live").children()[0]
}

#[doc = include_str!("wheel_events.doc/first_row.md")]
fn first_row(rt: &UiRuntime) -> NodeId {
    rt.tree().get(list_node(rt)).expect("live").children()[0]
}

#[doc = include_str!("wheel_events.doc/refuse.md")]
fn refuse(rt: &mut UiRuntime) {
    rt.on_command(ON_WHEEL, |e: &mut UiEvent| e.prevent_default());
}

#[doc = include_str!("wheel_events.doc/an_authored_on_wheel_actually_fires.md")]
#[test]
fn an_authored_on_wheel_actually_fires() {
    // The gap this closes, and it is the third of the same one: `on:wheel`
    // hashed to a symbol nothing dispatched, so the binding compiled, the
    // handler never ran, and nothing said so.
    let mut rt = staged(&list_of_rows(Watch::Rows));
    let seen: Rc<RefCell<Vec<(f32, f32, f32)>>> = Rc::new(RefCell::new(Vec::new()));
    let sink = Rc::clone(&seen);
    rt.on_command(ON_WHEEL, move |e: &mut UiEvent| {
        if let Some(wheel) = e.wheel() {
            sink.borrow_mut()
                .push((wheel.delta_y(), wheel.position().x, wheel.position().y));
        }
    });

    rt.handle_scroll(40.0, OVER_FIRST_ROW);
    assert_eq!(
        seen.borrow().len(),
        1,
        "the handler runs and can read the notch off the envelope"
    );
    let (delta, x, y) = seen.borrow()[0];
    assert!(
        (delta - 40.0).abs() < f32::EPSILON,
        "the delta reaches the handler unchanged — sign and magnitude"
    );
    assert!((x - OVER_FIRST_ROW.x).abs() < f32::EPSILON);
    assert!((y - OVER_FIRST_ROW.y).abs() < f32::EPSILON);

    // The other direction of the same claim: a downward notch is a different
    // number, not merely "an event happened".
    rt.handle_scroll(-15.0, OVER_FIRST_ROW);
    assert_eq!(seen.borrow().len(), 2);
    assert!((seen.borrow()[1].0 + 15.0).abs() < f32::EPSILON);
}

#[doc = include_str!("wheel_events.doc/a_wheel_is_not_a_pointer_event.md")]
#[test]
fn a_wheel_is_not_a_pointer_event() {
    // The payload decision, pinned. A notch could have ridden in a synthesised
    // `PointerEvent` — and then a handler would read a pointer that "moved" to
    // the wheel's position, a move that never happened and which the model
    // could not tell from a real one. It carries its own variant instead, so
    // asking for the wrong one gets nothing rather than something plausible.
    let mut rt = staged(&list_of_rows(Watch::Rows));
    let payload: Rc<Cell<(bool, bool)>> = Rc::new(Cell::new((false, false)));
    let sink = Rc::clone(&payload);
    rt.on_command(ON_WHEEL, move |e: &mut UiEvent| {
        sink.set((e.wheel().is_some(), e.pointer().is_some()));
    });

    rt.handle_scroll(40.0, OVER_FIRST_ROW);
    assert_eq!(
        payload.get(),
        (true, false),
        "a wheel envelope carries a wheel and no pointer"
    );

    // And the negative half, which is what makes the first half mean anything: a
    // real pointer move over the same node fires `pointermove`, not `wheel`.
    let mut rt = staged(&View::single(
        ViewNode::element(CONTAINER)
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("list".into())))
            .with_event(ViewEventBinding::new(EVENT_WHEEL, ON_WHEEL))
            .with_event(ViewEventBinding::new(EVENT_POINTER_MOVE, ON_MOVE))
            .with_children(ViewFragment::from_nodes(vec![
                ViewNode::element(CONTAINER)
                    .with_prop(ViewProperty::new(CLASS, ViewValue::Text("row".into()))),
            ])),
    ));
    let counts: Rc<Cell<(u32, u32)>> = Rc::new(Cell::new((0, 0)));
    let wheels = Rc::clone(&counts);
    rt.on_command(ON_WHEEL, move |_e: &mut UiEvent| {
        let (w, m) = wheels.get();
        wheels.set((w + 1, m));
    });
    let moves = Rc::clone(&counts);
    rt.on_command(ON_MOVE, move |_e: &mut UiEvent| {
        let (w, m) = moves.get();
        moves.set((w, m + 1));
    });

    rt.handle_pointer(
        PointerEvent::new(
            PointerId::MOUSE,
            PointerDevice::Mouse,
            PointerEventKind::Move,
            OVER_FIRST_ROW,
        )
        .as_trusted(),
    );
    assert_eq!(counts.get(), (0, 1), "a move is not a notch");
    rt.handle_scroll(40.0, OVER_FIRST_ROW);
    assert_eq!(counts.get(), (1, 1), "and a notch is not a move");
}

#[doc = include_str!("wheel_events.doc/prevent_default_on_wheel_suppresses_the_scroll.md")]
#[test]
fn prevent_default_on_wheel_suppresses_the_scroll() {
    // THE acceptance test. The wheel runs before the container moves, so
    // refusing it is reachable — which is the only way a handler could ever
    // decline a scroll it did not want.
    let view = list_of_rows(Watch::Rows);
    let mut rt = staged(&view);
    refuse(&mut rt);
    let list = list_node(&rt);

    rt.handle_scroll(40.0, OVER_FIRST_ROW);
    assert_eq!(
        rt.scroll_top(list),
        0.0,
        "the refused notch scrolled nothing"
    );

    // The negative half, so the claim above is attributable to the refusal and
    // not to the list being unscrollable: the identical notch on the identical
    // tree, with a handler that merely watches, scrolls it.
    let mut rt = staged(&view);
    let ran = Rc::new(Cell::new(false));
    let sink = Rc::clone(&ran);
    rt.on_command(ON_WHEEL, move |_e: &mut UiEvent| sink.set(true));
    let list = list_node(&rt);
    rt.handle_scroll(40.0, OVER_FIRST_ROW);
    assert!(ran.get(), "the watching handler did run");
    assert!(
        rt.scroll_top(list) > 0.0,
        "watching a notch does not consume it"
    );
}

#[doc = include_str!("wheel_events.doc/a_refused_wheel_is_a_claim_not_a_fall_through.md")]
#[test]
fn a_refused_wheel_is_a_claim_not_a_fall_through() {
    // `handle_scroll`'s return value is what `App::on_scroll` is gated on: a
    // host that windows its own list scrolls it when the runtime reports that
    // nothing took the notch. So a refusal must NOT report `None` — a listener
    // that declines the scroll has taken the event, and reporting otherwise
    // would scroll the host's list out from under it, which is the one outcome
    // the author was refusing.
    let view = list_of_rows(Watch::Rows);
    let mut rt = staged(&view);
    refuse(&mut rt);
    let row = first_row(&rt);
    let list = list_node(&rt);

    let taker = rt.handle_scroll(40.0, OVER_FIRST_ROW);
    assert_eq!(
        taker,
        Some(row),
        "the refusing node is reported as the node that took the notch"
    );
    assert_ne!(
        taker,
        Some(list),
        "and it is the listener's node, not the container that would have scrolled"
    );
    assert_eq!(rt.scroll_top(list), 0.0);

    // The negative half: with nothing scrollable and nothing bound, `None` still
    // means `None` — the fall-through the host relies on is intact.
    let bare = View::single(
        ViewNode::element(CONTAINER)
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("row".into()))),
    );
    let mut plain = UiRuntime::new(RuntimeId::from_raw(2), Size::new(200.0, 100.0));
    let sheet = compile(CSS);
    let _ = plain.render_now(&bare, std::slice::from_ref(&sheet));
    assert_eq!(
        plain.handle_scroll(40.0, OVER_FIRST_ROW),
        None,
        "nothing bound the wheel and nothing scrolls, so nothing took it"
    );
}

#[doc = include_str!("wheel_events.doc/the_wheel_targets_the_node_under_the_point.md")]
#[test]
fn the_wheel_targets_the_node_under_the_point() {
    // Not the scroll container that would take the notch. The row is what the
    // pointer is over, and a click there would land on the row, so a wheel there
    // names the row — otherwise a row could never hear its own wheel.
    let mut rt = staged(&list_of_rows(Watch::Rows));
    let targets: Rc<RefCell<Vec<Option<NodeId>>>> = Rc::new(RefCell::new(Vec::new()));
    let sink = Rc::clone(&targets);
    rt.on_command(ON_WHEEL, move |e: &mut UiEvent| {
        sink.borrow_mut().push(e.target());
    });
    let row = first_row(&rt);
    let list = list_node(&rt);

    rt.handle_scroll(40.0, OVER_FIRST_ROW);
    assert_eq!(targets.borrow().as_slice(), &[Some(row)]);
    assert_ne!(row, list, "the row and the list are two nodes");
}

#[doc = include_str!("wheel_events.doc/a_wheel_bubbles_to_an_ancestor_binding.md")]
#[test]
fn a_wheel_bubbles_to_an_ancestor_binding() {
    // The binding is resolved by walking the hit node's ancestors, so a listener
    // on the list hears a notch aimed at a row inside it — which is how a
    // scroller refuses a scroll aimed anywhere within it without binding every
    // row.
    let mut rt = staged(&list_of_rows(Watch::TheList));
    let targets: Rc<RefCell<Vec<Option<NodeId>>>> = Rc::new(RefCell::new(Vec::new()));
    let sink = Rc::clone(&targets);
    rt.on_command(ON_WHEEL, move |e: &mut UiEvent| {
        sink.borrow_mut().push(e.target());
    });
    let list = list_node(&rt);

    rt.handle_scroll(40.0, OVER_FIRST_ROW);
    assert_eq!(
        targets.borrow().as_slice(),
        &[Some(list)],
        "the ancestor's binding resolved, and the event names the bound node"
    );

    // And the refusal reaches from up there too: the list refusing a notch aimed
    // at its own row stops its own scroll.
    let mut rt = staged(&list_of_rows(Watch::TheList));
    refuse(&mut rt);
    let list = list_node(&rt);
    assert_eq!(rt.handle_scroll(40.0, OVER_FIRST_ROW), Some(list));
    assert_eq!(rt.scroll_top(list), 0.0);
}

#[doc = include_str!("wheel_events.doc/an_unbound_wheel_name_is_never_dispatched.md")]
#[test]
fn an_unbound_wheel_name_is_never_dispatched() {
    // The cost gate, from the outside. A tree that binds nothing must pay
    // nothing — no hit test, no ancestor walk — and the only externally visible
    // consequence of getting it wrong would be a handler firing for an event it
    // never asked for. So: register a handler for the command WITHOUT any node
    // binding `on:wheel`, and it must stay quiet.
    let mut rt = staged(&list_of_rows(Watch::Nothing));
    let ran = Rc::new(Cell::new(false));
    let sink = Rc::clone(&ran);
    rt.on_command(ON_WHEEL, move |_e: &mut UiEvent| sink.set(true));
    let list = list_node(&rt);

    rt.handle_scroll(40.0, OVER_FIRST_ROW);
    assert!(
        !ran.get(),
        "no node bound the name, so nothing was dispatched"
    );
    // And the scroll it would have refused happened, exactly as it did before
    // the event existed.
    assert!(rt.scroll_top(list) > 0.0);
}
