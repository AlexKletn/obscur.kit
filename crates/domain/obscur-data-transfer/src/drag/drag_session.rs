#![doc = include_str!("drag_session.doc/_module.md")]

use crate::drag::{DragEffect, DragSource, DropTarget};
use obscur_geometry::Point;
use obscur_identity::NodeId;

#[doc = include_str!("drag_session.doc/DragPhase.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DragPhase {
    #[doc = include_str!("drag_session.doc/Started.md")]
    Started,
    #[doc = include_str!("drag_session.doc/Over.md")]
    Over,
    #[doc = include_str!("drag_session.doc/Dropped.md")]
    Dropped,
    #[doc = include_str!("drag_session.doc/Cancelled.md")]
    Cancelled,
}

impl DragPhase {
    #[doc = include_str!("drag_session.doc/is_terminal.md")]
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Dropped | Self::Cancelled)
    }

    #[doc = include_str!("drag_session.doc/is_active.md")]
    #[must_use]
    pub const fn is_active(self) -> bool {
        matches!(self, Self::Started | Self::Over)
    }
}

#[doc = include_str!("drag_session.doc/DragSession.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct DragSession {
    source: DragSource,
    position: Point,
    over: Option<NodeId>,
    effect: DragEffect,
    phase: DragPhase,
}

impl DragSession {
    #[doc = include_str!("drag_session.doc/start.md")]
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

    #[doc = include_str!("drag_session.doc/move_to.md")]
    pub fn move_to(&mut self, position: Point) {
        if self.phase.is_terminal() {
            return;
        }
        self.position = position;
    }

    #[doc = include_str!("drag_session.doc/enter.md")]
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

    #[doc = include_str!("drag_session.doc/leave.md")]
    pub fn leave(&mut self) {
        if self.phase.is_terminal() {
            return;
        }
        self.over = None;
        self.effect = DragEffect::None;
        self.phase = DragPhase::Started;
    }

    #[doc = include_str!("drag_session.doc/drop.md")]
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

    #[doc = include_str!("drag_session.doc/cancel.md")]
    pub fn cancel(&mut self) {
        if self.phase.is_terminal() {
            return;
        }
        self.phase = DragPhase::Cancelled;
        self.over = None;
        self.effect = DragEffect::None;
    }

    #[doc = include_str!("drag_session.doc/phase.md")]
    #[must_use]
    pub const fn phase(&self) -> DragPhase {
        self.phase
    }

    #[doc = include_str!("drag_session.doc/position.md")]
    #[must_use]
    pub const fn position(&self) -> Point {
        self.position
    }

    #[doc = include_str!("drag_session.doc/over.md")]
    #[must_use]
    pub const fn over(&self) -> Option<NodeId> {
        self.over
    }

    #[doc = include_str!("drag_session.doc/effect.md")]
    #[must_use]
    pub const fn effect(&self) -> DragEffect {
        self.effect
    }

    #[doc = include_str!("drag_session.doc/source.md")]
    #[must_use]
    pub const fn source(&self) -> &DragSource {
        &self.source
    }

    #[doc = include_str!("drag_session.doc/is_active_2.md")]
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.phase.is_active()
    }

    #[doc = include_str!("drag_session.doc/is_terminal_2.md")]
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
