#![doc = include_str!("keyboard_events.doc/_module.md")]
#![forbid(unsafe_code)]

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::{Point, Size};
use obscur_kit::identity::{ComponentId, InternedSymbol, NodeId, RuntimeId};
use obscur_kit::interaction::command::{
    KeyEvent, LogicalKey, Modifier, Modifiers, NamedKey, PhysicalKey, Shortcut,
};
use obscur_kit::interaction::event::UiEvent;
use obscur_kit::interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{
    CLASS, EVENT_INPUT, EVENT_KEY_DOWN, EVENT_KEY_UP, LOCAL_NAME, VALUE, View, ViewEventBinding,
    ViewNode, ViewProperty, ViewValue,
};

const ON_KEY: InternedSymbol = InternedSymbol::from_raw(0x7D01);
const ON_KEY_UP: InternedSymbol = InternedSymbol::from_raw(0x7D02);
const ON_EDIT: InternedSymbol = InternedSymbol::from_raw(0x7D03);
const SAVE: InternedSymbol = InternedSymbol::from_raw(0x7D04);

const CSS: &str = ".field{width:200px;height:40px}";

fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("keyboard_events.doc/field.md")]
fn field(value: &str) -> View {
    View::single(
        ViewNode::element(ComponentId::from_raw(0x00D0_0004))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("field".into())))
            .with_prop(ViewProperty::new(
                LOCAL_NAME,
                ViewValue::Text("input".into()),
            ))
            .with_prop(ViewProperty::new(VALUE, ViewValue::Text(value.into())))
            .with_event(ViewEventBinding::new(EVENT_INPUT, ON_EDIT))
            .with_event(ViewEventBinding::new(EVENT_KEY_DOWN, ON_KEY))
            .with_event(ViewEventBinding::new(EVENT_KEY_UP, ON_KEY_UP)),
    )
}

#[doc = include_str!("keyboard_events.doc/focused_field.md")]
fn focused_field(value: &str) -> (UiRuntime, NodeId) {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 300.0));
    let sheet = compile(CSS);
    let view = field(value);
    let _ = rt.render_now(&view, std::slice::from_ref(&sheet));

    // Press inside the field so it takes focus, the way a user would.
    for kind in [PointerEventKind::Down, PointerEventKind::Up] {
        rt.handle_pointer(
            PointerEvent::new(
                PointerId::MOUSE,
                PointerDevice::Mouse,
                kind,
                Point::new(10.0, 10.0),
            )
            .as_trusted(),
        );
    }
    let node = rt.focused().expect("the press focused the field");
    (rt, node)
}

fn press(ch: char) -> KeyEvent {
    KeyEvent::press(physical_of(ch), Some(LogicalKey::character(ch))).as_trusted()
}

fn physical_of(ch: char) -> PhysicalKey {
    match ch {
        'a' => PhysicalKey::KeyA,
        's' => PhysicalKey::KeyS,
        _ => PhysicalKey::KeyB,
    }
}

#[doc = include_str!("keyboard_events.doc/an_authored_on_keydown_actually_fires.md")]
#[test]
fn an_authored_on_keydown_actually_fires() {
    // The gap this closes: `on:keydown="…"` compiled, hashed to a symbol, bound a
    // command — and nothing in the workspace ever dispatched it, so the handler
    // was never called and nothing said so. Four reserved event names existed;
    // `keydown` was not one of them.
    let (mut rt, _node) = focused_field("");
    let seen: Rc<RefCell<Vec<(char, bool)>>> = Rc::new(RefCell::new(Vec::new()));
    let sink = Rc::clone(&seen);
    rt.on_command(ON_KEY, move |e: &mut UiEvent| {
        if let Some(key) = e.key()
            && let Some(LogicalKey::Character(ch)) = key.logical()
        {
            sink.borrow_mut().push((ch, key.is_trusted()));
        }
    });

    rt.handle_key(press('a'));
    assert_eq!(
        seen.borrow().as_slice(),
        [('a', true)],
        "the handler runs, reads the key off the envelope, and sees its trust"
    );
}

#[doc = include_str!("keyboard_events.doc/keyup_is_delivered_too.md")]
#[test]
fn keyup_is_delivered_too() {
    let (mut rt, _node) = focused_field("");
    let ups = Rc::new(Cell::new(0_u32));
    let downs = Rc::new(Cell::new(0_u32));
    let u = Rc::clone(&ups);
    let d = Rc::clone(&downs);
    rt.on_command(ON_KEY_UP, move || u.set(u.get() + 1));
    rt.on_command(ON_KEY, move || d.set(d.get() + 1));

    rt.handle_key(press('a'));
    rt.handle_key(
        KeyEvent::release(PhysicalKey::KeyA, Some(LogicalKey::character('a'))).as_trusted(),
    );

    assert_eq!(downs.get(), 1, "one press, one keydown");
    assert_eq!(
        ups.get(),
        1,
        "one release, one keyup — and not a second keydown"
    );
}

#[doc = include_str!("keyboard_events.doc/prevent_default_on_keydown_suppresses_the_text_edit.md")]
#[test]
fn prevent_default_on_keydown_suppresses_the_text_edit() {
    // THE acceptance test for this change. `prevent_default` was consulted only
    // for §2.6's routing chain; the runtime's own default actions ran regardless,
    // and on the key path the text edit ran BEFORE any dispatch at all — so no
    // listener could refuse it however correctly it was written.
    let (mut rt, _node) = focused_field("");
    rt.on_command(ON_KEY, |e: &mut UiEvent| e.prevent_default());
    rt.on_command(ON_EDIT, |_e: &mut UiEvent| {});

    rt.handle_key(press('a'));
    assert_eq!(
        rt.editing_text(),
        Some(""),
        "the refused character never reached the editing session"
    );

    // And the negative half, so the assertion above is attributable to the
    // refusal rather than to the field being inert.
    let (mut rt, _node) = focused_field("");
    rt.on_command(ON_KEY, |_e: &mut UiEvent| {});
    rt.on_command(ON_EDIT, |_e: &mut UiEvent| {});
    rt.handle_key(press('a'));
    assert_eq!(
        rt.editing_text(),
        Some("a"),
        "an unrefused character edits the field as it always did"
    );
}

#[doc = include_str!("keyboard_events.doc/a_shortcut_resolves_with_nothing_focused.md")]
#[test]
fn a_shortcut_resolves_with_nothing_focused() {
    // §2.6's route ends at the window surface and the application. Those two
    // links were unreachable from the keyboard: `handle_key` returned early when
    // nothing was focused, so `Ctrl+S` did nothing until the user had clicked
    // something first.
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 300.0));
    let sheet = compile(CSS);
    let _ = rt.render_now(&field(""), std::slice::from_ref(&sheet));
    assert_eq!(rt.focused(), None);

    rt.bind_command(
        Shortcut::new(LogicalKey::character('s')).with(Modifier::Ctrl),
        SAVE,
    );
    let saves = Rc::new(Cell::new(0_u32));
    let s = Rc::clone(&saves);
    rt.on_command(SAVE, move || s.set(s.get() + 1));

    rt.handle_key(KeyEvent::press(PhysicalKey::ControlLeft, None).as_trusted());
    let dispatch = rt.handle_key(press('s'));

    assert_eq!(
        saves.get(),
        1,
        "the chord ran its handler with nothing focused"
    );
    assert_eq!(dispatch.command(), Some(SAVE));
    assert_eq!(
        dispatch.target(),
        None,
        "and it targeted no node, because there is none"
    );
}

#[doc = include_str!("keyboard_events.doc/the_envelope_carries_the_chord.md")]
#[test]
fn the_envelope_carries_the_chord() {
    let (mut rt, _node) = focused_field("");
    let chord: Rc<Cell<Modifiers>> = Rc::new(Cell::new(Modifiers::new()));
    let sink = Rc::clone(&chord);
    rt.on_command(ON_KEY, move |e: &mut UiEvent| {
        if let Some(key) = e.key() {
            sink.set(key.modifiers());
        }
    });

    rt.handle_key(KeyEvent::press(PhysicalKey::ControlLeft, None).as_trusted());
    rt.handle_key(press('a'));

    assert!(
        chord.get().contains(Modifier::Ctrl),
        "the handler reads the chord off the event it holds, not off the runtime"
    );
}

#[doc = include_str!("keyboard_events.doc/a_correction_replaces_the_folded_chord.md")]
#[test]
fn a_correction_replaces_the_folded_chord() {
    // The stale-modifier case: a modifier released while the window was not
    // focused produces no key event, so the fold keeps believing it is held.
    let (mut rt, _node) = focused_field("");
    rt.handle_key(KeyEvent::press(PhysicalKey::AltLeft, None).as_trusted());
    assert!(rt.held_modifiers().contains(Modifier::Alt));

    rt.set_modifiers(Modifiers::new());
    assert!(
        rt.held_modifiers().is_empty(),
        "the platform's answer replaces the fold rather than merging with it"
    );
}

#[doc = include_str!("keyboard_events.doc/an_auto_repeat_does_not_re_activate.md")]
#[test]
fn an_auto_repeat_does_not_re_activate() {
    // A button with Space resting on it must not run its command at the
    // platform's repeat rate.
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 300.0));
    let sheet = compile(".field{width:200px;height:40px}");
    let button = View::single(
        ViewNode::element(ComponentId::from_raw(0x00D0_0001))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("field".into())))
            .with_event(ViewEventBinding::new(
                obscur_kit::view::view::EVENT_CLICK,
                SAVE,
            )),
    );
    let _ = rt.render_now(&button, std::slice::from_ref(&sheet));
    for kind in [PointerEventKind::Down, PointerEventKind::Up] {
        rt.handle_pointer(
            PointerEvent::new(
                PointerId::MOUSE,
                PointerDevice::Mouse,
                kind,
                Point::new(10.0, 10.0),
            )
            .as_trusted(),
        );
    }
    let runs = Rc::new(Cell::new(0_u32));
    let r = Rc::clone(&runs);
    rt.on_command(SAVE, move || r.set(r.get() + 1));

    let space = || {
        KeyEvent::press(PhysicalKey::Space, Some(LogicalKey::named(NamedKey::Space))).as_trusted()
    };
    rt.handle_key(space());
    assert_eq!(runs.get(), 1, "the first press activates");
    rt.handle_key(space().as_repeat());
    rt.handle_key(space().as_repeat());
    assert_eq!(
        runs.get(),
        1,
        "and the platform's repeats do not activate again"
    );
}
