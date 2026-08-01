#![doc = include_str!("text_input_events.doc/_module.md")]
#![forbid(unsafe_code)]

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use obscur_identity::{ComponentId, InternedSymbol};
use obscur_interaction::command::{KeyEvent, LogicalKey, NamedKey, PhysicalKey};
use obscur_interaction::pointer::PointerEventKind;
use obscur_runtime::frame::FrameReason;
use obscur_testkit::fixture::{
    EditablePanel, InteractionPanel, TestFixture, char_press, mouse, named_press,
};
use obscur_testkit::selector::SemanticSelector;
use obscur_view::view::{
    CLASS, EVENT_CHANGE, EVENT_INPUT, View, ViewEventBinding, ViewFragment, ViewNode, ViewProperty,
    ViewValue,
};

// ---- Text entry offered per focus (spec §5.12, §6.15) --------------------

/// The question a host asks before raising an on-screen keyboard: does what
/// holds focus actually take typed text?
///
/// Worth an explicit test rather than trusting the editing session, because a
/// host cannot see the session — it can only see this answer, and a wrong
/// `true` is a keyboard covering half a tablet in an app with no field.
#[test]
fn focus_accepts_text_only_while_an_editable_holds_focus() {
    let mut runtime = TestFixture::runtime();
    let sheet = EditablePanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&EditablePanel::view(""), sheets);

    assert!(
        !runtime.focus_accepts_text(),
        "nothing focused: no text entry is wanted"
    );

    // Press inside the editable `b`, drain the focus frame.
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
    let _ = runtime.run_due_frame(&EditablePanel::view(""), sheets);
    assert!(runtime.focused().is_some(), "the press focused a node");
    assert!(
        runtime.focus_accepts_text(),
        "an editable holds focus: text entry is wanted"
    );
}

/// The other half, and the case the calculator hit: a focused control that is
/// not editable wants no keyboard. A button takes focus like a field does, so
/// "something is focused" is not the test — "the focused thing binds
/// `EVENT_INPUT`" is.
#[test]
fn a_focused_button_wants_no_text_entry() {
    let mut runtime = TestFixture::runtime();
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&InteractionPanel::view(), sheets);

    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
    let _ = runtime.run_due_frame(&InteractionPanel::view(), sheets);
    assert!(
        runtime.focused().is_some(),
        "the press focused the plain node"
    );
    assert!(
        !runtime.focus_accepts_text(),
        "a non-editable holds focus: no keyboard"
    );
}

// ---- EVENT_INPUT: per-keystroke text input dispatch ----------------------

#[test]
fn typing_into_a_focused_editable_edits_app_state_and_re_renders() {
    // (b) End-to-end: typing 'h' then 'i' into a focused `on-input` editable
    // drives the runtime-owned editing session; after each mutating edit the
    // app mirrors the RESULTING TEXT into its own Rc<RefCell<String>>, which
    // the reactive builder reads back so the re-render shows "hi", and
    // Backspace deletes back to "h".
    let mut runtime = TestFixture::runtime();

    let text = Rc::new(RefCell::new(String::new()));
    {
        let s = text.clone();
        runtime.on_text_input(EditablePanel::CMD_INPUT, move |resulting| {
            *s.borrow_mut() = resulting.to_owned();
        });
    }
    {
        let s = text.clone();
        runtime.set_root_view(move || {
            let current = s.borrow().clone();
            EditablePanel::view(&current)
        });
    }
    runtime.set_stylesheets(vec![EditablePanel::sheet()]);

    // First reactive frame populates the tree/boxes so the press can focus.
    runtime.request_frame(FrameReason::UiCommit);
    runtime.run_due_frame_reactive().expect("first frame due");

    // Focus the editable by pressing inside `b`, drain the focus frame.
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
    runtime.run_due_frame_reactive().expect("focus frame due");
    assert!(runtime.focused().is_some(), "the press focused a node");

    // Type 'h' then 'i': each keystroke edits the app's own string.
    let h = runtime.handle_key(char_press('h'));
    assert_eq!(
        h.command(),
        None,
        "a plain char resolves no shortcut command"
    );
    runtime.handle_key(char_press('i'));
    assert_eq!(&*text.borrow(), "hi", "typing edited the app's string");

    // The reactive re-render reflects the edited string.
    runtime
        .run_due_frame_reactive()
        .expect("re-render frame due");
    assert_eq!(
        SemanticSelector::first_text(&runtime),
        "hi",
        "the rebuilt view shows the text"
    );

    // Backspace deletes the last character; the re-render follows.
    runtime.handle_key(KeyEvent::press(
        PhysicalKey::Backspace,
        Some(LogicalKey::named(NamedKey::Backspace)),
    ));
    assert_eq!(&*text.borrow(), "h", "Backspace deleted the last char");
    runtime
        .run_due_frame_reactive()
        .expect("re-render frame due");
    assert_eq!(
        SemanticSelector::first_text(&runtime),
        "h",
        "the rebuilt view shows the deletion"
    );
}

#[test]
fn focusing_an_empty_placeholdered_field_seeds_an_empty_buffer_then_typing_yields_just_the_typed_text()
 {
    // Placeholder is a non-editable overlay (CSS `::placeholder` /
    // `:placeholder-shown`), NOT the field value. Focusing an empty
    // placeholdered field must seed an EMPTY editing buffer (caret at 0) — the
    // `.placeholder` hint text must never leak in — and typing 'x' must yield
    // exactly "x", never "Type here…x".
    let mut runtime = TestFixture::runtime();
    let mirror = Rc::new(RefCell::new(String::new()));
    {
        let s = mirror.clone();
        runtime.on_text_input(EditablePanel::CMD_INPUT, move |resulting| {
            *s.borrow_mut() = resulting.to_owned();
        });
    }
    let sheet = EditablePanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&EditablePanel::placeholder_view(), sheets);

    // Focus the editable by pressing inside it; the runtime auto-seeds.
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
    assert!(
        runtime.focused().is_some(),
        "the press focused the editable"
    );
    assert_eq!(
        runtime.editing_text(),
        Some(""),
        "an empty placeholdered field seeds an EMPTY buffer, not the placeholder text"
    );
    let sel = runtime.editing_selection().expect("session active");
    assert!(sel.is_collapsed());
    assert_eq!(
        sel.focus(),
        0,
        "caret seeded at offset 0 in the empty field"
    );

    runtime.handle_key(char_press('x'));
    assert_eq!(
        runtime.editing_text(),
        Some("x"),
        "typing yields exactly the typed text"
    );
    assert_eq!(
        &*mirror.borrow(),
        "x",
        "on-input delivered exactly \"x\", not the placeholder"
    );
}

#[test]
fn ctrl_char_on_a_focused_editable_is_not_inserted_as_text() {
    // (c) A command modifier makes a char an editing chord (Ctrl+A =
    // SelectAll) or a shortcut, never inserted text: Ctrl+'a' on a focused
    // editable changes no text, so the app's mirror stays empty.
    let mut runtime = TestFixture::runtime();
    let text = Rc::new(RefCell::new(String::new()));
    {
        let s = text.clone();
        runtime.on_text_input(EditablePanel::CMD_INPUT, move |resulting| {
            *s.borrow_mut() = resulting.to_owned();
        });
    }
    let sheet = EditablePanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&EditablePanel::view(""), sheets);

    // Focus the editable.
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
    let _ = runtime.run_due_frame(&EditablePanel::view(""), sheets);
    assert!(runtime.focused().is_some());

    // Hold Ctrl, then press 'a': it is treated as a (possible) shortcut.
    runtime.handle_key(KeyEvent::press(PhysicalKey::ControlLeft, None));
    runtime.handle_key(KeyEvent::press(
        PhysicalKey::KeyA,
        Some(LogicalKey::character('a')),
    ));
    assert_eq!(&*text.borrow(), "", "Ctrl+a was not inserted as text");
}

#[test]
fn char_press_with_no_focused_editable_does_not_dispatch_text() {
    // (d) A char over a focused NON-editable node (no `on-input` in its
    // ancestor chain) is not text — the shortcut/activation path is
    // byte-identical and no TextEdit is dispatched.
    let mut runtime = TestFixture::runtime();
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&InteractionPanel::view(), sheets);
    let (_panel, a, _b) = InteractionPanel::nodes(&runtime);

    let text = Rc::new(RefCell::new(String::new()));
    {
        let s = text.clone();
        runtime.on_text_input(EditablePanel::CMD_INPUT, move |resulting| {
            *s.borrow_mut() = resulting.to_owned();
        });
    }

    // Focus `a`, which binds no `on-input`.
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
    let _ = runtime.run_due_frame(&InteractionPanel::view(), sheets);
    assert_eq!(runtime.focused(), Some(a));

    let dispatch = runtime.handle_key(char_press('a'));
    assert_eq!(dispatch.target(), Some(a), "routed to the focused node");
    assert_eq!(dispatch.command(), None);
    assert_eq!(&*text.borrow(), "", "no editable → no text dispatched");
}

#[test]
fn text_input_with_no_registered_handler_is_a_harmless_noop() {
    // (e) A focused editable bound to `on-input` but with NO handler
    // registered: typing consumes the key as text (so it is not re-read as a
    // shortcut) yet dispatches nothing and never panics.
    let mut runtime = TestFixture::runtime();
    let sheet = EditablePanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&EditablePanel::view(""), sheets);

    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
    let _ = runtime.run_due_frame(&EditablePanel::view(""), sheets);
    assert!(runtime.focused().is_some());
    assert!(!runtime.has_text_input_handler(EditablePanel::CMD_INPUT));

    let dispatch = runtime.handle_key(char_press('h'));
    assert!(
        dispatch.is_handled(),
        "the key still routed to the focused node"
    );
    assert_eq!(dispatch.command(), None, "no shortcut resolved");
}

// ---- EVENT_CHANGE as a COMMIT (blur / single-line Enter) event -----------
// The HTML `change`/`input` event model (WHATWG HTML §4.10.5.5 *Common input
// element APIs*): for a text field `input` fires per keystroke, while `change`
// fires ONCE on COMMIT (blur, or Enter for a single-line input) carrying the
// committed value — and NOT on a focus/click. (For a checkbox/radio/select the
// toggling click IS the change; that click→change path is covered by
// `clicking_a_checkbox_fires_its_change_command_via_activate` in
// `tests/click_commands.rs`.)

/// Field `a`'s command (bound to BOTH `on-input` and `on-change`, as the
/// `Input` widget binds both symbols to one command).
const CMD_FIELD_A: InternedSymbol = InternedSymbol::from_raw(0x1301);
/// Field `b`'s command — a second editable to move focus to (blur `a`).
const CMD_FIELD_B: InternedSymbol = InternedSymbol::from_raw(0x1302);

/// A panel (100x100) with two editable single-line fields: `a` (y∈[0,50)) and
/// `b` (y∈[50,100)), each binding BOTH `on-input` and `on-change` to its own
/// command — mirroring the `Input` widget, which binds `EVENT_INPUT` and
/// `EVENT_CHANGE` to the same symbol. Neither carries a `type`, so both are
/// single-line (`Enter` = commit, not newline).
fn two_editable_fields_view() -> View {
    View::from_fragment(ViewFragment::from_nodes(vec![
        ViewNode::element(ComponentId::from_raw(600))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into())))
            .with_children(ViewFragment::from_nodes(vec![
                ViewNode::element(ComponentId::from_raw(601))
                    .with_prop(ViewProperty::new(CLASS, ViewValue::Text("a".into())))
                    .with_event(ViewEventBinding::new(EVENT_INPUT, CMD_FIELD_A))
                    .with_event(ViewEventBinding::new(EVENT_CHANGE, CMD_FIELD_A)),
                ViewNode::element(ComponentId::from_raw(602))
                    .with_prop(ViewProperty::new(CLASS, ViewValue::Text("b".into())))
                    .with_event(ViewEventBinding::new(EVENT_INPUT, CMD_FIELD_B))
                    .with_event(ViewEventBinding::new(EVENT_CHANGE, CMD_FIELD_B)),
            ])),
    ]))
}

#[test]
fn clicking_into_a_text_field_fires_no_change() {
    // (a) A click into an editable text field is a FOCUS, not a commit: it must
    // resolve NO activation command and fire NO `change` (WHATWG HTML §4.10.5.5
    // — a text field's `change` fires on commit, never on focus/click). This is
    // the fix for the old bug where an editable's `EVENT_CHANGE` was resolved as
    // a pointer-click activation and fired on every focus-click.
    let mut runtime = TestFixture::runtime();
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&two_editable_fields_view(), sheets);

    let changes = Rc::new(Cell::new(0));
    {
        let c = changes.clone();
        runtime.on_change(CMD_FIELD_A, move |_| c.set(c.get() + 1));
    }

    // Click into field `a` (Down+Up at (50, 25)).
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
    let up = runtime.handle_pointer(mouse(PointerEventKind::Up, 50.0, 25.0));

    assert!(runtime.focused().is_some(), "the click focused the field");
    assert_eq!(
        up.command(),
        None,
        "an editable click resolves NO activation command (change is not a click)"
    );
    assert_eq!(
        changes.get(),
        0,
        "clicking into a text field fires NO change"
    );
}

#[test]
fn input_fires_per_keystroke_and_change_fires_once_on_blur_with_the_committed_text() {
    // (b) The full HTML input/change call sequence on a text field:
    //   type "hi"  → `input` fires per char (×2), `change` fires 0×;
    //   click into the (already-focused) field → `change` fires 0×;
    //   blur (focus another field) → `change` fires ONCE with the committed "hi".
    let mut runtime = TestFixture::runtime();
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&two_editable_fields_view(), sheets);

    // Separate registries, both keyed by the SAME field command: per-keystroke
    // `on_text_input` and commit `on_change` — proving they never conflate.
    let inputs = Rc::new(RefCell::new(Vec::<String>::new()));
    let committed = Rc::new(RefCell::new(Vec::<String>::new()));
    {
        let i = inputs.clone();
        runtime.on_text_input(CMD_FIELD_A, move |t| i.borrow_mut().push(t.to_owned()));
    }
    {
        let c = committed.clone();
        runtime.on_change(CMD_FIELD_A, move |t| c.borrow_mut().push(t.to_owned()));
    }

    // Focus field `a`.
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));

    // Type "hi": each keystroke fires `input`, none fires `change`.
    runtime.handle_key(char_press('h'));
    runtime.handle_key(char_press('i'));
    assert_eq!(
        &*inputs.borrow(),
        &["h".to_owned(), "hi".to_owned()],
        "input per keystroke"
    );
    assert!(committed.borrow().is_empty(), "typing fires NO change");

    // Click again into the SAME field: still no commit, no change.
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
    runtime.handle_pointer(mouse(PointerEventKind::Up, 50.0, 25.0));
    assert!(
        committed.borrow().is_empty(),
        "a focus-click fires NO change"
    );

    // Blur: move focus to field `b` (press at (50, 75)) — this COMMITS `a`.
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 75.0));
    assert_eq!(
        &*committed.borrow(),
        &["hi".to_owned()],
        "blur fires `change` ONCE with the committed text"
    );
    // `input` was not fired again by the blur.
    assert_eq!(inputs.borrow().len(), 2, "blur fires no extra `input`");
}

#[test]
fn blur_of_an_unchanged_field_fires_no_change() {
    // The HTML change model fires `change` only when the value ACTUALLY changed
    // since focus: focusing a field and blurring it without editing fires
    // nothing (WHATWG HTML §4.10.5.5).
    let mut runtime = TestFixture::runtime();
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&two_editable_fields_view(), sheets);

    let changes = Rc::new(Cell::new(0));
    {
        let c = changes.clone();
        runtime.on_change(CMD_FIELD_A, move |_| c.set(c.get() + 1));
    }

    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0)); // focus a
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 75.0)); // blur to b
    assert_eq!(
        changes.get(),
        0,
        "blurring an UNCHANGED field fires no change"
    );
}

#[test]
fn enter_on_a_single_line_input_commits_change_without_inserting_and_does_not_refire() {
    // (c) `Enter` in a single-line `input` is a COMMIT (WHATWG HTML §4.10.5.5 /
    // implicit form submission §4.10.21.3): it fires `change` with the committed
    // value but inserts NO newline (multiline insertion is a separate item), and
    // rebaselines so a second `Enter` with no further edits does NOT re-fire.
    let mut runtime = TestFixture::runtime();
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&two_editable_fields_view(), sheets);

    let committed = Rc::new(RefCell::new(Vec::<String>::new()));
    {
        let c = committed.clone();
        runtime.on_change(CMD_FIELD_A, move |t| c.borrow_mut().push(t.to_owned()));
    }

    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0)); // focus a
    runtime.handle_key(char_press('h'));
    runtime.handle_key(char_press('i'));

    // Enter: commit "hi", but insert nothing (the session text stays "hi").
    runtime.handle_key(named_press(NamedKey::Enter));
    assert_eq!(
        runtime.editing_text(),
        Some("hi"),
        "single-line Enter inserts NO newline"
    );
    assert_eq!(
        &*committed.borrow(),
        &["hi".to_owned()],
        "Enter committed `change` once"
    );

    // A second Enter with no edits does not re-fire (baseline was advanced).
    runtime.handle_key(named_press(NamedKey::Enter));
    assert_eq!(
        committed.borrow().len(),
        1,
        "Enter with no new edit does not re-fire change"
    );

    // A subsequent blur (still unchanged since the Enter commit) also stays quiet.
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 75.0));
    assert_eq!(
        committed.borrow().len(),
        1,
        "blur after an Enter-commit does not re-fire"
    );
}

#[test]
fn on_change_and_on_text_input_are_distinct_registries() {
    // Guard the separation directly: registering only `on_text_input` for a
    // field's command leaves `has_change_handler` false (and vice-versa), so an
    // app can bind per-keystroke `input` and commit `change` independently even
    // though the widget binds both events to one symbol.
    let mut runtime = TestFixture::runtime();
    runtime.on_text_input(CMD_FIELD_A, |_| {});
    assert!(runtime.has_text_input_handler(CMD_FIELD_A));
    assert!(
        !runtime.has_change_handler(CMD_FIELD_A),
        "input registry is not the change registry"
    );
    runtime.on_change(CMD_FIELD_A, |_| {});
    assert!(runtime.has_change_handler(CMD_FIELD_A));
}
