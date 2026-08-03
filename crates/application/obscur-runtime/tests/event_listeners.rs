#![doc = include_str!("event_listeners.doc/_module.md")]
#![forbid(unsafe_code)]

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use obscur_geometry::Point;
use obscur_identity::{InternedSymbol, NodeId};
use obscur_interaction::command::{KeyEvent, LogicalKey, NamedKey, PhysicalKey};
use obscur_interaction::event::{EventOrigin, EventPhase, EventTrust, ListenerPhase, UiEvent};
use obscur_interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};
use obscur_runtime::UiRuntime;
use obscur_style::matching::Stylesheet;
use obscur_testkit::assertion::counting_handler;
use obscur_testkit::fixture::{InteractionPanel, TestFixture, mouse};
use obscur_view::view::View;

/// One command symbol, bound by BOTH rows of [`two_row_view`].
const CMD_PICK: InternedSymbol = InternedSymbol::from_raw(0x9E11);

/// What the phase-order case records per step: the phase, `current_target`,
/// and `target` — the last of which must never change.
type PhaseLog = Rc<RefCell<Vec<(EventPhase, Option<NodeId>, Option<NodeId>)>>>;

/// What the keyboard-activation case records: the key, the origin, and
/// whether the event was trusted.
type KeySeen = Rc<Cell<Option<(Option<KeyEvent>, EventOrigin, bool)>>>;

/// A `panel` (100x100) holding `a` (y∈[0,50)) and `b` (y∈[50,100)), **both**
/// binding `on-click → CMD_PICK`.
///
/// One symbol on two nodes is the shape that used to be inexpressible: a
/// zero-argument handler is told only *that* the command fired, so the only
/// way to learn *which* row it came from was a bespoke signature
/// (`on_row_click(id, FnMut(usize, u32))`), carved out for the one case that
/// forced it. With the event delivered, `e.target()` answers it for every
/// case.
fn two_row_view() -> View {
    InteractionPanel::command_view(CMD_PICK, CMD_PICK)
}

/// A runtime with [`two_row_view`] rendered; returns `(runtime, panel, a, b)`.
fn two_rows(sheets: &[Stylesheet]) -> (UiRuntime, NodeId, NodeId, NodeId) {
    let mut runtime = TestFixture::runtime();
    let _ = runtime.render_now(&two_row_view(), sheets);
    let (panel, a, b) = InteractionPanel::nodes(&runtime);
    (runtime, panel, a, b)
}

/// Drives a trusted press-and-release at `(x, y)` — a real click.
fn click_at(runtime: &mut UiRuntime, x: f32, y: f32) {
    runtime.handle_pointer(mouse(PointerEventKind::Down, x, y));
    runtime.handle_pointer(mouse(PointerEventKind::Up, x, y));
}

/// §2.3 "- `target`;". **The case that was impossible.** One command symbol
/// is bound by two sibling nodes; a single handler learns which one the user
/// hit, from the event.
#[test]
fn one_symbol_on_two_nodes_and_the_handler_learns_which_was_hit() {
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let (mut runtime, _panel, a, b) = two_rows(sheets);

    let hit = Rc::new(Cell::new(None));
    {
        let sink = Rc::clone(&hit);
        runtime.on_command(CMD_PICK, move |e: &mut UiEvent| sink.set(e.target()));
    }

    click_at(&mut runtime, 50.0, 25.0);
    assert_eq!(hit.get(), Some(a), "the click on the first row reports `a`");

    click_at(&mut runtime, 50.0, 75.0);
    assert_eq!(
        hit.get(),
        Some(b),
        "the SAME handler, the SAME symbol, and a different target — which no \
         zero-argument handler could have distinguished"
    );
}

/// §2.4: the payload a pointer event must carry reaches the handler —
/// "pointer ID; device type; buttons; logical position and delta; pressure;
/// tilt; rotation; primary-pointer status; hover capability."
#[test]
fn a_handler_reads_the_pointer_payload_the_platform_reported() {
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let (mut runtime, _panel, _a, _b) = two_rows(sheets);

    let seen: Rc<Cell<Option<PointerEvent>>> = Rc::new(Cell::new(None));
    {
        let sink = Rc::clone(&seen);
        runtime.on_command(CMD_PICK, move |e: &mut UiEvent| sink.set(e.pointer()));
    }

    click_at(&mut runtime, 50.0, 75.0);

    let payload = seen.get().expect("the click delivered its pointer payload");
    assert_eq!(payload.pointer(), PointerId::MOUSE, "pointer ID");
    assert_eq!(payload.device(), PointerDevice::Mouse, "device type");
    assert_eq!(
        payload.kind(),
        PointerEventKind::Up,
        "a click is completed by the release, so that is the event delivered"
    );
    assert_eq!(payload.position(), Point::new(50.0, 75.0), "position");
    assert_eq!(payload.pressure(), 0.0, "a mouse reports no pressure");
    assert!(payload.is_primary(), "primary-pointer status");
    assert!(payload.hover_capable(), "a mouse hovers");
    assert!(payload.delta().is_zero(), "a release moved nothing");
}

/// §2.3 "- trusted/untrusted origin.": platform input arrives trusted, and
/// the handler can read it.
#[test]
fn a_handler_can_read_that_the_gesture_was_trusted() {
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let (mut runtime, _panel, _a, _b) = two_rows(sheets);

    let trust = Rc::new(Cell::new(None));
    {
        let sink = Rc::clone(&trust);
        runtime.on_command(CMD_PICK, move |e: &mut UiEvent| sink.set(Some(e.trust())));
    }

    click_at(&mut runtime, 50.0, 25.0);

    assert_eq!(
        trust.get(),
        Some(EventTrust::Trusted),
        "a platform-vouched gesture reports trusted = true to the handler"
    );
}

/// §2.3: "Event propagation: 1. capture; 2. target; 3. bubble" — and
/// "- `target`; - `current_target`;": the phases run outermost-first then
/// innermost-first, `current_target` moves at every step, and `target` never
/// does.
#[test]
fn the_phases_run_in_order_and_only_current_target_moves() {
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let (mut runtime, panel, a, _b) = two_rows(sheets);
    let root = runtime.tree().root().expect("root after a frame");

    let log: PhaseLog = Rc::new(RefCell::new(Vec::new()));
    let recorder = |log: &PhaseLog| {
        let sink = Rc::clone(log);
        move |e: &mut UiEvent| {
            sink.borrow_mut()
                .push((e.phase(), e.current_target(), e.target()));
        }
    };
    for node in [root, panel, a] {
        for phase in [ListenerPhase::Capture, ListenerPhase::Bubble] {
            runtime.add_event_listener(node, CMD_PICK, phase, recorder(&log));
        }
    }

    click_at(&mut runtime, 50.0, 25.0);

    let steps: Vec<(EventPhase, Option<NodeId>)> = log
        .borrow()
        .iter()
        .map(|(phase, current, _)| (*phase, *current))
        .collect();
    assert_eq!(
        steps,
        vec![
            (EventPhase::Capture, Some(root)),
            (EventPhase::Capture, Some(panel)),
            // At the target BOTH of its listeners run, in registration
            // order, whichever direction they asked for — the registration
            // choice stops mattering once the event has arrived.
            (EventPhase::Target, Some(a)),
            (EventPhase::Target, Some(a)),
            (EventPhase::Bubble, Some(panel)),
            (EventPhase::Bubble, Some(root)),
        ],
        "capture strictly outermost-first, then the target, then bubble \
         strictly innermost-first"
    );
    assert!(
        log.borrow().iter().all(|(_, _, target)| *target == Some(a)),
        "`target` is the node that was hit and does not move while the event \
         propagates — which is the whole reason it is a second value"
    );
}

/// §2.3 "- `stop_propagation`;": the ancestor's bubble listener does not run,
/// and neither does the default action, because §2.6's chain is the outward
/// leg of the same propagation.
#[test]
fn stop_propagation_at_the_target_keeps_an_ancestor_from_running() {
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let (mut runtime, panel, a, _b) = two_rows(sheets);

    let (ancestor_ran, ancestor) = counting_handler();
    runtime.add_event_listener(panel, CMD_PICK, ListenerPhase::Bubble, ancestor);
    let (handler_ran, handler) = counting_handler();
    runtime.on_command(CMD_PICK, handler);

    // Baseline: with nothing stopping it, both run.
    click_at(&mut runtime, 50.0, 25.0);
    assert_eq!((ancestor_ran.get(), handler_ran.get()), (1, 1));

    // Now the target stops propagation.
    runtime.add_event_listener(a, CMD_PICK, ListenerPhase::Bubble, |e: &mut UiEvent| {
        e.stop_propagation();
    });
    click_at(&mut runtime, 50.0, 25.0);
    assert_eq!(
        (ancestor_ran.get(), handler_ran.get()),
        (1, 1),
        "neither the ancestor's listener nor the command's handler saw the \
         second click"
    );
}

/// §2.3 "- `stop_immediate_propagation`;": the *remaining listeners on the
/// same node* are skipped — the one thing that distinguishes it from the
/// ordinary stop, and the reason a node may carry more than one listener.
#[test]
fn stop_immediate_propagation_skips_a_sibling_listener_on_the_same_node() {
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let (mut runtime, _panel, a, _b) = two_rows(sheets);

    // An ordinary stop first: the second listener on the same node still runs.
    let (ordinary_ran, ordinary) = counting_handler();
    runtime.add_event_listener(a, CMD_PICK, ListenerPhase::Bubble, |e: &mut UiEvent| {
        e.stop_propagation();
    });
    runtime.add_event_listener(a, CMD_PICK, ListenerPhase::Bubble, ordinary);
    click_at(&mut runtime, 50.0, 25.0);
    assert_eq!(
        ordinary_ran.get(),
        1,
        "`stop_propagation` leaves same-node listeners running"
    );

    // The immediate form, on a fresh runtime: it does not.
    let (mut runtime, _panel, a, _b) = two_rows(sheets);
    let (immediate_ran, immediate) = counting_handler();
    runtime.add_event_listener(a, CMD_PICK, ListenerPhase::Bubble, |e: &mut UiEvent| {
        e.stop_immediate_propagation();
    });
    runtime.add_event_listener(a, CMD_PICK, ListenerPhase::Bubble, immediate);
    click_at(&mut runtime, 50.0, 25.0);
    assert_eq!(
        immediate_ran.get(),
        0,
        "`stop_immediate_propagation` skips the rest of this node's listeners"
    );
}

/// §2.3: "Event propagation: … 4. default action." and "- `prevent_default`;"
///
/// The default action of an activation is running the node's bound command
/// handler — the reading `handle_pointer` already takes where it gates that
/// step on trust. So `prevent_default` suppresses the handler while leaving
/// the listener walk intact, which is exactly what separates it from a stop.
#[test]
fn prevent_default_suppresses_the_default_action_but_not_the_walk() {
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let (mut runtime, panel, a, _b) = two_rows(sheets);

    let (handler_ran, handler) = counting_handler();
    runtime.on_command(CMD_PICK, handler);
    let (ancestor_ran, ancestor) = counting_handler();
    runtime.add_event_listener(panel, CMD_PICK, ListenerPhase::Bubble, ancestor);
    runtime.add_event_listener(a, CMD_PICK, ListenerPhase::Bubble, |e: &mut UiEvent| {
        e.prevent_default();
    });

    click_at(&mut runtime, 50.0, 25.0);

    assert_eq!(
        handler_ran.get(),
        0,
        "the default action — the bound command's handler — did not run"
    );
    assert_eq!(
        ancestor_ran.get(),
        1,
        "but propagation continued: preventing a default is not stopping a \
         propagation"
    );
}

/// A capture listener sees a command aimed at a **descendant** before the
/// descendant does, and can take the interaction away from it. This is what
/// §2.3's capture phase is *for*, and nothing could express it before.
#[test]
fn a_capture_listener_can_take_the_interaction_from_its_descendant() {
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let (mut runtime, panel, a, _b) = two_rows(sheets);

    let order = Rc::new(RefCell::new(Vec::new()));
    {
        let sink = Rc::clone(&order);
        runtime.add_event_listener(
            panel,
            CMD_PICK,
            ListenerPhase::Capture,
            move |e: &mut UiEvent| {
                sink.borrow_mut().push("capture");
                e.stop_propagation();
            },
        );
    }
    {
        let sink = Rc::clone(&order);
        runtime.add_event_listener(a, CMD_PICK, ListenerPhase::Bubble, move || {
            sink.borrow_mut().push("target");
        });
    }
    let (handler_ran, handler) = counting_handler();
    runtime.on_command(CMD_PICK, handler);

    click_at(&mut runtime, 50.0, 25.0);

    assert_eq!(
        *order.borrow(),
        vec!["capture"],
        "the ancestor ran first and the target never ran"
    );
    assert_eq!(handler_ran.get(), 0, "and no handler claimed the command");
}

/// The zero-argument shape is a convenience, not a legacy: it still
/// registers, still runs, and coexists with the event-taking shape on the
/// same runtime and the same symbol.
#[test]
fn the_zero_argument_handler_shape_still_works() {
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let (mut runtime, panel, a, _b) = two_rows(sheets);

    // A zero-argument LISTENER, an event-taking listener, and a
    // zero-argument HANDLER, all for one symbol.
    let (plain_listener_ran, plain_listener) = counting_handler();
    runtime.add_event_listener(a, CMD_PICK, ListenerPhase::Bubble, plain_listener);
    let saw_target = Rc::new(Cell::new(None));
    {
        let sink = Rc::clone(&saw_target);
        runtime.add_event_listener(
            panel,
            CMD_PICK,
            ListenerPhase::Bubble,
            move |e: &mut UiEvent| sink.set(e.target()),
        );
    }
    let (plain_handler_ran, plain_handler) = counting_handler();
    runtime.on_command(CMD_PICK, plain_handler);

    click_at(&mut runtime, 50.0, 25.0);

    assert_eq!(plain_listener_ran.get(), 1);
    assert_eq!(plain_handler_ran.get(), 1);
    assert_eq!(saw_target.get(), Some(a));
}

/// §2.6's keyboard leg carries its own payload: an Enter activation delivers
/// the key that caused it, and the key's own trust travels with it.
///
/// CHANGED 2026-08-03. This used to assert `trusted == true` with the reason
/// "handle_key is the platform adapter's door" — `handle_key` conferred trust on
/// every key that reached it. It no longer does: `KeyEvent` carries a `trust`
/// field of its own and `UiEvent::from_key` reports it, exactly as
/// `PointerEvent` and `UiEvent::from_pointer` already did. A field the runtime
/// overwrote would be a lie, and the pointer side had already settled the
/// question — trust is conferred at the platform boundary, by the converter, and
/// fails closed everywhere else. The winit runner now calls `as_trusted`.
///
/// Both directions are asserted below, which is more than the old case did.
#[test]
fn an_enter_activation_delivers_the_key_that_caused_it() {
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let (mut runtime, _panel, a, _b) = two_rows(sheets);

    let seen: KeySeen = Rc::new(Cell::new(None));
    {
        let sink = Rc::clone(&seen);
        runtime.on_command(CMD_PICK, move |e: &mut UiEvent| {
            sink.set(Some((e.key(), e.origin(), e.is_trusted())));
        });
    }

    // A press seats focus on `a` without completing a click.
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
    assert_eq!(runtime.focused(), Some(a));
    let enter = KeyEvent::press(PhysicalKey::Enter, Some(LogicalKey::named(NamedKey::Enter)));
    runtime.handle_key(enter);

    let (key, origin, trusted) = seen.get().expect("the Enter activation ran the handler");
    assert_eq!(
        key,
        Some(enter),
        "the handler reads the key, not just that one arrived"
    );
    assert_eq!(origin, EventOrigin::Keyboard);
    assert!(
        !trusted,
        "a key nobody marked is untrusted — `handle_key` is reachable by more \
         than the platform adapter, so it confers nothing"
    );

    // And the other direction: what the converter marks arrives marked.
    let trusted_enter =
        KeyEvent::press(PhysicalKey::Enter, Some(LogicalKey::named(NamedKey::Enter))).as_trusted();
    runtime.handle_key(trusted_enter);
    let (key, _origin, trusted) = seen.get().expect("the second activation ran the handler");
    assert_eq!(key, Some(trusted_enter));
    assert!(trusted, "the platform boundary is what confers trust");
}

/// A programmatic [`UiRuntime::dispatch_command`] has no device behind it, so
/// it reports an empty payload and a synthetic origin rather than a
/// fabricated click at (0, 0).
#[test]
fn a_programmatic_dispatch_reports_a_synthetic_origin_and_no_payload() {
    let mut runtime = TestFixture::runtime();
    let seen: Rc<Cell<Option<(EventOrigin, bool, bool)>>> = Rc::new(Cell::new(None));
    {
        let sink = Rc::clone(&seen);
        runtime.on_command(CMD_PICK, move |e: &mut UiEvent| {
            sink.set(Some((e.origin(), e.payload().is_empty(), e.is_trusted())));
        });
    }

    assert!(runtime.dispatch_command(CMD_PICK));

    assert_eq!(
        seen.get(),
        Some((EventOrigin::Synthetic, true, false)),
        "no device, no payload, and no trust conferred by omission"
    );
}

/// `clear_node_commands` reaps a node's listeners along with its handler —
/// otherwise a churning subtree would leak one closure per rebuilt row.
#[test]
fn clearing_a_nodes_commands_also_clears_its_listeners() {
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let (mut runtime, _panel, a, _b) = two_rows(sheets);

    runtime.on_command_at(a, CMD_PICK, || {});
    runtime.add_event_listener(a, CMD_PICK, ListenerPhase::Capture, || {});
    runtime.add_event_listener(a, CMD_PICK, ListenerPhase::Bubble, || {});
    assert_eq!(runtime.event_listener_count(a, CMD_PICK), 2);

    assert_eq!(
        runtime.clear_node_commands(a),
        3,
        "one handler and two listeners"
    );
    assert_eq!(runtime.event_listener_count(a, CMD_PICK), 0);
    assert!(!runtime.has_node_command_handler(a, CMD_PICK));
}
