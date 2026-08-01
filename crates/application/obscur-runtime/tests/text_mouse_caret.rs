#![doc = include_str!("text_mouse_caret.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_interaction::command::NamedKey;
use obscur_interaction::pointer::PointerEventKind;
use obscur_runtime::UiRuntime;
use obscur_testkit::fixture::{
    EditablePanel, InteractionPanel, TestFixture, char_press, finger_at, mouse, mouse_at,
    named_press,
};

/// The per-glyph advance the mock shaper emits at the default 16px font size
/// (`font_size * ADVANCE_RATIO` = 9.6): every ASCII glyph in "hello" is this
/// wide, so the caret x of boundary `i` is exactly `i * ADVANCE`.
const ADVANCE: f32 = 16.0 * obscur_text::shaping::MockTextShaper::ADVANCE_RATIO;

/// The layout-box origin x of the focused editable's text leaf — the left edge
/// the click-caret local x is measured from.
fn editing_text_origin_x(runtime: &UiRuntime) -> f32 {
    let text_node = runtime
        .tree()
        .iter()
        .find_map(|(id, node)| node.kind().text().map(|_| id))
        .expect("a text node after a frame");
    runtime
        .layout_boxes()
        .get(&text_node)
        .expect("the text leaf has a box")
        .origin()
        .x
}

#[test]
fn click_places_caret_at_nearest_character_boundary() {
    // (a) A pointer click in the focused field places the caret at the nearest
    // cluster boundary: mid-way between offsets 2 and 3 snaps to 2, a click at
    // the start snaps to 0, and a click past the last glyph snaps to the end.
    let (mut runtime, _mirror) = EditablePanel::focused("hello");
    let ox = editing_text_origin_x(&runtime);

    runtime.handle_pointer(mouse(PointerEventKind::Down, ox + ADVANCE * 2.5, 25.0));
    runtime.handle_pointer(mouse(PointerEventKind::Up, ox + ADVANCE * 2.5, 25.0));
    let sel = runtime.editing_selection().expect("session active");
    assert!(sel.is_collapsed(), "a plain click collapses to a caret");
    assert_eq!(sel.focus(), 2, "click at 2.5·advance snaps to boundary 2");

    runtime.handle_pointer(mouse(PointerEventKind::Down, ox, 25.0));
    runtime.handle_pointer(mouse(PointerEventKind::Up, ox, 25.0));
    assert_eq!(
        runtime.editing_selection().unwrap().focus(),
        0,
        "click at the start → 0"
    );

    runtime.handle_pointer(mouse(PointerEventKind::Down, ox + ADVANCE * 6.0, 25.0));
    runtime.handle_pointer(mouse(PointerEventKind::Up, ox + ADVANCE * 6.0, 25.0));
    assert_eq!(
        runtime.editing_selection().unwrap().focus(),
        5,
        "click past the end → 5"
    );
}

#[test]
fn click_up_on_a_field_fires_no_activation_command() {
    // A Down+Up on a text field sets a caret but must NOT spuriously fire a
    // click/change command: the field binds only `on-input`, so activation's
    // click/change lookup finds nothing.
    let (mut runtime, _mirror) = EditablePanel::focused("hello");
    let ox = editing_text_origin_x(&runtime);
    runtime.handle_pointer(mouse(PointerEventKind::Down, ox + ADVANCE * 3.0, 25.0));
    let up = runtime.handle_pointer(mouse(PointerEventKind::Up, ox + ADVANCE * 3.0, 25.0));
    assert_eq!(
        up.command(),
        None,
        "a click on a text field resolves no activation command"
    );
    assert_eq!(
        runtime.editing_selection().unwrap().focus(),
        3,
        "the caret still moved to 3"
    );
}

#[test]
fn drag_selects_a_range_from_press_anchor_to_pointer() {
    // (b) Press at offset 1 (anchor), drag to the offset-4 position → the
    // selection is the range [1, 4) anchored at the press; releasing clears the
    // drag so a later move no longer extends it.
    let (mut runtime, _mirror) = EditablePanel::focused("hello");
    let ox = editing_text_origin_x(&runtime);

    runtime.handle_pointer(mouse(PointerEventKind::Down, ox + ADVANCE * 1.0, 25.0));
    assert_eq!(
        runtime.editing_selection().unwrap().focus(),
        1,
        "press caret at 1"
    );

    runtime.handle_pointer(mouse(PointerEventKind::Move, ox + ADVANCE * 4.0, 25.0));
    let sel = runtime.editing_selection().unwrap();
    assert!(!sel.is_collapsed(), "the drag opened a selection");
    assert_eq!(
        (sel.anchor(), sel.focus()),
        (1, 4),
        "range [1,4) anchored at the press"
    );

    runtime.handle_pointer(mouse(PointerEventKind::Up, ox + ADVANCE * 4.0, 25.0));
    runtime.handle_pointer(mouse(PointerEventKind::Move, ox + ADVANCE * 2.0, 25.0));
    let sel = runtime.editing_selection().unwrap();
    assert_eq!(
        (sel.anchor(), sel.focus()),
        (1, 4),
        "no drag armed after Up → the selection is unchanged"
    );
}

#[test]
fn click_on_a_non_editable_does_not_open_a_session() {
    // (c) A click on a NON-editable node opens no editing session and never
    // panics — the caret/drag path is inert when nothing editable is clicked.
    let mut runtime = TestFixture::runtime();
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&InteractionPanel::view(), sheets);
    let (_panel, a, _b) = InteractionPanel::nodes(&runtime);

    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
    runtime.handle_pointer(mouse(PointerEventKind::Up, 50.0, 25.0));
    assert_eq!(runtime.focused(), Some(a));
    assert!(
        runtime.editing_text().is_none(),
        "no editable clicked → no session"
    );
}

#[test]
fn click_offset_is_the_inverse_of_the_render_caret_x() {
    // (d) The click-caret offset is the exact inverse of the render caret x:
    // the rendered caret x of boundary `i` is `i·advance` past the text origin,
    // and clicking there returns `i`.
    let (mut runtime, _mirror) = EditablePanel::focused("hello");
    let ox = editing_text_origin_x(&runtime);
    for i in 0..=5usize {
        #[allow(clippy::cast_precision_loss)]
        let x = ox + ADVANCE * i as f32;
        runtime.handle_pointer(mouse(PointerEventKind::Down, x, 25.0));
        runtime.handle_pointer(mouse(PointerEventKind::Up, x, 25.0));
        assert_eq!(
            runtime.editing_selection().unwrap().focus(),
            i,
            "click at the caret x of offset {i} returns {i}",
        );
    }
}

#[test]
fn space_key_inserts_a_space_in_a_focused_field() {
    // Regression: the platform delivers Space as `NamedKey::Space` (a control
    // that is NOT being edited uses it for activation), NOT a `Character`, so
    // the editing key path must itself turn it into an inserted `' '` —
    // otherwise a focused field silently swallows every space. Type "ab",
    // press Space, type "cd": the value is "ab cd".
    let (mut runtime, mirror) = EditablePanel::focused("");
    for c in ['a', 'b'] {
        runtime.handle_key(char_press(c));
    }
    assert!(
        runtime
            .handle_key(named_press(NamedKey::Space))
            .is_handled(),
        "Space is consumed as an edit in a focused field",
    );
    for c in ['c', 'd'] {
        runtime.handle_key(char_press(c));
    }
    assert_eq!(runtime.editing_text(), Some("ab cd"));
    assert_eq!(
        mirror.borrow().as_str(),
        "ab cd",
        "the on-input mirror sees the space"
    );
}

#[test]
fn double_click_selects_the_word_under_the_pointer() {
    // (a) Fix 1 (§5.11): a DOUBLE click — two presses on the same point within
    // the multi-click window — selects the WORD under the pointer via
    // `NavigationUnit::Word`. Clicking inside "ab" of "ab cd" selects "ab".
    let (mut runtime, _mirror) = EditablePanel::focused("ab cd");
    let ox = editing_text_origin_x(&runtime);
    let x = ox + ADVANCE * 1.0; // inside the first word "ab"
    let t = 1_000_000;
    runtime.handle_pointer(mouse_at(PointerEventKind::Down, x, 25.0, t));
    runtime.handle_pointer(mouse_at(PointerEventKind::Up, x, 25.0, t));
    // The second press within the window is the double click.
    runtime.handle_pointer(mouse_at(PointerEventKind::Down, x, 25.0, t + 1_000));
    let sel = runtime.editing_selection().expect("session active");
    assert!(
        !sel.is_collapsed(),
        "the double click opened a word selection"
    );
    assert_eq!(
        (sel.start(), sel.end()),
        (0, 2),
        "the word 'ab' under the pointer is selected"
    );
}

/// A click run belongs to the pointer making it.
///
/// Everything the multi-click test looks at — target, point, elapsed time —
/// is held identical here, so the *pointer* is the only axis that differs.
/// With one shared history the second finger read the first finger's press
/// and counted itself as the second click of a double, selecting a word
/// nobody double-tapped.
#[test]
fn a_second_pointer_does_not_continue_the_first_pointers_click_run() {
    let (mut runtime, _mirror) = EditablePanel::focused("ab cd");
    let ox = editing_text_origin_x(&runtime);
    let x = ox + ADVANCE * 1.0; // inside the first word "ab"
    let t = 1_000_000;
    runtime.handle_pointer(finger_at(0, PointerEventKind::Down, x, 25.0, t));
    runtime.handle_pointer(finger_at(0, PointerEventKind::Up, x, 25.0, t));
    // A different finger, same spot, well inside the window.
    runtime.handle_pointer(finger_at(1, PointerEventKind::Down, x, 25.0, t + 1_000));

    let sel = runtime.editing_selection().expect("session active");
    assert!(
        sel.is_collapsed(),
        "the second finger's first press is a single click of its own; it \
         seats a caret rather than completing someone else's double click"
    );
}

#[test]
fn triple_click_selects_the_line_under_the_pointer() {
    // (b) Fix 1 (§5.11): a TRIPLE click selects the whole LINE under the pointer
    // via `NavigationUnit::LineBoundary` — for a single-line field that is the
    // whole value "ab cd".
    let (mut runtime, _mirror) = EditablePanel::focused("ab cd");
    let ox = editing_text_origin_x(&runtime);
    let x = ox + ADVANCE * 1.0;
    let t = 1_000_000;
    runtime.handle_pointer(mouse_at(PointerEventKind::Down, x, 25.0, t));
    runtime.handle_pointer(mouse_at(PointerEventKind::Up, x, 25.0, t));
    runtime.handle_pointer(mouse_at(PointerEventKind::Down, x, 25.0, t + 1_000));
    runtime.handle_pointer(mouse_at(PointerEventKind::Up, x, 25.0, t + 1_000));
    // The third press within the window is the triple click.
    runtime.handle_pointer(mouse_at(PointerEventKind::Down, x, 25.0, t + 2_000));
    let sel = runtime.editing_selection().expect("session active");
    assert_eq!(
        (sel.start(), sel.end()),
        (0, 5),
        "the whole single line 'ab cd' is selected"
    );
}

#[test]
fn single_click_still_collapses_the_caret_byte_identically() {
    // (c) Fix 1 byte-identical guard: a lone click (outside any prior press's
    // multi-click window) still collapses to a plain caret at the clicked
    // boundary — unchanged from the pre-fix single-click behaviour.
    let (mut runtime, _mirror) = EditablePanel::focused("ab cd");
    let ox = editing_text_origin_x(&runtime);
    let x = ox + ADVANCE * 4.0; // inside "cd"
    runtime.handle_pointer(mouse_at(PointerEventKind::Down, x, 25.0, 1_000_000));
    runtime.handle_pointer(mouse_at(PointerEventKind::Up, x, 25.0, 1_000_000));
    let sel = runtime.editing_selection().expect("session active");
    assert!(sel.is_collapsed(), "a single click collapses to a caret");
    assert_eq!(sel.focus(), 4, "the caret sits at the clicked boundary");
}
