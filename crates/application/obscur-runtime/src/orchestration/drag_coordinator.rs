#![doc = include_str!("drag_coordinator.doc/_module.md")]

use obscur_data_transfer::drag::{DragEffect, DragPhase, DragSession, DragSource, DropTarget};
use obscur_geometry::Point;
use obscur_identity::{FastMap, NodeId, VersionStamp};

use crate::transaction::{
    ProvisionalTransaction, TransactionCommit, TransactionValidator, UiOperation,
};

#[doc = include_str!("drag_coordinator.doc/DragOutcome.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DragOutcome {
    #[doc = include_str!("drag_coordinator.doc/Dropped.md")]
    Dropped {
        #[doc = include_str!("drag_coordinator.doc/target.md")]
        target: NodeId,
        #[doc = include_str!("drag_coordinator.doc/effect.md")]
        effect: DragEffect,
        #[doc = include_str!("drag_coordinator.doc/commit.md")]
        commit: TransactionCommit,
    },
    #[doc = include_str!("drag_coordinator.doc/Cancelled.md")]
    Cancelled,
}

#[doc = include_str!("drag_coordinator.doc/DragCoordinator.md")]
#[derive(Debug, Default)]
pub struct DragCoordinator {
    #[doc = include_str!("drag_coordinator.doc/session.md")]
    session: Option<DragSession>,
    #[doc = include_str!("drag_coordinator.doc/targets.md")]
    targets: FastMap<NodeId, DropTarget>,
    #[doc = include_str!("drag_coordinator.doc/provisional.md")]
    provisional: ProvisionalTransaction,
}

impl DragCoordinator {
    #[doc = include_str!("drag_coordinator.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("drag_coordinator.doc/register_target.md")]
    pub fn register_target(&mut self, target: DropTarget) {
        self.targets.insert(target.node(), target);
    }

    #[doc = include_str!("drag_coordinator.doc/unregister_target.md")]
    pub fn unregister_target(&mut self, node: NodeId) {
        self.targets.remove(&node);
    }

    #[doc = include_str!("drag_coordinator.doc/is_target.md")]
    #[must_use]
    pub fn is_target(&self, node: NodeId) -> bool {
        self.targets.contains_key(&node)
    }

    #[doc = include_str!("drag_coordinator.doc/begin.md")]
    pub fn begin(&mut self, source: DragSource) -> bool {
        if self.is_active() {
            return false;
        }
        self.session = Some(DragSession::start(source));
        self.provisional = ProvisionalTransaction::new();
        true
    }

    #[doc = include_str!("drag_coordinator.doc/is_active.md")]
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.session
            .as_ref()
            .is_some_and(|session| session.phase().is_active())
    }

    #[doc = include_str!("drag_coordinator.doc/session_2.md")]
    #[must_use]
    pub fn session(&self) -> Option<&DragSession> {
        self.session.as_ref()
    }

    #[doc = include_str!("drag_coordinator.doc/pointer_moved.md")]
    pub fn pointer_moved(&mut self, position: Point, under: Option<NodeId>) -> bool {
        let Some(session) = self.session.as_mut() else {
            return false;
        };
        if session.phase().is_terminal() {
            return false;
        }
        session.move_to(position);

        // The target under the pointer, but only if it will actually take this
        // payload: a target that refuses the offer is not a drop site, and
        // entering it would stage work a drop could never do.
        let accepting = under
            .and_then(|node| self.targets.get(&node))
            .filter(|target| target.accepts(session.source().payload().offer()));

        let was_over = session.over();
        match accepting {
            Some(target) => {
                // `enter` re-negotiates the effect, so it is called even when the
                // target is unchanged only if the target IS changed — otherwise a
                // move within one target would churn the session every pixel.
                if was_over != Some(target.node()) {
                    session.enter(target, DragEffect::Copy);
                    self.provisional = Self::stage_for(target.node());
                    return true;
                }
            }
            None => {
                if was_over.is_some() {
                    session.leave();
                    // Over nothing: a drop here would do no work, so nothing is
                    // staged.
                    self.provisional = ProvisionalTransaction::new();
                    return true;
                }
            }
        }
        false
    }

    #[doc = include_str!("drag_coordinator.doc/release.md")]
    pub fn release(
        &mut self,
        validator: &TransactionValidator,
        version: VersionStamp,
    ) -> Option<DragOutcome> {
        let session = self.session.as_mut()?;
        if session.phase().is_terminal() {
            return None;
        }
        let target = session.over();
        let effect = session.effect();
        // `drop` reports whether it landed anywhere valid; it refuses over nothing.
        let landed = session.drop();
        let staged = std::mem::take(&mut self.provisional);

        let outcome = if let (true, Some(node)) = (landed, target) {
            match staged.commit(validator, version) {
                Ok(commit) => DragOutcome::Dropped {
                    target: node,
                    effect,
                    commit,
                },
                // Validation rejected the batch: apply none of it. The drop is
                // reported as a cancel because, as far as any domain is
                // concerned, that is what happened.
                Err(_) => DragOutcome::Cancelled,
            }
        } else {
            staged.cancel();
            DragOutcome::Cancelled
        };
        self.session = None;
        Some(outcome)
    }

    #[doc = include_str!("drag_coordinator.doc/cancel.md")]
    pub fn cancel(&mut self) -> bool {
        let Some(session) = self.session.as_mut() else {
            return false;
        };
        session.cancel();
        // Consuming the staged transaction is what makes this final: it cannot be
        // committed afterwards, because it no longer exists.
        std::mem::take(&mut self.provisional).cancel();
        self.session = None;
        true
    }

    #[doc = include_str!("drag_coordinator.doc/stage_for.md")]
    fn stage_for(node: NodeId) -> ProvisionalTransaction {
        let mut provisional = ProvisionalTransaction::new();
        provisional.stage(UiOperation::Rerender { node });
        provisional
    }

    #[doc = include_str!("drag_coordinator.doc/phase.md")]
    #[must_use]
    pub fn phase(&self) -> Option<DragPhase> {
        self.session.as_ref().map(DragSession::phase)
    }
}

#[cfg(test)]
mod tests {
    use super::{DragCoordinator, DragOutcome};
    use obscur_data_transfer::drag::{DragSource, DropTarget};
    use obscur_data_transfer::offer::{DataOffer, MimeType};
    use obscur_data_transfer::transfer::TransferPayload;
    use obscur_geometry::Point;
    use obscur_identity::{NodeId, VersionStamp};

    use crate::transaction::{TransactionValidator, UiOperation};

    fn node(raw: u64) -> NodeId {
        NodeId::from_raw(raw)
    }

    /// A drag carrying `text/plain`.
    fn text_source() -> DragSource {
        DragSource::new(node(1), TransferPayload::new(DataOffer::text("hello")))
    }

    fn version() -> VersionStamp {
        VersionStamp::from_raw(1)
    }

    fn coordinator_over_target() -> DragCoordinator {
        let mut coordinator = DragCoordinator::new();
        coordinator.register_target(DropTarget::new(node(2)));
        assert!(coordinator.begin(text_source()));
        coordinator
    }

    #[test]
    fn a_drop_on_a_target_commits_exactly_that_targets_work() {
        let mut coordinator = coordinator_over_target();
        assert!(
            coordinator.pointer_moved(Point::new(10.0, 10.0), Some(node(2))),
            "entering a target is an observable change"
        );

        let outcome = coordinator
            .release(&TransactionValidator::new(), version())
            .expect("a drag was in flight");
        let DragOutcome::Dropped { target, commit, .. } = outcome else {
            panic!("expected a drop, got {outcome:?}");
        };
        assert_eq!(target, node(2));
        assert_eq!(
            commit.operations(),
            &[UiOperation::Rerender { node: node(2) }],
            "the drop's work is the target's re-render, and nothing else"
        );
        assert!(!coordinator.is_active(), "the session is over");
    }

    #[test]
    fn a_cancelled_drag_applies_nothing_however_far_it_wandered() {
        // THE commit/cancel guarantee: staging never touched a domain, so
        // abandoning is free.
        let mut coordinator = DragCoordinator::new();
        coordinator.register_target(DropTarget::new(node(2)));
        coordinator.register_target(DropTarget::new(node(3)));
        assert!(coordinator.begin(text_source()));

        coordinator.pointer_moved(Point::new(1.0, 1.0), Some(node(2)));
        coordinator.pointer_moved(Point::new(2.0, 2.0), Some(node(3)));
        coordinator.pointer_moved(Point::new(3.0, 3.0), Some(node(2)));

        assert!(coordinator.cancel(), "a drag was in flight");
        assert!(!coordinator.is_active());
        // Nothing to apply: `cancel` consumed the staged transaction, so there is
        // no commit for a caller to accidentally use.
        assert!(
            coordinator
                .release(&TransactionValidator::new(), version())
                .is_none()
        );
    }

    #[test]
    fn releasing_over_nothing_cancels() {
        let mut coordinator = coordinator_over_target();
        coordinator.pointer_moved(Point::new(1.0, 1.0), Some(node(2)));
        // Wander off the target before letting go.
        coordinator.pointer_moved(Point::new(99.0, 99.0), None);

        let outcome = coordinator
            .release(&TransactionValidator::new(), version())
            .expect("a drag was in flight");
        assert_eq!(outcome, DragOutcome::Cancelled);
    }

    #[test]
    fn a_target_that_refuses_the_payload_is_not_a_drop_site() {
        // The offer is `text/plain`; this target takes only images. Entering it
        // must stage nothing, because a drop there could never do anything.
        let mut coordinator = DragCoordinator::new();
        coordinator.register_target(DropTarget::new(node(2)).accepting(MimeType::new("image/png")));
        assert!(coordinator.begin(text_source()));

        assert!(
            !coordinator.pointer_moved(Point::new(1.0, 1.0), Some(node(2))),
            "a refusing target is not entered"
        );
        let outcome = coordinator
            .release(&TransactionValidator::new(), version())
            .expect("a drag was in flight");
        assert_eq!(outcome, DragOutcome::Cancelled, "no valid drop happened");
    }

    #[test]
    fn moving_within_one_target_does_not_churn() {
        // The frame-request guard: only crossing a boundary is a change.
        let mut coordinator = coordinator_over_target();
        assert!(coordinator.pointer_moved(Point::new(1.0, 1.0), Some(node(2))));
        assert!(
            !coordinator.pointer_moved(Point::new(2.0, 2.0), Some(node(2))),
            "still over the same target: nothing observable changed"
        );
        // …but the position did track.
        assert_eq!(
            coordinator.session().expect("live").position(),
            Point::new(2.0, 2.0)
        );
    }

    #[test]
    fn leaving_a_target_unstages_its_work() {
        let mut coordinator = coordinator_over_target();
        coordinator.pointer_moved(Point::new(1.0, 1.0), Some(node(2)));
        assert!(
            coordinator.pointer_moved(Point::new(9.0, 9.0), None),
            "leaving is an observable change"
        );
        // Re-enter and drop: only the final target's work is committed, not both.
        coordinator.pointer_moved(Point::new(1.0, 1.0), Some(node(2)));
        let outcome = coordinator
            .release(&TransactionValidator::new(), version())
            .expect("live");
        let DragOutcome::Dropped { commit, .. } = outcome else {
            panic!("expected a drop");
        };
        assert_eq!(
            commit.operations().len(),
            1,
            "one drop, one target, one re-render — hovers do not accumulate"
        );
    }

    #[test]
    fn a_second_drag_cannot_start_while_one_is_in_flight() {
        let mut coordinator = coordinator_over_target();
        assert!(
            !coordinator.begin(text_source()),
            "two sessions would each stage work with no answer to which one commits"
        );
    }

    #[test]
    fn there_is_nothing_to_release_or_cancel_without_a_drag() {
        let mut coordinator = DragCoordinator::new();
        assert!(!coordinator.is_active());
        assert!(!coordinator.cancel());
        assert!(
            coordinator
                .release(&TransactionValidator::new(), version())
                .is_none()
        );
    }
}
