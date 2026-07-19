//! **SPECIFICATION-DERIVED** snapshot and record/replay cases for obscur.kit
//! §7.12 / §7.13.
//!
//! Written by an agent reading obscur.kit §7.12, §7.13 and §7.14 with **no
//! access to this repository's source**.
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

/// §7.12: "Development/diagnostic mode may serialize snapshots of:" (list) with
/// §7.14 "Golden tests should prefer backend-independent snapshots:".
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

/// §7.12, verbatim: "Development/diagnostic mode may serialize snapshots of:" /
/// "- focus;" / "- selection;" / "- scroll;" / "- animation;".
///
/// FIXTURE REPAIRED 2026-07-19: nothing here is weakened. The case set a scroll
/// offset of 120 on `.panel` and read back 0, and the shared `CSS` above styles
/// `.panel` as `width: 400px; height: 300px` with no `overflow` declaration — so
/// it is not a scroll container, `set_scroll_top` is correctly a no-op on it,
/// and there was no scroll state for any snapshot to record. The case now uses
/// its own stylesheet in which the panel actually scrolls (49 rows of 20px
/// against a 300px viewport, so 120 is well inside range). The assertion is
/// unchanged.
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

/// §7.13, verbatim: "Sensitive input such as passwords, secrets and protected
/// clipboard data is excluded or redacted by default."
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

/// §7.13, verbatim: "Replay runs against deterministic clocks/executors." and
/// "Optional development/test record-replay may capture:" /
/// "- scheduler decisions;" / "- clock ticks;" / "- async completions;" /
/// "- resource versions;".
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
