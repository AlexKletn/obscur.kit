#![doc = include_str!("spec_ch5_editing_ime.doc/_module.md")]
#![forbid(unsafe_code)]

#[path = "spec_ch5_support/mod.rs"]
mod support;

use std::cell::RefCell;
use std::rc::Rc;

use obscur_kit::geometry::{Point, Size};
use obscur_kit::identity::{InternedSymbol, NodeId, RuntimeId};
use obscur_kit::interaction::command::{KeyEvent, LogicalKey, NamedKey, PhysicalKey};
use obscur_kit::interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::text::ime::{CompositionEvent, Preedit};
use obscur_kit::view::view::View;
use obscur_kit::widgets::input::Input;

use support::compile;

#[doc = include_str!("spec_ch5_editing_ime.doc/CMD.md")]
const CMD: InternedSymbol = InternedSymbol::from_raw(0x0E01);

#[doc = include_str!("spec_ch5_editing_ime.doc/VW.md")]
const VW: f32 = 400.0;

#[doc = include_str!("spec_ch5_editing_ime.doc/Events.md")]
#[derive(Default)]
struct Events {
    input: Vec<String>,
    change: Vec<String>,
}

#[doc = include_str!("spec_ch5_editing_ime.doc/Field.md")]
struct Field {
    runtime: UiRuntime,
    events: Rc<RefCell<Events>>,
    sheets: Vec<Stylesheet>,
    view: View,
    node: NodeId,
}

impl Field {
    #[doc = include_str!("spec_ch5_editing_ime.doc/single_line.md")]
    fn single_line(value: &str) -> Self {
        Self::build(Input::with_value(CMD, value).view(), 200.0)
    }

    #[doc = include_str!("spec_ch5_editing_ime.doc/multiline.md")]
    fn multiline(value: &str, width: f32) -> Self {
        Self::build(Input::with_value(CMD, value).multiline(true).view(), width)
    }

    fn build(view: View, width: f32) -> Self {
        let events: Rc<RefCell<Events>> = Rc::default();
        let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VW, VW));

        let sink = Rc::clone(&events);
        runtime.on_text_input(CMD, move |text| {
            sink.borrow_mut().input.push(text.to_owned());
        });
        let sink = Rc::clone(&events);
        runtime.on_change(CMD, move |text| {
            sink.borrow_mut().change.push(text.to_owned());
        });

        let sheets = vec![compile(&format!(
            "input, textarea {{ width: {width}px; height: 80px; font-size: 16px }}"
        ))];
        let _ = runtime.render_now(&view, &sheets);
        let node = support::at(&runtime, &[0]);
        let mut field = Self {
            runtime,
            events,
            sheets,
            view,
            node,
        };
        field.focus();
        field
    }

    #[doc = include_str!("spec_ch5_editing_ime.doc/focus.md")]
    fn focus(&mut self) {
        let target = support::at(&self.runtime, &[0]);
        self.node = target;
        let b = self
            .runtime
            .layout_boxes()
            .get(&target)
            .copied()
            .expect("the field is arranged");
        let centre = Point::new(
            b.origin().x + b.size().width / 2.0,
            b.origin().y + b.size().height / 2.0,
        );
        self.runtime.handle_pointer(PointerEvent::new(
            PointerId::MOUSE,
            PointerDevice::Mouse,
            PointerEventKind::Down,
            centre,
        ));
        self.runtime.handle_pointer(PointerEvent::new(
            PointerId::MOUSE,
            PointerDevice::Mouse,
            PointerEventKind::Up,
            centre,
        ));
    }

    #[doc = include_str!("spec_ch5_editing_ime.doc/type_char.md")]
    #[allow(dead_code)]
    fn type_char(&mut self, ch: char) {
        let logical = Some(LogicalKey::character(ch));
        self.runtime
            .handle_key(KeyEvent::press(PhysicalKey::KeyA, logical));
        self.runtime
            .handle_key(KeyEvent::release(PhysicalKey::KeyA, logical));
    }

    #[doc = include_str!("spec_ch5_editing_ime.doc/press.md")]
    fn press(&mut self, key: NamedKey) {
        let logical = Some(LogicalKey::named(key));
        self.runtime
            .handle_key(KeyEvent::press(PhysicalKey::KeyA, logical));
        self.runtime
            .handle_key(KeyEvent::release(PhysicalKey::KeyA, logical));
    }

    #[doc = include_str!("spec_ch5_editing_ime.doc/value.md")]
    fn value(&self) -> String {
        self.runtime.editing_text().unwrap_or_default().to_owned()
    }

    #[doc = include_str!("spec_ch5_editing_ime.doc/caret.md")]
    fn caret(&self) -> usize {
        self.runtime.editing_selection().map_or(0, |s| s.focus())
    }

    #[doc = include_str!("spec_ch5_editing_ime.doc/inputs.md")]
    fn inputs(&self) -> usize {
        self.events.borrow().input.len()
    }

    #[doc = include_str!("spec_ch5_editing_ime.doc/changes.md")]
    fn changes(&self) -> usize {
        self.events.borrow().change.len()
    }

    #[doc = include_str!("spec_ch5_editing_ime.doc/frame.md")]
    fn frame(&mut self) {
        let _ = self.runtime.render_now(&self.view, &self.sheets);
    }

    #[doc = include_str!("spec_ch5_editing_ime.doc/painted_runs.md")]
    fn painted_runs(&self) -> Vec<String> {
        self.runtime
            .snapshot()
            .display_list()
            .commands()
            .iter()
            .filter_map(|c| match c {
                PaintCommand::DrawText { run, .. } => Some(run.text().to_owned()),
                _ => None,
            })
            .collect()
    }
}

#[doc = include_str!("spec_ch5_editing_ime.doc/ed_01_backspace_deletes_one_grapheme_cluster.md")]
#[test]
fn ed_01_backspace_deletes_one_grapheme_cluster() {
    let family = "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}";
    let mut field = Field::single_line(family);
    let before = field.inputs();

    field.press(NamedKey::Backspace);

    assert_eq!(
        field.value(),
        "",
        "one Backspace removes the whole ZWJ cluster; a code-point delete leaves \
         a trailing joiner"
    );
    assert_eq!(field.caret(), 0, "the caret lands at offset 0");
    assert_eq!(
        field.inputs() - before,
        1,
        "exactly one input event for the one deletion"
    );
}

#[doc = include_str!("spec_ch5_editing_ime.doc/ed_02_home_and_end_are_visual_line_not_paragraph.md")]
#[test]
fn ed_02_home_and_end_are_visual_line_not_paragraph() {
    // One paragraph with no newline, narrow enough to wrap into at least two line
    // boxes (the exact count depends on the shaper's metrics).
    let text = "alpha beta gamma delta epsilon zeta eta theta";
    let mut field = Field::multiline(text, 120.0);
    field.frame();

    let runs = field.painted_runs();
    assert!(
        runs.len() >= 2,
        "the setup requires the text to wrap into at least 2 line boxes, got {runs:?}"
    );
    // `k` — the logical offset where line 1 begins — comes from the engine's own
    // line boxes, not from a pixel width, so the case stays font-independent.
    let k = runs[0].chars().count();
    let n = text.chars().count();
    assert!(
        k < n,
        "line 0 must not be the whole paragraph (k={k}, n={n})"
    );

    // From offset k: Home stays at k (the start of line 1), End goes to the end of
    // line 1 — which is the paragraph end ONLY when line 1 is the last line. The
    // original assertion demanded `== n`, baking in a two-line wrap the setup never
    // guarantees (it requires >= 2); under a shaper that wraps to more lines, End is
    // still visual-line and stops at the end of *this* line. CORRECTED 2026-07-20 by
    // the owner: assert the end of line 1 from the engine's own runs, not `n`.
    let end_of_line_1 = k + runs[1].chars().count();
    seat_caret(&mut field, k);
    field.press(NamedKey::Home);
    assert_eq!(
        field.caret(),
        k,
        "Home from the start of line 1 stays on line 1, it does not jump to 0"
    );
    field.press(NamedKey::End);
    let end_from_line_1 = field.caret();
    assert!(
        end_from_line_1 <= end_of_line_1 && end_from_line_1 + 1 >= end_of_line_1,
        "End from the start of line 1 stops at the end of THAT visual line \
         (~{end_of_line_1}, modulo a trailing collapsed space), not at the \
         paragraph end {n}"
    );

    // From offset 0: End reaches the end of line 0 (k, modulo a trailing
    // collapsed space), NOT the end of the paragraph.
    seat_caret(&mut field, 0);
    field.press(NamedKey::End);
    let end_of_line_0 = field.caret();
    assert!(
        end_of_line_0 <= k && end_of_line_0 + 1 >= k,
        "End from offset 0 stops at the end of the FIRST VISUAL LINE \
         (k={k}, got {end_of_line_0}), not at the paragraph end {n}"
    );
    assert_ne!(
        end_of_line_0, n,
        "End must not scan to the buffer end on wrapped text"
    );
}

#[doc = include_str!("spec_ch5_editing_ime.doc/ime_01_a_preedit_is_never_in_the_value.md")]
#[test]
fn ime_01_a_preedit_is_never_in_the_value() {
    let mut field = Field::single_line("");

    assert!(field.runtime.handle_ime(CompositionEvent::Start));
    assert!(field.runtime.handle_ime(CompositionEvent::Update(
        Preedit::new("にほn").with_caret(3)
    )));
    field.frame();

    assert_eq!(field.value(), "", "the preedit is not in the current value");
    assert_eq!(field.inputs(), 0, "composing fires no input event");
    assert_eq!(field.changes(), 0, "composing fires no change event");
    assert!(
        field.painted_runs().iter().any(|r| r.contains("にほn")),
        "the preedit IS rendered, with preedit styling; painted runs: {:?}",
        field.painted_runs()
    );

    assert!(
        field
            .runtime
            .handle_ime(CompositionEvent::Commit("日本".into()))
    );
    assert_eq!(field.value(), "日本", "the commit lands in the value");
    assert_eq!(field.inputs(), 1, "exactly one input event, at the commit");
    assert_eq!(
        field.changes(),
        0,
        "change awaits blur/commit-of-the-edit-session, not the IME commit"
    );
}

#[doc = include_str!("spec_ch5_editing_ime.doc/ime_02_one_composition_is_one_undo_unit.md")]
#[test]
fn ime_02_one_composition_is_one_undo_unit() {
    let mut field = Field::single_line("ab");
    seat_caret(&mut field, 2);

    assert!(field.runtime.handle_ime(CompositionEvent::Start));
    for preedit in ["n", "に", "日本"] {
        assert!(
            field
                .runtime
                .handle_ime(CompositionEvent::Update(Preedit::new(preedit)))
        );
    }
    assert!(
        field
            .runtime
            .handle_ime(CompositionEvent::Commit("日本".into()))
    );
    assert_eq!(field.value(), "ab日本");

    undo(&mut field);
    assert_eq!(
        field.value(),
        "ab",
        "one Undo removes the WHOLE composition — not 'ab日', 'abに' or 'abn'"
    );
    assert_eq!(field.caret(), 2, "the caret returns to the anchor");

    undo(&mut field);
    assert_eq!(
        field.value(),
        "ab",
        "there is nothing earlier in the local undo stack"
    );
}

#[doc = include_str!("spec_ch5_editing_ime.doc/ime_03_an_external_edit_cancels_the_composition.md")]
#[test]
fn ime_03_an_external_edit_cancels_the_composition() {
    let mut field = Field::single_line("hello world");
    seat_caret(&mut field, 11);
    assert!(field.runtime.handle_ime(CompositionEvent::Start));
    assert!(
        field
            .runtime
            .handle_ime(CompositionEvent::Update(Preedit::new("wo")))
    );

    // The application programmatically shortens the value under the composition.
    field.runtime.set_editing_text("hi");

    assert_eq!(field.value(), "hi", "the external edit lands");
    assert!(
        field.runtime.composition_preedit().is_none(),
        "the composition is cancelled, not left pointing at a stale range"
    );
    assert!(
        field.caret() <= 2,
        "the caret is clamped into the new value, got {}",
        field.caret()
    );

    // A subsequent commit must not write into the old [6,11) range.
    let _ = field
        .runtime
        .handle_ime(CompositionEvent::Commit("X".into()));
    assert!(
        field.value() == "hi" || field.value().starts_with("hi") || field.value().ends_with("hi"),
        "a commit after cancellation starts fresh or is rejected; it must never \
         splice into the stale range. Value: {:?}",
        field.value()
    );
    assert!(
        field.value().chars().count() <= 3,
        "no out-of-range write occurred, got {:?}",
        field.value()
    );
}

// DELETED 2026-07-19: `ime_05_a_key_plus_a_text_delivery_inserts_once`.
//
// It encoded §5.12's "- duplicate keydown/text delivery is avoided." (with §2.6
// "IME and text input are separate from keydown.") by typing 'a' through
// `handle_key` AND then issuing `CompositionEvent::Start` +
// `Commit("a")`, on the stated premise that "a real platform adapter delivers
// both a keydown carrying the logical key and a separate text delivery carrying
// the string, for one physical press", and that the IME commit is "this
// toolkit's only text-delivery channel distinct from keydown". It expected "a"
// and measured "aa".
//
// The substitution does not survive scrutiny, and the translator flagged it at
// the time. §5.12's rule is about ONE physical press arriving on two wires from
// the platform, where the runtime must recognise the pair and insert once. This
// runtime has exactly two input entry points — `handle_key`, whose `LogicalKey`
// already carries the character, and `handle_ime`, a full typed composition
// lifecycle. A keydown insert followed by a separate composition start/commit is
// not one press delivered twice; it is two distinct inputs, and inserting twice
// is the only correct response to them. There is no third "text delivery"
// channel for the rule to be about, so the case could not construct its own
// premise and no weakening of it would test anything.
//
// Candidate spec amendment, and the finding this leaves behind: §5.12's rule
// presupposes an input surface this toolkit does not have. Either state that a
// platform adapter must fold text delivery into the `KeyEvent` it already sends
// (making the rule an adapter obligation, testable at the adapter boundary), or
// add the separate text-delivery event the rule guards, and say how a runtime
// correlates it with the keydown for the same press — a timestamp, a sequence
// number, or an explicit "text follows" flag. Until one of those exists the rule
// is unimplementable and untestable, not merely untested.

// ---------------------------------------------------------------------------
// Driving helpers. These DRIVE; they carry no expectations.
// ---------------------------------------------------------------------------

#[doc = include_str!("spec_ch5_editing_ime.doc/seat_caret.md")]
fn seat_caret(field: &mut Field, offset: usize) {
    let text = field.value();
    let node = field.node;
    field.runtime.begin_text_editing(node, text.clone());
    let steps = text.chars().count().saturating_sub(offset);
    for _ in 0..steps {
        field.press(NamedKey::ArrowLeft);
    }
}

#[doc = include_str!("spec_ch5_editing_ime.doc/undo.md")]
fn undo(field: &mut Field) {
    field
        .runtime
        .handle_key(KeyEvent::press(PhysicalKey::ControlLeft, None));
    let z = Some(LogicalKey::character('z'));
    field
        .runtime
        .handle_key(KeyEvent::press(PhysicalKey::KeyZ, z));
    field
        .runtime
        .handle_key(KeyEvent::release(PhysicalKey::KeyZ, z));
    field
        .runtime
        .handle_key(KeyEvent::release(PhysicalKey::ControlLeft, None));
}
