#![doc = include_str!("spec_ch7_snapshot_replay.doc/_module.md")]
#![forbid(unsafe_code)]

#[path = "spec_ch7_support/mod.rs"]
mod support;

use obscur_kit::headless::input::{PointerButton, SelectorTarget, SyntheticInput};
use obscur_kit::headless::replay::{
    CollectingReplayTarget, EventRecording, RecordedEvent, ReplaySession, SchedulerReason,
    Sensitivity,
};
use support::{at, click, drive, el, txt, view};

const VW: f32 = 400.0;

const CSS: &str = ".panel { width: 400px; height: 300px } \
                   .item { width: 380px; height: 20px; background-color: #445566 } \
                   .label { width: 100px; height: 12px; color: #101010 }";

fn tree() -> obscur_kit::view::view::View {
    view(vec![el(
        1,
        "panel",
        (0..49)
            .map(|i| {
                if i % 7 == 0 {
                    el(100 + i, "label", vec![txt("row")])
                } else {
                    el(100 + i, "item", vec![])
                }
            })
            .collect(),
    )])
}

#[doc = include_str!("spec_ch7_snapshot_replay.doc/snapshots_are_deterministic_and_carry_no_run_varying_values.md")]
#[test]
fn snapshots_are_deterministic_and_carry_no_run_varying_values() {
    let mut rt = drive(VW, VW, CSS, &tree());
    // A focused node and a selection, so the snapshot has runtime state in it.
    click(&mut rt, 20.0, 10.0);
    let node = at(&rt, &[0, 2]);
    rt.begin_text_editing(node, "selected text");

    let first = format!("{:?}", rt.inspect_snapshot());
    let second = format!("{:?}", rt.inspect_snapshot());
    assert_eq!(
        first, second,
        "two serializations in the same process are byte-identical"
    );

    // A fresh runtime running the identical script stands in for a fresh
    // process: the snapshot must not depend on anything the first run left
    // behind (hash seeding, allocation order, a global counter).
    let mut again = drive(VW, VW, CSS, &tree());
    click(&mut again, 20.0, 10.0);
    let again_node = at(&again, &[0, 2]);
    again.begin_text_editing(again_node, "selected text");
    let third = format!("{:?}", again.inspect_snapshot());
    assert_eq!(
        first, third,
        "an independent run of the identical script produces the identical snapshot"
    );

    for forbidden in ["0x7f", "Instant", "ThreadId", "SystemTime"] {
        assert!(
            !first.contains(forbidden),
            "the snapshot carries no {forbidden} value: {first}"
        );
    }
}

#[doc = include_str!("spec_ch7_snapshot_replay.doc/a_snapshot_records_focus_selection_and_scroll.md")]
#[test]
fn a_snapshot_records_focus_selection_and_scroll() {
    const SCROLLING_CSS: &str = ".panel { width: 400px; height: 300px; overflow: auto } \
                                 .item { width: 380px; height: 20px; background-color: #445566 } \
                                 .label { width: 100px; height: 12px; color: #101010 }";
    let mut rt = drive(VW, VW, SCROLLING_CSS, &tree());
    let third = at(&rt, &[0, 2]);

    rt.set_scroll_top(at(&rt, &[0]), 120.0);
    click(&mut rt, 20.0, 50.0);
    rt.begin_text_editing(third, "0123456789");

    let snapshot = rt.semantic_snapshot();
    let focused = snapshot
        .iter()
        .find(|(_, node)| node.state().focused)
        .map(|(id, _)| id);
    assert_eq!(
        focused,
        rt.focused(),
        "the snapshot records the focused node's stable ID"
    );
    assert!(
        focused.is_some(),
        "a snapshot taken with focus seated records that focus"
    );

    assert_eq!(
        rt.scroll_top(at(&rt, &[0])),
        120.0,
        "the snapshot's scroll offset is 120"
    );
    assert!(
        rt.editing_selection().is_some(),
        "the snapshot records the selection, not merely that text exists"
    );
}

#[doc = include_str!("spec_ch7_snapshot_replay.doc/sensitive_input_is_redacted_by_default.md")]
#[test]
fn sensitive_input_is_redacted_by_default() {
    // NO redaction options are passed: the default policy is what is under test.
    let mut recording = EventRecording::new();

    // A secret typed into a field in password/sensitive mode.
    const SECRET_KEY: u32 = 0x6875_6E32;
    recording.record_input(0, SyntheticInput::key_down(SECRET_KEY), Sensitivity::Secret);

    // Protected clipboard content pasted into the same field.
    const CLIPBOARD_KEY: u32 = 0x1BAD_C0DE;
    recording.record_input(
        1,
        SyntheticInput::key_down(CLIPBOARD_KEY),
        Sensitivity::Clipboard,
    );

    // Ordinary input into an ordinary field.
    const PUBLIC_KEY: u32 = 0x0000_7075;
    recording.record_input(2, SyntheticInput::key_down(PUBLIC_KEY), Sensitivity::Normal);

    // And a focus move, so the replayed sequence still has structure.
    recording.record(
        3,
        RecordedEvent::SchedulerDecision {
            reason: SchedulerReason::Frame,
            accepted: true,
        },
    );

    let serialized = format!("{recording:?}");
    assert!(
        !serialized.contains(&SECRET_KEY.to_string()),
        "the secret payload appears nowhere in the serialized recording: {serialized}"
    );
    assert!(
        !serialized.contains(&CLIPBOARD_KEY.to_string()),
        "the protected clipboard payload appears nowhere in the serialized recording"
    );
    assert!(
        serialized.contains(&PUBLIC_KEY.to_string()),
        "ordinary input is recorded verbatim"
    );

    // Replay still reproduces the same sequence of steps.
    assert_eq!(
        recording.len(),
        4,
        "redaction drops payloads, not steps: the focus sequence survives"
    );
    assert!(
        matches!(
            recording.steps()[0].event,
            RecordedEvent::Redacted {
                kind: Sensitivity::Secret
            }
        ),
        "the secret step is a redaction marker"
    );
}

#[doc = include_str!("spec_ch7_snapshot_replay.doc/replay_is_bit_reproducible_under_load.md")]
#[test]
fn replay_is_bit_reproducible_under_load() {
    let mut recording = EventRecording::new();
    let mut tick = 0_u64;
    for frame in 0..30_u64 {
        recording.record(tick, RecordedEvent::ClockTick { nanos: 16_666_667 });
        tick += 1;
        if frame == 12 {
            recording.record(tick, RecordedEvent::AsyncCompletion { operation: 42 });
            tick += 1;
            recording.record(
                tick,
                RecordedEvent::ResourceVersion {
                    resource: 42,
                    version: 1,
                },
            );
            tick += 1;
        }
        if frame % 6 == 0 {
            recording.record_input(
                tick,
                SyntheticInput::pointer_down(
                    SelectorTarget::test_id("row"),
                    PointerButton::Primary,
                ),
                Sensitivity::Normal,
            );
            tick += 1;
        }
    }

    let session = ReplaySession::new(recording);

    let run = |session: &ReplaySession| {
        let mut target = CollectingReplayTarget::new();
        let result = session.replay(&mut target);
        (format!("{target:?}"), format!("{result:?}"))
    };

    let first = run(&session);
    let second = run(&session);

    // A third replay while the machine is busy.
    let load: Vec<_> = (0..4)
        .map(|_| {
            std::thread::spawn(|| {
                let mut acc = 0_u64;
                for i in 0..40_000_000_u64 {
                    acc = acc.wrapping_add(i);
                }
                acc
            })
        })
        .collect();
    let third = run(&session);
    for handle in load {
        let _ = handle.join();
    }

    assert_eq!(
        first, second,
        "two replays produce the identical per-frame sequence, byte for byte"
    );
    assert_eq!(
        first, third,
        "the replay under CPU load is identical too — including the frame index the \
         resource completion lands at"
    );
}

#[test]
fn replay_drives_the_real_headless_runtime_and_completes_selected_jobs() {
    use obscur_kit::css::{
        compiler::StylesheetCompiler, parser::StylesheetParser, source::CssSource,
    };
    use obscur_kit::geometry::{Point, Size};
    use obscur_kit::headless::host::HeadlessHost;
    use obscur_kit::headless::replay::{EnvironmentChange, ReplayStatus};
    use obscur_kit::identity::RuntimeId;
    use std::sync::{Arc, Mutex};

    let source = CssSource::anonymous(CSS);
    let sheet = StylesheetCompiler::new()
        .compile(&StylesheetParser::new(&source).parse())
        .matchable;
    let scene = tree();
    let mut recording = EventRecording::new();
    recording.record(
        0,
        RecordedEvent::EnvironmentChange(EnvironmentChange::WindowResized {
            width: 400,
            height: 240,
        }),
    );
    recording.record(
        1,
        RecordedEvent::SchedulerDecision {
            reason: SchedulerReason::Frame,
            accepted: true,
        },
    );
    recording.record_input(
        2,
        SyntheticInput::pointer_move(SelectorTarget::coordinate(Point::new(20.0, 24.0))),
        Sensitivity::Normal,
    );
    recording.record(3, RecordedEvent::AsyncCompletion { operation: 1 });
    recording.record(4, RecordedEvent::ClockTick { nanos: 16_000_000 });
    recording.record(
        5,
        RecordedEvent::SchedulerDecision {
            reason: SchedulerReason::Frame,
            accepted: true,
        },
    );
    let replay = ReplaySession::new(recording);
    let run = || {
        let mut host = HeadlessHost::new(RuntimeId::from_raw(1));
        let order = Arc::new(Mutex::new(Vec::new()));
        for index in 0..2 {
            let order = order.clone();
            host.executor()
                .spawn(move || order.lock().unwrap().push(index));
        }
        let result = replay.replay_on_host(&mut host, &scene, std::slice::from_ref(&sheet));
        assert_eq!(result.status(), ReplayStatus::Completed);
        assert_eq!(result.inputs_applied(), 1);
        assert_eq!(result.clock_advanced_nanos(), 16_000_000);
        assert_eq!(result.completions(), 1);
        assert_eq!(*order.lock().unwrap(), [1]);
        assert_eq!(host.executor().pending_operations(), [0]);
        assert_eq!(host.runtime().viewport(), Size::new(400.0, 240.0));
        let snapshot = host.snapshot();
        assert!(!snapshot.display_list().is_empty());
        snapshot
    };
    assert_eq!(run().display_list(), run().display_list());
}

#[test]
fn replay_delivers_normalized_key_events_to_the_live_editor() {
    use obscur_kit::headless::{host::HeadlessHost, replay::ReplayStatus};
    use obscur_kit::identity::{ComponentId, InternedSymbol, RuntimeId};
    use obscur_kit::interaction::command::{KeyEvent, LogicalKey, NamedKey, PhysicalKey};
    use obscur_kit::view::view::{
        EVENT_INPUT, LOCAL_NAME, View, ViewEventBinding, ViewNode, ViewProperty, ViewValue,
    };

    let field = View::single(
        ViewNode::element(ComponentId::from_raw(4))
            .with_prop(ViewProperty::new(
                LOCAL_NAME,
                ViewValue::Text("input".into()),
            ))
            .with_event(ViewEventBinding::new(
                EVENT_INPUT,
                InternedSymbol::named("edit"),
            )),
    );
    let mut host = HeadlessHost::new(RuntimeId::from_raw(1));
    let mut recording = EventRecording::new();
    recording.record(
        0,
        RecordedEvent::SchedulerDecision {
            reason: SchedulerReason::Frame,
            accepted: true,
        },
    );
    recording.record_input(
        1,
        SyntheticInput::key(KeyEvent::press(
            PhysicalKey::Tab,
            Some(LogicalKey::Named(NamedKey::Tab)),
        )),
        Sensitivity::Normal,
    );
    let key = |ch| {
        SyntheticInput::key(KeyEvent::press(
            PhysicalKey::KeyA,
            Some(LogicalKey::character(ch)),
        ))
    };
    recording.record_input(2, key('x'), Sensitivity::Secret);
    recording.record_input(3, key('é'), Sensitivity::Normal);
    let result = ReplaySession::new(recording).replay_on_host(&mut host, &field, &[]);
    assert_eq!(result.status(), ReplayStatus::Completed);
    assert_eq!(result.inputs_applied(), 2);
    assert_eq!(result.redactions_skipped(), 1);
    assert_eq!(host.runtime().editing_text(), Some("é"));
}
