#![doc = include_str!("spec_ch9_view_contract.doc/_module.md")]
#![forbid(unsafe_code)]

#[path = "spec_ch9_support/mod.rs"]
mod support;

use obscur_kit::geometry::{Point, Size};
use obscur_kit::identity::{InternedSymbol, RuntimeId};
use obscur_kit::interaction::command::{KeyEvent, LogicalKey, NamedKey, PhysicalKey};
use obscur_kit::interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::widgets::input::Input;
use support::{Shared, compile};

const VW: f32 = 1000.0;

#[doc = include_str!("spec_ch9_view_contract.doc/typing_into_a_bound_input_does_not_mutate_its_source.md")]
#[test]
fn typing_into_a_bound_input_does_not_mutate_its_source() {
    let source = Shared::new(String::from("abc"));
    let observed = Shared::new(Vec::<String>::new());

    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VW, VW));
    let sheets = [compile(".field{width:200px;height:24px}")];

    let change_command = InternedSymbol::from_raw(
        obscur_kit::ui_ir::name_binding::attr_symbol("field-change").raw(),
    );

    let sink = observed.clone();
    rt.on_change(change_command, move |text| {
        let mut all = sink.get();
        all.push(text.to_string());
        sink.set(all);
    });

    let read = source.clone();
    let build = move || Input::with_value(change_command, read.get()).view();

    let v = build();
    let _ = rt.render_now(&v, &sheets);
    let node = rt
        .tree()
        .root()
        .and_then(|root| rt.tree().get(root).map(|n| n.child_nodes().to_vec()))
        .and_then(|kids| kids.first().copied())
        .expect("the input node");

    // Focus the field through the real input path, exactly as a user would.
    let (x, y, w, h) = support::geom(&rt, node);
    let centre = Point::new(x + w / 2.0, y + h / 2.0);
    let _ = rt.handle_pointer(PointerEvent::new(
        PointerId::MOUSE,
        PointerDevice::Mouse,
        PointerEventKind::Down,
        centre,
    ));
    let _ = rt.handle_pointer(PointerEvent::new(
        PointerId::MOUSE,
        PointerDevice::Mouse,
        PointerEventKind::Up,
        centre,
    ));

    let _ = rt.handle_key(KeyEvent::press(
        PhysicalKey::KeyA,
        Some(LogicalKey::character('d')),
    ));
    // A single-line field commits its `change` on Enter (WHATWG HTML §4.10.5.5).
    let _ = rt.handle_key(KeyEvent::press(
        PhysicalKey::Enter,
        Some(LogicalKey::named(NamedKey::Enter)),
    ));

    assert_eq!(
        source.get(),
        "abc",
        "the bound source value is unchanged: values flow down, not back up"
    );
    assert_eq!(
        observed.get(),
        vec![String::from("abcd")],
        "a typed change event carrying the new text was delivered upward"
    );
}
