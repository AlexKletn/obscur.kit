//! Drives a drag-and-drop session from pointer input: [`DragCoordinator`].

use std::collections::HashMap;

use obscur_data_transfer::drag::{DragEffect, DragPhase, DragSession, DragSource, DropTarget};
use obscur_geometry::Point;
use obscur_identity::{NodeId, VersionStamp};

use crate::transaction::{
    ProvisionalTransaction, TransactionCommit, TransactionValidator, UiOperation,
};

/// How a drag ended.
///
/// Returned by [`DragCoordinator::release`] so the caller can tell the two apart
/// without inspecting the session, and so the payload reaches the host on the one
/// path where it means something.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DragOutcome {
    /// The drop landed on `target`, and `commit` is the work it caused —
    /// already validated, ready for
    /// [`apply_transaction`](crate::runtime::UiRuntime::apply_transaction).
    Dropped {
        /// The node that accepted the drop.
        target: NodeId,
        /// The negotiated effect (`Copy` / `Move` / …).
        effect: DragEffect,
        /// The invalidation the drop caused, as one atomic transaction.
        commit: TransactionCommit,
    },
    /// The drag ended without a valid drop: released over nothing, over a target
    /// that refused the payload, or abandoned. **Nothing was applied.**
    Cancelled,
}

/// Drives a [`DragSession`] from pointer input and gates its effect behind a
/// [`ProvisionalTransaction`] (spec §3.2 provisional commit/cancel, §6.21 drag
/// and drop).
///
/// The [`DragSession`] state machine already knows the lifecycle; what it
/// deliberately leaves out — *"any provisional layout transaction that
/// materialises placeholders or insertion slots while a drag hovers is the
/// runtime's concern"* — is this type.
///
/// # What is provisional, and what is not
///
/// A drag's **live** state is not provisional: the pointer position and which
/// target is hovered change every frame and are simply read off the
/// [`session`](Self::session). Deferring those would mean a drag you cannot see.
///
/// What is provisional is the drop's **effect**. While the pointer is over an
/// accepting target, this coordinator stages the work that landing there would
/// cause — a [`Rerender`](UiOperation::Rerender) of that target, whose content
/// changes when it receives the payload. That staging is what
/// [`release`](Self::release) then either commits or throws away:
///
/// * a drop **commits** it — validated first, and applied as one transaction, so
///   the frame does exactly the work the drop caused and no more;
/// * a cancel **discards** it — and because
///   [`ProvisionalTransaction::cancel`] consumes the transaction, a cancelled
///   drag cannot later commit. The guarantee is in the type, not in a comment.
///
/// A drag that wanders across five targets and is then abandoned therefore
/// applies nothing at all: each hover restages, and the final discard is free
/// because nothing was ever applied.
///
/// # Not carried
///
/// The drag **preview** ([`DragPreview`](obscur_data_transfer::drag::DragPreview))
/// is not painted — nothing here reaches the scene, so a host that wants a
/// pointer-following preview must draw it. Autoscroll
/// ([`DragAutoscroll`](obscur_data_transfer::drag::DragAutoscroll)) is likewise
/// not driven. Both are scene/scroll concerns, and neither is what
/// commit/cancel is about.
#[derive(Debug, Default)]
pub struct DragCoordinator {
    /// The live session, or [`None`] when no drag is in flight.
    session: Option<DragSession>,
    /// The drop targets a host registered, by node.
    targets: HashMap<NodeId, DropTarget>,
    /// The work the current hover would cause, staged and unapplied. Rebuilt on
    /// every enter/leave, because only the target the drop *lands* on does any.
    provisional: ProvisionalTransaction,
}

impl DragCoordinator {
    /// Creates a coordinator with no session and no registered targets.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers `target` as a drop site, replacing any previous registration for
    /// its node.
    pub fn register_target(&mut self, target: DropTarget) {
        self.targets.insert(target.node(), target);
    }

    /// Removes the drop target registered for `node`, if any.
    pub fn unregister_target(&mut self, node: NodeId) {
        self.targets.remove(&node);
    }

    /// Whether `node` is a registered drop target.
    #[must_use]
    pub fn is_target(&self, node: NodeId) -> bool {
        self.targets.contains_key(&node)
    }

    /// Begins a drag from `source`.
    ///
    /// Returns `false` — starting nothing — when a drag is already in flight: two
    /// concurrent sessions would each stage their own provisional work and there
    /// would be no answer to which one a drop commits.
    pub fn begin(&mut self, source: DragSource) -> bool {
        if self.is_active() {
            return false;
        }
        self.session = Some(DragSession::start(source));
        self.provisional = ProvisionalTransaction::new();
        true
    }

    /// Whether a drag is in flight.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.session
            .as_ref()
            .is_some_and(|session| session.phase().is_active())
    }

    /// The live session, for a host that wants to draw the drag.
    #[must_use]
    pub fn session(&self) -> Option<&DragSession> {
        self.session.as_ref()
    }

    /// Tracks the pointer to `position`, with `under` the node it is currently
    /// over (as hit-tested by the caller).
    ///
    /// Drives the session's `enter` / `leave` and restages the provisional work
    /// for the new hover. Returns whether anything observable changed, so a caller
    /// can request a frame only when the drag actually moved between targets.
    ///
    /// A no-op when no drag is in flight.
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

    /// Ends the drag as the pointer is released.
    ///
    /// A drop over an accepting target **commits** the staged work: it is
    /// validated and returned as a [`TransactionCommit`] for the caller to apply.
    /// Anything else **cancels**: the staged work is discarded and nothing is
    /// applied.
    ///
    /// A commit whose validation fails also cancels rather than half-applying —
    /// which is the atomicity [`ProvisionalTransaction::commit`] exists to give.
    ///
    /// Returns [`None`] when no drag was in flight.
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

    /// Abandons the drag (Esc, focus loss, a cancelled pointer), discarding the
    /// staged work.
    ///
    /// Returns whether a drag was in flight to abandon.
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

    /// The work a drop on `node` would cause: that node re-renders, because it is
    /// about to receive the payload and its content changes.
    ///
    /// Deliberately narrow. The drag's *source* is not staged: whether the item
    /// leaves its origin is the application's decision (a `Copy` leaves it in
    /// place), and the runtime does not get to assume it.
    fn stage_for(node: NodeId) -> ProvisionalTransaction {
        let mut provisional = ProvisionalTransaction::new();
        provisional.stage(UiOperation::Rerender { node });
        provisional
    }

    /// The phase of the live session, for tests and diagnostics.
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
        assert!(coordinator.release(&TransactionValidator::new(), version()).is_none());
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
        assert!(coordinator.release(&TransactionValidator::new(), version()).is_none());
    }
}
