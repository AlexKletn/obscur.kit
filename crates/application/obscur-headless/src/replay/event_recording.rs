//! An ordered, timestamped log of recorded steps for record & replay (spec §7.13).

use super::{RecordedEvent, RecordedStep, RedactionPolicy, Sensitivity};
use crate::input::SyntheticInput;

#[doc = include_str!("../../docs/api/replay/event_recording.doc/EventRecording.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct EventRecording {
    steps: Vec<RecordedStep>,
    policy: RedactionPolicy,
    ticks_are_clock_nanos: bool,
}

impl EventRecording {
    /// Creates an empty recording using the default (redacting) policy.
    #[must_use]
    pub fn new() -> Self {
        Self::with_policy(RedactionPolicy::default())
    }

    /// Creates an empty recording using an explicit redaction `policy`.
    #[must_use]
    pub fn with_policy(policy: RedactionPolicy) -> Self {
        Self {
            steps: Vec::new(),
            policy,
            ticks_are_clock_nanos: false,
        }
    }

    pub(crate) fn from_host_clock() -> Self {
        Self {
            ticks_are_clock_nanos: true,
            ..Self::new()
        }
    }

    pub(crate) fn pointer_timestamp(&self, step: &RecordedStep) -> Option<u64> {
        self.ticks_are_clock_nanos.then_some(step.tick / 1_000)
    }

    /// Returns the redaction policy this recording was captured under.
    #[must_use]
    pub fn policy(&self) -> RedactionPolicy {
        self.policy
    }

    /// Appends `event` at `tick`. Structured events pass through unchanged; use
    /// [`record_input`](Self::record_input) to record input through the
    /// redaction policy.
    pub fn record(&mut self, tick: u64, event: RecordedEvent) {
        self.steps.push(RecordedStep::new(tick, event));
    }

    #[doc = include_str!("../../docs/api/replay/event_recording.doc/record_input.md")]
    pub fn record_input(&mut self, tick: u64, input: SyntheticInput, sensitivity: Sensitivity) {
        let event = if self.policy.redacts(sensitivity) {
            // The payload is intentionally dropped here; redaction is irreversible.
            RecordedEvent::Redacted { kind: sensitivity }
        } else {
            RecordedEvent::Input(input)
        };
        self.steps.push(RecordedStep::new(tick, event));
    }

    /// Returns the recorded steps in order.
    #[must_use]
    pub fn steps(&self) -> &[RecordedStep] {
        &self.steps
    }

    /// Returns the number of recorded steps.
    #[must_use]
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    /// Returns `true` when no steps have been recorded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Iterates over the recorded steps in order.
    pub fn iter(&self) -> std::slice::Iter<'_, RecordedStep> {
        self.steps.iter()
    }
}

impl Default for EventRecording {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> IntoIterator for &'a EventRecording {
    type Item = &'a RecordedStep;
    type IntoIter = std::slice::Iter<'a, RecordedStep>;

    fn into_iter(self) -> Self::IntoIter {
        self.steps.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::super::SchedulerReason;
    use super::*;
    use crate::input::{PointerButton, SelectorTarget, SyntheticInput};
    use obscur_identity::NodeId;

    fn some_input() -> SyntheticInput {
        SyntheticInput::pointer_down(
            SelectorTarget::node(NodeId::from_raw(1)),
            PointerButton::Primary,
        )
    }

    #[test]
    fn default_policy_redacts_secrets_and_clipboard() {
        let policy = RedactionPolicy::default();
        assert!(policy.redact_secrets);
        assert!(policy.redact_clipboard);
        assert!(policy.redacts(Sensitivity::Secret));
        assert!(policy.redacts(Sensitivity::Clipboard));
        assert!(!policy.redacts(Sensitivity::Normal));
    }

    #[test]
    fn allow_all_policy_redacts_nothing() {
        let policy = RedactionPolicy::allow_all();
        assert!(!policy.redacts(Sensitivity::Secret));
        assert!(!policy.redacts(Sensitivity::Clipboard));
        assert!(!policy.redacts(Sensitivity::Normal));
    }

    #[test]
    fn record_appends_in_order() {
        let mut rec = EventRecording::new();
        assert!(rec.is_empty());

        rec.record(0, RecordedEvent::ClockTick { nanos: 5 });
        rec.record(1, RecordedEvent::AsyncCompletion { operation: 9 });
        rec.record(
            2,
            RecordedEvent::SchedulerDecision {
                reason: SchedulerReason::Frame,
                accepted: true,
            },
        );

        assert_eq!(rec.len(), 3);
        assert!(!rec.is_empty());
        assert_eq!(rec.steps()[0].tick, 0);
        assert_eq!(rec.steps()[0].event, RecordedEvent::ClockTick { nanos: 5 });
        assert_eq!(
            rec.steps()[1].event,
            RecordedEvent::AsyncCompletion { operation: 9 }
        );
    }

    #[test]
    fn record_input_normal_is_verbatim() {
        let mut rec = EventRecording::new();
        let input = some_input();
        rec.record_input(0, input.clone(), Sensitivity::Normal);

        assert_eq!(rec.len(), 1);
        assert_eq!(rec.steps()[0].event, RecordedEvent::Input(input));
    }

    #[test]
    fn default_policy_redacts_secret_input_dropping_payload() {
        let mut rec = EventRecording::new();
        rec.record_input(3, some_input(), Sensitivity::Secret);

        assert_eq!(rec.len(), 1);
        // The original payload is absent — only the marker remains.
        assert_eq!(
            rec.steps()[0].event,
            RecordedEvent::Redacted {
                kind: Sensitivity::Secret
            }
        );
        assert!(!matches!(rec.steps()[0].event, RecordedEvent::Input(_)));
    }

    #[test]
    fn default_policy_redacts_clipboard_input() {
        let mut rec = EventRecording::new();
        rec.record_input(0, some_input(), Sensitivity::Clipboard);

        assert_eq!(
            rec.steps()[0].event,
            RecordedEvent::Redacted {
                kind: Sensitivity::Clipboard
            }
        );
    }

    #[test]
    fn policy_with_redaction_off_records_sensitive_verbatim() {
        let mut rec = EventRecording::with_policy(RedactionPolicy::allow_all());
        let input = some_input();
        rec.record_input(0, input.clone(), Sensitivity::Secret);

        assert_eq!(rec.steps()[0].event, RecordedEvent::Input(input));
    }

    #[test]
    fn iter_yields_steps_in_order() {
        let mut rec = EventRecording::new();
        rec.record(0, RecordedEvent::ClockTick { nanos: 1 });
        rec.record(1, RecordedEvent::ClockTick { nanos: 2 });

        let ticks: Vec<u64> = rec.iter().map(|s| s.tick).collect();
        assert_eq!(ticks, vec![0, 1]);

        let via_into: Vec<u64> = (&rec).into_iter().map(|s| s.tick).collect();
        assert_eq!(via_into, vec![0, 1]);
    }
}
