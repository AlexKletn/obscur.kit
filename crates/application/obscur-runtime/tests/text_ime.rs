#![doc = include_str!("text_ime.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_runtime::UiRuntime;
use obscur_scene::command::PaintCommand;
use obscur_testkit::fixture::{EditablePanel, TestFixture, char_press};
use obscur_text::ime::{CompositionEvent, Preedit};

/// Collects the composed text (concatenated across every `DrawText` run) of a
/// freshly rendered frame — the DISPLAYED text, including any spliced preedit.
fn composed_text(runtime: &mut UiRuntime, seed: &str) -> String {
    let sheet = EditablePanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let result = runtime.render_now(&EditablePanel::view(seed), sheets);
    result
        .snapshot()
        .display_list()
        .commands()
        .iter()
        .filter_map(|c| match c {
            PaintCommand::DrawText { run, .. } => Some(run.text()),
            _ => None,
        })
        .collect()
}

#[test]
fn ime_preedit_is_shown_but_not_committed_then_commit_folds_it_in() {
    // (a) Focus "ab" (caret at end). Start a composition, then update it with
    // the preedit "ni": the BUFFER stays "ab" (preedit is not committed text,
    // §5.12), `composition_preedit` is "ni", and the DISPLAYED text splices it
    // to "abni". Committing "に" replaces the (empty) composition span, so the
    // buffer becomes "abに", the composition is cleared, and the field's
    // `on-input` fires with the resulting text.
    let (mut runtime, mirror) = EditablePanel::focused("ab");
    assert_eq!(
        runtime.editing_selection().unwrap().focus(),
        2,
        "caret at end"
    );

    assert!(runtime.handle_ime(CompositionEvent::Start), "start applied");
    assert!(
        runtime.handle_ime(CompositionEvent::Update(Preedit::new("ni"))),
        "update applied"
    );

    assert_eq!(
        runtime.editing_text(),
        Some("ab"),
        "buffer unchanged by preedit"
    );
    assert_eq!(
        runtime.composition_preedit(),
        Some("ni"),
        "preedit is shown"
    );
    assert_eq!(
        composed_text(&mut runtime, "ab"),
        "abni",
        "preedit spliced for display"
    );

    assert!(
        runtime.handle_ime(CompositionEvent::Commit(Box::from("に"))),
        "commit applied"
    );
    assert_eq!(
        runtime.editing_text(),
        Some("abに"),
        "commit folded the text in"
    );
    assert_eq!(runtime.composition_preedit(), None, "composition cleared");
    assert_eq!(
        &*mirror.borrow(),
        "abに",
        "EVENT_INPUT fired with the committed text"
    );
}

#[test]
fn ime_cancel_after_update_drops_the_preedit_with_no_buffer_change() {
    // (b) Cancel after an Update discards the preedit and leaves the buffer
    // untouched: "ab" in, "ab" out, no preedit.
    let (mut runtime, mirror) = EditablePanel::focused("ab");
    runtime.handle_ime(CompositionEvent::Start);
    runtime.handle_ime(CompositionEvent::Update(Preedit::new("ni")));
    assert_eq!(runtime.composition_preedit(), Some("ni"));

    assert!(
        runtime.handle_ime(CompositionEvent::Cancel),
        "cancel applied"
    );
    assert_eq!(
        runtime.editing_text(),
        Some("ab"),
        "buffer unchanged by cancel"
    );
    assert_eq!(runtime.composition_preedit(), None, "preedit dropped");
    assert_eq!(&*mirror.borrow(), "", "cancel fires no EVENT_INPUT");
    assert_eq!(
        composed_text(&mut runtime, "ab"),
        "ab",
        "display back to buffer text"
    );
}

#[test]
fn ime_with_no_active_session_is_a_noop() {
    // (c) With no focused editable there is no session, so every composition
    // event is a no-op returning false — and no session springs into being.
    let mut runtime = TestFixture::runtime();
    assert!(
        !runtime.handle_ime(CompositionEvent::Start),
        "start no-op with no session"
    );
    assert!(
        !runtime.handle_ime(CompositionEvent::Update(Preedit::new("x"))),
        "update no-op with no session"
    );
    assert!(
        !runtime.handle_ime(CompositionEvent::Commit(Box::from("x"))),
        "commit no-op with no session"
    );
    assert!(runtime.editing_text().is_none(), "still no editing session");
    assert!(runtime.composition_preedit().is_none(), "no preedit");
}

#[test]
fn ime_preedit_renders_an_underline_over_its_x_range() {
    // (d) An active preedit paints a thin underline (height ~1) beneath the
    // composing glyphs, spanning the preedit's x-range in the spliced display
    // text: "ab" (caret at 2) + preedit "ni" underlines x∈[2·adv, 4·adv) past
    // the text origin. The caret (width ~1) is a separate fill.
    let (mut runtime, _mirror) = EditablePanel::focused("ab");
    runtime.handle_ime(CompositionEvent::Start);
    runtime.handle_ime(CompositionEvent::Update(Preedit::new("ni")));

    let sheet = EditablePanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let result = runtime.render_now(&EditablePanel::view("ab"), sheets);
    let list = result.snapshot().display_list();

    let advance = 16.0_f32 * obscur_text::shaping::MockTextShaper::ADVANCE_RATIO;
    let text_origin_x = list
        .commands()
        .iter()
        .find_map(|c| match c {
            PaintCommand::DrawText { origin, .. } => Some(origin.x),
            _ => None,
        })
        .expect("a text run");
    // The underline: a thin (height ~1) horizontal fill spanning two advances.
    let underline = list
        .commands()
        .iter()
        .find_map(|c| match c {
            PaintCommand::DrawPath { path, .. } => path
                .bounds()
                .filter(|r| (r.size.height - 1.0).abs() < 1e-6 && r.size.width > 1.0),
            _ => None,
        })
        .expect("a thin preedit underline fill");
    assert!(
        (underline.origin.x - (text_origin_x + advance * 2.0)).abs() < 1e-3,
        "underline starts at the preedit's left edge (2·advance)"
    );
    assert!(
        (underline.size.width - advance * 2.0).abs() < 1e-3,
        "underline spans the two preedit glyphs"
    );
    assert!(list.is_balanced(), "the display list stays balanced");
}

#[test]
fn no_ime_keystroke_path_is_byte_identical() {
    // (e) The ordinary (no-composition) keystroke path is unaffected: a runtime
    // that types 'x' produces the exact same display list as one that runs a
    // full Start→Cancel composition cycle (leaving no preedit) before typing.
    let sheet = EditablePanel::sheet();
    let sheets = std::slice::from_ref(&sheet);

    let (mut plain, _m1) = EditablePanel::focused("hi");
    plain.handle_key(char_press('x'));
    let plain_cmds: Vec<_> = plain
        .render_now(&EditablePanel::view("hi"), sheets)
        .snapshot()
        .display_list()
        .commands()
        .to_vec();

    let (mut cycled, _m2) = EditablePanel::focused("hi");
    cycled.handle_ime(CompositionEvent::Start);
    cycled.handle_ime(CompositionEvent::Cancel);
    cycled.handle_key(char_press('x'));
    let cycled_cmds: Vec<_> = cycled
        .render_now(&EditablePanel::view("hi"), sheets)
        .snapshot()
        .display_list()
        .commands()
        .to_vec();

    assert_eq!(
        plain_cmds, cycled_cmds,
        "a cancelled composition leaves paint byte-identical"
    );
}
