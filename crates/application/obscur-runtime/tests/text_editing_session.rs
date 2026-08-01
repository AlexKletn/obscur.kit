#![doc = include_str!("text_editing_session.doc/_module.md")]
#![forbid(unsafe_code)]

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use obscur_identity::{ComponentId, InternedSymbol};
use obscur_interaction::command::{KeyEvent, NamedKey, PhysicalKey};
use obscur_interaction::pointer::PointerEventKind;
use obscur_runtime::UiRuntime;
use obscur_runtime::frame::FrameReason;
use obscur_scene::command::PaintCommand;
use obscur_testkit::fixture::{
    EditablePanel, InteractionPanel, TestFixture, char_press, mouse, named_press,
};
use obscur_testkit::selector::SemanticSelector;
use obscur_text::ime::CompositionEvent;
use obscur_view::view::{
    CLASS, EVENT_CHANGE, EVENT_INPUT, READONLY, View, ViewEventBinding, ViewFragment, ViewNode,
    ViewProperty, ViewValue,
};

// ---- Runtime-owned editing session: caret + selection --------------------

#[test]
fn seed_places_caret_at_end_then_type_and_backspace() {
    // (a) A focused editable seeded with "hi" starts with the caret at the
    // end (index 2); typing 'x' -> "hix" caret 3; Backspace -> "hi".
    let (mut runtime, mirror) = EditablePanel::focused("hi");
    assert_eq!(runtime.editing_text(), Some("hi"));
    let sel = runtime.editing_selection().expect("session active");
    assert!(sel.is_collapsed());
    assert_eq!(sel.focus(), 2, "caret seeded at the end");

    runtime.handle_key(char_press('x'));
    assert_eq!(runtime.editing_text(), Some("hix"));
    assert_eq!(runtime.editing_selection().unwrap().focus(), 3);
    assert_eq!(&*mirror.borrow(), "hix", "on-input got the resulting text");

    runtime.handle_key(named_press(NamedKey::Backspace));
    assert_eq!(runtime.editing_text(), Some("hi"));
    assert_eq!(runtime.editing_selection().unwrap().focus(), 2);
    assert_eq!(&*mirror.borrow(), "hi");
}

#[test]
fn arrow_and_shift_extend_then_typing_replaces_selection() {
    // (b) ArrowLeft moves the caret to 1; Shift+ArrowLeft extends the
    // selection to [0,1); typing 'z' replaces that selection -> "zi".
    let (mut runtime, _mirror) = EditablePanel::focused("hi");

    runtime.handle_key(named_press(NamedKey::ArrowLeft));
    let sel = runtime.editing_selection().unwrap();
    assert!(sel.is_collapsed());
    assert_eq!(sel.focus(), 1);

    runtime.handle_key(KeyEvent::press(PhysicalKey::ShiftLeft, None));
    runtime.handle_key(named_press(NamedKey::ArrowLeft));
    let sel = runtime.editing_selection().unwrap();
    assert!(!sel.is_collapsed(), "shift extended the selection");
    assert_eq!((sel.start(), sel.end()), (0, 1));
    runtime.handle_key(KeyEvent::release(PhysicalKey::ShiftLeft, None));

    runtime.handle_key(char_press('z'));
    assert_eq!(runtime.editing_text(), Some("zi"), "insert replaced [0,1)");
}

#[test]
fn home_end_and_ctrl_a_select_all_then_replace() {
    // (c) Home -> caret 0, End -> caret at len; Ctrl+A selects all and typing
    // replaces the whole field.
    let (mut runtime, _mirror) = EditablePanel::focused("hi");

    runtime.handle_key(named_press(NamedKey::Home));
    assert_eq!(runtime.editing_selection().unwrap().focus(), 0);
    runtime.handle_key(named_press(NamedKey::End));
    assert_eq!(runtime.editing_selection().unwrap().focus(), 2);

    runtime.handle_key(KeyEvent::press(PhysicalKey::ControlLeft, None));
    runtime.handle_key(char_press('a')); // Ctrl+A -> SelectAll
    let sel = runtime.editing_selection().unwrap();
    assert_eq!(
        (sel.start(), sel.end()),
        (0, 2),
        "select-all covers the field"
    );
    runtime.handle_key(KeyEvent::release(PhysicalKey::ControlLeft, None));

    runtime.handle_key(char_press('Z'));
    assert_eq!(
        runtime.editing_text(),
        Some("Z"),
        "typing replaced everything"
    );
}

#[test]
fn mutating_edit_fires_on_input_but_caret_move_does_not() {
    // (d) A mutating edit fires EVENT_INPUT with the resulting text; a
    // caret-only move updates the session without firing on-input or
    // corrupting the text.
    let (mut runtime, mirror) = EditablePanel::focused("hi");

    runtime.handle_key(char_press('!'));
    assert_eq!(&*mirror.borrow(), "hi!", "mutating edit delivered the text");

    // A sentinel proves the caret move does not re-fire the handler.
    *mirror.borrow_mut() = String::from("SENTINEL");
    runtime.handle_key(named_press(NamedKey::ArrowLeft));
    assert_eq!(
        runtime.editing_text(),
        Some("hi!"),
        "text intact after a move"
    );
    assert_eq!(
        &*mirror.borrow(),
        "SENTINEL",
        "a caret move does not fire on-input"
    );
}

#[test]
fn typing_renders_session_text_and_a_caret_in_the_frame() {
    // End-to-end: focus an editable "hi", type 'x' so the session is
    // "hix" with the caret at index 3, render a frame, and confirm the display
    // list paints the SESSION text "hix" plus a caret rect at 3·advance past
    // the text origin. The retained view still carries "hi" — the session
    // override is what makes the typed 'x' visible.
    let (mut runtime, _mirror) = EditablePanel::focused("hi");
    runtime.handle_key(char_press('x'));
    assert_eq!(runtime.editing_text(), Some("hix"));
    assert_eq!(runtime.editing_selection().unwrap().focus(), 3);

    let sheet = EditablePanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let result = runtime.render_now(&EditablePanel::view("hi"), sheets);
    let list = result.snapshot().display_list();

    // The painted text is the live session text "hix" (concatenated across any
    // wrapped runs), not the retained "hi".
    let drawn: String = list
        .commands()
        .iter()
        .filter_map(|c| match c {
            PaintCommand::DrawText { run, .. } => Some(run.text()),
            _ => None,
        })
        .collect();
    assert_eq!(drawn, "hix", "the frame paints the live session text");

    // The first text run's origin x is the editing text box's left edge; the
    // caret (the thin width-1 fill) sits 3 advances (3·9.6) past it.
    let advance = 16.0_f32 * obscur_text::shaping::MockTextShaper::ADVANCE_RATIO;
    let text_origin_x = list
        .commands()
        .iter()
        .find_map(|c| match c {
            PaintCommand::DrawText { origin, .. } => Some(origin.x),
            _ => None,
        })
        .expect("a text run");
    let caret = list
        .commands()
        .iter()
        .find_map(|c| match c {
            PaintCommand::DrawPath { path, .. } => {
                path.bounds().filter(|r| (r.size.width - 1.0).abs() < 1e-6)
            }
            _ => None,
        })
        .expect("a thin caret fill");
    assert!(
        (caret.origin.x - (text_origin_x + advance * 3.0)).abs() < 1e-3,
        "caret at {} != text_origin {text_origin_x} + 3·advance {}",
        caret.origin.x,
        advance * 3.0
    );
    assert!(list.is_balanced(), "the display list stays balanced");
}

// ---- Controlled-value reseed while focused (Defect A) --------------------
// The HTML controlled-input model (WHATWG HTML §4.10.5.5): a text control
// reflects its `value`, so an app rewrite of the bound value MID-FOCUS must
// reach the display. The session (the sole source of truth for the focused
// field, an uncontrolled input) reseeds ONLY on an EXTERNAL rewrite — never on
// the user's own keystroke mirrored back.

/// Drives a reactive, controlled editable: `transform` maps each keystroke's
/// resulting text to the value the app stores (identity = passthrough,
/// uppercase = a normalizing handler), which the reactive builder reflects back
/// into the field. Returns the runtime plus the shared app value. Focus lands
/// on the field (empty), ready for keystrokes.
fn controlled_field(
    transform: impl Fn(&str) -> String + 'static,
) -> (UiRuntime, Rc<RefCell<String>>) {
    let mut runtime = TestFixture::runtime();
    let value = Rc::new(RefCell::new(String::new()));
    {
        let s = value.clone();
        runtime.on_text_input(EditablePanel::CMD_INPUT, move |resulting| {
            *s.borrow_mut() = transform(resulting);
        });
    }
    {
        let s = value.clone();
        runtime.set_root_view(move || EditablePanel::view(&s.borrow()));
    }
    runtime.set_stylesheets(vec![EditablePanel::sheet()]);
    // First reactive frame builds the tree/boxes; then focus the field.
    runtime.request_frame(FrameReason::UiCommit);
    runtime.run_due_frame_reactive().expect("first frame");
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
    runtime.run_due_frame_reactive().expect("focus frame");
    assert!(runtime.focused().is_some(), "the press focused the field");
    (runtime, value)
}

#[test]
fn external_value_rewrite_mid_focus_reseeds_the_session_and_shows_it() {
    // (A1) Type into a focused field, then an app `set` of a DIFFERENT value
    // mid-focus: the session reseeds and compose shows the NEW value, not the
    // stale buffer that used to persist until blur.
    let (mut runtime, value) = controlled_field(str::to_owned); // passthrough
    for c in "abc".chars() {
        runtime.handle_key(char_press(c));
    }
    runtime.run_due_frame_reactive().expect("rebuild frame");
    assert_eq!(
        runtime.editing_text(),
        Some("abc"),
        "typing filled the session"
    );
    assert_eq!(
        SemanticSelector::first_text(&runtime),
        "abc",
        "the passthrough value reached the tree"
    );

    // The app rewrites the bound value to something else while still focused
    // (a "Clear"-style set, out of band from the session's own edits).
    *value.borrow_mut() = "reset".to_string();
    runtime.request_frame(FrameReason::UiCommit);
    runtime
        .run_due_frame_reactive()
        .expect("external-rewrite frame");

    assert_eq!(
        runtime.editing_text(),
        Some("reset"),
        "the session reseeded from the external value (not the stale buffer)"
    );
    assert_eq!(SemanticSelector::first_text(&runtime), "reset");
    let sel = runtime.editing_selection().expect("session active");
    assert!(
        sel.is_collapsed() && sel.focus() <= "reset".len(),
        "the caret is clamped into the new value (sane)"
    );
}

#[test]
fn a_normalizing_on_input_uppercases_and_the_display_tracks_the_model() {
    // (A2) An `on-input` that uppercases writes back a value that DIFFERS from
    // the session buffer, so the session reseeds to the uppercased model value
    // — the displayed text is uppercased and the caret stays sane, keystroke
    // after keystroke (continuous controlled normalization).
    let (mut runtime, _value) = controlled_field(str::to_uppercase);

    runtime.handle_key(char_press('a'));
    runtime.run_due_frame_reactive().expect("rebuild after 'a'");
    assert_eq!(
        runtime.editing_text(),
        Some("A"),
        "the model uppercased 'a' → 'A'"
    );
    assert_eq!(
        runtime.editing_selection().unwrap().focus(),
        1,
        "caret at the end"
    );

    runtime.handle_key(char_press('b'));
    runtime.run_due_frame_reactive().expect("rebuild after 'b'");
    assert_eq!(
        runtime.editing_text(),
        Some("AB"),
        "typing keeps uppercasing"
    );
    assert_eq!(SemanticSelector::first_text(&runtime), "AB");
    assert_eq!(
        runtime.editing_selection().unwrap().focus(),
        2,
        "caret stays sane at the end"
    );
}

#[test]
fn a_plain_keystroke_does_not_reseed_and_preserves_the_caret() {
    // (A3) A plain passthrough keystroke does NOT reseed — the retained value
    // catches up to the session's OWN edit, so the discriminator sees no
    // external change and leaves the buffer + caret untouched (no fight with
    // the user). Proven by editing in the MIDDLE: a wrongful reseed would fling
    // the caret to the end.
    let (mut runtime, _value) = controlled_field(str::to_owned); // passthrough
    for c in "hello".chars() {
        runtime.handle_key(char_press(c));
    }
    runtime.run_due_frame_reactive().expect("rebuild frame");
    // Move the caret into the middle (byte 2), a non-mutating edit.
    runtime.handle_key(named_press(NamedKey::ArrowLeft));
    runtime.handle_key(named_press(NamedKey::ArrowLeft));
    runtime.handle_key(named_press(NamedKey::ArrowLeft));
    assert_eq!(
        runtime.editing_selection().unwrap().focus(),
        2,
        "caret in the middle"
    );

    // A reactive frame with UNCHANGED state must not reseed (retained == the
    // session's own value), so the caret is preserved where the user left it.
    runtime.request_frame(FrameReason::UiCommit);
    runtime
        .run_due_frame_reactive()
        .expect("no-op rebuild frame");
    assert_eq!(
        runtime.editing_text(),
        Some("hello"),
        "text untouched (no reseed)"
    );
    assert_eq!(
        runtime.editing_selection().unwrap().focus(),
        2,
        "the caret stayed put — a plain keystroke never reseeds/refights"
    );
}

// ---- readonly: focusable + selectable but NOT editable -------------------
// (WHATWG HTML §attr-input-readonly). Distinct from `disabled`
// (`disabled_control_takes_no_focus_and_fires_no_command`, in
// `tests/click_commands.rs`, covers the disabled gate, which is deliberately
// unchanged).

/// App-owned command a readonly editable binds `on-change` to; used to prove a
/// readonly field never fires `change` (its value can't change).
const CMD_RO_CHANGE: InternedSymbol = InternedSymbol::from_raw(0x1249);

/// A **readonly** editable view: same shape as [`EditablePanel::view`] (a
/// focusable `b` binding `on-input` with a seeded text child), plus the
/// reserved `READONLY` structural-state attribute (as the `Input` widget emits)
/// and an `on-change → CMD_RO_CHANGE` binding — so the runtime gates only its
/// mutation path while focus/caret/selection stay live.
fn readonly_input_view(text: &str) -> View {
    View::from_fragment(ViewFragment::from_nodes(vec![
        ViewNode::element(ComponentId::from_raw(500))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into())))
            .with_children(ViewFragment::from_nodes(vec![
                ViewNode::element(ComponentId::from_raw(501))
                    .with_prop(ViewProperty::new(CLASS, ViewValue::Text("b".into())))
                    .with_prop(ViewProperty::new(READONLY, ViewValue::Bool(true)))
                    .with_event(ViewEventBinding::new(EVENT_INPUT, EditablePanel::CMD_INPUT))
                    .with_event(ViewEventBinding::new(EVENT_CHANGE, CMD_RO_CHANGE))
                    .with_children(ViewFragment::from_nodes(vec![ViewNode::text(
                        text.to_string(),
                    )])),
            ])),
    ]))
}

/// Focuses the readonly editable seeded with `seed` (via a press inside `b`)
/// and returns the runtime plus a mirror of any `on-input` text and a flag set
/// if `on-change` ever fires (it must not on a readonly field).
fn focused_readonly(seed: &str) -> (UiRuntime, Rc<RefCell<String>>, Rc<Cell<bool>>) {
    let mut runtime = TestFixture::runtime();
    let mirror = Rc::new(RefCell::new(String::new()));
    let changed = Rc::new(Cell::new(false));
    {
        let s = mirror.clone();
        runtime.on_text_input(EditablePanel::CMD_INPUT, move |resulting| {
            resulting.clone_into(&mut s.borrow_mut());
        });
        let c = changed.clone();
        runtime.on_change(CMD_RO_CHANGE, move |_| c.set(true));
    }
    let sheet = EditablePanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&readonly_input_view(seed), sheets);
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
    (runtime, mirror, changed)
}

#[test]
fn readonly_field_focuses_and_places_a_caret_and_selects_all() {
    // (a) A readonly field IS focusable and selectable (WHATWG HTML
    // §attr-input-readonly): a press seats focus + a caret, and Ctrl+A selects
    // the whole value — none of which is gated on readonly.
    let (mut runtime, _mirror, _changed) = focused_readonly("hi");
    assert!(runtime.focused().is_some(), "a readonly field takes focus");
    assert_eq!(
        runtime.editing_text(),
        Some("hi"),
        "session seeded from its value"
    );
    let sel = runtime.editing_selection().expect("session active");
    assert!(sel.is_collapsed(), "caret placed (collapsed) on focus");

    // Select-all works: the selection becomes the whole value (non-empty).
    runtime.handle_key(KeyEvent::press(PhysicalKey::ControlLeft, None));
    runtime.handle_key(char_press('a'));
    runtime.handle_key(KeyEvent::release(PhysicalKey::ControlLeft, None));
    let sel = runtime.editing_selection().expect("session active");
    assert!(
        !sel.is_collapsed(),
        "select-all selected the text (selectable)"
    );
    assert_eq!(
        (sel.start(), sel.end()),
        (0, 2),
        "the whole value is selected"
    );
    assert_eq!(
        runtime.editing_text(),
        Some("hi"),
        "selecting did not change the value"
    );
}

#[test]
fn readonly_field_typing_and_backspace_change_nothing_and_fire_no_change() {
    // (b) A readonly field CANNOT be edited: typing, Backspace, Delete and a
    // (single-line) Enter are all no-ops on its value, no `on-input` fires, and
    // `on-change` never fires (its value can't change).
    let (mut runtime, mirror, changed) = focused_readonly("hi");

    runtime.handle_key(char_press('x'));
    assert_eq!(runtime.editing_text(), Some("hi"), "typing inserts nothing");
    runtime.handle_key(named_press(NamedKey::Backspace));
    assert_eq!(
        runtime.editing_text(),
        Some("hi"),
        "Backspace deletes nothing"
    );
    runtime.handle_key(named_press(NamedKey::Delete));
    assert_eq!(runtime.editing_text(), Some("hi"), "Delete deletes nothing");
    assert_eq!(&*mirror.borrow(), "", "no mutating edit fired on-input");

    // Enter (single-line) commits — but a readonly value never changed, so no
    // `change` fires; then blur (focus a non-editable) also fires nothing.
    runtime.handle_key(named_press(NamedKey::Enter));
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 75.0));
    assert!(!changed.get(), "a readonly field never fires on-change");
}

#[test]
fn readonly_field_ime_commit_inserts_nothing() {
    // IME is a mutation entry too: a composition commit on a readonly field is
    // inert — the buffer is unchanged and no `on-input` fires.
    let (mut runtime, mirror, _changed) = focused_readonly("ab");
    assert!(
        !runtime.handle_ime(CompositionEvent::Start),
        "readonly rejects IME start"
    );
    assert!(
        !runtime.handle_ime(CompositionEvent::Commit("に".into())),
        "readonly rejects IME commit"
    );
    assert_eq!(runtime.editing_text(), Some("ab"), "IME committed nothing");
    assert_eq!(&*mirror.borrow(), "", "no on-input fired");
}

#[test]
fn no_editable_focused_makes_a_char_press_a_noop() {
    // (e) With a focused NON-editable node there is no session, so a char
    // press mutates nothing and routes exactly as before.
    let mut runtime = TestFixture::runtime();
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&InteractionPanel::view(), sheets);
    let (_panel, a, _b) = InteractionPanel::nodes(&runtime);

    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
    assert_eq!(runtime.focused(), Some(a));
    assert!(
        runtime.editing_text().is_none(),
        "non-editable focus has no session"
    );

    let dispatch = runtime.handle_key(char_press('a'));
    assert_eq!(dispatch.target(), Some(a), "routed to the focused node");
    assert_eq!(dispatch.command(), None);
    assert!(runtime.editing_text().is_none(), "still no session");
}
