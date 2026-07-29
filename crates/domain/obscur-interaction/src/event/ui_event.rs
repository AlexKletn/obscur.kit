//! The typed event handed to a handler as it propagates (spec §2.3, §4.7).

use obscur_identity::NodeId;

use super::{EventOrigin, EventPayload, EventPhase, EventPropagation, EventTrust};
use crate::command::KeyEvent;
use crate::pointer::PointerEvent;

/// The typed event that travels §2.3's propagation path and is handed to every
/// handler along it (spec §2.3, §4.7).
///
/// §2.3 enumerates the API an event must expose, and this is that list:
///
/// > Event API includes:
/// >
/// > - `target`;
/// > - `current_target`;
/// > - `stop_propagation`;
/// > - `stop_immediate_propagation`;
/// > - `prevent_default`;
/// > - trusted/untrusted origin.
///
/// [`target`](Self::target) is the node the event is *aimed* at — the hit-test
/// result for a pointer, the focused node for a key — and does not change while
/// the event propagates. [`current_target`](Self::current_target) is the node
/// whose handler is running *right now* and changes at every step; the pair is
/// the whole reason a bubbled handler can tell "what was clicked" from "what I am
/// attached to". [`propagation`](Self::propagation) holds the three steering
/// flags, [`trust`](Self::trust) is §2.3's trusted/untrusted origin, and
/// [`payload`](Self::payload) is §2.4's device state.
///
/// # What this type used to be
///
/// Until this change `UiEvent` carried only `origin`, `target`, `phase`,
/// `propagation` and a timestamp, and — more to the point — **nothing
/// constructed one**: `UiEvent` and `EventDispatcher` appeared outside this crate
/// in exactly two places, a prelude re-export and a spec test. The runtime
/// resolved input to a node and then ran a `FnMut()` that was told nothing at
/// all, so `current_target`, the pointer's position, the key, and the trust flag
/// were unreachable from application code even though every one of them was
/// already modeled here. The fields added below (`current_target`, `trust`,
/// `payload`) are the ones a handler could not have obtained; the rest of the gap
/// was delivery, and lives in `obscur_runtime`.
///
/// # A target is optional, and that is not a placeholder
///
/// [`target`](Self::target) is `Option<NodeId>` because §2.6's routing chain ends
/// in two links that are **not tree positions** — *"4. window; 5. application
/// fallback"*. A command dispatched with nothing focused reaches those links and
/// only those, and it genuinely has no target node; reporting some sentinel node
/// would be a lie a handler could act on.
///
/// # Not `Eq`
///
/// The pointer payload carries `f32` pressure, position, tilt and rotation, so
/// this type is `PartialEq` only. It was `Eq` while it carried no payload.
///
/// The timestamp is a *logical* value supplied by the caller (for example, from
/// the platform event or a test harness); this crate never reads a wall clock,
/// so routing stays pure and deterministic.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiEvent {
    /// The source that produced this event.
    pub origin: EventOrigin,
    /// §2.3's `target`: the node the event is aimed at — the hit-test result, or
    /// the focused node. Constant for the whole propagation; `None` when the
    /// event has no tree position (a window- or application-level command
    /// dispatched with nothing focused).
    pub target: Option<NodeId>,
    /// §2.3's `current_target`: the node whose handler is running now. Advanced
    /// at every step of the path, so it differs from [`target`](Self::target) for
    /// every capture and bubble step. `None` for the window and application links
    /// of §2.6's chain, which are not nodes.
    pub current_target: Option<NodeId>,
    /// The propagation phase the event currently occupies.
    pub phase: EventPhase,
    /// The mutable propagation-control flags carried by the event.
    pub propagation: EventPropagation,
    /// §2.3's trusted/untrusted origin: whether a platform adapter vouched for
    /// this as a real user gesture.
    pub trust: EventTrust,
    /// §2.4's device state (pointer) or §2.6's key pair (keyboard), typed per
    /// kind.
    pub payload: EventPayload,
    /// A logical timestamp in microseconds, supplied by the caller — never read
    /// from a clock.
    pub timestamp_micros: u64,
}

impl UiEvent {
    /// Creates an event aimed at `target` from `origin`, seated at the
    /// [`Target`](EventPhase::Target) phase with fresh propagation control, no
    /// payload, an untrusted origin and a zero timestamp.
    ///
    /// `current_target` starts equal to `target`, which is exactly true at the
    /// target phase; a dispatcher overwrites it at every other step.
    #[must_use]
    pub const fn new(origin: EventOrigin, target: NodeId) -> Self {
        Self {
            origin,
            target: Some(target),
            current_target: Some(target),
            phase: EventPhase::Target,
            propagation: EventPropagation::new(),
            trust: EventTrust::Untrusted,
            payload: EventPayload::Empty,
            timestamp_micros: 0,
        }
    }

    /// Creates an event with **no** target node — the honest envelope for a
    /// command that reaches only §2.6's window and application links because
    /// nothing was focused and nothing was hit.
    #[must_use]
    pub const fn detached(origin: EventOrigin) -> Self {
        Self {
            origin,
            target: None,
            current_target: None,
            phase: EventPhase::Target,
            propagation: EventPropagation::new(),
            trust: EventTrust::Untrusted,
            payload: EventPayload::Empty,
            timestamp_micros: 0,
        }
    }

    /// Creates the envelope for a [`PointerEvent`], carrying its payload, its
    /// trust and its timestamp through unchanged, with no target yet (spec §2.4).
    ///
    /// The target is deliberately absent: hit-testing is the runtime's job, not
    /// the payload's, so whoever resolves the hit seats it with
    /// [`at_target`](Self::at_target). The [`origin`](Self::origin) follows the
    /// device — a touchscreen contact is [`EventOrigin::Touch`], everything else
    /// is [`EventOrigin::Pointer`] — because §2.4 lists touch as its own source
    /// and its hover behaviour differs.
    #[must_use]
    pub const fn from_pointer(pointer: PointerEvent) -> Self {
        let origin = if pointer.device().is_touchscreen() {
            EventOrigin::Touch
        } else {
            EventOrigin::Pointer
        };
        let mut event = Self::detached(origin);
        event.trust = pointer.trust();
        event.timestamp_micros = pointer.timestamp();
        event.payload = EventPayload::Pointer(pointer);
        event
    }

    /// Creates the envelope for a [`KeyEvent`], with no target yet (spec §2.6).
    ///
    /// [`KeyEvent`] carries no trust of its own — unlike [`PointerEvent`], which
    /// has three producers and therefore has to say which one it came from — so
    /// the envelope starts [`Untrusted`](EventTrust::Untrusted) and whoever
    /// received the OS key confers trust with [`as_trusted`](Self::as_trusted).
    /// The day a key-injection path exists, that call is what it must not make.
    #[must_use]
    pub const fn from_key(key: KeyEvent) -> Self {
        let mut event = Self::detached(EventOrigin::Keyboard);
        event.payload = EventPayload::Key(key);
        event
    }

    /// Returns this event aimed at `target`, with `current_target` seated there
    /// too (fluent).
    #[must_use]
    pub const fn at_target(mut self, target: NodeId) -> Self {
        self.target = Some(target);
        self.current_target = Some(target);
        self
    }

    /// Returns this event with `current_target` moved to `node`, leaving
    /// [`target`](Self::target) untouched (fluent).
    #[must_use]
    pub const fn at_current_target(mut self, node: Option<NodeId>) -> Self {
        self.current_target = node;
        self
    }

    /// Returns this event marked as a real user gesture (fluent).
    ///
    /// **Only the layer that received the OS input may call this**, for the
    /// reason spelled out on [`EventTrust`]: trust is conferred at the platform
    /// boundary and never assumed, so a forgotten call costs an action while a
    /// careless one forges a gesture.
    #[must_use]
    pub const fn as_trusted(mut self) -> Self {
        self.trust = EventTrust::Trusted;
        self
    }

    /// Returns this event carrying `payload` (fluent).
    #[must_use]
    pub const fn with_payload(mut self, payload: EventPayload) -> Self {
        self.payload = payload;
        self
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

    /// Returns §2.3's `target`: the node the event is aimed at.
    #[must_use]
    pub const fn target(&self) -> Option<NodeId> {
        self.target
    }

    /// Returns §2.3's `current_target`: the node whose handler is running now.
    #[must_use]
    pub const fn current_target(&self) -> Option<NodeId> {
        self.current_target
    }

    /// Returns the propagation phase this event currently occupies.
    #[must_use]
    pub const fn phase(&self) -> EventPhase {
        self.phase
    }

    /// Returns §2.3's trusted/untrusted origin.
    #[must_use]
    pub const fn trust(&self) -> EventTrust {
        self.trust
    }

    /// Returns whether a platform adapter vouched for this as a real user
    /// gesture (§2.3).
    #[must_use]
    pub const fn is_trusted(&self) -> bool {
        self.trust.is_trusted()
    }

    /// Returns the typed payload this event carries.
    #[must_use]
    pub const fn payload(&self) -> EventPayload {
        self.payload
    }

    /// Returns §2.4's pointer payload, or `None` when this is not a pointer
    /// event.
    #[must_use]
    pub const fn pointer(&self) -> Option<PointerEvent> {
        self.payload.pointer()
    }

    /// Returns the keyboard payload, or `None` when this is not a key event.
    #[must_use]
    pub const fn key(&self) -> Option<KeyEvent> {
        self.payload.key()
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

    /// Requests that propagation stop at once, skipping any remaining handler on
    /// the current node, delegating to
    /// [`EventPropagation::stop_immediate_propagation`].
    pub const fn stop_immediate_propagation(&mut self) {
        self.propagation.stop_immediate_propagation();
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

    /// Returns whether the remaining handlers on the *current* node are to be
    /// skipped as well (`stopImmediatePropagation`).
    #[must_use]
    pub const fn is_immediate_stopped(&self) -> bool {
        self.propagation.stop_immediate
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
    use obscur_geometry::Point;

    use super::*;
    use crate::command::{LogicalKey, PhysicalKey};
    use crate::pointer::{PointerDevice, PointerEventKind, PointerId};

    fn node(raw: u64) -> NodeId {
        NodeId::from_raw(raw)
    }

    #[test]
    fn new_defaults() {
        let e = UiEvent::new(EventOrigin::Pointer, node(1));
        assert_eq!(e.origin(), EventOrigin::Pointer);
        assert_eq!(e.target(), Some(node(1)));
        assert_eq!(e.current_target(), Some(node(1)));
        assert_eq!(e.phase(), EventPhase::Target);
        assert_eq!(e.timestamp(), 0);
        assert!(!e.is_stopped());
        assert!(!e.is_default_prevented());
        // Trust is never acquired by omission — see `EventTrust`.
        assert!(!e.is_trusted());
        assert!(e.payload().is_empty());
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
        assert!(
            !e.is_immediate_stopped(),
            "the ordinary stop leaves same-node handlers running"
        );
    }

    #[test]
    fn stop_immediate_propagation_also_skips_the_current_node() {
        let mut e = UiEvent::new(EventOrigin::Touch, node(3));
        e.stop_immediate_propagation();
        assert!(e.is_stopped());
        assert!(e.is_immediate_stopped());
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

    /// §2.3: `target` and `current_target` are two values, and only one of them
    /// moves.
    #[test]
    fn current_target_moves_while_target_stays() {
        let e = UiEvent::new(EventOrigin::Pointer, node(7))
            .at_phase(EventPhase::Bubble)
            .at_current_target(Some(node(1)));
        assert_eq!(e.target(), Some(node(7)), "the hit node does not change");
        assert_eq!(
            e.current_target(),
            Some(node(1)),
            "the listener's node does"
        );
    }

    /// §2.6's window and application links are not tree positions, so an event
    /// that reaches only those has no target at all.
    #[test]
    fn a_detached_event_has_no_target() {
        let e = UiEvent::detached(EventOrigin::Keyboard);
        assert_eq!(e.target(), None);
        assert_eq!(e.current_target(), None);
    }

    #[test]
    fn from_pointer_carries_payload_trust_and_timestamp() {
        let pointer = PointerEvent::new(
            PointerId::MOUSE,
            PointerDevice::Mouse,
            PointerEventKind::Down,
            Point::new(12.0, 34.0),
        )
        .with_pressure(0.25)
        .with_timestamp(999)
        .as_trusted();

        let e = UiEvent::from_pointer(pointer).at_target(node(5));

        assert_eq!(e.origin(), EventOrigin::Pointer);
        assert_eq!(e.target(), Some(node(5)));
        assert!(e.is_trusted(), "trust travels with the event");
        assert_eq!(e.timestamp(), 999);
        let carried = e.pointer().expect("a pointer event carries its payload");
        assert_eq!(carried.position(), Point::new(12.0, 34.0));
        assert_eq!(carried.pressure(), 0.25);
        assert_eq!(e.key(), None);
    }

    /// §2.4 lists touch as its own source, and its hover behaviour differs, so a
    /// contact is not filed under the generic pointer origin.
    #[test]
    fn a_touch_contact_reports_the_touch_origin() {
        let contact = PointerEvent::new(
            PointerId::touch(0),
            PointerDevice::Touch,
            PointerEventKind::Down,
            Point::new(0.0, 0.0),
        );
        assert_eq!(UiEvent::from_pointer(contact).origin(), EventOrigin::Touch);
    }

    #[test]
    fn from_key_carries_the_key_and_is_untrusted_until_vouched_for() {
        let key = KeyEvent::press(PhysicalKey::Enter, Some(LogicalKey::character('\r')));
        let e = UiEvent::from_key(key);
        assert_eq!(e.origin(), EventOrigin::Keyboard);
        assert_eq!(e.key(), Some(key));
        assert_eq!(e.pointer(), None);
        assert!(!e.is_trusted());
        assert!(e.as_trusted().is_trusted());
    }
}
