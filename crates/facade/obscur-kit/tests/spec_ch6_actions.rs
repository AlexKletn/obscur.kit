#![doc = include_str!("spec_ch6_actions.doc/_module.md")]
#![forbid(unsafe_code)]

#[path = "spec_ch6_support/mod.rs"]
mod support;

use std::cell::Cell;
use std::rc::Rc;

use obscur_kit::identity::InternedSymbol;
use obscur_kit::interaction::command::KeyEvent;
use obscur_kit::interaction::command::{
    Command, CommandState, LogicalKey, Modifier, Modifiers, PhysicalKey, Shortcut,
};
use obscur_kit::interaction::pointer::PointerEventKind;
use obscur_kit::runtime::UiRuntime;

use support::{at, drive, el, pointer, view};

const CSS: &str = ".panel{width:200px;height:200px} .item{width:200px;height:50px}";

#[doc = include_str!("spec_ch6_actions.doc/focused_runtime.md")]
fn focused_runtime() -> (UiRuntime, obscur_kit::identity::NodeId) {
    let v = view(vec![el(1, "panel", vec![el(2, "item", vec![])])]);
    let mut rt = drive(200.0, 200.0, CSS, &v);
    let item = at(&rt, &[0, 0]);
    rt.handle_pointer(pointer(PointerEventKind::Down, 10.0, 10.0));
    rt.handle_pointer(pointer(PointerEventKind::Up, 10.0, 10.0));
    assert_eq!(
        rt.focused(),
        Some(item),
        "the pointer press seats focus on the item"
    );
    (rt, item)
}

#[doc = include_str!("spec_ch6_actions.doc/press_ctrl_s.md")]
fn press_ctrl_s(rt: &mut UiRuntime) {
    rt.handle_key(KeyEvent::press(PhysicalKey::ControlLeft, None));
    rt.handle_key(KeyEvent::press(
        PhysicalKey::KeyS,
        Some(LogicalKey::character('s')),
    ));
}

// -------------------------------------------------------------------------
// P6-10
// -------------------------------------------------------------------------

#[doc = include_str!("spec_ch6_actions.doc/disabling_an_action_by_id_also_disables_its_keyboard_shortcut.md")]
#[test]
fn disabling_an_action_by_id_also_disables_its_keyboard_shortcut() {
    let save = InternedSymbol::from_raw(0xA1);
    let (mut rt, item) = focused_runtime();

    let shortcut = Shortcut::new(LogicalKey::character('s')).with(Modifier::Ctrl);
    rt.register_command(Command::new(save, "Save").with_shortcut(shortcut));
    rt.bind_command(shortcut, save);

    let counter = Rc::new(Cell::new(0u32));
    let bumped = Rc::clone(&counter);
    rt.on_command_at(item, save, move || bumped.set(bumped.get() + 1));

    // One mutation, by ID.
    assert!(
        rt.set_command_state(save, CommandState::new().enabled(false)),
        "§6.7: the action is addressable by its stable ID"
    );

    press_ctrl_s(&mut rt);
    assert_eq!(
        counter.get(),
        0,
        "§6.7: a disabled action's shortcut must not fire its handler — a disabled Delete \
         must not delete"
    );

    let state = rt
        .command_registry()
        .state(save)
        .expect("§6.7: the action's state is readable by ID");
    assert!(
        !state.is_enabled(),
        "§6.7: every surface reads `enabled = false` from the single action"
    );
    assert!(
        !rt.command_registry().permits_dispatch(save),
        "§6.7: dispatch through any surface is gated by the action's enabled state"
    );

    // Re-enabling makes exactly the same shortcut work again — proof the gate is
    // the action's state and not a torn-down key binding.
    rt.set_command_state(save, CommandState::new().enabled(true));
    press_ctrl_s(&mut rt);
    assert_eq!(
        counter.get(),
        1,
        "§6.7: re-enabling the action restores every surface, the shortcut included"
    );
    assert_eq!(
        rt.command_router().resolve(
            LogicalKey::character('s'),
            Modifiers::new().with(Modifier::Ctrl)
        ),
        Some(save),
        "§6.7: the shortcut stays bound to the action while disabled — disabling is not \
         unbinding"
    );
}

// -------------------------------------------------------------------------
// P6-11
// -------------------------------------------------------------------------

#[doc = include_str!("spec_ch6_actions.doc/checked_is_semantic_state_and_hiding_an_action_does_not_unregister_it.md")]
#[test]
fn checked_is_semantic_state_and_hiding_an_action_does_not_unregister_it() {
    let grid = InternedSymbol::from_raw(0xB2);
    let (mut rt, item) = focused_runtime();

    rt.register_command(Command::new(grid, "Grid"));
    rt.set_command_state(
        grid,
        CommandState::new().with_checked(Some(false)).visible(true),
    );

    let counter = Rc::new(Cell::new(0u32));
    let bumped = Rc::clone(&counter);
    rt.on_command_at(item, grid, move || bumped.set(bumped.get() + 1));

    rt.set_command_state(grid, CommandState::new().with_checked(Some(true)));
    assert_eq!(
        rt.command_registry()
            .state(grid)
            .expect("the action's state")
            .checked(),
        Some(true),
        "§6.7: `checked` is a semantic toggle state, not a glyph in the title"
    );
    assert_eq!(
        rt.command_registry().get(grid).map(Command::title),
        Some("Grid"),
        "§6.7: checking an action does not rewrite its title"
    );

    rt.set_command_state(
        grid,
        CommandState::new().with_checked(Some(true)).visible(false),
    );
    assert!(
        !rt.command_registry()
            .state(grid)
            .expect("the action's state")
            .is_visible(),
        "§6.7: the action is hidden from rendered surfaces"
    );
    assert!(
        rt.command_registry().contains(grid),
        "§6.7: hiding is not unregistering — the stable ID `view.grid` still resolves"
    );
    assert_eq!(
        rt.command_registry().get(grid).map(Command::title),
        Some("Grid"),
        "§6.7: the hidden action is still fully readable by ID"
    );
}
