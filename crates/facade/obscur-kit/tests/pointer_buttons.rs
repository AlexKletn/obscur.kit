#![doc = include_str!("pointer_buttons.doc/_module.md")]
#![forbid(unsafe_code)]

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::{Point, Size};
use obscur_kit::identity::{ComponentId, InternedSymbol, NodeId, RuntimeId};
use obscur_kit::interaction::event::UiEvent;
use obscur_kit::interaction::pointer::{
    PointerButton, PointerButtons, PointerDevice, PointerEvent, PointerEventKind, PointerId,
};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{
    CLASS, EVENT_CLICK, EVENT_CONTEXT_MENU, EVENT_POINTER_DOWN, EVENT_POINTER_LEAVE, LOCAL_NAME,
    View, ViewEventBinding, ViewFragment, ViewNode, ViewProperty, ViewValue,
};

const ON_CLICK: InternedSymbol = InternedSymbol::from_raw(0x7F01);
const ON_MENU: InternedSymbol = InternedSymbol::from_raw(0x7F02);
const ON_DOWN: InternedSymbol = InternedSymbol::from_raw(0x7F03);
const ON_LEAVE: InternedSymbol = InternedSymbol::from_raw(0x7F04);

const BUTTON: ComponentId = ComponentId::from_raw(0x00D0_0001);
const CONTAINER: ComponentId = ComponentId::from_raw(0x00C0_0001);

const CSS: &str = ".panel{width:300px;height:300px}.a{width:200px;height:40px}";

fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("pointer_buttons.doc/control.md")]
fn control() -> View {
    View::single(
        ViewNode::element(CONTAINER)
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into())))
            .with_children(ViewFragment::from_nodes(vec![
                ViewNode::element(BUTTON)
                    .with_prop(ViewProperty::new(CLASS, ViewValue::Text("a".into())))
                    .with_prop(ViewProperty::new(
                        LOCAL_NAME,
                        ViewValue::Text("button".into()),
                    ))
                    .with_event(ViewEventBinding::new(EVENT_CLICK, ON_CLICK))
                    .with_event(ViewEventBinding::new(EVENT_CONTEXT_MENU, ON_MENU))
                    .with_event(ViewEventBinding::new(EVENT_POINTER_DOWN, ON_DOWN))
                    .with_event(ViewEventBinding::new(EVENT_POINTER_LEAVE, ON_LEAVE)),
            ])),
    )
}

#[doc = include_str!("pointer_buttons.doc/staged.md")]
fn staged() -> UiRuntime {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 300.0));
    let sheet = compile(CSS);
    let _ = rt.render_now(&control(), std::slice::from_ref(&sheet));
    rt
}

#[doc = include_str!("pointer_buttons.doc/the_control.md")]
fn the_control(rt: &UiRuntime) -> NodeId {
    rt.tree()
        .iter()
        .find_map(|(id, node)| (node.kind().component_id() == Some(BUTTON)).then_some(id))
        .expect("the staged control is in the committed tree")
}

#[doc = include_str!("pointer_buttons.doc/held.md")]
fn held(buttons: &[PointerButton]) -> PointerButtons {
    let mut set = PointerButtons::new();
    for button in buttons {
        set.press(*button);
    }
    set
}

#[doc = include_str!("pointer_buttons.doc/at.md")]
fn at(kind: PointerEventKind, button: PointerButton, after: &[PointerButton]) -> PointerEvent {
    PointerEvent::new(
        PointerId::MOUSE,
        PointerDevice::Mouse,
        kind,
        Point::new(10.0, 10.0),
    )
    .with_buttons(held(after))
    .with_button(button)
    .as_trusted()
}

#[doc = include_str!("pointer_buttons.doc/press.md")]
fn press(button: PointerButton) -> PointerEvent {
    at(PointerEventKind::Down, button, &[button])
}

#[doc = include_str!("pointer_buttons.doc/release.md")]
fn release(button: PointerButton) -> PointerEvent {
    at(PointerEventKind::Up, button, &[])
}

#[doc = include_str!("pointer_buttons.doc/counter.md")]
fn counter(rt: &mut UiRuntime, command: InternedSymbol) -> Rc<Cell<u32>> {
    let runs = Rc::new(Cell::new(0_u32));
    let sink = Rc::clone(&runs);
    rt.on_command(command, move || sink.set(sink.get() + 1));
    runs
}

#[doc = include_str!("pointer_buttons.doc/an_authored_on_contextmenu_fires_on_a_right_press.md")]
#[test]
fn an_authored_on_contextmenu_fires_on_a_right_press() {
    // The gap this closes is two gaps that were one: `contextmenu` was not a
    // reserved name because nothing emitted it, and nothing emitted it because
    // the runner dropped every button but the left one before the runtime ever
    // saw it.
    let mut rt = staged();
    let seen: Rc<RefCell<Vec<Option<PointerButton>>>> = Rc::new(RefCell::new(Vec::new()));
    let sink = Rc::clone(&seen);
    rt.on_command(ON_MENU, move |e: &mut UiEvent| {
        sink.borrow_mut().push(e.pointer().and_then(|p| p.button()));
    });

    rt.handle_pointer(press(PointerButton::Secondary));
    assert_eq!(
        seen.borrow().as_slice(),
        [Some(PointerButton::Secondary)],
        "the handler runs and reads the moved button off the envelope"
    );

    // The negative half, so the assertion above is attributable to the button
    // and not merely to a press: the primary one asks for no menu.
    let mut rt = staged();
    let menus = counter(&mut rt, ON_MENU);
    rt.handle_pointer(press(PointerButton::Primary));
    rt.handle_pointer(release(PointerButton::Primary));
    assert_eq!(menus.get(), 0, "a left click is not a context request");

    // And neither does the middle one, which is the button most likely to be
    // mistaken for "any non-primary press".
    let mut rt = staged();
    let menus = counter(&mut rt, ON_MENU);
    rt.handle_pointer(press(PointerButton::Middle));
    assert_eq!(menus.get(), 0);
}

#[doc = include_str!("pointer_buttons.doc/a_secondary_press_focuses_and_refusing_the_menu_stops_it.md")]
#[test]
fn a_secondary_press_focuses_and_refusing_the_menu_stops_it() {
    // What refusing `contextmenu` suppresses, established rather than assumed:
    // the focus move is the whole of what a secondary press performs, so it is
    // the whole of what the refusal reaches. Right-clicking a control focuses it
    // — a menu command acts on what has focus — and an application drawing its
    // own menu can decline even that.
    let mut rt = staged();
    rt.on_command(ON_MENU, |_e: &mut UiEvent| {});
    rt.handle_pointer(press(PointerButton::Secondary));
    let control = the_control(&rt);
    assert_eq!(
        rt.focused(),
        Some(control),
        "an unrefused right press focuses what it landed on"
    );

    let mut rt = staged();
    rt.on_command(ON_MENU, UiEvent::prevent_default);
    rt.handle_pointer(press(PointerButton::Secondary));
    assert_eq!(
        rt.focused(),
        None,
        "and refusing it leaves focus exactly where it was"
    );
}

#[doc = include_str!("pointer_buttons.doc/a_right_click_does_not_activate_the_control.md")]
#[test]
fn a_right_click_does_not_activate_the_control() {
    // The regression that delivering the other four buttons would otherwise
    // introduce, and the loudest one: every default action of a press is written
    // against a press, so a right or middle click would have run `on:click`.
    for button in [PointerButton::Secondary, PointerButton::Middle] {
        let mut rt = staged();
        let clicks = counter(&mut rt, ON_CLICK);
        rt.handle_pointer(press(button));
        rt.handle_pointer(release(button));
        assert_eq!(clicks.get(), 0, "{button:?} must not activate the control");
    }

    // The positive half, so "does not activate" is not "activation is broken".
    let mut rt = staged();
    let clicks = counter(&mut rt, ON_CLICK);
    rt.handle_pointer(press(PointerButton::Primary));
    rt.handle_pointer(release(PointerButton::Primary));
    assert_eq!(clicks.get(), 1, "the primary button is what engages");
}

#[doc = include_str!("pointer_buttons.doc/a_chord_is_a_real_state.md")]
#[test]
fn a_chord_is_a_real_state() {
    // Two buttons held at once, which is unrepresentable if the held set is
    // guessed per event. The failure it prevents is concrete: the press anchor
    // is per pointer, not per button, so a secondary release arriving while the
    // primary is still down would consume that press — and click the control the
    // user has not let go of yet.
    let mut rt = staged();
    let clicks = counter(&mut rt, ON_CLICK);

    rt.handle_pointer(at(
        PointerEventKind::Down,
        PointerButton::Primary,
        &[PointerButton::Primary],
    ));
    rt.handle_pointer(at(
        PointerEventKind::Down,
        PointerButton::Secondary,
        &[PointerButton::Primary, PointerButton::Secondary],
    ));
    rt.handle_pointer(at(
        PointerEventKind::Up,
        PointerButton::Secondary,
        &[PointerButton::Primary],
    ));
    assert_eq!(
        clicks.get(),
        0,
        "the primary button is still down, so nothing has been clicked"
    );

    rt.handle_pointer(at(PointerEventKind::Up, PointerButton::Primary, &[]));
    assert_eq!(
        clicks.get(),
        1,
        "and letting it go clicks once — the press survived the other release"
    );
}

#[doc = include_str!("pointer_buttons.doc/the_raw_press_reports_every_button.md")]
#[test]
fn the_raw_press_reports_every_button() {
    // `pointerdown` is a report of the device, not of the primary button: an
    // application watching presses hears all five, and the envelope says which.
    // This is the direction `a_right_click_does_not_activate_the_control` must
    // not be read too far in — the button is suppressed from *engaging*, not
    // from being reported.
    for button in [
        PointerButton::Primary,
        PointerButton::Secondary,
        PointerButton::Middle,
        PointerButton::Back,
        PointerButton::Forward,
    ] {
        let mut rt = staged();
        let seen: Rc<Cell<Option<PointerButton>>> = Rc::new(Cell::new(None));
        let sink = Rc::clone(&seen);
        rt.on_command(ON_DOWN, move |e: &mut UiEvent| {
            sink.set(e.pointer().and_then(|p| p.button()));
        });
        rt.handle_pointer(press(button));
        assert_eq!(seen.get(), Some(button), "{button:?}");
    }
}

#[doc = include_str!("pointer_buttons.doc/a_pointer_that_left_the_window_hovers_nothing.md")]
#[test]
fn a_pointer_that_left_the_window_hovers_nothing() {
    // `WindowEvent::CursorLeft` was unhandled, so `:hover` never cleared when the
    // pointer left the window — the last row it was over stayed highlighted for
    // the rest of the session. Converting the event is only half of it: a leave
    // carries the position the pointer went out by, so the hit test still finds
    // the node under it, and honouring that answer would leave the chain exactly
    // where the bug did.
    let mut rt = staged();
    let control = the_control(&rt);
    let left = counter(&mut rt, ON_LEAVE);

    let moved = rt.handle_pointer(
        PointerEvent::new(
            PointerId::MOUSE,
            PointerDevice::Mouse,
            PointerEventKind::Move,
            Point::new(10.0, 10.0),
        )
        .as_trusted(),
    );
    assert!(moved.hover_changed());
    assert_eq!(
        rt.hovered(),
        Some(control),
        "the pointer is over the control"
    );
    assert_eq!(left.get(), 0);

    let gone = rt.handle_pointer(
        PointerEvent::new(
            PointerId::MOUSE,
            PointerDevice::Mouse,
            PointerEventKind::Leave,
            // Deliberately the position it went out by, which is still over the
            // control: that is what the runner has to send, because winit's
            // `CursorLeft` carries no position of its own.
            Point::new(10.0, 10.0),
        )
        .as_trusted(),
    );
    assert!(gone.hover_changed());
    assert_eq!(
        rt.hovered(),
        None,
        "nothing is under a pointer that is gone"
    );
    assert_eq!(
        left.get(),
        1,
        "and the node it was over hears its own pointerleave"
    );
}
