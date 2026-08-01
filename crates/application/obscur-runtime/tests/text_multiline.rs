#![doc = include_str!("text_multiline.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_geometry::{Point, Rect};
use obscur_identity::NodeId;
use obscur_interaction::command::NamedKey;
use obscur_interaction::pointer::PointerEventKind;
use obscur_runtime::UiRuntime;
use obscur_scene::command::{DisplayList, PaintCommand};
use obscur_testkit::fixture::{
    EditablePanel, SingleLineField, TestFixture, TextArea, char_press, mouse, named_press,
};
use obscur_testkit::selector::NodePath;

/// Focuses the empty `textarea` and returns the runtime with an (empty) session.
fn focused_textarea() -> UiRuntime {
    focused_area(&TextArea::sheet())
}

/// Focuses the empty **scrolling** `<textarea>` and returns the runtime with an
/// (empty) session, ready for typing.
fn focused_scrolling_textarea() -> UiRuntime {
    focused_area(&TextArea::scrolling_sheet())
}

/// The shared arrangement of the two above: render the empty textarea under
/// `sheet`, then press inside it so the runtime seeds a session.
fn focused_area(sheet: &obscur_style::matching::Stylesheet) -> UiRuntime {
    let mut runtime = TestFixture::runtime();
    runtime.on_text_input(EditablePanel::CMD_INPUT, move |_| {});
    let sheets = std::slice::from_ref(sheet);
    let _ = runtime.render_now(&TextArea::view(""), sheets);
    runtime.handle_pointer(mouse(PointerEventKind::Down, 20.0, 10.0));
    assert!(
        runtime.focused().is_some(),
        "the press focused the textarea"
    );
    runtime
}

/// The textarea's editable node id (root → panel → area) and its text-leaf id.
fn textarea_nodes(runtime: &UiRuntime) -> (NodeId, NodeId) {
    let field = NodePath::new(&[0, 0]).resolve(runtime);
    let text = NodePath::new(&[0, 0, 0]).resolve(runtime);
    (field, text)
}

/// Re-renders the seed view (retained text `""`); the SESSION holds the typed
/// value, so compose paints the live multi-line text + line-aware caret.
fn render_textarea_frame(runtime: &mut UiRuntime) -> DisplayList {
    render_area_frame(runtime, &TextArea::sheet())
}

/// Re-renders the scrolling-textarea seed view (retained text `""`; the SESSION
/// holds the typed value) so the field's grown box + scroll offset lay out.
fn render_scrolling_textarea_frame(runtime: &mut UiRuntime) -> DisplayList {
    render_area_frame(runtime, &TextArea::scrolling_sheet())
}

/// The shared body of the two above.
fn render_area_frame(
    runtime: &mut UiRuntime,
    sheet: &obscur_style::matching::Stylesheet,
) -> DisplayList {
    let sheets = std::slice::from_ref(sheet);
    let result = runtime.render_now(&TextArea::view(""), sheets);
    result.snapshot().display_list().clone()
}

/// The painted caret rect (the thin, width-1 fill), or `None` if absent.
fn painted_caret_rect(list: &DisplayList) -> Option<Rect> {
    list.commands().iter().find_map(|c| match c {
        PaintCommand::DrawPath { path, .. } => {
            path.bounds().filter(|r| (r.size.width - 1.0).abs() < 1e-6)
        }
        _ => None,
    })
}

/// Types `text` into the focused editable one printable char at a time, mapping
/// a `\n` to a real `Enter` key press (through the runtime's editing keymap).
fn type_multiline(runtime: &mut UiRuntime, text: &str) {
    for ch in text.chars() {
        if ch == '\n' {
            runtime.handle_key(named_press(NamedKey::Enter));
        } else {
            runtime.handle_key(char_press(ch));
        }
    }
}

// ---- Multi-line editing: textarea, Enter → newline, line-aware caret -------

#[test]
fn enter_in_a_textarea_inserts_a_newline_and_moves_the_caret_to_the_next_line() {
    // (a) Enter in a MULTILINE field inserts `\n` (CSS Text 3 §4.1.1 segment
    // break) and the caret advances past it — to line 2, column 0. Typing "ab"
    // then Enter yields the buffer "ab\n" with the caret at byte 3, and the
    // painted caret sits on the SECOND line (y ≈ line-height) at x ≈ 0.
    let mut runtime = focused_textarea();
    type_multiline(&mut runtime, "ab\n");
    assert_eq!(
        runtime.editing_text(),
        Some("ab\n"),
        "Enter inserted a newline"
    );
    assert_eq!(
        runtime.editing_selection().unwrap().focus(),
        3,
        "caret is past the newline (line 2, col 0)",
    );

    let (_, text_node) = textarea_nodes(&runtime);
    let text_origin = runtime
        .layout_boxes()
        .get(&text_node)
        .unwrap()
        .bounds
        .origin;
    let list = render_textarea_frame(&mut runtime);
    let caret = painted_caret_rect(&list).expect("a caret is painted");
    assert!(
        (caret.origin.y - text_origin.y - TextArea::LINE_HEIGHT).abs() < 1e-3,
        "the caret is on the 2nd line (y ≈ line-height 20): rel y {}",
        caret.origin.y - text_origin.y,
    );
    assert!(
        (caret.origin.x - text_origin.x).abs() < 1e-3,
        "the caret is at column 0 on line 2: rel x {}",
        caret.origin.x - text_origin.x,
    );
}

#[test]
fn arrow_up_and_down_move_the_caret_between_wrapped_lines() {
    // (b) In a textarea holding "ab\ncd" (caret at end, line 2 col 2), ArrowUp
    // moves to line 1 at roughly the same x (byte 2, end of "ab"); ArrowDown
    // moves back to line 2 at that x (byte 5, end of "cd").
    let mut runtime = focused_textarea();
    type_multiline(&mut runtime, "ab\ncd");
    assert_eq!(runtime.editing_text(), Some("ab\ncd"));
    assert_eq!(
        runtime.editing_selection().unwrap().focus(),
        5,
        "caret at end"
    );

    runtime.handle_key(named_press(NamedKey::ArrowUp));
    assert_eq!(
        runtime.editing_selection().unwrap().focus(),
        2,
        "ArrowUp lands on line 1 at ~the same x (byte 2)",
    );

    runtime.handle_key(named_press(NamedKey::ArrowDown));
    assert_eq!(
        runtime.editing_selection().unwrap().focus(),
        5,
        "ArrowDown returns to line 2 at ~the same x (byte 5)",
    );
}

#[test]
fn caret_on_the_second_line_paints_within_that_line_not_the_summed_width() {
    // (c) With "ab\ncd" (caret at end), the painted caret sits on line 2 at
    // y ≈ line-height, and its x is measured WITHIN line 2 ("cd", ~2 advances) —
    // NOT the summed width of both lines ("abcd", ~4 advances), which is the
    // single-line bug this replaces.
    let mut runtime = focused_textarea();
    type_multiline(&mut runtime, "ab\ncd");

    let (_, text_node) = textarea_nodes(&runtime);
    let text_origin = runtime
        .layout_boxes()
        .get(&text_node)
        .unwrap()
        .bounds
        .origin;
    let list = render_textarea_frame(&mut runtime);
    let caret = painted_caret_rect(&list).expect("a caret");
    let advance = 16.0_f32 * obscur_text::shaping::MockTextShaper::ADVANCE_RATIO;
    assert!(
        (caret.origin.y - text_origin.y - TextArea::LINE_HEIGHT).abs() < 1e-3,
        "caret y is on line 2 (≈ line-height 20): rel y {}",
        caret.origin.y - text_origin.y,
    );
    let rel_x = caret.origin.x - text_origin.x;
    assert!(
        (rel_x - advance * 2.0).abs() < 1e-2,
        "caret x is line-2-local (~2 advances = {}), got {rel_x}",
        advance * 2.0,
    );
    assert!(
        rel_x < advance * 3.0,
        "caret x is NOT the summed width of both lines (~4 advances): {rel_x}",
    );
}

#[test]
fn a_click_on_the_second_line_resolves_to_a_second_line_byte() {
    // (d) A pointer press within the SECOND line's y-band resolves to a byte on
    // line 2 (>= 3, the start of "cd" after "ab\n") — the 2D hit-test picks the
    // line by y then the byte by x, so a click never lands on line 1.
    let mut runtime = focused_textarea();
    type_multiline(&mut runtime, "ab\ncd");
    // Lay out the multi-line text box (measured from the session) so the second
    // line has a real y-band to click into.
    let _ = render_textarea_frame(&mut runtime);

    let (_, text_node) = textarea_nodes(&runtime);
    let origin = runtime
        .layout_boxes()
        .get(&text_node)
        .unwrap()
        .bounds
        .origin;
    // Click into line 2 (y ∈ [20, 40)), a little in from the left.
    runtime.handle_pointer(mouse(
        PointerEventKind::Down,
        origin.x + 12.0,
        origin.y + 25.0,
    ));
    let focus = runtime.editing_selection().unwrap().focus();
    assert!(
        (3..=5).contains(&focus),
        "a click on line 2 resolves to a line-2 byte (3..=5), got {focus}",
    );
}

#[test]
fn enter_in_a_single_line_input_does_not_insert_a_newline() {
    // (e) A single-line `input`'s Enter is NOT an edit — it is left for the
    // activation/submit path — so the buffer is unchanged (no `\n`), proving the
    // multiline Enter behaviour is gated on `type=textarea`.
    let mut runtime = focused_single_line_field();
    for c in "ab".chars() {
        runtime.handle_key(char_press(c));
    }
    assert_eq!(runtime.editing_text(), Some("ab"));
    runtime.handle_key(named_press(NamedKey::Enter));
    assert_eq!(
        runtime.editing_text(),
        Some("ab"),
        "single-line Enter inserts no newline (activation, not an edit)",
    );
}

/// Focuses the single-line `field` (seeded empty) and returns the runtime. The
/// `on-input` handler is registered but the resulting text is not mirrored.
fn focused_single_line_field() -> UiRuntime {
    let mut runtime = TestFixture::runtime();
    runtime.on_text_input(EditablePanel::CMD_INPUT, move |_| {});
    let sheet = SingleLineField::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&SingleLineField::view(""), sheets);
    // Press inside the field to focus it; the runtime seeds an empty session.
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 10.0));
    assert!(runtime.focused().is_some(), "the press focused the field");
    runtime
}

// ---- Multi-line VERTICAL caret scroll (textarea, CSS Overflow 3 §3.1) ------

#[test]
fn typing_past_the_box_scrolls_the_textarea_caret_line_into_view() {
    // (a) Typing enough lines to exceed the 60-px (3-line) box scrolls the
    // overflow container's `scroll_top` past 0 and keeps the caret's LINE inside
    // the visible box (CSS Overflow 3 §3.1 scroll-into-view) — the vertical
    // analogue of the single-line horizontal caret scroll.
    let mut runtime = focused_scrolling_textarea();
    // Six lines (120 px of content) in a 3-line (60 px) box; the caret ends on
    // line 6.
    type_multiline(&mut runtime, "l1\nl2\nl3\nl4\nl5\nl6");
    let (field, _text) = textarea_nodes(&runtime);
    // Lay out the grown box + scroll offset.
    let list = render_scrolling_textarea_frame(&mut runtime);

    // The overflow container scrolled: the caret at the bottom pushed `scroll_top`
    // to its max (120 content − 60 box = 60).
    let scroll_top = runtime.scroll_top(field);
    assert!(
        scroll_top > 0.0,
        "typing past the box scrolls it (scroll_top {scroll_top})"
    );
    assert!(
        (scroll_top - 60.0).abs() < 1e-3,
        "scrolled to the max (60): {scroll_top}"
    );

    // The painted caret line, offset by the container's `translate(0, −scroll_top)`,
    // sits within the visible box's y-band — the last line is visible, not clipped.
    let field_box = runtime.layout_boxes().get(&field).unwrap().bounds;
    let caret = painted_caret_rect(&list).expect("a caret is painted");
    let caret_top = caret.origin.y - scroll_top;
    let caret_bottom = caret.bottom() - scroll_top;
    assert!(
        caret_top >= field_box.origin.y - 1e-3 && caret_bottom <= field_box.bottom() + 1e-3,
        "caret line y [{caret_top}, {caret_bottom}] within the box [{}, {}]",
        field_box.origin.y,
        field_box.bottom(),
    );
    // The unscrolled caret sits below the box (the last line, before the scroll
    // transform), proving the top lines are scrolled out + clipped.
    assert!(
        caret.origin.y > field_box.bottom(),
        "the caret's unscrolled line is below the box (top lines scrolled out): {} vs {}",
        caret.origin.y,
        field_box.bottom(),
    );

    // A scroll transform brackets the content: `translate(0, −scroll_top)`.
    let has_scroll_transform = list.commands().iter().any(
        |c| matches!(c, PaintCommand::PushTransform(t) if (t.coeffs[5] + scroll_top).abs() < 0.01),
    );
    assert!(
        has_scroll_transform,
        "the content is bracketed in a scroll transform"
    );
}

#[test]
fn a_textarea_whose_content_fits_stays_unscrolled() {
    // (b) A scrolling textarea whose content fits its 3-line box keeps
    // `scroll_top == 0` — byte-identical to the unscrolled paint (the caret
    // follow is a no-op when nothing overflows).
    let mut runtime = focused_scrolling_textarea();
    type_multiline(&mut runtime, "l1\nl2"); // 40 px of content in a 60-px box
    let (field, _text) = textarea_nodes(&runtime);
    let _ = render_scrolling_textarea_frame(&mut runtime);
    assert_eq!(
        runtime.scroll_top(field),
        0.0,
        "a fitting textarea does not scroll"
    );
}

#[test]
fn the_wheel_still_scrolls_the_textarea_sharing_the_caret_scroll_top() {
    // (c) The wheel/overflow path is intact and shares the SAME `scroll_top` the
    // caret follow drives: after typing scrolls it to the max, a wheel-up gesture
    // moves that same value back (CSS Overflow 3 §scroll-container).
    let mut runtime = focused_scrolling_textarea();
    type_multiline(&mut runtime, "l1\nl2\nl3\nl4\nl5\nl6");
    let (field, _text) = textarea_nodes(&runtime);
    let _ = render_scrolling_textarea_frame(&mut runtime);
    let before = runtime.scroll_top(field);
    assert!(
        (before - 60.0).abs() < 1e-3,
        "caret follow put scroll_top at the max: {before}"
    );

    // Wheel up by 25 px over the field: the SAME container scrolls back.
    let target = runtime.handle_scroll(-25.0, Point::new(20.0, 10.0));
    assert_eq!(
        target,
        Some(field),
        "the wheel targets the textarea container"
    );
    let after = runtime.scroll_top(field);
    assert!(
        after < before && (after - 35.0).abs() < 1e-3,
        "the wheel moved the shared scroll_top back (before {before}, after {after})",
    );
}

// ---- MEASURE↔COMPOSE editing consistency (CSS Text 3 §4.1.1) --------------

/// Focuses the empty `wrapfield` and returns the runtime with an (empty) session.
fn focused_wrap_field() -> UiRuntime {
    let mut runtime = TestFixture::runtime();
    runtime.on_text_input(EditablePanel::CMD_INPUT, move |_| {});
    let sheet = SingleLineField::wrapping_sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&SingleLineField::wrapping_view(""), sheets);
    runtime.handle_pointer(mouse(PointerEventKind::Down, 20.0, 10.0));
    assert!(
        runtime.focused().is_some(),
        "the press focused the wrap field"
    );
    runtime
}

/// Re-renders the seed view (retained text `""`) and returns the display list;
/// the SESSION still holds the typed value, so this is the lag frame.
fn render_wrap_frame(runtime: &mut UiRuntime) -> DisplayList {
    let sheet = SingleLineField::wrapping_sheet();
    let sheets = std::slice::from_ref(&sheet);
    let result = runtime.render_now(&SingleLineField::wrapping_view(""), sheets);
    result.snapshot().display_list().clone()
}

#[test]
fn an_edited_wrapping_field_is_measured_from_its_session_so_box_equals_painted_lines() {
    // The MEASURE↔COMPOSE invariant (CSS Text 3 §4.1.1): while a field is
    // focused, its box is SIZED and PAINTED from the SAME session buffer with
    // the SAME processing (raw, the field's real `white-space`). The retained
    // tree lags the session between a keystroke and the app's reactive rebuild —
    // here it stays `""` — so a pass reading the retained value would size a
    // 1-line box while compose painted several lines. With the session threaded
    // into measure, the arranged text box height equals the painted line count ×
    // line-height, and differs from the stale retained box.
    let mut runtime = focused_wrap_field();

    // Baseline: the (empty) retained/session text box is a single 20px line.
    let text_node = NodePath::new(&[0, 0, 0]).resolve(&runtime);
    let stale_height = runtime
        .layout_boxes()
        .get(&text_node)
        .expect("a text box")
        .bounds
        .size
        .height;

    // Type a space-separated value that wraps to several lines in the 40px box
    // (this fixture is a single-line-typed `input`, so wrapping is driven by
    // spaces; the textarea `\n` path is covered by the multiline tests above).
    for c in "aa aa aa aa".chars() {
        runtime.handle_key(char_press(c));
    }
    assert_eq!(
        runtime.editing_text(),
        Some("aa aa aa aa"),
        "session holds the value"
    );

    // The lag frame: the retained text is still "", the session drives paint.
    let list = render_wrap_frame(&mut runtime);
    let text_node = NodePath::new(&[0, 0, 0]).resolve(&runtime);
    let measured_height = runtime
        .layout_boxes()
        .get(&text_node)
        .expect("a text box")
        .bounds
        .size
        .height;
    // One `DrawText` run is emitted per wrapped line, so the run count IS the
    // painted line count.
    let painted_lines = list
        .commands()
        .iter()
        .filter(|c| matches!(c, PaintCommand::DrawText { .. }))
        .count();

    let line_height = 20.0_f32;
    assert!(
        painted_lines >= 2,
        "the session buffer wraps to multiple lines (got {painted_lines})",
    );
    assert!(
        (measured_height - painted_lines as f32 * line_height).abs() < 1e-3,
        "measured box height {measured_height} == painted lines {painted_lines} × {line_height}",
    );
    assert!(
        measured_height > stale_height + 1e-3,
        "the box grew past the stale retained (1-line) height {stale_height} → measure reads \
         the SESSION, not the lagging retained tree (measured {measured_height})",
    );
}
