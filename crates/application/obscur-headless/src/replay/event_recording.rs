//! An ordered, timestamped log of recorded steps for record & replay (spec §7.13).

use crate::input::SyntheticInput;

/// How sensitive a recorded input is — the classification that drives redaction.
///
/// A caller tags each recorded input with its sensitivity so the
/// [`RedactionPolicy`] can decide whether the payload is stored verbatim or
/// dropped and replaced with a [`RecordedEvent::Redacted`] marker (spec §7.13).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Sensitivity {
    /// Ordinary, non-sensitive input; recorded verbatim.
    #[default]
    Normal,
    /// A secret (password, token, credential); redacted by default.
    Secret,
    /// Protected clipboard data; redacted by default.
    Clipboard,
}

/// Which classes of sensitive input are redacted when recorded (spec §7.13).
///
/// Redaction is **default-on**: [`RedactionPolicy::default`] redacts both
/// secrets and protected clipboard data. Redaction is also **irreversible** —
/// when a class is redacted the original payload is dropped, never stored, so a
/// recording can never leak a redacted secret no matter how it is later
/// inspected or replayed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RedactionPolicy {
    /// Whether [`Sensitivity::Secret`] inputs are redacted. Default `true`.
    pub redact_secrets: bool,
    /// Whether [`Sensitivity::Clipboard`] inputs are redacted. Default `true`.
    pub redact_clipboard: bool,
}

impl Default for RedactionPolicy {
    /// The safe default: redact both secrets and protected clipboard data.
    fn default() -> Self {
        Self {
            redact_secrets: true,
            redact_clipboard: true,
        }
    }
}

impl RedactionPolicy {
    /// A policy that redacts both secrets and clipboard data (the default).
    #[must_use]
    pub fn redacting() -> Self {
        Self::default()
    }

    /// A policy that records everything verbatim, redacting nothing.
    ///
    /// This opts out of the default protection; use it only for recordings that
    /// are known not to carry sensitive input.
    #[must_use]
    pub const fn allow_all() -> Self {
        Self {
            redact_secrets: false,
            redact_clipboard: false,
        }
    }

    /// Returns `true` when input of the given `sensitivity` must be redacted
    /// under this policy. [`Sensitivity::Normal`] is never redacted.
    #[must_use]
    pub const fn redacts(&self, sensitivity: Sensitivity) -> bool {
        match sensitivity {
            Sensitivity::Normal => false,
            Sensitivity::Secret => self.redact_secrets,
            Sensitivity::Clipboard => self.redact_clipboard,
        }
    }
}

/// One recorded happening, covering the categories a record-replay run may
/// capture (spec §7.13): normalized platform events, scheduler decisions, clock
/// ticks, async completions, resource versions, and window/environment changes —
/// plus a [`Redacted`](RecordedEvent::Redacted) placeholder for input whose
/// payload was dropped by the [`RedactionPolicy`].
#[derive(Debug, Clone, PartialEq)]
pub enum RecordedEvent {
    /// A normalized platform input event.
    Input(SyntheticInput),
    /// A deterministic clock tick, advancing virtual time by `nanos`.
    ClockTick {
        /// Nanoseconds the clock advanced.
        nanos: u64,
    },
    /// An async completion delivered for the given `operation`.
    AsyncCompletion {
        /// The raw operation id the completion resolves.
        operation: u64,
    },
    /// A scheduler decision, recorded as a typed marker.
    SchedulerDecision {
        /// Why the scheduler made this decision.
        reason: SchedulerReason,
        /// Whether the scheduled work was accepted (vs. coalesced/dropped).
        accepted: bool,
    },
    /// A resource whose version changed.
    ResourceVersion {
        /// The raw resource id.
        resource: u64,
        /// The new version of that resource.
        version: u64,
    },
    /// A window/environment change.
    EnvironmentChange(EnvironmentChange),
    /// A placeholder for input whose payload was redacted and dropped. Carries
    /// only the [`Sensitivity`] class, never the original data (spec §7.13).
    Redacted {
        /// Which sensitivity class was redacted.
        kind: Sensitivity,
    },
}

/// Why the scheduler reached a recorded decision (a small typed marker).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SchedulerReason {
    /// A frame was scheduled.
    Frame,
    /// A timer fired.
    Timer,
    /// Idle work was scheduled.
    Idle,
    /// Work was coalesced with pending work of the same kind.
    Coalesced,
}

/// A recorded window/environment change (spec §7.13).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnvironmentChange {
    /// The window was resized to `(width, height)` in physical pixels.
    WindowResized {
        /// New window width.
        width: u32,
        /// New window height.
        height: u32,
    },
    /// The window's visibility changed (`true` = visible).
    VisibilityChanged {
        /// Whether the window is now visible.
        visible: bool,
    },
}

/// One entry in an [`EventRecording`]: a monotonic `tick` and its `event`.
///
/// `tick` is either a monotonic sequence index or the deterministic clock value
/// at record time; the recording preserves entries in the order they were added.
#[derive(Debug, Clone, PartialEq)]
pub struct RecordedStep {
    /// The monotonic tick (sequence index or clock value) at record time.
    pub tick: u64,
    /// What happened at this tick.
    pub event: RecordedEvent,
}

impl RecordedStep {
    /// Creates a recorded step pairing `tick` with `event`.
    #[must_use]
    pub const fn new(tick: u64, event: RecordedEvent) -> Self {
        Self { tick, event }
    }
}

/// An ordered, timestamped log of recorded steps together with the
/// [`RedactionPolicy`] under which it was captured (spec §7.13).
///
/// Steps are appended in order via [`record`](Self::record) /
/// [`record_input`](Self::record_input). Recording an input goes through the
/// policy: a sensitive class that the policy redacts is stored as a
/// [`RecordedEvent::Redacted`] marker with its payload dropped, so a recording
/// never carries redacted secrets.
#[derive(Debug, Clone, PartialEq)]
pub struct EventRecording {
    steps: Vec<RecordedStep>,
    policy: RedactionPolicy,
}

impl EventRecording {
    /// Creates an empty recording using the default (redacting) policy.
    #[must_use]
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            policy: RedactionPolicy::default(),
        }
    }

    /// Creates an empty recording using an explicit redaction `policy`.
    #[must_use]
    pub fn with_policy(policy: RedactionPolicy) -> Self {
        Self {
            steps: Vec::new(),
            policy,
        }
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

    /// Records a normalized platform `input` at `tick`, applying the redaction
    /// policy for its `sensitivity`.
    ///
    /// When the policy redacts that class, only a [`RecordedEvent::Redacted`]
    /// marker (carrying the [`Sensitivity`] class) is stored — the original
    /// `input` payload is dropped and never persisted (spec §7.13). A
    /// [`Sensitivity::Normal`] input, or any class the policy does not redact,
    /// is recorded verbatim as [`RecordedEvent::Input`].
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
    use super::*;
    use crate::input::{PointerButton, SelectorTarget, SyntheticInput};
    use obscur_identity::NodeId;

    fn some_input() -> SyntheticInput {
        SyntheticInput::pointer_down(SelectorTarget::node(NodeId::from_raw(1)), PointerButton::Primary)
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
