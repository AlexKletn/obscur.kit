//! The base routing envelope shared by all input events (spec §4.7).

use obscur_identity::NodeId;

use super::{EventOrigin, EventPhase, EventPropagation};

/// The routing envelope carried by every input event through the event model
/// (spec §4.7).
///
/// [`UiEvent`] holds only what routing needs: where the event came from
/// ([`origin`](Self::origin)), the hit-test [`target`](Self::target) it is aimed
/// at, the [`phase`](Self::phase) it currently occupies (updated as it
/// propagates), the mutable [`propagation`](Self::propagation) control, and a
/// logical [`timestamp`](Self::timestamp). Specific payloads — pointer
/// coordinates, key codes, gesture deltas — live in sibling modules and pair
/// with this envelope rather than living inside it.
///
/// The timestamp is a *logical* value supplied by the caller (for example, from
/// the platform event or a test harness); this crate never reads a wall clock,
/// so routing stays pure and deterministic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiEvent {
    /// The source that produced this event.
    pub origin: EventOrigin,
    /// The hit-test node the event is aimed at.
    pub target: NodeId,
    /// The propagation phase the event currently occupies.
    pub phase: EventPhase,
    /// The mutable propagation-control flags carried by the event.
    pub propagation: EventPropagation,
    /// A logical timestamp in microseconds, supplied by the caller — never read
    /// from a clock.
    pub timestamp_micros: u64,
}

impl UiEvent {
    /// Creates an event aimed at `target` from `origin`, seated at the
    /// [`Target`](EventPhase::Target) phase with fresh propagation control and a
    /// zero timestamp.
    #[must_use]
    pub const fn new(origin: EventOrigin, target: NodeId) -> Self {
        Self {
            origin,
            target,
            phase: EventPhase::Target,
            propagation: EventPropagation::new(),
            timestamp_micros: 0,
        }
    }

    /// Returns this event with its logical timestamp set (fluent).
    #[must_use]
    pub const fn with_timestamp(mut self, timestamp_micros: u64) -> Self {
        self.timestamp_micros = timestamp_micros;
        self
    }

    /// Returns this event seated at `phase` (fluent).
    #[must_use]
    pub const fn at_phase(mut self, phase: EventPhase) -> Self {
        self.phase = phase;
        self
    }

    /// Returns the source that produced this event.
    #[must_use]
    pub const fn origin(&self) -> EventOrigin {
        self.origin
    }

    /// Returns the hit-test node this event is aimed at.
    #[must_use]
    pub const fn target(&self) -> NodeId {
        self.target
    }

    /// Returns the propagation phase this event currently occupies.
    #[must_use]
    pub const fn phase(&self) -> EventPhase {
        self.phase
    }

    /// Returns the logical timestamp, in microseconds, supplied by the caller.
    #[must_use]
    pub const fn timestamp(&self) -> u64 {
        self.timestamp_micros
    }

    /// Returns mutable access to the propagation control for advanced steering.
    pub const fn propagation_mut(&mut self) -> &mut EventPropagation {
        &mut self.propagation
    }

    /// Requests that propagation stop after the current node finishes, delegating
    /// to [`EventPropagation::stop_propagation`].
    pub const fn stop_propagation(&mut self) {
        self.propagation.stop_propagation();
    }

    /// Suppresses the host's default action, delegating to
    /// [`EventPropagation::prevent_default`].
    pub const fn prevent_default(&mut self) {
        self.propagation.prevent_default();
    }

    /// Returns whether propagation has been stopped, delegating to
    /// [`EventPropagation::is_stopped`].
    #[must_use]
    pub const fn is_stopped(&self) -> bool {
        self.propagation.is_stopped()
    }

    /// Returns whether the host's default action has been suppressed, delegating
    /// to [`EventPropagation::is_default_prevented`].
    #[must_use]
    pub const fn is_default_prevented(&self) -> bool {
        self.propagation.is_default_prevented()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(raw: u64) -> NodeId {
        NodeId::from_raw(raw)
    }

    #[test]
    fn new_defaults() {
        let e = UiEvent::new(EventOrigin::Pointer, node(1));
        assert_eq!(e.origin(), EventOrigin::Pointer);
        assert_eq!(e.target(), node(1));
        assert_eq!(e.phase(), EventPhase::Target);
        assert_eq!(e.timestamp(), 0);
        assert!(!e.is_stopped());
        assert!(!e.is_default_prevented());
    }

    #[test]
    fn builder_sets_timestamp_and_phase() {
        let e = UiEvent::new(EventOrigin::Keyboard, node(2))
            .with_timestamp(1_234)
            .at_phase(EventPhase::Capture);
        assert_eq!(e.timestamp(), 1_234);
        assert_eq!(e.phase(), EventPhase::Capture);
    }

    #[test]
    fn stop_propagation_delegates_to_is_stopped() {
        let mut e = UiEvent::new(EventOrigin::Touch, node(3));
        assert!(!e.is_stopped());
        e.stop_propagation();
        assert!(e.is_stopped());
    }

    #[test]
    fn prevent_default_delegates() {
        let mut e = UiEvent::new(EventOrigin::Touch, node(3));
        e.prevent_default();
        assert!(e.is_default_prevented());
    }

    #[test]
    fn propagation_mut_exposes_control() {
        let mut e = UiEvent::new(EventOrigin::Gesture, node(4));
        e.propagation_mut().stop_immediate_propagation();
        assert!(e.is_stopped());
    }
}
