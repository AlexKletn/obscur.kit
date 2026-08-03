#![doc = include_str!("focus_events.doc/_module.md")]
#![forbid(unsafe_code)]

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::{Point, Size};
use obscur_kit::identity::{ComponentId, InternedSymbol, NodeId, RuntimeId};
use obscur_kit::interaction::command::{KeyEvent, LogicalKey, PhysicalKey};
use obscur_kit::interaction::event::UiEvent;
use obscur_kit::interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{
    CLASS, DISABLED, EVENT_BLUR, EVENT_CHANGE, EVENT_FOCUS, EVENT_INPUT, EVENT_POINTER_DOWN,
    LOCAL_NAME, VALUE, View, ViewEventBinding, ViewFragment, ViewNode, ViewProperty, ViewValue,
};

const ON_FOCUS: InternedSymbol = InternedSymbol::from_raw(0x7F01);
const ON_BLUR: InternedSymbol = InternedSymbol::from_raw(0x7F02);
const ON_DOWN: InternedSymbol = InternedSymbol::from_raw(0x7F03);
const ON_CHANGE: InternedSymbol = InternedSymbol::from_raw(0x7F04);
const ON_EDIT: InternedSymbol = InternedSymbol::from_raw(0x7F05);

const BUTTON: ComponentId = ComponentId::from_raw(0x00D0_0001);
const INPUT: ComponentId = ComponentId::from_raw(0x00D0_0004);
const CONTAINER: ComponentId = ComponentId::from_raw(0x00C0_0001);

const CSS: &str = ".panel{width:300px;height:300px}.a{width:200px;height:40px}\
                  .b{width:200px;height:40px}";

#[doc = include_str!("focus_events.doc/Note.md")]
type Note = (&'static str, Option<NodeId>);

#[doc = include_str!("focus_events.doc/Envelope.md")]
type Envelope = (bool, u64, Option<Point>);

fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("focus_events.doc/watched.md")]
fn watched(node: ViewNode) -> ViewNode {
    node.with_event(ViewEventBinding::new(EVENT_FOCUS, ON_FOCUS))
        .with_event(ViewEventBinding::new(EVENT_BLUR, ON_BLUR))
}

#[doc = include_str!("focus_events.doc/row.md")]
fn row(class: &str, watch: bool) -> ViewNode {
    let node = ViewNode::element(BUTTON)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())));
    if watch { watched(node) } else { node }
}

#[doc = include_str!("focus_events.doc/rows.md")]
fn rows(rows_watch: bool, panel_watches: bool) -> View {
    let panel = ViewNode::element(CONTAINER)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into())))
        .with_children(ViewFragment::from_nodes(vec![
            row("a", rows_watch),
            row("b", rows_watch),
        ]));
    View::single(if panel_watches { watched(panel) } else { panel })
}

#[doc = include_str!("focus_events.doc/field.md")]
fn field(class: &str, value: &str) -> ViewNode {
    watched(
        ViewNode::element(INPUT)
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
            .with_prop(ViewProperty::new(
                LOCAL_NAME,
                ViewValue::Text("input".into()),
            ))
            .with_prop(ViewProperty::new(VALUE, ViewValue::Text(value.into())))
            // `on:input` is what makes a node an editable field to the runtime,
            // and `on:change` is what its blur commits.
            .with_event(ViewEventBinding::new(EVENT_INPUT, ON_EDIT))
            .with_event(ViewEventBinding::new(EVENT_CHANGE, ON_CHANGE))
            .with_children(ViewFragment::from_nodes(vec![ViewNode::text(
                value.to_owned(),
            )])),
    )
}

#[doc = include_str!("focus_events.doc/two_fields.md")]
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

#[doc = include_str!("focus_events.doc/staged.md")]
fn staged(view: &View) -> UiRuntime {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 300.0));
    let sheet = compile(CSS);
    let _ = rt.render_now(view, std::slice::from_ref(&sheet));
    rt
}

#[doc = include_str!("focus_events.doc/watch.md")]
fn watch(rt: &mut UiRuntime) -> Rc<RefCell<Vec<Note>>> {
    let seen: Rc<RefCell<Vec<Note>>> = Rc::new(RefCell::new(Vec::new()));
    for (command, label) in [(ON_BLUR, "blur"), (ON_FOCUS, "focus")] {
        let sink = Rc::clone(&seen);
        rt.on_command(command, move |e: &mut UiEvent| {
            sink.borrow_mut().push((label, e.target()));
        });
    }
    seen
}

#[doc = include_str!("focus_events.doc/parent_of.md")]
fn parent_of(rt: &UiRuntime, node: NodeId) -> NodeId {
    rt.tree()
        .get(node)
        .expect("a live node")
        .logical_parent()
        .parent()
        .expect("a row has a panel above it")
}

#[doc = include_str!("focus_events.doc/at.md")]
fn at(kind: PointerEventKind, x: f32, y: f32) -> PointerEvent {
    PointerEvent::new(
        PointerId::MOUSE,
        PointerDevice::Mouse,
        kind,
        Point::new(x, y),
    )
    .as_trusted()
    .with_timestamp(1_000)
}

#[doc = include_str!("focus_events.doc/press_at.md")]
fn press_at(rt: &mut UiRuntime, x: f32, y: f32) -> Option<NodeId> {
    let target = rt.handle_pointer(at(PointerEventKind::Down, x, y)).target();
    rt.handle_pointer(at(PointerEventKind::Up, x, y));
    target
}

#[doc = include_str!("focus_events.doc/an_authored_on_focus_actually_fires.md")]
#[test]
fn an_authored_on_focus_actually_fires() {
    // The gap this closes, and it is the same one `on:keydown` and
    // `on:pointerdown` had: the name compiled, hashed to a symbol, bound a
    // command — and nothing in the workspace dispatched it, so the handler never
    // ran and nothing said so. `docs/api/06-input.md` said as much in as many
    // words: "`focus`, `blur` … are not reserved names, because nothing emits
    // them."
    let view = rows(true, false);
    let mut rt = staged(&view);
    let seen = watch(&mut rt);

    let a = press_at(&mut rt, 10.0, 10.0).expect("row a is under (10, 10)");
    assert_eq!(rt.focused(), Some(a), "the press focused it");
    assert_eq!(
        *seen.borrow(),
        vec![("focus", Some(a))],
        "the handler runs, and the event names the node that gained focus"
    );

    // The negative half: an unbound node is silent. `b` binds neither name here,
    // so moving focus onto it fires `blur` on `a` and nothing else — the events
    // are resolved from bindings, not announced unconditionally.
    let mut rt = staged(&rows(false, false));
    let seen = watch(&mut rt);
    press_at(&mut rt, 10.0, 10.0);
    assert!(
        seen.borrow().is_empty(),
        "no binding, no dispatch: {:?}",
        seen.borrow()
    );
}

#[doc = include_str!("focus_events.doc/a_move_between_two_nodes_blurs_before_it_focuses.md")]
#[test]
fn a_move_between_two_nodes_blurs_before_it_focuses() {
    // THE ordering claim, and it is a sequence rather than a pair of counts: the
    // node losing focus hears about it before the node gaining it does, so a
    // teardown on blur is never interleaved with the next node's setup.
    let view = rows(true, false);
    let mut rt = staged(&view);
    let seen = watch(&mut rt);

    let a = press_at(&mut rt, 10.0, 10.0).expect("row a is under (10, 10)");
    let b = press_at(&mut rt, 10.0, 60.0).expect("row b is under (10, 60)");
    assert_ne!(a, b, "the two rows are two nodes");

    assert_eq!(
        *seen.borrow(),
        vec![
            // Arriving from nothing: `focus` alone, because there is no node to
            // blur. The first control of a session does not blur the void.
            ("focus", Some(a)),
            // And the move: both halves, blur first.
            ("blur", Some(a)),
            ("focus", Some(b)),
        ]
    );
}

#[doc = include_str!("focus_events.doc/re_pressing_the_focused_node_fires_neither.md")]
#[test]
fn re_pressing_the_focused_node_fires_neither() {
    // The negative half of the pair above, and the reason the transition is
    // guarded on inequality rather than on "a press happened": a second click
    // inside the field you are already typing in is not a blur and not a focus.
    // Firing them would also destroy the caret and the selection, which is the
    // damage `sync_editing_to_focus` returns early to avoid.
    let view = rows(true, false);
    let mut rt = staged(&view);
    let seen = watch(&mut rt);

    let a = press_at(&mut rt, 10.0, 10.0).expect("row a is under (10, 10)");
    let after_first = seen.borrow().len();
    assert_eq!(after_first, 1, "the first press focused it");

    press_at(&mut rt, 12.0, 12.0);
    press_at(&mut rt, 14.0, 14.0);
    assert_eq!(rt.focused(), Some(a), "focus never left");
    assert_eq!(
        seen.borrow().len(),
        after_first,
        "re-pressing the focused node moved nothing, so it announced nothing"
    );
}

#[doc = include_str!("focus_events.doc/clearing_focus_blurs_with_no_focus_to_follow.md")]
#[test]
fn clearing_focus_blurs_with_no_focus_to_follow() {
    // The case the pair is easiest to get wrong: focus can change with NO node
    // newly focused. A press that lands on nothing clears focus, and the node
    // that had it must still hear `blur` — a runtime that only announced
    // transitions between two nodes would go silent exactly when a form has just
    // been abandoned.
    let view = rows(true, false);
    let mut rt = staged(&view);
    let seen = watch(&mut rt);

    let a = press_at(&mut rt, 10.0, 10.0).expect("row a is under (10, 10)");
    seen.borrow_mut().clear();

    // Outside the surface: nothing is hit, so the press targets no node.
    let target = rt
        .handle_pointer(at(PointerEventKind::Down, 400.0, 400.0))
        .target();
    assert_eq!(target, None, "the press landed on nothing");
    assert_eq!(rt.focused(), None, "and so it cleared focus");
    assert_eq!(
        *seen.borrow(),
        vec![("blur", Some(a))],
        "the blur fired alone — there was no node to focus"
    );
}

#[doc = include_str!("focus_events.doc/a_press_on_a_disabled_control_blurs_nothing.md")]
#[test]
fn a_press_on_a_disabled_control_blurs_nothing() {
    // The carve-out the press path already had, now visible through the events:
    // clicking a disabled control is not a blur. Focus, the editing session and
    // the press anchor stay where they were, so there is no transition to
    // announce — and announcing one would tell an application its field had been
    // abandoned when the user had merely clicked a greyed-out button.
    let panel = ViewNode::element(CONTAINER)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into())))
        .with_children(ViewFragment::from_nodes(vec![
            row("a", true),
            row("b", true).with_prop(ViewProperty::new(DISABLED, ViewValue::Bool(true))),
        ]));
    let view = View::single(panel);
    let mut rt = staged(&view);
    let seen = watch(&mut rt);

    let a = press_at(&mut rt, 10.0, 10.0).expect("row a is under (10, 10)");
    seen.borrow_mut().clear();

    press_at(&mut rt, 10.0, 60.0);
    assert_eq!(rt.focused(), Some(a), "focus stayed on the enabled row");
    assert!(
        seen.borrow().is_empty(),
        "no move, no pair: {:?}",
        seen.borrow()
    );
}

#[doc = include_str!("focus_events.doc/prevent_default_on_the_pair_is_inert.md")]
#[test]
fn prevent_default_on_the_pair_is_inert() {
    // The documented refusal, asserted rather than only written down. By the time
    // either half is dispatched the focus has already moved, so there is no
    // default action left to suppress and `prevent_default` changes nothing. No
    // suppression was invented to make the story symmetrical.
    let view = rows(true, false);
    let mut rt = staged(&view);
    rt.on_command(ON_BLUR, |e: &mut UiEvent| e.prevent_default());
    rt.on_command(ON_FOCUS, |e: &mut UiEvent| e.prevent_default());

    let a = press_at(&mut rt, 10.0, 10.0).expect("row a is under (10, 10)");
    assert_eq!(rt.focused(), Some(a), "the refused focus landed anyway");
    let b = press_at(&mut rt, 10.0, 60.0).expect("row b is under (10, 60)");
    assert_eq!(
        rt.focused(),
        Some(b),
        "and the refused blur did not hold focus back"
    );
}

#[doc = include_str!("focus_events.doc/pointerdown_is_where_a_focus_move_is_refused.md")]
#[test]
fn pointerdown_is_where_a_focus_move_is_refused() {
    // The other side of the same claim, and the reason the pair does not need to
    // be cancellable: the refusal exists, one event earlier. A `pointerdown`
    // runs BEFORE the press moves focus, so refusing it suppresses the move —
    // and therefore the pair, which describes a move that did not happen.
    //
    // Bound on the panel, which the raw pointer binding reaches by bubbling from
    // either row; the focus pair, one level down, does not bubble.
    let view = View::single(
        ViewNode::element(CONTAINER)
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into())))
            .with_event(ViewEventBinding::new(EVENT_POINTER_DOWN, ON_DOWN))
            .with_children(ViewFragment::from_nodes(vec![
                row("a", true),
                row("b", true),
            ])),
    );
    let mut rt = staged(&view);
    let seen = watch(&mut rt);
    let a = press_at(&mut rt, 10.0, 10.0).expect("row a is under (10, 10)");
    seen.borrow_mut().clear();

    rt.on_command(ON_DOWN, |e: &mut UiEvent| e.prevent_default());
    press_at(&mut rt, 10.0, 60.0);
    assert_eq!(rt.focused(), Some(a), "the refused press moved no focus");
    assert!(
        seen.borrow().is_empty(),
        "and so announced no move: {:?}",
        seen.borrow()
    );

    // The negative half, so the silence above is attributable to the refusal and
    // not to row `b` being unreachable: the same press, unrefused, moves focus
    // and announces both halves.
    let mut rt = staged(&view);
    let seen = watch(&mut rt);
    let a = press_at(&mut rt, 10.0, 10.0).expect("row a is under (10, 10)");
    seen.borrow_mut().clear();
    rt.on_command(ON_DOWN, |_e: &mut UiEvent| {});
    let b = press_at(&mut rt, 10.0, 60.0).expect("row b is under (10, 60)");
    assert_eq!(rt.focused(), Some(b));
    assert_eq!(*seen.borrow(), vec![("blur", Some(a)), ("focus", Some(b))]);
}

#[doc = include_str!("focus_events.doc/the_change_a_blur_commits_arrives_before_the_blur.md")]
#[test]
fn the_change_a_blur_commits_arrives_before_the_blur() {
    // The ordering a form relies on. A field losing focus commits its `change`
    // (spec §5.11) and then hears `blur`, which is the order HTML fires the two
    // in — so a handler reading the field's value on blur sees the committed
    // one rather than the value focus arrived with.
    let view = two_fields();
    let mut rt = staged(&view);
    let seen: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    for (command, label) in [(ON_BLUR, "blur"), (ON_FOCUS, "focus")] {
        let sink = Rc::clone(&seen);
        rt.on_command(command, move |_e: &mut UiEvent| {
            sink.borrow_mut().push(label.to_owned());
        });
    }
    {
        let sink = Rc::clone(&seen);
        rt.on_change(ON_CHANGE, move |text: &str| {
            sink.borrow_mut().push(format!("change:{text}"));
        });
    }

    press_at(&mut rt, 10.0, 10.0);
    rt.handle_key(
        KeyEvent::press(PhysicalKey::KeyX, Some(LogicalKey::character('x'))).as_trusted(),
    );
    // Read the live buffer rather than spelling the expected string: *where* the
    // click put the caret is another test's claim, and this one is about order.
    let typed = rt.editing_text().expect("a live session").to_owned();
    assert_ne!(typed, "alpha", "the key edited the field");
    seen.borrow_mut().clear();

    press_at(&mut rt, 10.0, 60.0);
    assert_eq!(
        *seen.borrow(),
        vec![format!("change:{typed}"), "blur".into(), "focus".into()],
        "the commit, then the blur it belongs to, then the next field's focus"
    );
}

#[doc = include_str!("focus_events.doc/the_binding_resolves_through_the_ancestor_chain.md")]
#[test]
fn the_binding_resolves_through_the_ancestor_chain() {
    // The resolution rule, and the reason it is not the DOM's. The DOM focuses
    // elements and its `focus` does not bubble; this runtime focuses whatever
    // the press HIT, and a press on a field's glyphs hits the field's text leaf.
    // So an `<input on:focus>` resolved on the focused node alone would fire or
    // not depending on whether the user clicked the text or the padding — which
    // is the failure this half pins.
    let view = two_fields();
    let mut rt = staged(&view);
    let seen = watch(&mut rt);
    let hit = press_at(&mut rt, 10.0, 10.0).expect("field a is under (10, 10)");
    assert!(
        rt.tree()
            .get(hit)
            .expect("a live node")
            .kind()
            .text()
            .is_some(),
        "the press landed on the field's TEXT, which is what makes this the hard case"
    );
    let a = parent_of(&rt, hit);
    assert_eq!(
        *seen.borrow(),
        vec![("focus", Some(a))],
        "the field heard its own focus, and the event names the field rather than the leaf"
    );

    // The other half: the walk stops at the first binding, so a nested pair is
    // one event and not one per level. Pressing the second field blurs the first
    // and focuses the second — two notes, not four.
    seen.borrow_mut().clear();
    let b_hit = press_at(&mut rt, 10.0, 60.0).expect("field b is under (10, 60)");
    let b = if rt
        .tree()
        .get(b_hit)
        .expect("a live node")
        .kind()
        .text()
        .is_some()
    {
        parent_of(&rt, b_hit)
    } else {
        b_hit
    };
    assert_ne!(a, b, "the two fields are two nodes");
    assert_eq!(*seen.borrow(), vec![("blur", Some(a)), ("focus", Some(b))]);
}

#[doc = include_str!("focus_events.doc/an_ancestor_binding_is_what_a_descendants_focus_resolves_to.md")]
#[test]
fn an_ancestor_binding_is_what_a_descendants_focus_resolves_to() {
    // The consequence of walking, stated rather than left to be discovered: a
    // panel's `on:focus` fires for a row inside it. That is the DOM's `focusin`,
    // and it is exactly why no `focusin` is reserved here — it would be a second
    // name for this one.
    let mut rt = staged(&rows(false, true));
    let seen = watch(&mut rt);
    let a = press_at(&mut rt, 10.0, 10.0).expect("row a is under (10, 10)");
    let panel = parent_of(&rt, a);
    assert_ne!(a, panel, "the row and the panel are two nodes");
    assert_eq!(
        *seen.borrow(),
        vec![("focus", Some(panel))],
        "the panel's binding resolved for the row's focus, and names the panel"
    );

    // And the nearest binding wins, so an ancestor never doubles a row that
    // binds the pair itself. Both rows and the panel watch here; the row's own
    // binding is what resolves, once.
    let view = View::single(
        watched(
            ViewNode::element(CONTAINER)
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into()))),
        )
        .with_children(ViewFragment::from_nodes(vec![
            row("a", true),
            row("b", true),
        ])),
    );
    let mut rt = staged(&view);
    let seen = watch(&mut rt);
    let a = press_at(&mut rt, 10.0, 10.0).expect("row a is under (10, 10)");
    assert_eq!(
        *seen.borrow(),
        vec![("focus", Some(a))],
        "one note, naming the row — the panel above it did not also hear it"
    );

    // And a command still routes past whatever resolved it: an `on_command_at`
    // listener on the panel sees the row's focus in the bubble phase, which is
    // the §2.6 half of the same delivery.
    let heard = Rc::new(Cell::new(0_u32));
    let count = Rc::clone(&heard);
    let panel = parent_of(&rt, a);
    rt.on_command_at(panel, ON_FOCUS, move |_e: &mut UiEvent| {
        count.set(count.get() + 1);
    });
    press_at(&mut rt, 10.0, 60.0);
    assert_eq!(
        heard.get(),
        1,
        "the ancestor listener heard the row's focus"
    );
}

#[doc = include_str!("focus_events.doc/the_envelope_carries_the_event_that_moved_focus.md")]
#[test]
fn the_envelope_carries_the_event_that_moved_focus() {
    // §2.3's trust and §2.4's payload reach the handler, because the envelope is
    // the causing event rather than a synthetic stand-in. A `blur` handler can
    // therefore see *what* took focus away — today always a pointer press, and
    // the position it landed at.
    let view = rows(true, false);
    let mut rt = staged(&view);
    let seen: Rc<RefCell<Vec<Envelope>>> = Rc::new(RefCell::new(Vec::new()));
    let sink = Rc::clone(&seen);
    rt.on_command(ON_FOCUS, move |e: &mut UiEvent| {
        sink.borrow_mut().push((
            e.is_trusted(),
            e.timestamp(),
            e.pointer().map(|p| p.position()),
        ));
    });

    press_at(&mut rt, 10.0, 10.0);
    assert_eq!(
        *seen.borrow(),
        vec![(true, 1_000, Some(Point::new(10.0, 10.0)))],
        "trust, the host clock and the press that conferred them"
    );
}
