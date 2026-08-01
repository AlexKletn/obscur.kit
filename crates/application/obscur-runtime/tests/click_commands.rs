#![doc = include_str!("click_commands.doc/_module.md")]
#![forbid(unsafe_code)]

use std::cell::Cell;
use std::rc::Rc;

use obscur_identity::InternedSymbol;
use obscur_interaction::pointer::PointerEventKind;
use obscur_testkit::fixture::{InteractionPanel, TestFixture, mouse};
use obscur_view::view::View;

/// App-owned command symbols the click tests bind and register against.
const CMD_PANEL: InternedSymbol = InternedSymbol::from_raw(0xC0DE);
const CMD_B: InternedSymbol = InternedSymbol::from_raw(0xB0B0);

/// The `panel` view (same geometry as [`InteractionPanel::view`]) but with click
/// bindings: the panel binds `on-click → CMD_PANEL`, its child `a` is unbound
/// (so a click on it bubbles to the panel), and its child `b` binds
/// `on-click → CMD_B` (so a click on it fires directly).
fn bound_view() -> View {
    InteractionPanel::ancestor_bound_view(CMD_PANEL, CMD_B)
}

#[test]
fn click_on_bound_node_runs_its_registered_handler() {
    let mut runtime = TestFixture::runtime();
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&bound_view(), sheets);
    let (_panel, _a, b) = InteractionPanel::nodes(&runtime);

    // Register the handler for `b`'s command with the SAME symbol the View
    // bound `on-click` to — the runtime only compares symbols.
    let counter = Rc::new(Cell::new(0));
    {
        let c = counter.clone();
        runtime.on_command(CMD_B, move || c.set(c.get() + 1));
    }
    assert!(runtime.has_command_handler(CMD_B));

    // Down then Up on `b` (50, 75) is a click on `b`.
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 75.0));
    let up = runtime.handle_pointer(mouse(PointerEventKind::Up, 50.0, 75.0));

    assert_eq!(up.target(), Some(b));
    assert_eq!(up.command(), Some(CMD_B), "the click resolved b's command");
    assert_eq!(counter.get(), 1, "the registered handler RAN exactly once");
}

#[test]
fn press_and_release_on_different_nodes_do_not_dispatch() {
    let mut runtime = TestFixture::runtime();
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&bound_view(), sheets);

    let counter = Rc::new(Cell::new(0));
    {
        let c = counter.clone();
        runtime.on_command(CMD_B, move || c.set(c.get() + 1));
    }
    {
        let c = counter.clone();
        runtime.on_command(CMD_PANEL, move || c.set(c.get() + 1));
    }

    // Press on `a` (50, 25), release on `b` (50, 75): not a click on either.
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
    let up = runtime.handle_pointer(mouse(PointerEventKind::Up, 50.0, 75.0));

    assert_eq!(up.command(), None, "a drag across nodes is not a click");
    assert_eq!(counter.get(), 0, "no handler ran");
}

#[test]
fn click_on_unbound_child_bubbles_to_the_bound_ancestor() {
    let mut runtime = TestFixture::runtime();
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&bound_view(), sheets);
    let (_panel, a, _b) = InteractionPanel::nodes(&runtime);

    let counter = Rc::new(Cell::new(0));
    {
        let c = counter.clone();
        runtime.on_command(CMD_PANEL, move || c.set(c.get() + 1));
    }

    // `a` (50, 25) has no click binding; the click bubbles up to the panel.
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
    let up = runtime.handle_pointer(mouse(PointerEventKind::Up, 50.0, 25.0));

    assert_eq!(up.target(), Some(a));
    assert_eq!(
        up.command(),
        Some(CMD_PANEL),
        "the click bubbled to the bound ancestor"
    );
    assert_eq!(counter.get(), 1, "the ancestor's handler ran");
}

/// §2.3's `current_target` is the node the handler is *attached to*; `target`
/// is the node the event *hit*. The whole point of the field is that those
/// two are not the same node, so this asserts they differ on the bubbled
/// click — a test that only checked the field exists would pass with
/// `current_target` hard-wired to `target`, and this one does not.
#[test]
fn current_target_is_the_bound_ancestor_not_the_hit_node() {
    let mut runtime = TestFixture::runtime();
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&bound_view(), sheets);
    let (panel, a, b) = InteractionPanel::nodes(&runtime);

    // `a` (50, 25) is unbound: the click HITS `a` and bubbles to the panel,
    // which is where the handler lives.
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
    let bubbled = runtime.handle_pointer(mouse(PointerEventKind::Up, 50.0, 25.0));

    assert_eq!(bubbled.target(), Some(a), "the pointer hit `a`");
    assert_eq!(
        bubbled.current_target(),
        Some(panel),
        "but the handler is attached to the panel"
    );
    assert_ne!(
        bubbled.current_target(),
        bubbled.target(),
        "current_target MUST differ from target on a bubbled event — equal \
         here would mean the field is just target under another name"
    );
    assert_ne!(panel, a, "the fixture's two nodes are genuinely distinct");

    // `b` (50, 75) binds `on-click` itself: hit node and handler node coincide.
    // This is the other half — the field is not simply "always the ancestor".
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 75.0));
    let direct = runtime.handle_pointer(mouse(PointerEventKind::Up, 50.0, 75.0));

    assert_eq!(direct.target(), Some(b));
    assert_eq!(
        direct.current_target(),
        Some(b),
        "a node carrying its own binding IS its own current_target"
    );
}

/// No binding resolved → no handler ran → nothing is a current target.
/// `current_target` tracks `command`, so it must be absent exactly when the
/// command is.
#[test]
fn current_target_is_absent_when_no_command_resolves() {
    let mut runtime = TestFixture::runtime();
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    // `panel_view` binds nothing at all.
    let _ = runtime.render_now(&InteractionPanel::view(), sheets);

    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
    let up = runtime.handle_pointer(mouse(PointerEventKind::Up, 50.0, 25.0));

    assert!(up.target().is_some(), "the pointer still hit a node");
    assert_eq!(up.command(), None, "nothing in the chain binds on-click");
    assert_eq!(
        up.current_target(),
        None,
        "no handler ran, so there is no current target"
    );
}

/// A `Move` is not a click: it resolves no command, so it has no
/// `current_target` even over a node whose ancestor is bound.
#[test]
fn a_move_over_a_bound_subtree_has_no_current_target() {
    let mut runtime = TestFixture::runtime();
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&bound_view(), sheets);

    let moved = runtime.handle_pointer(mouse(PointerEventKind::Move, 50.0, 25.0));
    assert_eq!(moved.command(), None);
    assert_eq!(moved.current_target(), None);
}

#[test]
fn click_with_no_registered_handler_is_a_harmless_noop() {
    let mut runtime = TestFixture::runtime();
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&bound_view(), sheets);

    // `b` binds a command in the View, but NO handler is registered for it.
    assert!(!runtime.has_command_handler(CMD_B));
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 75.0));
    let up = runtime.handle_pointer(mouse(PointerEventKind::Up, 50.0, 75.0));

    // The command still resolves (it is bound on the node), but nothing ran
    // and nothing panicked.
    assert_eq!(up.command(), Some(CMD_B));
}

#[test]
fn dispatching_a_click_requests_a_frame() {
    let mut runtime = TestFixture::runtime();
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&bound_view(), sheets);

    let counter = Rc::new(Cell::new(0));
    {
        let c = counter.clone();
        runtime.on_command(CMD_B, move || c.set(c.get() + 1));
    }

    // The press moves focus → requests a frame; drain it so the pending
    // state is clean before the release.
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 75.0));
    let _ = runtime.run_due_frame(&bound_view(), sheets);
    assert!(!runtime.has_pending_frame(), "the press frame drained");

    // The release completes the click: the dispatch alone requests a frame
    // (hover and focus are unchanged from the press on the same node).
    runtime.handle_pointer(mouse(PointerEventKind::Up, 50.0, 75.0));
    assert!(
        runtime.has_pending_frame(),
        "dispatching the click requested a re-render frame"
    );
    assert_eq!(counter.get(), 1);
}
