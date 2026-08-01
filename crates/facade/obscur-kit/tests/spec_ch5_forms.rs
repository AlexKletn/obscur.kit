#![doc = include_str!("spec_ch5_forms.doc/_module.md")]
#![forbid(unsafe_code)]

#[path = "spec_ch5_support/mod.rs"]
mod support;

use std::cell::RefCell;
use std::rc::Rc;

use obscur_kit::geometry::{Point, Size};
use obscur_kit::identity::{InternedSymbol, NodeId, RuntimeId};
use obscur_kit::interaction::command::{KeyEvent, LogicalKey, PhysicalKey};
use obscur_kit::interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{View, ViewFragment};
use obscur_kit::widgets::input::Input;

use support::{at, compile};

#[doc = include_str!("spec_ch5_forms.doc/CMD_A.md")]
const CMD_A: InternedSymbol = InternedSymbol::from_raw(0x0F01);
#[doc = include_str!("spec_ch5_forms.doc/CMD_B.md")]
const CMD_B: InternedSymbol = InternedSymbol::from_raw(0x0F02);

#[doc = include_str!("spec_ch5_forms.doc/VW.md")]
const VW: f32 = 400.0;

#[doc = include_str!("spec_ch5_forms.doc/sheets.md")]
fn sheets() -> Vec<Stylesheet> {
    vec![compile(
        "input { display: block; width: 160px; height: 40px; font-size: 16px }",
    )]
}

#[doc = include_str!("spec_ch5_forms.doc/click.md")]
fn click(runtime: &mut UiRuntime, node: NodeId) {
    let b = runtime
        .layout_boxes()
        .get(&node)
        .copied()
        .expect("the node is arranged");
    let centre = Point::new(
        b.origin().x + b.size().width / 2.0,
        b.origin().y + b.size().height / 2.0,
    );
    runtime.handle_pointer(PointerEvent::new(
        PointerId::MOUSE,
        PointerDevice::Mouse,
        PointerEventKind::Down,
        centre,
    ));
    runtime.handle_pointer(PointerEvent::new(
        PointerId::MOUSE,
        PointerDevice::Mouse,
        PointerEventKind::Up,
        centre,
    ));
}

#[doc = include_str!("spec_ch5_forms.doc/type_char.md")]
fn type_char(runtime: &mut UiRuntime, ch: char) {
    let logical = Some(LogicalKey::character(ch));
    runtime.handle_key(KeyEvent::press(PhysicalKey::KeyA, logical));
    runtime.handle_key(KeyEvent::release(PhysicalKey::KeyA, logical));
}

#[doc = include_str!("spec_ch5_forms.doc/painted.md")]
fn painted(runtime: &UiRuntime) -> Vec<String> {
    runtime
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

#[doc = include_str!("spec_ch5_forms.doc/fm_01_a_controlled_value_does_not_drift.md")]
#[test]
fn fm_01_a_controlled_value_does_not_drift() {
    let proposals: Rc<RefCell<Vec<String>>> = Rc::default();
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VW, VW));
    let sink = Rc::clone(&proposals);
    runtime.on_text_input(CMD_A, move |text| {
        sink.borrow_mut().push(text.to_owned());
    });

    let sheets = sheets();
    // The application's value prop is the CONSTANT "a" — it never updates in
    // response to events, which is what "controlled" means.
    let controlled = || Input::with_value(CMD_A, "a").view();

    let _ = runtime.render_now(&controlled(), &sheets);
    let field = at(&runtime, &[0]);
    click(&mut runtime, field);

    type_char(&mut runtime, 'b');
    assert_eq!(
        proposals.borrow().len(),
        1,
        "one keystroke, one input event"
    );
    assert_eq!(
        proposals.borrow()[0],
        "ab",
        "the input event carries the PROPOSED value"
    );

    // The frame commits from the application's (unchanged) value.
    let _ = runtime.render_now(&controlled(), &sheets);
    assert_eq!(
        runtime.editing_text(),
        Some("a"),
        "the field's current value is the application's, not its own buffer"
    );
    assert!(
        painted(&runtime).iter().any(|r| r == "a") && !painted(&runtime).iter().any(|r| r == "ab"),
        "the rendered text is 'a'; painted runs: {:?}",
        painted(&runtime)
    );
    assert!(
        runtime.editing_selection().is_none_or(|s| s.focus() <= 1),
        "the caret is clamped into the one-character value"
    );

    type_char(&mut runtime, 'b');
    assert_eq!(proposals.borrow().len(), 2, "typing again proposes again");
    assert_eq!(proposals.borrow()[1], "ab", "and proposes 'ab' again");
    let _ = runtime.render_now(&controlled(), &sheets);
    assert_eq!(
        runtime.editing_text(),
        Some("a"),
        "the field still shows 'a'"
    );
}

#[doc = include_str!("spec_ch5_forms.doc/fm_02_input_fires_per_keystroke_and_change_fires_once_at_commit.md")]
#[test]
fn fm_02_input_fires_per_keystroke_and_change_fires_once_at_commit() {
    let inputs: Rc<RefCell<Vec<String>>> = Rc::default();
    let changes: Rc<RefCell<Vec<String>>> = Rc::default();
    let b_inputs: Rc<RefCell<Vec<String>>> = Rc::default();
    let b_changes: Rc<RefCell<Vec<String>>> = Rc::default();

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VW, VW));
    for (cmd, i, c) in [
        (CMD_A, Rc::clone(&inputs), Rc::clone(&changes)),
        (CMD_B, Rc::clone(&b_inputs), Rc::clone(&b_changes)),
    ] {
        runtime.on_text_input(cmd, move |t| i.borrow_mut().push(t.to_owned()));
        runtime.on_change(cmd, move |t| c.borrow_mut().push(t.to_owned()));
    }

    let sheets = sheets();
    let two_fields = View::from_fragment(ViewFragment::from_nodes(vec![
        Input::new(CMD_A).view().root().nodes()[0].clone(),
        Input::with_value(CMD_B, "x").view().root().nodes()[0].clone(),
    ]));
    let _ = runtime.render_now(&two_fields, &sheets);

    let first = at(&runtime, &[0]);
    let second = at(&runtime, &[1]);

    // Field 1: type "ab", then blur by focusing field 2.
    click(&mut runtime, first);
    type_char(&mut runtime, 'a');
    type_char(&mut runtime, 'b');
    assert_eq!(
        *inputs.borrow(),
        vec!["a".to_owned(), "ab".to_owned()],
        "two input events, carrying 'a' then 'ab'"
    );
    assert_eq!(
        changes.borrow().len(),
        0,
        "change has not fired before the commit"
    );

    click(&mut runtime, second);
    assert_eq!(
        *changes.borrow(),
        vec!["ab".to_owned()],
        "exactly one change event, at blur, carrying 'ab'"
    );

    // Field 2: focused, nothing typed, then blurred.
    click(&mut runtime, first);
    assert_eq!(
        b_inputs.borrow().len(),
        0,
        "an untouched field fires no input event"
    );
    assert_eq!(
        b_changes.borrow().len(),
        0,
        "an untouched field fires no change event on blur — a spurious commit \
         would re-trigger every save on every focus traversal"
    );
}

#[doc = include_str!("spec_ch5_forms.doc/fm_03_readonly_is_not_disabled.md")]
#[test]
fn fm_03_readonly_is_not_disabled() {
    let inputs: Rc<RefCell<Vec<String>>> = Rc::default();
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VW, VW));
    let sink = Rc::clone(&inputs);
    runtime.on_text_input(CMD_A, move |t| sink.borrow_mut().push(t.to_owned()));
    let sink = Rc::clone(&inputs);
    runtime.on_text_input(CMD_B, move |t| sink.borrow_mut().push(t.to_owned()));

    let sheets = sheets();
    let view = View::from_fragment(ViewFragment::from_nodes(vec![
        Input::with_value(CMD_A, "abc")
            .readonly(true)
            .view()
            .root()
            .nodes()[0]
            .clone(),
        Input::with_value(CMD_B, "abc")
            .disabled(true)
            .view()
            .root()
            .nodes()[0]
            .clone(),
    ]));
    let _ = runtime.render_now(&view, &sheets);
    let readonly = at(&runtime, &[0]);
    let disabled = at(&runtime, &[1]);

    // --- readonly: focusable, selectable, not editable ---------------------
    click(&mut runtime, readonly);
    assert_eq!(
        runtime.focused(),
        Some(readonly),
        "focus lands on a readonly field"
    );
    select_all(&mut runtime);
    let selection = runtime
        .editing_selection()
        .expect("a readonly field has a selection");
    assert_eq!(
        (selection.start(), selection.end()),
        (0, 3),
        "select-all establishes [0,3) on a readonly field — this is what makes \
         it copyable"
    );
    type_char(&mut runtime, 'x');
    assert_eq!(
        inputs.borrow().len(),
        0,
        "typing into a readonly field fires no input event"
    );
    assert_eq!(
        runtime.editing_text(),
        Some("abc"),
        "the readonly value is unchanged"
    );

    // --- disabled: inert ---------------------------------------------------
    let focus_before = runtime.focused();
    click(&mut runtime, disabled);
    assert_eq!(
        runtime.focused(),
        focus_before,
        "focus does NOT move to a disabled field"
    );
    select_all(&mut runtime);
    assert_ne!(
        runtime.editing_selection().map(|s| (s.start(), s.end())),
        Some((0, 3)).filter(|_| runtime.focused() == Some(disabled)),
        "no selection is established on a disabled field"
    );
    type_char(&mut runtime, 'x');
    assert_eq!(
        inputs.borrow().len(),
        0,
        "a disabled field dispatches no keyboard events"
    );
}

#[doc = include_str!("spec_ch5_forms.doc/select_all.md")]
fn select_all(runtime: &mut UiRuntime) {
    runtime.handle_key(KeyEvent::press(PhysicalKey::ControlLeft, None));
    let a = Some(LogicalKey::character('a'));
    runtime.handle_key(KeyEvent::press(PhysicalKey::KeyA, a));
    runtime.handle_key(KeyEvent::release(PhysicalKey::KeyA, a));
    runtime.handle_key(KeyEvent::release(PhysicalKey::ControlLeft, None));
}
