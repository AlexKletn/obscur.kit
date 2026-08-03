#![doc = include_str!("pointer_events.doc/_module.md")]
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
use obscur_kit::text::shaping::MockTextShaper;
use obscur_kit::view::view::{
    CLASS, EVENT_CLICK, EVENT_INPUT, EVENT_POINTER_CANCEL, EVENT_POINTER_DOWN, EVENT_POINTER_ENTER,
    EVENT_POINTER_LEAVE, EVENT_POINTER_MOVE, EVENT_POINTER_UP, LOCAL_NAME, VALUE, View,
    ViewEventBinding, ViewFragment, ViewNode, ViewProperty, ViewValue,
};

const ON_DOWN: InternedSymbol = InternedSymbol::from_raw(0x7E01);
const ON_UP: InternedSymbol = InternedSymbol::from_raw(0x7E02);
const ON_MOVE: InternedSymbol = InternedSymbol::from_raw(0x7E03);
const ON_CANCEL: InternedSymbol = InternedSymbol::from_raw(0x7E04);
const ON_CLICK: InternedSymbol = InternedSymbol::from_raw(0x7E05);
const ON_EDIT: InternedSymbol = InternedSymbol::from_raw(0x7E06);
const ON_ENTER: InternedSymbol = InternedSymbol::from_raw(0x7E07);
const ON_LEAVE: InternedSymbol = InternedSymbol::from_raw(0x7E08);

#[doc = include_str!("pointer_events.doc/Crossing.md")]
type Crossing = (&'static str, Option<NodeId>);

const BUTTON: ComponentId = ComponentId::from_raw(0x00D0_0001);
const INPUT: ComponentId = ComponentId::from_raw(0x00D0_0004);
const CONTAINER: ComponentId = ComponentId::from_raw(0x00C0_0001);

const CSS: &str = ".panel{width:300px;height:300px}.a{width:200px;height:40px}\
                  .b{width:200px;height:40px}";

/// The per-glyph advance the default mock shaper emits at 16px — the same
/// constant `text_mouse_caret.rs` measures caret offsets against.
const ADVANCE: f32 = 16.0 * MockTextShaper::ADVANCE_RATIO;

fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("pointer_events.doc/watched.md")]
fn watched(node: ViewNode) -> ViewNode {
    node.with_event(ViewEventBinding::new(EVENT_POINTER_DOWN, ON_DOWN))
        .with_event(ViewEventBinding::new(EVENT_POINTER_UP, ON_UP))
        .with_event(ViewEventBinding::new(EVENT_POINTER_MOVE, ON_MOVE))
        .with_event(ViewEventBinding::new(EVENT_POINTER_CANCEL, ON_CANCEL))
}

#[doc = include_str!("pointer_events.doc/button.md")]
fn button() -> View {
    View::single(watched(
        ViewNode::element(BUTTON)
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("a".into())))
            .with_prop(ViewProperty::new(
                LOCAL_NAME,
                ViewValue::Text("button".into()),
            ))
            .with_event(ViewEventBinding::new(EVENT_CLICK, ON_CLICK)),
    ))
}

#[doc = include_str!("pointer_events.doc/field.md")]
fn field(class: &str, value: &str) -> ViewNode {
    watched(
        ViewNode::element(INPUT)
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
            .with_prop(ViewProperty::new(
                LOCAL_NAME,
                ViewValue::Text("input".into()),
            ))
            .with_prop(ViewProperty::new(VALUE, ViewValue::Text(value.into())))
            .with_event(ViewEventBinding::new(EVENT_INPUT, ON_EDIT))
            // The laid-out text, which is what maps a pointer x onto a character
            // offset — a field carrying only a `value` prop has no glyph for a
            // drag-selection to reach.
            .with_children(ViewFragment::from_nodes(vec![ViewNode::text(
                value.to_owned(),
            )])),
    )
}

#[doc = include_str!("pointer_events.doc/two_fields.md")]
fn two_fields() -> View {
    View::single(
        ViewNode::element(CONTAINER)
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into())))
            .with_children(ViewFragment::from_nodes(vec![
                field("a", "alpha"),
                field("b", "beta"),
            ])),
    )
}

#[doc = include_str!("pointer_events.doc/staged.md")]
fn staged(view: &View) -> UiRuntime {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 300.0));
    let sheet = compile(CSS);
    let _ = rt.render_now(view, std::slice::from_ref(&sheet));
    rt
}

#[doc = include_str!("pointer_events.doc/crossed_row.md")]
fn crossed_row(class: &str, watch: bool) -> ViewNode {
    let row = ViewNode::element(BUTTON)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())));
    if watch { crossed(row) } else { row }
}

#[doc = include_str!("pointer_events.doc/crossed.md")]
fn crossed(node: ViewNode) -> ViewNode {
    node.with_event(ViewEventBinding::new(EVENT_POINTER_ENTER, ON_ENTER))
        .with_event(ViewEventBinding::new(EVENT_POINTER_LEAVE, ON_LEAVE))
}

#[doc = include_str!("pointer_events.doc/crossed_rows.md")]
fn crossed_rows(rows_watch: bool, panel_watches: bool) -> View {
    let panel = ViewNode::element(CONTAINER)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into())))
        .with_children(ViewFragment::from_nodes(vec![
            crossed_row("a", rows_watch),
            crossed_row("b", rows_watch),
        ]));
    View::single(if panel_watches { crossed(panel) } else { panel })
}

#[doc = include_str!("pointer_events.doc/parent_of.md")]
fn parent_of(rt: &UiRuntime, node: NodeId) -> NodeId {
    rt.tree()
        .get(node)
        .expect("a live node")
        .logical_parent()
        .parent()
        .expect("a row has a panel above it")
}

#[doc = include_str!("pointer_events.doc/watch_crossings.md")]
fn watch_crossings(rt: &mut UiRuntime) -> Rc<RefCell<Vec<Crossing>>> {
    let seen: Rc<RefCell<Vec<Crossing>>> = Rc::new(RefCell::new(Vec::new()));
    for (command, label) in [(ON_ENTER, "enter"), (ON_LEAVE, "leave")] {
        let sink = Rc::clone(&seen);
        rt.on_command(command, move |e: &mut UiEvent| {
            sink.borrow_mut().push((label, e.target()));
        });
    }
    seen
}

#[doc = include_str!("pointer_events.doc/at.md")]
fn at(kind: PointerEventKind, x: f32, y: f32) -> PointerEvent {
    PointerEvent::new(
        PointerId::MOUSE,
        PointerDevice::Mouse,
        kind,
        Point::new(x, y),
    )
    .as_trusted()
}

#[doc = include_str!("pointer_events.doc/finger_at.md")]
fn finger_at(kind: PointerEventKind, x: f32, y: f32) -> PointerEvent {
    PointerEvent::new(
        PointerId::touch(0),
        PointerDevice::Touch,
        kind,
        Point::new(x, y),
    )
    .as_trusted()
}

#[doc = include_str!("pointer_events.doc/text_origin_x.md")]
fn text_origin_x(rt: &UiRuntime) -> f32 {
    let text_node = rt
        .tree()
        .iter()
        .find_map(|(id, node)| node.kind().text().map(|_| id))
        .expect("a text leaf after a frame");
    rt.layout_boxes()
        .get(&text_node)
        .expect("the text leaf has a box")
        .origin()
        .x
}

#[doc = include_str!("pointer_events.doc/an_authored_on_pointerdown_actually_fires.md")]
#[test]
fn an_authored_on_pointerdown_actually_fires() {
    // The gap this closes, and it is the same one `on:keydown` had: the name
    // compiled, hashed to a symbol, bound a command — and nothing in the
    // workspace dispatched it, so the handler never ran and nothing said so.
    let mut rt = staged(&button());
    let seen: Rc<RefCell<Vec<(PointerDevice, f32, bool)>>> = Rc::new(RefCell::new(Vec::new()));
    let sink = Rc::clone(&seen);
    rt.on_command(ON_DOWN, move |e: &mut UiEvent| {
        if let Some(p) = e.pointer() {
            sink.borrow_mut()
                .push((p.device(), p.position().x, p.is_trusted()));
        }
    });

    rt.handle_pointer(at(PointerEventKind::Down, 10.0, 10.0));
    assert_eq!(
        seen.borrow().len(),
        1,
        "the handler runs and reads §2.4's payload off the envelope"
    );
    let (device, x, trusted) = seen.borrow()[0];
    assert_eq!(device, PointerDevice::Mouse);
    assert!((x - 10.0).abs() < f32::EPSILON);
    assert!(
        trusted,
        "and the trust the host conferred survives the trip"
    );

    // The negative half: a move is not a press, so the same binding stays quiet.
    rt.handle_pointer(at(PointerEventKind::Move, 12.0, 12.0));
    assert_eq!(
        seen.borrow().len(),
        1,
        "a move fires no `pointerdown` — one name, one kind"
    );
}

#[doc = include_str!("pointer_events.doc/each_raw_name_fires_on_its_own_kind.md")]
#[test]
fn each_raw_name_fires_on_its_own_kind() {
    let mut rt = staged(&button());
    let counts: Rc<Cell<[u32; 4]>> = Rc::new(Cell::new([0; 4]));
    for (index, command) in [ON_DOWN, ON_UP, ON_MOVE, ON_CANCEL].into_iter().enumerate() {
        let c = Rc::clone(&counts);
        rt.on_command(command, move || {
            let mut seen = c.get();
            seen[index] += 1;
            c.set(seen);
        });
    }

    rt.handle_pointer(at(PointerEventKind::Down, 10.0, 10.0));
    rt.handle_pointer(at(PointerEventKind::Move, 11.0, 11.0));
    rt.handle_pointer(at(PointerEventKind::Up, 11.0, 11.0));
    rt.handle_pointer(at(PointerEventKind::Cancel, 11.0, 11.0));

    assert_eq!(
        counts.get(),
        [1, 1, 1, 1],
        "each kind fired its own name exactly once and no other"
    );

    // The negative half: a crossing KIND is dispatched under no name of the four.
    //
    // 2026-08-03. This used to read "the crossings have no reserved name,
    // because nothing emits one" — `pointerenter` / `pointerleave` did not
    // exist. They do now, but they are emitted from the hover transition rather
    // than from these two kinds, so the assertion is unchanged and its reason is
    // narrower: an `Enter` or a `Leave` still dispatches none of the four,
    // because a kind without a name of its own must dispatch nothing rather than
    // something else's. What the crossings DO emit is pinned in
    // `a_crossing_fires_pointerenter_and_pointerleave` below.
    rt.handle_pointer(at(PointerEventKind::Enter, 11.0, 11.0));
    rt.handle_pointer(at(PointerEventKind::Leave, 400.0, 400.0));
    assert_eq!(
        counts.get(),
        [1, 1, 1, 1],
        "a crossing dispatches none of the four"
    );
}

#[doc = include_str!("pointer_events.doc/prevent_default_on_pointerdown_leaves_focus_where_it_was.md")]
#[test]
fn prevent_default_on_pointerdown_leaves_focus_where_it_was() {
    // THE acceptance test for the press half. A `pointerdown` runs before the
    // runtime moves focus, so refusing it is reachable — which is the only way a
    // handler could ever have declined a focus move it did not want.
    let view = two_fields();
    let mut rt = staged(&view);
    let first = rt
        .handle_pointer(at(PointerEventKind::Down, 10.0, 10.0))
        .target()
        .expect("the first field is under (10, 10)");
    rt.handle_pointer(at(PointerEventKind::Up, 10.0, 10.0));
    assert_eq!(rt.focused(), Some(first));

    // The second field is a different node, or this test asserts nothing.
    let second_probe = staged(&view)
        .handle_pointer(at(PointerEventKind::Down, 10.0, 60.0))
        .target()
        .expect("the second field is under (10, 60)");
    assert_ne!(first, second_probe, "the two fields are two nodes");

    rt.on_command(ON_DOWN, |e: &mut UiEvent| e.prevent_default());
    let dispatch = rt.handle_pointer(at(PointerEventKind::Down, 10.0, 60.0));
    assert_eq!(
        rt.focused(),
        Some(first),
        "the refused press moved no focus"
    );
    assert!(!dispatch.focus_changed());

    // And the negative half, so the claim above is attributable to the refusal
    // rather than to the second field being unfocusable.
    let mut rt = staged(&view);
    rt.handle_pointer(at(PointerEventKind::Down, 10.0, 10.0));
    rt.handle_pointer(at(PointerEventKind::Up, 10.0, 10.0));
    rt.on_command(ON_DOWN, |_e: &mut UiEvent| {});
    rt.handle_pointer(at(PointerEventKind::Down, 10.0, 60.0));
    assert_eq!(
        rt.focused(),
        Some(second_probe),
        "an unrefused press moves focus exactly as it always did"
    );
}

#[doc = include_str!("pointer_events.doc/prevent_default_on_pointerup_runs_no_on_click.md")]
#[test]
fn prevent_default_on_pointerup_runs_no_on_click() {
    // THE acceptance test for the release half. `prevent_default` had nothing to
    // refuse on the pointer path at all: a click was resolved and activated
    // before any raw event existed to decline it.
    let view = button();
    let mut rt = staged(&view);
    let clicks = Rc::new(Cell::new(0_u32));
    let c = Rc::clone(&clicks);
    rt.on_command(ON_CLICK, move || c.set(c.get() + 1));
    rt.on_command(ON_UP, |e: &mut UiEvent| e.prevent_default());

    rt.handle_pointer(at(PointerEventKind::Down, 10.0, 10.0));
    let up = rt.handle_pointer(at(PointerEventKind::Up, 10.0, 10.0));
    assert_eq!(clicks.get(), 0, "the refused release ran no `on:click`");
    assert_eq!(
        up.command(),
        None,
        "and reported no activation command, because none resolved"
    );

    // The negative half.
    let mut rt = staged(&view);
    let clicks = Rc::new(Cell::new(0_u32));
    let c = Rc::clone(&clicks);
    rt.on_command(ON_CLICK, move || c.set(c.get() + 1));
    rt.on_command(ON_UP, |_e: &mut UiEvent| {});
    rt.handle_pointer(at(PointerEventKind::Down, 10.0, 10.0));
    let up = rt.handle_pointer(at(PointerEventKind::Up, 10.0, 10.0));
    assert_eq!(
        clicks.get(),
        1,
        "an unrefused release clicks as it always did"
    );
    assert_eq!(up.command(), Some(ON_CLICK));
}

#[doc = include_str!("pointer_events.doc/a_refused_press_means_no_click_can_follow.md")]
#[test]
fn a_refused_press_means_no_click_can_follow() {
    // The consequence of suppressing the press anchor, stated as its own claim:
    // a click is a release matched against a press, so refusing the press is
    // enough on its own — the release need not also be refused.
    let view = button();
    let mut rt = staged(&view);
    let clicks = Rc::new(Cell::new(0_u32));
    let c = Rc::clone(&clicks);
    rt.on_command(ON_CLICK, move || c.set(c.get() + 1));
    rt.on_command(ON_DOWN, |e: &mut UiEvent| e.prevent_default());
    rt.on_command(ON_UP, |_e: &mut UiEvent| {});

    rt.handle_pointer(at(PointerEventKind::Down, 10.0, 10.0));
    let up = rt.handle_pointer(at(PointerEventKind::Up, 10.0, 10.0));
    assert_eq!(clicks.get(), 0, "no anchor, so the release is not a click");
    assert_eq!(up.command(), None);

    // The negative half: the very same release, with only the press unrefused.
    let mut rt = staged(&view);
    let clicks = Rc::new(Cell::new(0_u32));
    let c = Rc::clone(&clicks);
    rt.on_command(ON_CLICK, move || c.set(c.get() + 1));
    rt.on_command(ON_DOWN, |_e: &mut UiEvent| {});
    rt.on_command(ON_UP, |_e: &mut UiEvent| {});
    rt.handle_pointer(at(PointerEventKind::Down, 10.0, 10.0));
    rt.handle_pointer(at(PointerEventKind::Up, 10.0, 10.0));
    assert_eq!(clicks.get(), 1);
}

#[doc = include_str!("pointer_events.doc/prevent_default_on_pointermove_suppresses_the_drag_selection.md")]
#[test]
fn prevent_default_on_pointermove_suppresses_the_drag_selection() {
    // The move half. Sweeping over a field selects text; a `pointermove`
    // listener that refuses is what stops it — an application drawing its own
    // marquee over a field needs exactly this.
    let view = two_fields();

    let mut rt = staged(&view);
    let ox = text_origin_x(&rt);
    rt.on_command(ON_MOVE, |e: &mut UiEvent| e.prevent_default());
    rt.handle_pointer(at(PointerEventKind::Down, ox + ADVANCE, 20.0));
    rt.handle_pointer(at(PointerEventKind::Move, ox + ADVANCE * 4.0, 20.0));
    let selection = rt.editing_selection().expect("the press seated a session");
    assert!(
        selection.is_collapsed(),
        "the refused move selected nothing; the caret stayed where the press put it"
    );
    assert_eq!(selection.focus(), 1);

    // And the negative half, so the assertion above is attributable to the
    // refusal rather than to the sweep being too short to select anything.
    let mut rt = staged(&view);
    let ox = text_origin_x(&rt);
    rt.on_command(ON_MOVE, |_e: &mut UiEvent| {});
    rt.handle_pointer(at(PointerEventKind::Down, ox + ADVANCE, 20.0));
    rt.handle_pointer(at(PointerEventKind::Move, ox + ADVANCE * 4.0, 20.0));
    let selection = rt.editing_selection().expect("the press seated a session");
    assert!(!selection.is_collapsed(), "an unrefused sweep selects");
    assert_eq!((selection.anchor(), selection.focus()), (1, 4));
}

#[doc = include_str!("pointer_events.doc/pointercancel_is_reported_and_its_refusal_is_inert.md")]
#[test]
fn pointercancel_is_reported_and_its_refusal_is_inert() {
    // The documented refusal. A cancel is the system taking the sequence away;
    // what follows it is not a default action but the unwinding of state that no
    // longer describes anything. Honouring a refusal would reinstate the stuck
    // `:active` and the never-released capture that `cancel_landed` exists to
    // clear, so the event reports the loss and does not offer to decline it.
    let view = button();
    for refuse in [false, true] {
        let mut rt = staged(&view);
        let cancels = Rc::new(Cell::new(0_u32));
        let clicks = Rc::new(Cell::new(0_u32));
        let n = Rc::clone(&cancels);
        let c = Rc::clone(&clicks);
        rt.on_command(ON_CANCEL, move |e: &mut UiEvent| {
            n.set(n.get() + 1);
            if refuse {
                e.prevent_default();
            }
        });
        rt.on_command(ON_CLICK, move || c.set(c.get() + 1));

        rt.handle_pointer(at(PointerEventKind::Down, 10.0, 10.0));
        rt.handle_pointer(at(PointerEventKind::Cancel, 10.0, 10.0));
        let up = rt.handle_pointer(at(PointerEventKind::Up, 10.0, 10.0));

        assert_eq!(
            cancels.get(),
            1,
            "the cancel was reported (refuse={refuse})"
        );
        assert_eq!(
            clicks.get(),
            0,
            "the press was discarded either way (refuse={refuse})"
        );
        assert_eq!(up.command(), None, "(refuse={refuse})");
    }
}

#[doc = include_str!("pointer_events.doc/an_unbound_raw_name_is_never_dispatched.md")]
#[test]
fn an_unbound_raw_name_is_never_dispatched() {
    // The cost claim's observable half. `pointermove` arrives at the pointer's
    // report rate, and resolving a binding walks the target's ancestor chain,
    // which allocates. That walk is skipped — not merely empty — unless some
    // node in the committed tree binds the name, and the bit that decides it is
    // recomputed once per structural frame rather than once per event.
    let only_press = View::single(
        ViewNode::element(BUTTON)
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("a".into())))
            .with_event(ViewEventBinding::new(EVENT_POINTER_DOWN, ON_DOWN))
            .with_event(ViewEventBinding::new(EVENT_CLICK, ON_CLICK)),
    );
    let mut rt = staged(&only_press);
    let moves = Rc::new(Cell::new(0_u32));
    let downs = Rc::new(Cell::new(0_u32));
    let m = Rc::clone(&moves);
    let d = Rc::clone(&downs);
    rt.on_command(ON_MOVE, move || m.set(m.get() + 1));
    rt.on_command(ON_DOWN, move || d.set(d.get() + 1));

    for step in 0..64 {
        #[expect(
            clippy::cast_precision_loss,
            reason = "a loop counter under 64 is exact as an f32"
        )]
        rt.handle_pointer(at(PointerEventKind::Move, 10.0 + step as f32 * 0.1, 10.0));
    }
    assert_eq!(
        moves.get(),
        0,
        "nothing binds `pointermove`, so nothing ran"
    );

    // The positive half in the same tree: the name that IS bound still fires, so
    // the gate is per-name and not a blanket off switch.
    rt.handle_pointer(at(PointerEventKind::Down, 10.0, 10.0));
    assert_eq!(downs.get(), 1, "the bound name is unaffected by the gate");
}

#[doc = include_str!("pointer_events.doc/a_raw_event_bubbles_to_an_ancestor_binding.md")]
#[test]
fn a_raw_event_bubbles_to_an_ancestor_binding() {
    // §2.3's phases, not a direct hit test: a press on a child resolves the
    // nearest bound ancestor, exactly as a click does.
    let view = View::single(
        watched(
            ViewNode::element(CONTAINER)
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into()))),
        )
        .with_children(ViewFragment::from_nodes(vec![
            ViewNode::element(BUTTON)
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("a".into()))),
        ])),
    );
    let mut rt = staged(&view);
    let targets: Rc<RefCell<Vec<Option<NodeId>>>> = Rc::new(RefCell::new(Vec::new()));
    let sink = Rc::clone(&targets);
    rt.on_command(ON_DOWN, move |e: &mut UiEvent| {
        sink.borrow_mut().push(e.target());
    });

    let hit = rt
        .handle_pointer(at(PointerEventKind::Down, 10.0, 10.0))
        .target()
        .expect("the child button is under (10, 10)");
    assert_eq!(
        targets.borrow().len(),
        1,
        "the unbound child's press reached the bound ancestor"
    );
    // The event's `target` is the node the binding resolved on — the ancestor —
    // while the hit is the child. Both are true at once, and that difference is
    // what "bubble" means here.
    assert_ne!(
        targets.borrow()[0],
        Some(hit),
        "the binding resolved on the ancestor, not on the hit child"
    );
}

#[doc = include_str!("pointer_events.doc/a_crossing_fires_pointerenter_and_pointerleave.md")]
#[test]
fn a_crossing_fires_pointerenter_and_pointerleave() {
    // THE acceptance test for the crossings, and it pins all four claims at once
    // in the one sequence: which nodes are addressed, that they are not the hit
    // target, that leaves come before enters, and that the two run in opposite
    // directions along the chain.
    let view = crossed_rows(true, true);
    let mut rt = staged(&view);
    let seen = watch_crossings(&mut rt);

    let a = rt
        .handle_pointer(at(PointerEventKind::Move, 10.0, 10.0))
        .target()
        .expect("row a is under (10, 10)");
    let panel = parent_of(&rt, a);
    assert_ne!(a, panel, "the row and the panel are two nodes");

    let b = rt
        .handle_pointer(at(PointerEventKind::Move, 10.0, 60.0))
        .target()
        .expect("row b is under (10, 60)");
    assert_ne!(a, b, "the two rows are two nodes");
    assert_eq!(parent_of(&rt, b), panel, "and they share the one panel");

    // The pointer leaves the surface: nothing is hit, so the whole chain unwinds.
    rt.handle_pointer(at(PointerEventKind::Leave, 400.0, 400.0));

    assert_eq!(
        *seen.borrow(),
        vec![
            // Arriving from nowhere enters the panel BEFORE the row inside it —
            // outermost-first, so a handler that opens a panel on entry runs
            // before the row in it asks to be highlighted.
            ("enter", Some(panel)),
            ("enter", Some(a)),
            // Sibling to sibling. One event, two crossings, and neither of them
            // names the node the move targeted: `a` is left while `b` is hit.
            // The leave comes first, so a teardown is never interleaved with the
            // next node's setup.
            ("leave", Some(a)),
            ("enter", Some(b)),
            // And out: innermost-first, so the row is left before the panel that
            // held it. The two sequences nest.
            ("leave", Some(b)),
            ("leave", Some(panel)),
        ]
    );

    // The negative half: a move that crosses no boundary crosses nothing. Two
    // reports inside the same row are not two enters, or `pointerenter` would be
    // `pointermove` under another name.
    let mut rt = staged(&view);
    let seen = watch_crossings(&mut rt);
    rt.handle_pointer(at(PointerEventKind::Move, 10.0, 10.0));
    let after_arrival = seen.borrow().len();
    rt.handle_pointer(at(PointerEventKind::Move, 12.0, 12.0));
    rt.handle_pointer(at(PointerEventKind::Move, 14.0, 14.0));
    assert_eq!(
        seen.borrow().len(),
        after_arrival,
        "moving within one row crossed nothing"
    );
}

#[doc = include_str!("pointer_events.doc/an_ancestor_binding_is_not_resolved_for_a_descendants_crossing.md")]
#[test]
fn an_ancestor_binding_is_not_resolved_for_a_descendants_crossing() {
    // The one place the crossings differ from every other event here: the
    // binding is resolved on the crossed node ALONE. The crossing set is already
    // the whole chain, so an ancestor walk would deliver the panel's binding once
    // for the panel and again for the row that crossed with it — two events for
    // one boundary, and one more for every level of nesting.
    let mut rt = staged(&crossed_rows(false, true));
    let seen = watch_crossings(&mut rt);

    let a = rt
        .handle_pointer(at(PointerEventKind::Move, 10.0, 10.0))
        .target()
        .expect("row a is under (10, 10)");
    let panel = parent_of(&rt, a);
    assert_eq!(
        *seen.borrow(),
        vec![("enter", Some(panel))],
        "the panel was entered once, and named itself — not once per descendant \
         that crossed with it, and never under the row's name"
    );

    // And moving between the rows crosses nothing the panel cares about: the
    // panel did not change, and the rows bind nothing, so a sweep down a list is
    // silent for an application that only watches the list itself.
    rt.handle_pointer(at(PointerEventKind::Move, 10.0, 60.0));
    assert_eq!(
        *seen.borrow(),
        vec![("enter", Some(panel))],
        "no second event once the panel is already entered"
    );

    // The negative half, so the count above is attributable to the resolution
    // rule and not to the rows being unreachable: the same sweep with the rows
    // bound is exactly one leave and one enter.
    let mut rt = staged(&crossed_rows(true, false));
    let seen = watch_crossings(&mut rt);
    rt.handle_pointer(at(PointerEventKind::Move, 10.0, 10.0));
    seen.borrow_mut().clear();
    rt.handle_pointer(at(PointerEventKind::Move, 10.0, 60.0));
    assert_eq!(seen.borrow().len(), 2, "one row left, one row entered");
}

#[doc = include_str!("pointer_events.doc/prevent_default_on_a_crossing_is_inert.md")]
#[test]
fn prevent_default_on_a_crossing_is_inert() {
    // The documented refusal, and the honest finding rather than an invented
    // symmetry: a hover transition has no default action to suppress. The chain
    // is already seated when the event is dispatched, and the restyle that
    // follows is not the crossing's default action — it is the crossing, stated
    // in CSS. So the event reports the boundary and does not offer to decline
    // it.
    let view = crossed_rows(true, false);
    for refuse in [false, true] {
        let mut rt = staged(&view);
        let enters = Rc::new(Cell::new(0_u32));
        let leaves = Rc::new(Cell::new(0_u32));
        let n = Rc::clone(&enters);
        let l = Rc::clone(&leaves);
        rt.on_command(ON_ENTER, move |e: &mut UiEvent| {
            n.set(n.get() + 1);
            if refuse {
                e.prevent_default();
            }
        });
        rt.on_command(ON_LEAVE, move |_e: &mut UiEvent| l.set(l.get() + 1));

        let first = rt.handle_pointer(at(PointerEventKind::Move, 10.0, 10.0));
        let second = rt.handle_pointer(at(PointerEventKind::Move, 10.0, 60.0));

        assert_eq!(enters.get(), 2, "(refuse={refuse})");
        assert!(
            first.hover_changed() && second.hover_changed(),
            "the chain advanced either way — a refusal does not un-hover (refuse={refuse})"
        );
        assert_eq!(
            leaves.get(),
            1,
            "and the refused enter did not stop the later leave (refuse={refuse})"
        );
    }
}

#[doc = include_str!("pointer_events.doc/a_tap_enters_and_then_leaves.md")]
#[test]
fn a_tap_enters_and_then_leaves() {
    // §2.4: "Touch does not synthesize persistent hover." The crossings say the
    // same thing the hover chain does, or they would contradict it — a row that
    // was entered and never left would leave an application holding a highlight
    // for a finger that is gone.
    let view = crossed_rows(true, false);
    let mut rt = staged(&view);
    let seen = watch_crossings(&mut rt);

    let a = rt
        .handle_pointer(finger_at(PointerEventKind::Down, 10.0, 10.0))
        .target()
        .expect("row a is under the finger");
    rt.handle_pointer(finger_at(PointerEventKind::Up, 10.0, 10.0));

    assert_eq!(
        *seen.borrow(),
        vec![("enter", Some(a)), ("leave", Some(a))],
        "the contact entered the row and the lift left it"
    );

    // The negative half, and the reason this is a device rule rather than an
    // event-kind rule: a mouse has a resting cursor, so the same press and
    // release leave the row entered.
    let mut rt = staged(&view);
    let seen = watch_crossings(&mut rt);
    let same_row = rt
        .handle_pointer(at(PointerEventKind::Down, 10.0, 10.0))
        .target()
        .expect("row a is under the cursor");
    rt.handle_pointer(at(PointerEventKind::Up, 10.0, 10.0));
    assert_eq!(
        *seen.borrow(),
        vec![("enter", Some(same_row))],
        "a released mouse is still over the row"
    );
}

#[doc = include_str!("pointer_events.doc/an_unbound_crossing_name_is_never_dispatched.md")]
#[test]
fn an_unbound_crossing_name_is_never_dispatched() {
    // The cost claim for the crossings. A chain change is not rare — sweeping
    // down a list crosses two boundaries per row — so the same per-name gate the
    // four raw kinds get has to cover these two, and it is the same bits.
    let leave_only = View::single(
        ViewNode::element(CONTAINER)
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into())))
            .with_children(ViewFragment::from_nodes(vec![
                ViewNode::element(BUTTON)
                    .with_prop(ViewProperty::new(CLASS, ViewValue::Text("a".into())))
                    .with_event(ViewEventBinding::new(EVENT_POINTER_LEAVE, ON_LEAVE)),
                ViewNode::element(BUTTON)
                    .with_prop(ViewProperty::new(CLASS, ViewValue::Text("b".into())))
                    .with_event(ViewEventBinding::new(EVENT_POINTER_LEAVE, ON_LEAVE)),
            ])),
    );
    let mut rt = staged(&leave_only);
    let enters = Rc::new(Cell::new(0_u32));
    let leaves = Rc::new(Cell::new(0_u32));
    let n = Rc::clone(&enters);
    let l = Rc::clone(&leaves);
    rt.on_command(ON_ENTER, move || n.set(n.get() + 1));
    rt.on_command(ON_LEAVE, move || l.set(l.get() + 1));

    for _ in 0..8 {
        rt.handle_pointer(at(PointerEventKind::Move, 10.0, 10.0));
        rt.handle_pointer(at(PointerEventKind::Move, 10.0, 60.0));
    }
    assert_eq!(
        enters.get(),
        0,
        "nothing binds `pointerenter`, so nothing ran"
    );
    // The positive half in the same tree, so the gate is per-name and not a
    // blanket off switch: sixteen sweeps, fifteen boundaries left.
    assert_eq!(leaves.get(), 15, "the bound name fired on every crossing");
}
