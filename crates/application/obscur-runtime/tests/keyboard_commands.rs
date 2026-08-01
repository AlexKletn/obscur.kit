#![doc = include_str!("keyboard_commands.doc/_module.md")]
#![forbid(unsafe_code)]

use std::cell::Cell;
use std::rc::Rc;

use obscur_identity::{ComponentId, InternedSymbol};
use obscur_interaction::command::{
    Command, CommandState, KeyEvent, LogicalKey, Modifier, NamedKey, PhysicalKey, Shortcut,
};
use obscur_interaction::pointer::PointerEventKind;
use obscur_runtime::KeyDispatch;
use obscur_testkit::fixture::{InteractionPanel, TestFixture, mouse};
use obscur_view::view::{
    CLASS, EVENT_CHANGE, View, ViewEventBinding, ViewFragment, ViewNode, ViewProperty, ViewValue,
};

/// App-owned command symbols the click tests bind and register against.
const CMD_PANEL: InternedSymbol = InternedSymbol::from_raw(0xC0DE);
const CMD_B: InternedSymbol = InternedSymbol::from_raw(0xB0B0);
/// A keyboard-shortcut command symbol the key tests bind and dispatch.
const CMD_SAVE: InternedSymbol = InternedSymbol::from_raw(0x5A7E);
/// A checkbox `on-change` command symbol the change-activation tests use.
const CMD_CHECK: InternedSymbol = InternedSymbol::from_raw(0xC4EC);

/// The `panel` view (same geometry as [`InteractionPanel::view`]) but with click
/// bindings: the panel binds `on-click → CMD_PANEL`, its child `a` is unbound
/// (so a click on it bubbles to the panel), and its child `b` binds
/// `on-click → CMD_B` (so a click on it fires directly).
fn bound_view() -> View {
    InteractionPanel::ancestor_bound_view(CMD_PANEL, CMD_B)
}

fn key_a() -> KeyEvent {
    KeyEvent::press(PhysicalKey::KeyA, Some(LogicalKey::character('a')))
}

#[test]
fn handle_key_routes_to_focused_node() {
    let mut runtime = TestFixture::runtime();
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&InteractionPanel::view(), sheets);
    let (_panel, a, _b) = InteractionPanel::nodes(&runtime);

    // With nothing focused, a key routes nowhere and schedules no frame.
    let unfocused = runtime.handle_key(key_a());
    assert_eq!(unfocused.target(), None);
    assert!(!unfocused.is_handled());
    assert!(!runtime.has_pending_frame());

    // Focus a via a press, drain its frame, then a key routes to a.
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
    let _ = runtime.run_due_frame(&InteractionPanel::view(), sheets);
    let dispatch = runtime.handle_key(key_a());
    assert_eq!(dispatch.target(), Some(a));
    assert!(dispatch.is_handled());
    // No shortcut bound, so nothing resolves to a command.
    assert_eq!(dispatch.command(), None);
    assert!(runtime.has_pending_frame());
}

#[test]
fn handle_key_resolves_a_bound_command_at_the_focused_node() {
    let mut runtime = TestFixture::runtime();
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&InteractionPanel::view(), sheets);
    let (_panel, a, _b) = InteractionPanel::nodes(&runtime);

    // Ctrl+S → CMD_SAVE.
    runtime.bind_command(
        Shortcut::new(LogicalKey::character('s')).with(Modifier::Ctrl),
        CMD_SAVE,
    );

    // Focus a, then hold Ctrl and press S.
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
    let _ = runtime.run_due_frame(&InteractionPanel::view(), sheets);

    // A physical Ctrl press folds into the held modifier set (no logical key).
    let ctrl = runtime.handle_key(KeyEvent::press(PhysicalKey::ControlLeft, None));
    assert_eq!(ctrl.target(), Some(a));
    assert_eq!(ctrl.command(), None, "a bare modifier resolves no command");

    // Now Ctrl is held: S resolves to the bound command at the focused node.
    let save = runtime.handle_key(KeyEvent::press(
        PhysicalKey::KeyS,
        Some(LogicalKey::character('s')),
    ));
    assert_eq!(save.target(), Some(a));
    assert_eq!(save.command(), Some(CMD_SAVE));
    assert!(save.resolved_command());

    // Releasing Ctrl clears the modifier: S alone no longer resolves.
    let _ = runtime.handle_key(KeyEvent::release(PhysicalKey::ControlLeft, None));
    let bare_s = runtime.handle_key(KeyEvent::press(
        PhysicalKey::KeyS,
        Some(LogicalKey::character('s')),
    ));
    assert_eq!(bare_s.command(), None);
}

/// §6.7: registering ONE unified `Command` binds its shortcut, and its ONE
/// `CommandState` governs every surface that fires it.
///
/// The command is registered once — no separate `bind_command` call — and the
/// keyboard finds it. Disabling that one descriptor then stops the shortcut
/// *and* the click, which is the property that makes this a unified model
/// rather than a parallel vocabulary: both surfaces funnel through
/// `dispatch_command`, which consults the registry.
#[test]
fn registered_command_binds_its_shortcut_and_its_state_gates_every_surface() {
    let mut runtime = TestFixture::runtime();
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&InteractionPanel::view(), sheets);

    let ran = Rc::new(Cell::new(0_u32));
    let counter = Rc::clone(&ran);
    runtime.on_command(CMD_SAVE, move || counter.set(counter.get() + 1));

    // ONE registration: the descriptor carries its own shortcut.
    let ctrl_s = Shortcut::new(LogicalKey::character('s')).with(Modifier::Ctrl);
    runtime.register_command(Command::new(CMD_SAVE, "Save").with_shortcut(ctrl_s));

    // The shortcut was bound by registering alone.
    assert_eq!(
        runtime.command_router().bindings_for(CMD_SAVE),
        vec![ctrl_s],
        "register_command must bind the descriptor's own shortcut"
    );
    // The descriptor is reachable by the SAME symbol the router resolves.
    assert_eq!(
        runtime.command_registry().get(CMD_SAVE).map(Command::title),
        Some("Save")
    );

    // Focus the node, hold Ctrl.
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
    let _ = runtime.run_due_frame(&InteractionPanel::view(), sheets);
    let _ = runtime.handle_key(KeyEvent::press(PhysicalKey::ControlLeft, None));

    let press_s = || KeyEvent::press(PhysicalKey::KeyS, Some(LogicalKey::character('s')));

    // Enabled: the shortcut runs the handler.
    let hit = runtime.handle_key(press_s());
    assert_eq!(hit.command(), Some(CMD_SAVE));
    assert_eq!(ran.get(), 1, "an enabled command's shortcut runs it");

    // Disable the ONE descriptor.
    assert!(runtime.set_command_state(CMD_SAVE, CommandState::new().enabled(false)));

    // Surface 1 — shortcut: still resolves, but is now inert.
    let blocked = runtime.handle_key(press_s());
    assert_eq!(
        blocked.command(),
        Some(CMD_SAVE),
        "the router still resolves the chord; the STATE is what refuses it"
    );
    assert_eq!(
        ran.get(),
        1,
        "a disabled command's shortcut must not run it"
    );

    // Surface 2 — click/activation on the node bound to the same symbol.
    assert!(!runtime.dispatch_command(CMD_SAVE));
    assert_eq!(ran.get(), 1, "a disabled command's click must not run it");

    // Hidden is independently inert.
    assert!(runtime.set_command_state(CMD_SAVE, CommandState::new().visible(false)));
    let _ = runtime.handle_key(press_s());
    assert_eq!(ran.get(), 1, "a hidden command's shortcut must not run it");

    // Re-enabling restores every surface at once.
    assert!(runtime.set_command_state(CMD_SAVE, CommandState::new()));
    let _ = runtime.handle_key(press_s());
    assert_eq!(ran.get(), 2, "re-enabling restores the shortcut");
    assert!(runtime.dispatch_command(CMD_SAVE));
    assert_eq!(ran.get(), 3, "re-enabling restores the click");
}

/// A symbol with no registered `Command` is unconstrained: the registry
/// describes commands, it does not gate symbols routed without one. This is
/// what keeps every existing raw-symbol application working.
#[test]
fn unregistered_command_symbol_is_not_gated_by_the_registry() {
    let mut runtime = TestFixture::runtime();
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&InteractionPanel::view(), sheets);

    let ran = Rc::new(Cell::new(0_u32));
    let counter = Rc::clone(&ran);
    runtime.on_command(CMD_SAVE, move || counter.set(counter.get() + 1));

    // Never registered as a Command, and a *different* command is disabled.
    runtime.register_command(Command::new(CMD_B, "Other"));
    assert!(runtime.set_command_state(CMD_B, CommandState::new().enabled(false)));

    assert!(runtime.command_registry().permits_dispatch(CMD_SAVE));
    assert!(runtime.dispatch_command(CMD_SAVE));
    assert_eq!(ran.get(), 1);
}

#[test]
fn handle_key_without_focus_is_a_noop() {
    let mut runtime = TestFixture::runtime();
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&InteractionPanel::view(), sheets);
    runtime.bind_command(Shortcut::new(LogicalKey::character('a')), CMD_SAVE);

    // Nothing focused: even a bound key resolves to nothing and schedules
    // no frame — the runtime state is unchanged.
    let dispatch = runtime.handle_key(key_a());
    assert_eq!(dispatch, KeyDispatch::unhandled());
    assert!(!dispatch.is_handled());
    assert_eq!(dispatch.command(), None);
    assert!(!runtime.has_pending_frame());
}

#[test]
fn handle_key_is_deterministic() {
    let build = || {
        let mut runtime = TestFixture::runtime();
        let sheet = InteractionPanel::sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&InteractionPanel::view(), sheets);
        runtime.bind_command(
            Shortcut::new(LogicalKey::character('s')).with(Modifier::Ctrl),
            CMD_SAVE,
        );
        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
        let _ = runtime.run_due_frame(&InteractionPanel::view(), sheets);
        runtime.handle_key(KeyEvent::press(PhysicalKey::ControlLeft, None));
        runtime.handle_key(KeyEvent::press(
            PhysicalKey::KeyS,
            Some(LogicalKey::character('s')),
        ))
    };
    assert_eq!(build(), build());
}

/// A `panel` (100x100) holding a single child `b` (100x50, at y∈[0,50)) that
/// binds `on-change → CMD_CHECK` — a checkbox-like control that fires its
/// *change* command (not a click command) when activated.
fn checkbox_view() -> View {
    View::from_fragment(ViewFragment::from_nodes(vec![
        ViewNode::element(ComponentId::from_raw(400))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into())))
            .with_children(ViewFragment::from_nodes(vec![
                ViewNode::element(ComponentId::from_raw(401))
                    .with_prop(ViewProperty::new(CLASS, ViewValue::Text("b".into())))
                    .with_event(ViewEventBinding::new(EVENT_CHANGE, CMD_CHECK)),
            ])),
    ]))
}

#[test]
fn bound_shortcut_key_press_runs_its_registered_handler() {
    // (b) A keyboard shortcut resolves to a command symbol AND runs the
    // handler registered for that same symbol — keyboard now dispatches.
    let mut runtime = TestFixture::runtime();
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&InteractionPanel::view(), sheets);

    // Bind Ctrl+S to CMD_SAVE and register the handler for the SAME symbol.
    runtime.bind_command(
        Shortcut::new(LogicalKey::character('s')).with(Modifier::Ctrl),
        CMD_SAVE,
    );
    let counter = Rc::new(Cell::new(0));
    {
        let c = counter.clone();
        runtime.on_command(CMD_SAVE, move || c.set(c.get() + 1));
    }

    // Focus a node so the key routes, then hold Ctrl and press S.
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
    let _ = runtime.run_due_frame(&InteractionPanel::view(), sheets);
    runtime.handle_key(KeyEvent::press(PhysicalKey::ControlLeft, None));
    let save = runtime.handle_key(KeyEvent::press(
        PhysicalKey::KeyS,
        Some(LogicalKey::character('s')),
    ));

    assert_eq!(save.command(), Some(CMD_SAVE), "the shortcut resolved");
    assert_eq!(counter.get(), 1, "the bound shortcut RAN its handler once");
}

#[test]
fn clicking_a_checkbox_fires_its_change_command_via_activate() {
    // (c) A checkbox-like node bound only to EVENT_CHANGE fires that change
    // command when clicked (Down+Up) — activation resolves click ELSE change.
    let mut runtime = TestFixture::runtime();
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&checkbox_view(), sheets);

    let counter = Rc::new(Cell::new(0));
    {
        let c = counter.clone();
        runtime.on_command(CMD_CHECK, move || c.set(c.get() + 1));
    }

    // The single child `b` occupies y∈[0,50); click at (50, 25).
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
    let up = runtime.handle_pointer(mouse(PointerEventKind::Up, 50.0, 25.0));

    assert_eq!(
        up.command(),
        Some(CMD_CHECK),
        "the click resolved the change command"
    );
    assert_eq!(counter.get(), 1, "the change handler RAN once on click");
}

#[test]
fn enter_on_focused_button_fires_its_click_command_via_activate() {
    // (d) Enter on a focused button (bound to EVENT_CLICK) runs the click
    // handler even with no keyboard shortcut bound.
    let mut runtime = TestFixture::runtime();
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&bound_view(), sheets);
    let (_panel, _a, b) = InteractionPanel::nodes(&runtime);

    let counter = Rc::new(Cell::new(0));
    {
        let c = counter.clone();
        runtime.on_command(CMD_B, move || c.set(c.get() + 1));
    }

    // Focus `b` (bound to EVENT_CLICK → CMD_B) via a press at (50, 75), drain
    // the focus frame, then press Enter: activation fires b's click command.
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 75.0));
    let _ = runtime.run_due_frame(&bound_view(), sheets);
    assert_eq!(runtime.focused(), Some(b));

    let enter = runtime.handle_key(KeyEvent::press(
        PhysicalKey::Enter,
        Some(LogicalKey::named(NamedKey::Enter)),
    ));
    assert_eq!(enter.target(), Some(b));
    // No shortcut is bound, so nothing resolved through the router...
    assert_eq!(enter.command(), None);
    // ...but Enter activated the focused button's click command.
    assert_eq!(
        counter.get(),
        1,
        "Enter activated the button's click handler"
    );
}

#[test]
fn space_on_focused_checkbox_fires_change_and_missing_handler_is_a_noop() {
    // (e-adjacent) Space activates too, firing the checkbox's change command;
    // and an activation whose command has no registered handler never panics.
    let mut runtime = TestFixture::runtime();
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&checkbox_view(), sheets);

    // Focus the checkbox with NO handler registered: Space must not panic.
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
    let _ = runtime.run_due_frame(&checkbox_view(), sheets);
    let space_noop = runtime.handle_key(KeyEvent::press(
        PhysicalKey::Space,
        Some(LogicalKey::named(NamedKey::Space)),
    ));
    assert!(space_noop.is_handled());

    // Now register the handler; a Space keypress fires the change command.
    let counter = Rc::new(Cell::new(0));
    {
        let c = counter.clone();
        runtime.on_command(CMD_CHECK, move || c.set(c.get() + 1));
    }
    runtime.handle_key(KeyEvent::press(
        PhysicalKey::Space,
        Some(LogicalKey::named(NamedKey::Space)),
    ));
    assert_eq!(counter.get(), 1, "Space activated the change handler");
}

#[test]
fn button_click_still_fires_event_click_after_activation_change() {
    // (e) Regression: a button click still dispatches EVENT_CLICK (activation
    // prefers the click binding over any change binding).
    let mut runtime = TestFixture::runtime();
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&bound_view(), sheets);
    let (_panel, _a, b) = InteractionPanel::nodes(&runtime);

    let counter = Rc::new(Cell::new(0));
    {
        let c = counter.clone();
        runtime.on_command(CMD_B, move || c.set(c.get() + 1));
    }

    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 75.0));
    let up = runtime.handle_pointer(mouse(PointerEventKind::Up, 50.0, 75.0));

    assert_eq!(up.target(), Some(b));
    assert_eq!(
        up.command(),
        Some(CMD_B),
        "click still resolves EVENT_CLICK"
    );
    assert_eq!(counter.get(), 1);
}
