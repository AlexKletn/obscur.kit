//! The active drag state machine and its lifecycle (spec §6.21 drag and drop —
//! enter/leave/over/drop/cancel).

use crate::drag::{DragEffect, DragSource, DropTarget};
use obscur_geometry::Point;
use obscur_identity::NodeId;

use super::DragPhase;

#[doc = include_str!("../../docs/api/drag/drag_session.doc/DragSession.md")]
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

    #[doc = include_str!("../../docs/api/drag/drag_session.doc/move_to.md")]
    pub fn move_to(&mut self, position: Point) {
        if self.phase.is_terminal() {
            return;
        }
        self.position = position;
    }

    #[doc = include_str!("../../docs/api/drag/drag_session.doc/enter.md")]
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

    #[doc = include_str!("../../docs/api/drag/drag_session.doc/leave.md")]
    pub fn leave(&mut self) {
        if self.phase.is_terminal() {
            return;
        }
        self.over = None;
        self.effect = DragEffect::None;
        self.phase = DragPhase::Started;
    }

    #[doc = include_str!("../../docs/api/drag/drag_session.doc/drop.md")]
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

    #[doc = include_str!("../../docs/api/drag/drag_session.doc/cancel.md")]
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
