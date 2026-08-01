#![doc = include_str!("event_recording.doc/_module.md")]

use crate::input::SyntheticInput;

#[doc = include_str!("event_recording.doc/Sensitivity.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Sensitivity {
    #[doc = include_str!("event_recording.doc/Normal.md")]
    #[default]
    Normal,
    #[doc = include_str!("event_recording.doc/Secret.md")]
    Secret,
    #[doc = include_str!("event_recording.doc/Clipboard.md")]
    Clipboard,
}

#[doc = include_str!("event_recording.doc/RedactionPolicy.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RedactionPolicy {
    #[doc = include_str!("event_recording.doc/redact_secrets.md")]
    pub redact_secrets: bool,
    #[doc = include_str!("event_recording.doc/redact_clipboard.md")]
    pub redact_clipboard: bool,
}

impl Default for RedactionPolicy {
    #[doc = include_str!("event_recording.doc/default.md")]
    fn default() -> Self {
        Self {
            redact_secrets: true,
            redact_clipboard: true,
        }
    }
}

impl RedactionPolicy {
    #[doc = include_str!("event_recording.doc/redacting.md")]
    #[must_use]
    pub fn redacting() -> Self {
        Self::default()
    }

    #[doc = include_str!("event_recording.doc/allow_all.md")]
    #[must_use]
    pub const fn allow_all() -> Self {
        Self {
            redact_secrets: false,
            redact_clipboard: false,
        }
    }

    #[doc = include_str!("event_recording.doc/redacts.md")]
    #[must_use]
    pub const fn redacts(&self, sensitivity: Sensitivity) -> bool {
        match sensitivity {
            Sensitivity::Normal => false,
            Sensitivity::Secret => self.redact_secrets,
            Sensitivity::Clipboard => self.redact_clipboard,
        }
    }
}

#[doc = include_str!("event_recording.doc/RecordedEvent.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum RecordedEvent {
    #[doc = include_str!("event_recording.doc/Input.md")]
    Input(SyntheticInput),
    #[doc = include_str!("event_recording.doc/ClockTick.md")]
    ClockTick {
        #[doc = include_str!("event_recording.doc/nanos.md")]
        nanos: u64,
    },
    #[doc = include_str!("event_recording.doc/AsyncCompletion.md")]
    AsyncCompletion {
        #[doc = include_str!("event_recording.doc/operation.md")]
        operation: u64,
    },
    #[doc = include_str!("event_recording.doc/SchedulerDecision.md")]
    SchedulerDecision {
        #[doc = include_str!("event_recording.doc/reason.md")]
        reason: SchedulerReason,
        #[doc = include_str!("event_recording.doc/accepted.md")]
        accepted: bool,
    },
    #[doc = include_str!("event_recording.doc/ResourceVersion.md")]
    ResourceVersion {
        #[doc = include_str!("event_recording.doc/resource.md")]
        resource: u64,
        #[doc = include_str!("event_recording.doc/version.md")]
        version: u64,
    },
    #[doc = include_str!("event_recording.doc/EnvironmentChange.md")]
    EnvironmentChange(EnvironmentChange),
    #[doc = include_str!("event_recording.doc/Redacted.md")]
    Redacted {
        #[doc = include_str!("event_recording.doc/kind.md")]
        kind: Sensitivity,
    },
}

#[doc = include_str!("event_recording.doc/SchedulerReason.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SchedulerReason {
    #[doc = include_str!("event_recording.doc/Frame.md")]
    Frame,
    #[doc = include_str!("event_recording.doc/Timer.md")]
    Timer,
    #[doc = include_str!("event_recording.doc/Idle.md")]
    Idle,
    #[doc = include_str!("event_recording.doc/Coalesced.md")]
    Coalesced,
}

#[doc = include_str!("event_recording.doc/EnvironmentChange_2.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnvironmentChange {
    #[doc = include_str!("event_recording.doc/WindowResized.md")]
    WindowResized {
        #[doc = include_str!("event_recording.doc/width.md")]
        width: u32,
        #[doc = include_str!("event_recording.doc/height.md")]
        height: u32,
    },
    #[doc = include_str!("event_recording.doc/VisibilityChanged.md")]
    VisibilityChanged {
        #[doc = include_str!("event_recording.doc/visible.md")]
        visible: bool,
    },
}

#[doc = include_str!("event_recording.doc/RecordedStep.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct RecordedStep {
    #[doc = include_str!("event_recording.doc/tick.md")]
    pub tick: u64,
    #[doc = include_str!("event_recording.doc/event.md")]
    pub event: RecordedEvent,
}

impl RecordedStep {
    #[doc = include_str!("event_recording.doc/new.md")]
    #[must_use]
    pub const fn new(tick: u64, event: RecordedEvent) -> Self {
        Self { tick, event }
    }
}

#[doc = include_str!("event_recording.doc/EventRecording.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct EventRecording {
    steps: Vec<RecordedStep>,
    policy: RedactionPolicy,
}

impl EventRecording {
    #[doc = include_str!("event_recording.doc/new_2.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            policy: RedactionPolicy::default(),
        }
    }

    #[doc = include_str!("event_recording.doc/with_policy.md")]
    #[must_use]
    pub fn with_policy(policy: RedactionPolicy) -> Self {
        Self {
            steps: Vec::new(),
            policy,
        }
    }

    #[doc = include_str!("event_recording.doc/policy.md")]
    #[must_use]
    pub fn policy(&self) -> RedactionPolicy {
        self.policy
    }

    #[doc = include_str!("event_recording.doc/record.md")]
    pub fn record(&mut self, tick: u64, event: RecordedEvent) {
        self.steps.push(RecordedStep::new(tick, event));
    }

    #[doc = include_str!("event_recording.doc/record_input.md")]
    pub fn record_input(&mut self, tick: u64, input: SyntheticInput, sensitivity: Sensitivity) {
        let event = if self.policy.redacts(sensitivity) {
            // The payload is intentionally dropped here; redaction is irreversible.
            RecordedEvent::Redacted { kind: sensitivity }
        } else {
            RecordedEvent::Input(input)
        };
        self.steps.push(RecordedStep::new(tick, event));
    }

    #[doc = include_str!("event_recording.doc/steps.md")]
    #[must_use]
    pub fn steps(&self) -> &[RecordedStep] {
        &self.steps
    }

    #[doc = include_str!("event_recording.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    #[doc = include_str!("event_recording.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    #[doc = include_str!("event_recording.doc/iter.md")]
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
