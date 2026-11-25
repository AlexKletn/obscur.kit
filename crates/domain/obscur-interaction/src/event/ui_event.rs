//! The typed event handed to a handler as it propagates (spec §2.3, §4.7).

use obscur_identity::NodeId;

use super::{EventOrigin, EventPayload, EventPhase, EventPropagation, EventTrust};
use crate::command::KeyEvent;
use crate::gamepad::GamepadEvent;
use crate::gesture::{SystemBackEvent, TouchpadGestureEvent};
use crate::pointer::{PointerEvent, WheelEvent};

#[doc = include_str!("../../docs/api/event/ui_event.doc/UiEvent.md")]
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
    #[doc = include_str!("../../docs/api/event/ui_event.doc/new.md")]
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

    #[doc = include_str!("../../docs/api/event/ui_event.doc/from_pointer.md")]
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

    #[doc = include_str!("../../docs/api/event/ui_event.doc/from_key.md")]
    #[must_use]
    pub const fn from_key(key: KeyEvent) -> Self {
        let mut event = Self::detached(EventOrigin::Keyboard);
        // The envelope reports what the key reported, the way `from_pointer`
        // already did. It used to carry neither, so a keyboard-originated event
        // arrived untrusted and unstamped however the platform had marked it,
        // and every caller that wanted trust had to re-confer it by hand.
        event.trust = key.trust();
        event.timestamp_micros = key.timestamp();
        event.payload = EventPayload::Key(key);
        event
    }

    /// Returns the gamepad report carried by this event.
    #[must_use]
    pub const fn gamepad(&self) -> Option<GamepadEvent> {
        match self.payload {
            EventPayload::Gamepad(event) => Some(event),
            _ => None,
        }
    }

    /// Returns predictive Back progress if the window host supplied it.
    #[must_use]
    pub const fn system_back(&self) -> Option<SystemBackEvent> {
        match self.payload {
            EventPayload::SystemBack(event) => Some(event),
            _ => None,
        }
    }

    /// Wraps a native touchpad gesture while preserving trust and time.
    #[must_use]
    pub const fn from_touchpad_gesture(gesture: TouchpadGestureEvent) -> Self {
        let mut event = Self::detached(EventOrigin::Pointer);
        event.trust = gesture.trust;
        event.timestamp_micros = gesture.timestamp_micros;
        event.payload = EventPayload::TouchpadGesture(gesture);
        event
    }

    /// Returns the native touchpad gesture payload when present.
    #[must_use]
    pub const fn touchpad_gesture(&self) -> Option<TouchpadGestureEvent> {
        match self.payload {
            EventPayload::TouchpadGesture(gesture) => Some(gesture),
            _ => None,
        }
    }

    #[doc = include_str!("../../docs/api/event/ui_event.doc/from_wheel.md")]
    #[must_use]
    pub const fn from_wheel(wheel: WheelEvent) -> Self {
        // The pointing device's origin, because that is where a notch comes
        // from: it is aimed by the same cursor a click is, and `is_direct_input`
        // is true of it for the same reason. A `Wheel` origin was considered and
        // refused — the payload already says which of the three kinds this is,
        // and a second enum saying the same thing is a second thing to keep in
        // step.
        let mut event = Self::detached(EventOrigin::Pointer);
        // The envelope reports what the notch reported, the way `from_pointer`
        // and `from_key` do. This used to leave both at their defaults, and the
        // reason was honest at the time: a wheel had no platform converter to
        // confer them, because `UiRuntime::handle_scroll` took two floats and
        // nothing at the door could know whether the notch came from a device or
        // from an application pushing one. `handle_wheel` is that door now.
        event.trust = wheel.trust();
        event.timestamp_micros = wheel.timestamp();
        event.payload = EventPayload::Wheel(wheel);
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

    #[doc = include_str!("../../docs/api/event/ui_event.doc/as_trusted.md")]
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

    /// The wheel notch this event carries, or `None` when it carries something else.
    #[must_use]
    pub const fn wheel(&self) -> Option<WheelEvent> {
        self.payload.wheel()
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

    /// A notch is the pointing device's, and it is not a pointer event: the
    /// origin says where it came from, the payload says what it is, and asking
    /// for the wrong payload gets nothing rather than a plausible-looking
    /// pointer at the wheel's position.
    #[test]
    fn from_wheel_carries_the_notch_under_the_pointer_origin() {
        let wheel = WheelEvent::new(Point::new(30.0, 40.0), -25.0);
        let e = UiEvent::from_wheel(wheel).at_target(node(9));
        assert_eq!(e.origin(), EventOrigin::Pointer);
        assert_eq!(e.target(), Some(node(9)));
        assert_eq!(e.wheel(), Some(wheel));
        assert_eq!(e.wheel().map(|w| w.delta_y()), Some(-25.0));
        assert_eq!(e.pointer(), None);
        assert_eq!(e.key(), None);
    }

    /// Unlike `from_pointer` / `from_key`, which report what their device event
    /// reported, a wheel has no converter to report from — so it arrives
    /// untrusted and unstamped, and a host that knows better says so on the
    /// envelope rather than being lied to by it.
    #[test]
    fn a_wheel_is_untrusted_and_unstamped_until_a_host_vouches() {
        let e = UiEvent::from_wheel(WheelEvent::new(Point::new(1.0, 1.0), 10.0));
        assert!(!e.is_trusted());
        assert_eq!(e.timestamp(), 0);
        let vouched = e.as_trusted().with_timestamp(77);
        assert!(vouched.is_trusted());
        assert_eq!(vouched.timestamp(), 77);
    }
}
