#![doc = include_str!("scroll_containers.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_geometry::Insets;
use obscur_identity::NodeId;
use obscur_interaction::pointer::PointerEventKind;
use obscur_runtime::UiRuntime;
use obscur_scene::command::PaintCommand;
use obscur_testkit::fixture::{EditablePanel, InteractionPanel, ScrollingList, TestFixture, mouse};
use obscur_view::view::{EVENT_INPUT, ViewEventBinding};

/// The 200×120 scroller of six 40px rows, with **row 3** bound as an editable
/// field — a field in the middle of a scrolling screen, where there is
/// content below it to scroll past. Returns `(runtime, scroller, rows)`.
fn reveal_fixture() -> (UiRuntime, NodeId, Vec<NodeId>) {
    let mut runtime = TestFixture::runtime();
    let view = ScrollingList::with_rows(|index, row| {
        if index == 3 {
            row.with_event(ViewEventBinding::new(EVENT_INPUT, EditablePanel::CMD_INPUT))
        } else {
            row
        }
    });
    let sheet = ScrollingList::sheet();
    let _ = runtime.render_now(&view, std::slice::from_ref(&sheet));
    let (scroller, rows) = ScrollingList::nodes(&runtime);
    (runtime, scroller, rows)
}

#[test]
fn revealing_a_node_below_the_fold_scrolls_the_minimum() {
    let (mut runtime, scroller, rows) = reveal_fixture();
    // Row 5 spans y 200..240; the scrollport is 120 tall. The least scroll
    // that shows all of it is 240 − 120 = 120.
    assert!(runtime.scroll_into_view(rows[5]));
    assert!(
        (runtime.scroll_top(scroller) - 120.0).abs() < f32::EPSILON,
        "got {}",
        runtime.scroll_top(scroller)
    );
}

#[test]
fn revealing_a_node_already_in_view_moves_nothing() {
    // The minimum scroll that reveals something already visible is none —
    // this is a reveal, not a recentre.
    let (mut runtime, scroller, rows) = reveal_fixture();
    assert!(!runtime.scroll_into_view(rows[1]));
    assert!((runtime.scroll_top(scroller) - 0.0).abs() < f32::EPSILON);
}

#[test]
fn revealing_a_node_above_the_fold_scrolls_back_up() {
    let (mut runtime, scroller, rows) = reveal_fixture();
    runtime.scroll_by(scroller, 120.0);
    assert!(runtime.scroll_into_view(rows[0]));
    assert!(
        (runtime.scroll_top(scroller) - 0.0).abs() < f32::EPSILON,
        "got {}",
        runtime.scroll_top(scroller)
    );
}

/// The case §6.15 exists for. The keyboard occludes the bottom of the
/// scrollport, so the region a field must be revealed *into* is smaller —
/// and a reveal that ignored it would decide the field was already visible
/// and leave it behind the keys.
#[test]
fn the_keyboard_shrinks_the_region_a_field_is_revealed_into() {
    // Row 3 spans y 120..160. With the whole 120px scrollport visible the
    // least scroll that shows it is 160 − 120 = 40.
    let (mut runtime, scroller, rows) = reveal_fixture();
    runtime.scroll_into_view(rows[3]);
    let without_keyboard = runtime.scroll_top(scroller);
    assert!(
        (without_keyboard - 40.0).abs() < f32::EPSILON,
        "got {without_keyboard}"
    );

    // Now a keyboard covering the bottom 240px of the 300px viewport leaves
    // the scrollport visible only down to y=60 — 60px of it — so the same row
    // has to come 60px further up: 160 − 60 = 100.
    let (mut runtime, scroller, rows) = reveal_fixture();
    runtime.set_keyboard_insets(Insets {
        top: 0.0,
        right: 0.0,
        bottom: 240.0,
        left: 0.0,
    });
    runtime.scroll_into_view(rows[3]);
    let with_keyboard = runtime.scroll_top(scroller);
    assert!(
        (with_keyboard - 100.0).abs() < f32::EPSILON,
        "got {with_keyboard}"
    );
    assert!(
        with_keyboard > without_keyboard,
        "the keyboard forced the field further up the scrollport"
    );
}

#[test]
fn a_node_with_no_scrollable_ancestor_cannot_be_revealed() {
    // Honest `false` rather than a silent no-op that looks like success: a
    // form laid out on the root has nowhere to go.
    let mut runtime = TestFixture::runtime();
    let sheet = InteractionPanel::sheet();
    let _ = runtime.render_now(&InteractionPanel::view(), std::slice::from_ref(&sheet));
    let (_panel, a, _b) = InteractionPanel::nodes(&runtime);
    assert!(!runtime.scroll_into_view(a));
}

/// A keyboard rising over the field the user is already typing in: nothing
/// about the focus changed, so only the keyboard's arrival can trigger the
/// reveal.
#[test]
fn a_keyboard_report_reveals_the_field_that_is_already_focused() {
    let (mut runtime, scroller, rows) = reveal_fixture();
    // Bring the editable row into view (scroll 40, so it sits at screen
    // y 80..120) and focus it by pressing on it.
    runtime.scroll_into_view(rows[3]);
    let seated = runtime.scroll_top(scroller);
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 100.0));
    assert_eq!(
        runtime.focused(),
        Some(rows[3]),
        "the press focused the editable row"
    );

    runtime.set_keyboard_insets(Insets {
        top: 0.0,
        right: 0.0,
        bottom: 240.0,
        left: 0.0,
    });
    assert!(
        runtime.scroll_top(scroller) > seated,
        "the field came up out from behind the keyboard: {} → {}",
        seated,
        runtime.scroll_top(scroller)
    );
}

/// The limit of scrolling as an answer to the keyboard, stated as a test
/// rather than left to be discovered.
///
/// A field at the very bottom of the content cannot be lifted above the
/// keyboard, because there is nothing below it to scroll past — the scroll is
/// already at its maximum. Revealing it needs the *other* half of §6.15's
/// "overlay/resize/pan behavior": the viewport shrinking, or the content
/// gaining room, when the keyboard appears. Neither is built, so this is what
/// currently happens, and a screen that puts a field flush against the bottom
/// of a scroller will have it covered.
#[test]
fn a_field_at_the_very_bottom_cannot_be_lifted_above_the_keyboard() {
    let (mut runtime, scroller, rows) = reveal_fixture();
    runtime.scroll_into_view(rows[5]);
    let at_end = runtime.scroll_top(scroller);
    assert!((at_end - 120.0).abs() < f32::EPSILON, "scrolled to the end");

    runtime.set_keyboard_insets(Insets {
        top: 0.0,
        right: 0.0,
        bottom: 240.0,
        left: 0.0,
    });
    assert!(!runtime.scroll_into_view(rows[5]), "there is nowhere to go");
    assert!((runtime.scroll_top(scroller) - at_end).abs() < f32::EPSILON);
}

#[test]
fn overflowing_content_makes_a_scroll_container_with_a_clamped_scroll_top() {
    let mut runtime = TestFixture::runtime();
    let sheet = ScrollingList::sheet();
    let _ = runtime.render_now(&ScrollingList::view(), std::slice::from_ref(&sheet));
    let (scroller, _rows) = ScrollingList::nodes(&runtime);

    // Content (240) overflows the 120-px box → it is a scroll container at 0.
    assert_eq!(runtime.scroll_top(scroller), 0.0);
    // scroll_by advances, clamped to content_height − scrollport = 240 − 120.
    assert_eq!(runtime.scroll_by(scroller, 80.0), 80.0);
    assert_eq!(runtime.scroll_top(scroller), 80.0);
    assert_eq!(
        runtime.scroll_by(scroller, 1000.0),
        120.0,
        "clamps to the scrollable range"
    );
    assert_eq!(
        runtime.scroll_by(scroller, -1000.0),
        0.0,
        "clamps at the top"
    );
    // set_scroll_top clamps identically; a non-container node is ignored.
    runtime.set_scroll_top(scroller, 60.0);
    assert_eq!(runtime.scroll_top(scroller), 60.0);
}

#[test]
fn scroll_offset_translates_content_and_reduces_to_identity_at_zero() {
    let mut runtime = TestFixture::runtime();
    let sheet = ScrollingList::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&ScrollingList::view(), sheets);
    let (scroller, _rows) = ScrollingList::nodes(&runtime);

    // At scroll_top 0 there is NO scroll PushTransform (identity) — the only
    // change from a plain overflow:hidden box is the scrollbar fills.
    let idle = runtime.snapshot();
    let idle_translates = idle
        .display_list()
        .commands()
        .iter()
        .filter(|c| matches!(c, PaintCommand::PushTransform(t) if t.coeffs[5] != 0.0))
        .count();
    assert_eq!(
        idle_translates, 0,
        "scroll transform is elided at scroll_top == 0"
    );

    // Scrolled: a PushTransform(translate(0,-80)) wraps the content.
    runtime.scroll_by(scroller, 80.0);
    let _ = runtime.render_now(&ScrollingList::view(), sheets);
    let scrolled = runtime.snapshot();
    let has =
        scrolled.display_list().commands().iter().any(
            |c| matches!(c, PaintCommand::PushTransform(t) if (t.coeffs[5] + 80.0).abs() < 0.01),
        );
    assert!(has, "scrolled content is wrapped in translate(0,-80)");
}

#[test]
fn pointer_hit_test_composes_the_container_scroll_offset() {
    let mut runtime = TestFixture::runtime();
    let sheet = ScrollingList::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&ScrollingList::view(), sheets);
    let (scroller, rows) = ScrollingList::nodes(&runtime);

    // Idle: a click at the top of the scrollport lands on row 0.
    let hit = runtime
        .handle_pointer(mouse(PointerEventKind::Move, 100.0, 20.0))
        .target();
    assert_eq!(hit, Some(rows[0]), "idle: (100,20) hits row 0");

    // Scroll up two rows: the SAME point now maps to row 2 (content y 20+80).
    runtime.scroll_by(scroller, 80.0);
    let hit = runtime
        .handle_pointer(mouse(PointerEventKind::Move, 100.0, 20.0))
        .target();
    assert_eq!(hit, Some(rows[2]), "scrolled: (100,20) hits row 2");
}

#[test]
fn absent_overflow_records_no_scroll_state_and_composes_identically() {
    // Byte-identical guard: a view with NO scroll container composes exactly as
    // before per-element scroll existed (no scroll transform, no scrollbar).
    let mut a = TestFixture::runtime();
    let mut b = TestFixture::runtime();
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let list_a = a
        .render_now(&InteractionPanel::view(), sheets)
        .snapshot()
        .display_list()
        .commands()
        .to_vec();
    // Populate b, then drive the scroll API against a non-container node (inert).
    let _ = b.render_now(&InteractionPanel::view(), sheets);
    let (panel, _a, _b) = InteractionPanel::nodes(&b);
    b.set_scroll_top(panel, 40.0);
    assert_eq!(
        b.scroll_top(panel),
        0.0,
        "a non-overflowing node owns no scroll position"
    );
    let list_b = b
        .render_now(&InteractionPanel::view(), sheets)
        .snapshot()
        .display_list()
        .commands()
        .to_vec();
    assert_eq!(
        list_a, list_b,
        "no scroll container ⇒ byte-identical display list"
    );
}
