//! The active drag state machine and its lifecycle (spec §6.21 drag and drop —
//! enter/leave/over/drop/cancel).

use crate::drag::{DragEffect, DragSource, DropTarget};
use obscur_geometry::Point;
use obscur_identity::NodeId;

/// The lifecycle phase of a [`DragSession`] (spec §6.21 enter/leave/over/drop/
/// cancel).
///
/// A session begins [`Started`](DragPhase::Started) — dragging but not yet over
/// any drop target — and moves to [`Over`](DragPhase::Over) while the pointer is
/// over an accepting target. It ends in one of two terminal phases:
/// [`Dropped`](DragPhase::Dropped) when a valid drop committed, or
/// [`Cancelled`](DragPhase::Cancelled) when the drag was abandoned or dropped
/// with no valid target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DragPhase {
    /// The drag has begun but the pointer is not over a drop target (spec
    /// §6.21).
    Started,
    /// The pointer is currently over an accepting drop target (spec §6.21).
    Over,
    /// A valid drop committed — terminal (spec §6.21).
    Dropped,
    /// The drag was cancelled — terminal (spec §6.21).
    Cancelled,
}

impl DragPhase {
    /// Returns whether this is a terminal phase —
    /// [`Dropped`](DragPhase::Dropped) or [`Cancelled`](DragPhase::Cancelled)
    /// (spec §6.21).
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Dropped | Self::Cancelled)
    }

    /// Returns whether this is an active phase —
    /// [`Started`](DragPhase::Started) or [`Over`](DragPhase::Over) (spec
    /// §6.21).
    #[must_use]
    pub const fn is_active(self) -> bool {
        matches!(self, Self::Started | Self::Over)
    }
}

/// The active drag state machine (spec §6.21 enter/leave/over/drop/cancel).
///
/// A session is created with [`start`](DragSession::start) from a
/// [`DragSource`] and driven by pointer events: [`move_to`](DragSession::move_to)
/// tracks the pointer, [`enter`](DragSession::enter) and
/// [`leave`](DragSession::leave) mark crossing into and out of a
/// [`DropTarget`], and [`drop`](DragSession::drop) or
/// [`cancel`](DragSession::cancel) terminate it. Throughout, the session holds
/// the current [`position`](DragSession::position), the target
/// [`over`](DragSession::over) the pointer (if any), the negotiated
/// [`effect`](DragSession::effect), and the [`phase`](DragSession::phase).
///
/// Once the session reaches a terminal phase (see
/// [`DragPhase::is_terminal`]) every mutating method is a no-op, so a committed
/// drop or cancel cannot be reopened. Any provisional layout transaction that
/// materialises placeholders or insertion slots while a drag hovers is the
/// runtime's concern and is not modeled here.
#[derive(Debug, Clone, PartialEq)]
pub struct DragSession {
    source: DragSource,
    position: Point,
    over: Option<NodeId>,
    effect: DragEffect,
    phase: DragPhase,
}

impl DragSession {
    /// Starts a session from `source` at the origin, over nothing, with no
    /// effect and phase [`Started`](DragPhase::Started) (spec §6.21).
    #[must_use]
    pub fn start(source: DragSource) -> DragSession {
        Self {
            source,
            position: Point::new(0.0, 0.0),
            over: None,
            effect: DragEffect::None,
            phase: DragPhase::Started,
        }
    }

    /// Updates the current pointer position (spec §6.21).
    ///
    /// A no-op once the session is terminal.
    pub fn move_to(&mut self, position: Point) {
        if self.phase.is_terminal() {
            return;
        }
        self.position = position;
    }

    /// Marks the pointer entering `target` and negotiates the effect (spec
    /// §6.21).
    ///
    /// If `target` accepts the dragged payload, the session records it as the
    /// current target, sets the effect to the target's negotiated choice for
    /// `preferred`, and enters phase [`Over`](DragPhase::Over). If the target
    /// rejects the payload, the session clears the current target and effect and
    /// returns to phase [`Started`](DragPhase::Started).
    ///
    /// A no-op once the session is terminal.
    pub fn enter(&mut self, target: &DropTarget, preferred: DragEffect) {
        if self.phase.is_terminal() {
            return;
        }
        if target.accepts(self.source.payload().offer()) {
            self.over = Some(target.node());
            self.effect = target.effect_for(preferred);
            self.phase = DragPhase::Over;
        } else {
            self.over = None;
            self.effect = DragEffect::None;
            self.phase = DragPhase::Started;
        }
    }

    /// Marks the pointer leaving the current target, clearing it and the effect
    /// and returning to phase [`Started`](DragPhase::Started) (spec §6.21).
    ///
    /// A no-op once the session is terminal.
    pub fn leave(&mut self) {
        if self.phase.is_terminal() {
            return;
        }
        self.over = None;
        self.effect = DragEffect::None;
        self.phase = DragPhase::Started;
    }

    /// Commits a drop, returning whether it succeeded (spec §6.21).
    ///
    /// Succeeds only when the session is in phase [`Over`](DragPhase::Over) with
    /// a non-[`None`](DragEffect::None) effect: it enters phase
    /// [`Dropped`](DragPhase::Dropped) and returns `true`. Otherwise — a drop
    /// with no valid target — it enters phase [`Cancelled`](DragPhase::Cancelled)
    /// and returns `false`.
    ///
    /// A no-op returning `false` once the session is already terminal.
    pub fn drop(&mut self) -> bool {
        if self.phase.is_terminal() {
            return false;
        }
        if self.phase == DragPhase::Over && !self.effect.is_none() {
            self.phase = DragPhase::Dropped;
            true
        } else {
            self.phase = DragPhase::Cancelled;
            false
        }
    }

    /// Cancels the drag, clearing the target and effect and entering phase
    /// [`Cancelled`](DragPhase::Cancelled) (spec §6.21).
    ///
    /// A no-op once the session is terminal.
    pub fn cancel(&mut self) {
        if self.phase.is_terminal() {
            return;
        }
        self.phase = DragPhase::Cancelled;
        self.over = None;
        self.effect = DragEffect::None;
    }

    /// Returns the current lifecycle phase (spec §6.21).
    #[must_use]
    pub const fn phase(&self) -> DragPhase {
        self.phase
    }

    /// Returns the current pointer position (spec §6.21).
    #[must_use]
    pub const fn position(&self) -> Point {
        self.position
    }

    /// Returns the drop target currently under the pointer, if any (spec §6.21).
    #[must_use]
    pub const fn over(&self) -> Option<NodeId> {
        self.over
    }

    /// Returns the currently negotiated effect (spec §6.21).
    #[must_use]
    pub const fn effect(&self) -> DragEffect {
        self.effect
    }

    /// Returns the drag source this session was started from (spec §6.21).
    #[must_use]
    pub const fn source(&self) -> &DragSource {
        &self.source
    }

    /// Returns whether the session is still active (spec §6.21).
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.phase.is_active()
    }

    /// Returns whether the session has reached a terminal phase (spec §6.21).
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        self.phase.is_terminal()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::offer::{DataOffer, MimeType};
    use crate::transfer::TransferPayload;

    fn source() -> DragSource {
        let payload = TransferPayload::new(DataOffer::text("x"));
        DragSource::new(NodeId::from_raw(2), payload)
    }

    #[test]
    fn phase_predicates() {
        assert!(DragPhase::Started.is_active());
        assert!(DragPhase::Over.is_active());
        assert!(!DragPhase::Dropped.is_active());
        assert!(DragPhase::Dropped.is_terminal());
        assert!(DragPhase::Cancelled.is_terminal());
        assert!(!DragPhase::Started.is_terminal());
    }

    #[test]
    fn full_lifecycle_start_move_enter_drop() {
        let mut session = DragSession::start(source());
        assert_eq!(session.phase(), DragPhase::Started);
        assert_eq!(session.position(), Point::new(0.0, 0.0));
        assert_eq!(session.over(), None);
        assert_eq!(session.effect(), DragEffect::None);
        assert!(session.is_active());

        session.move_to(Point::new(5.0, 6.0));
        assert_eq!(session.position(), Point::new(5.0, 6.0));

        let target = DropTarget::new(NodeId::from_raw(1));
        session.enter(&target, DragEffect::Move);
        assert_eq!(session.phase(), DragPhase::Over);
        assert_eq!(session.over(), Some(NodeId::from_raw(1)));
        assert_eq!(session.effect(), DragEffect::Move);

        assert!(session.drop());
        assert_eq!(session.phase(), DragPhase::Dropped);
        assert!(session.is_terminal());
    }

    #[test]
    fn enter_rejecting_target_clears_over() {
        let mut session = DragSession::start(source());
        let target = DropTarget::new(NodeId::from_raw(1)).accepting(MimeType::image_png());
        session.enter(&target, DragEffect::Copy);
        assert_eq!(session.over(), None);
        assert_eq!(session.effect(), DragEffect::None);
        assert_eq!(session.phase(), DragPhase::Started);
    }

    #[test]
    fn leave_resets_to_started() {
        let mut session = DragSession::start(source());
        let target = DropTarget::new(NodeId::from_raw(1));
        session.enter(&target, DragEffect::Copy);
        session.leave();
        assert_eq!(session.over(), None);
        assert_eq!(session.effect(), DragEffect::None);
        assert_eq!(session.phase(), DragPhase::Started);
    }

    #[test]
    fn drop_with_no_target_cancels() {
        let mut session = DragSession::start(source());
        assert!(!session.drop());
        assert_eq!(session.phase(), DragPhase::Cancelled);
        assert!(session.is_terminal());
    }

    #[test]
    fn cancel_terminates() {
        let mut session = DragSession::start(source());
        let target = DropTarget::new(NodeId::from_raw(1));
        session.enter(&target, DragEffect::Copy);
        session.cancel();
        assert_eq!(session.phase(), DragPhase::Cancelled);
        assert_eq!(session.over(), None);
        assert_eq!(session.effect(), DragEffect::None);
    }

    #[test]
    fn terminal_methods_are_no_ops() {
        let mut session = DragSession::start(source());
        let target = DropTarget::new(NodeId::from_raw(1));
        session.enter(&target, DragEffect::Move);
        assert!(session.drop());

        // All mutators are no-ops once terminal.
        session.move_to(Point::new(9.0, 9.0));
        session.enter(&target, DragEffect::Copy);
        session.leave();
        session.cancel();
        assert!(!session.drop());
        assert_eq!(session.phase(), DragPhase::Dropped);
        assert_ne!(session.position(), Point::new(9.0, 9.0));
    }

    #[test]
    fn source_accessor_returns_origin() {
        let session = DragSession::start(source());
        assert_eq!(session.source().origin(), NodeId::from_raw(2));
    }
}
