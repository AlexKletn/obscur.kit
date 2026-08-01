#![doc = include_str!("command_routing.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_identity::{InternedSymbol, NodeId, RuntimeId};
use obscur_interaction::pointer::PointerEventKind;
use obscur_runtime::{AppCommands, CommandTarget, UiRuntime};
use obscur_style::matching::Stylesheet;
use obscur_testkit::assertion::counting_handler;
use obscur_testkit::fixture::{InteractionPanel, TestFixture, mouse};

/// A keyboard-shortcut command symbol the routing tests bind and dispatch.
const CMD_SAVE: InternedSymbol = InternedSymbol::from_raw(0x5A7E);

/// A runtime with the panel rendered and focus seated on child `a`.
/// Returns `(runtime, panel, a)` — `panel` is `a`'s parent, i.e. the
/// nearest ancestor link of the chain.
fn focused_on_a(sheets: &[Stylesheet]) -> (UiRuntime, NodeId, NodeId) {
    let mut runtime = TestFixture::runtime();
    let _ = runtime.render_now(&InteractionPanel::view(), sheets);
    let (panel, a, _b) = InteractionPanel::nodes(&runtime);
    // A press on `a`'s box seats focus there.
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
    let _ = runtime.run_due_frame(&InteractionPanel::view(), sheets);
    assert_eq!(
        runtime.focused(),
        Some(a),
        "fixture precondition: the press must seat focus on `a`"
    );
    (runtime, panel, a)
}

/// §2.6 link 3: a command the focused node does not handle must reach an
/// **ancestor's** handler.
///
/// Before the routing chain existed this could not happen at all: dispatch
/// was one flat `HashMap<InternedSymbol, handler>` lookup with no per-node
/// registration, so there was nothing for an ancestor to register *as*.
#[test]
fn a_command_reaches_an_ancestors_handler_when_the_focused_node_has_none() {
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let (mut runtime, panel, a) = focused_on_a(sheets);

    // The ONLY handler is on the ancestor. Nothing at the focused node,
    // nothing at the window, no application fallback.
    let (ran, handler) = counting_handler();
    runtime.on_command_at(panel, CMD_SAVE, handler);
    assert!(
        !runtime.has_node_command_handler(a, CMD_SAVE),
        "the focused node must have no handler — that is the premise"
    );
    assert!(!runtime.has_command_handler(CMD_SAVE));

    assert!(
        runtime.dispatch_command(CMD_SAVE),
        "the chain must walk from the focused node up to its ancestor"
    );
    assert_eq!(ran.get(), 1);
}

/// §2.6 link 5: a command nothing else claims must reach the **application
/// fallback**.
#[test]
fn a_command_reaches_the_app_fallback_when_nothing_else_claims_it() {
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let (mut runtime, panel, a) = focused_on_a(sheets);

    let app = AppCommands::new();
    let (ran, handler) = counting_handler();
    app.on_command(CMD_SAVE, handler);
    runtime.set_app_commands(app);

    // Every earlier link is empty: focused node, scope, ancestor, window.
    assert!(!runtime.has_node_command_handler(a, CMD_SAVE));
    assert!(!runtime.has_node_command_handler(panel, CMD_SAVE));
    assert!(runtime.focus_scopes().is_empty());
    assert!(!runtime.has_command_handler(CMD_SAVE));

    assert!(
        runtime.dispatch_command(CMD_SAVE),
        "an unclaimed command must fall through to the application fallback"
    );
    assert_eq!(ran.get(), 1);
}

/// The window link is **per window**; the application link **spans** them.
///
/// This is what makes links 4 and 5 two links rather than one map consulted
/// twice: a `UiRuntime` is one window surface, so its own handlers are that
/// window's alone, while a shared `AppCommands` registered ONCE serves every
/// window that has it installed.
#[test]
fn the_window_link_is_per_window_and_the_app_link_is_shared_across_them() {
    let app = AppCommands::new();
    let mut first = UiRuntime::new(RuntimeId::from_raw(1), TestFixture::VIEWPORT);
    let mut second = UiRuntime::new(RuntimeId::from_raw(2), TestFixture::VIEWPORT);
    first.set_app_commands(app.clone());
    second.set_app_commands(app.clone());
    assert!(
        first
            .app_commands()
            .shares_registry_with(&second.app_commands())
    );

    // A window handler on the FIRST window only, and one shared fallback.
    let (win_ran, win_handler) = counting_handler();
    first.on_command(CMD_SAVE, win_handler);
    let (app_ran, app_handler) = counting_handler();
    app.on_command(CMD_SAVE, app_handler);

    // First window: its own handler claims the command; the fallback is
    // never reached.
    assert!(first.dispatch_command(CMD_SAVE));
    assert_eq!((win_ran.get(), app_ran.get()), (1, 0));

    // Second window: the SAME symbol has no handler here, so it falls
    // through to the fallback — which was registered once, on neither
    // runtime.
    assert!(second.dispatch_command(CMD_SAVE));
    assert_eq!(
        (win_ran.get(), app_ran.get()),
        (1, 1),
        "the window link must not leak between windows, and the app link must span them"
    );

    // A runtime with a *separate* registry does not see that fallback.
    let mut third = UiRuntime::new(RuntimeId::from_raw(3), TestFixture::VIEWPORT);
    third.set_app_commands(AppCommands::new());
    assert!(!third.dispatch_command(CMD_SAVE));
    assert_eq!((win_ran.get(), app_ran.get()), (1, 1));
}

/// With nothing focused the positional links have no anchor, so the route is
/// just the two non-positional ones. A route is never empty.
#[test]
fn a_route_without_an_origin_is_window_then_application() {
    let runtime = TestFixture::runtime();
    assert_eq!(
        runtime.command_route(None),
        vec![CommandTarget::Window, CommandTarget::App]
    );
}
