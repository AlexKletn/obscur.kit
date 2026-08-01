#![doc = include_str!("spec_ch6_data_transfer.doc/_module.md")]
#![forbid(unsafe_code)]

#[path = "spec_ch6_support/mod.rs"]
mod support;

use obscur_kit::data_transfer::clipboard::{
    ClipboardProvider, ClipboardRequest, ClipboardResult, MemoryClipboard,
};
use obscur_kit::data_transfer::drag::{DragEffect, DragPhase, DragSession, DragSource, DropTarget};
use obscur_kit::data_transfer::offer::{DataOffer, DataRepresentation, MimeType};
use obscur_kit::data_transfer::transfer::TransferPayload;
use obscur_kit::identity::NodeId;
use obscur_kit::interaction::pointer::PointerEventKind;
use obscur_kit::runtime::CommandTarget;
use obscur_kit::runtime::orchestration::DragOutcome;

use support::{at, drive, el, pointer, view};

// -------------------------------------------------------------------------
// P6-31
// -------------------------------------------------------------------------

#[doc = include_str!("spec_ch6_data_transfer.doc/a_clipboard_offer_keeps_every_representation_and_refuses_absent_ones.md")]
#[test]
fn a_clipboard_offer_keeps_every_representation_and_refuses_absent_ones() {
    let offer = DataOffer::new()
        .with(DataRepresentation::eager(
            MimeType::text_plain(),
            b"hello".to_vec(),
        ))
        .with(DataRepresentation::eager(
            MimeType::text_html(),
            b"<b>hello</b>".to_vec(),
        ))
        .with(DataRepresentation::eager(
            MimeType::image_png(),
            vec![0x89u8, 0x50, 0x4e, 0x47],
        ));

    let mut clipboard = MemoryClipboard::new();
    assert_eq!(
        clipboard.execute(ClipboardRequest::copy(TransferPayload::new(offer))),
        ClipboardResult::Written,
        "§6.20: the offer is written"
    );

    let ClipboardResult::Read(read_back) = clipboard.execute(ClipboardRequest::read_any()) else {
        panic!("§6.20: reading back the clipboard yields the offer");
    };

    assert_eq!(
        read_back.available_mimes(),
        vec![
            MimeType::text_plain(),
            MimeType::text_html(),
            MimeType::image_png()
        ],
        "§6.20: exactly the three offered MIME types, in the order registered"
    );
    assert_eq!(
        read_back
            .representation_for(&MimeType::text_html())
            .and_then(DataRepresentation::inline_bytes),
        Some(&b"<b>hello</b>"[..]),
        "§6.20: text/html reads back as itself"
    );
    assert_eq!(
        read_back
            .representation_for(&MimeType::text_plain())
            .and_then(DataRepresentation::inline_bytes),
        Some(&b"hello"[..]),
        "§6.20: text/plain reads back as itself"
    );

    let rtf = MimeType::new("text/rtf");
    assert!(
        !read_back.has_mime(&rtf),
        "§6.20: a flavour that was never offered is not silently present"
    );
    assert_eq!(
        clipboard.execute(ClipboardRequest::read(vec![rtf])),
        ClipboardResult::Empty,
        "§6.20: requesting an absent type fails loudly — it does not degrade to text/plain"
    );
}

// -------------------------------------------------------------------------
// P6-34
// -------------------------------------------------------------------------

#[doc = include_str!("spec_ch6_data_transfer.doc/a_copy_command_routes_focused_node_scope_ancestors_window_then_app.md")]
#[test]
fn a_copy_command_routes_focused_node_scope_ancestors_window_then_app() {
    let css = ".panel{width:200px;height:200px} .scope{width:200px;height:100px} \
               .item{width:200px;height:50px}";
    let v = view(vec![el(
        1,
        "panel",
        vec![el(2, "scope", vec![el(3, "item", vec![])])],
    )]);
    let rt = drive(200.0, 200.0, css, &v);

    let panel = at(&rt, &[0]);
    let scope_node = at(&rt, &[0, 0]);
    let item = at(&rt, &[0, 0, 0]);

    let mut rt = rt;
    let scope_id = obscur_kit::interaction::focus::FocusScopeId::from_raw(1);
    rt.open_focus_scope(
        obscur_kit::interaction::focus::FocusScope::new(scope_id).with_member(item),
    );

    let route = rt.command_route(Some(item));
    let index = |t: CommandTarget| {
        route
            .iter()
            .position(|x| *x == t)
            .unwrap_or_else(|| panic!("§2.6: {t:?} is a link of the route {route:?}"))
    };

    assert_eq!(
        route.first(),
        Some(&CommandTarget::Node(item)),
        "§2.6: link 1 — the focused node comes first, not the window"
    );
    assert_eq!(
        route.get(1),
        Some(&CommandTarget::Scope(scope_id)),
        "§2.6: link 2 — the focus scope follows the focused node"
    );
    assert!(
        index(CommandTarget::Node(scope_node)) < index(CommandTarget::Node(panel)),
        "§2.6: link 3 — ancestors follow the scope, nearest first"
    );
    assert_eq!(
        route[route.len() - 2..],
        [CommandTarget::Window, CommandTarget::App],
        "§2.6: links 4 and 5 — the window, then the application fallback, last"
    );
    assert!(
        index(CommandTarget::Node(panel)) < index(CommandTarget::Window),
        "§2.6: every ancestor is offered the command before the window is"
    );
}

// -------------------------------------------------------------------------
// P6-35
// -------------------------------------------------------------------------

#[doc = include_str!("spec_ch6_data_transfer.doc/a_drag_across_two_targets_enters_each_once_and_drops_on_the_second_only.md")]
#[test]
fn a_drag_across_two_targets_enters_each_once_and_drops_on_the_second_only() {
    let css = ".page{width:200px;height:100px;display:flex} \
               .a{width:100px;height:100px} .b{width:100px;height:100px}";
    let v = view(vec![el(
        1,
        "page",
        vec![el(2, "a", vec![]), el(3, "b", vec![])],
    )]);
    let mut rt = drive(200.0, 100.0, css, &v);

    let a = at(&rt, &[0, 0]);
    let b = at(&rt, &[0, 1]);
    rt.register_drop_target(DropTarget::new(a));
    rt.register_drop_target(DropTarget::new(b));

    let source = DragSource::new(a, TransferPayload::new(DataOffer::text("payload")));
    assert!(rt.begin_drag(source), "§6.21: the drag starts");

    let mut over_trail = Vec::new();
    for (x, y) in [(50.0, 50.0), (60.0, 50.0), (150.0, 50.0), (160.0, 50.0)] {
        rt.handle_pointer(pointer(PointerEventKind::Move, x, y));
        over_trail.push(rt.drag_session().and_then(DragSession::over));
    }
    assert_eq!(
        over_trail,
        vec![Some(a), Some(a), Some(b), Some(b)],
        "§6.21: A is entered once and left exactly when B is entered — one active target"
    );

    rt.handle_pointer(pointer(PointerEventKind::Up, 160.0, 50.0));
    let outcome = rt.take_drag_outcome().expect("§6.21: the drag finished");
    let DragOutcome::Dropped { target, .. } = outcome else {
        panic!("§6.21: the release over B is a drop, got {outcome:?}");
    };
    assert_eq!(
        target, b,
        "§6.21: only B receives the drop; A receives none"
    );
}

// -------------------------------------------------------------------------
// P6-37
// -------------------------------------------------------------------------

#[doc = include_str!("spec_ch6_data_transfer.doc/the_negotiated_drop_effect_respects_both_source_and_target.md")]
#[test]
fn the_negotiated_drop_effect_respects_both_source_and_target() {
    let origin = NodeId::from_raw(1);
    let payload = || TransferPayload::new(DataOffer::text("payload"));
    let source = || {
        DragSource::new(origin, payload()).with_effects(vec![DragEffect::Copy, DragEffect::Move])
    };

    // (a) the target accepts only `move`.
    let move_only = DropTarget::new(NodeId::from_raw(2))
        .accepting(MimeType::text_plain())
        .with_effects(vec![DragEffect::Move]);
    let mut session = DragSession::start(source());
    session.enter(&move_only, DragEffect::Copy);
    assert!(
        session.drop(),
        "§6.21: the drop lands on an accepting target"
    );
    assert_eq!(
        session.effect(),
        DragEffect::Move,
        "§6.21: the source is told the effect was `move`, so it can remove the original"
    );

    // (b) the target accepts both and the user holds the copy modifier.
    let both = DropTarget::new(NodeId::from_raw(3))
        .accepting(MimeType::text_plain())
        .with_effects(vec![DragEffect::Copy, DragEffect::Move]);
    let mut session = DragSession::start(source());
    session.enter(&both, DragEffect::Copy);
    assert!(session.drop(), "§6.21: the drop lands");
    assert_eq!(
        session.effect(),
        DragEffect::Copy,
        "§6.21: the copy modifier selects `copy` from the mutually allowed set"
    );

    // (c) the target rejects the data.
    let rejecting = DropTarget::new(NodeId::from_raw(4)).with_effects(Vec::new());
    let mut session = DragSession::start(source());
    session.enter(&rejecting, DragEffect::Copy);
    assert!(
        !session.drop(),
        "§6.21: a target that rejected the data receives no drop"
    );
    assert_eq!(
        session.effect(),
        DragEffect::None,
        "§6.21: the source is told no effect took place"
    );
    assert_eq!(
        session.phase(),
        DragPhase::Cancelled,
        "§6.21: the drag ends with a cancel/none outcome"
    );
}
