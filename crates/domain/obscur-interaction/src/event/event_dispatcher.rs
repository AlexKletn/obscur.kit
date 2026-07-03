//! Computes the ordered capture → target → bubble propagation path (spec §4.7).

use obscur_identity::NodeId;

use super::{DispatchResult, EventPhase, UiEvent};

/// One node visited in one phase along an event's propagation path (spec §4.7).
///
/// A [`DispatchStep`] pairs the [`node`](Self::node) reached with the
/// [`phase`](Self::phase) it is reached in, so an ordered list of steps fully
/// describes how an event threads the logical tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DispatchStep {
    /// The node visited at this step.
    pub node: NodeId,
    /// The propagation phase this node is visited in.
    pub phase: EventPhase,
}

impl DispatchStep {
    /// Creates a step visiting `node` during `phase`.
    #[must_use]
    pub const fn new(node: NodeId, phase: EventPhase) -> Self {
        Self { node, phase }
    }
}

/// The stateless helper that maps a hit chain to a logical propagation path
/// (spec §4.7 hit testing → event routing).
///
/// Given the ancestor chain from the root down to a hit-test target,
/// [`EventDispatcher`] produces the ordered capture → target → bubble sequence an
/// event travels, and can [`dispatch`](Self::dispatch) an event along it. The
/// dispatcher carries no state and reads no clock: it is a pure, deterministic
/// function of its inputs. Running the handlers attached to each node is the
/// runtime's concern, not this crate's — here [`dispatch`](Self::dispatch) only
/// computes and validates the path and honors any pre-set propagation control.
#[derive(Debug, Clone, Copy, Default)]
pub struct EventDispatcher;

impl EventDispatcher {
    /// Creates a dispatcher.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Computes the ordered propagation path from a root → target ancestor chain.
    ///
    /// `ancestors_root_to_target` is the path from the ROOT down to and
    /// INCLUDING the target as its last element. The result is: a
    /// [`Capture`](EventPhase::Capture) step for each ancestor above the target
    /// (root → …), then a single [`Target`](EventPhase::Target) step for the
    /// target, then a [`Bubble`](EventPhase::Bubble) step for each of those
    /// ancestors in REVERSE (… → root).
    ///
    /// An empty input yields an empty path; a single element (the target alone)
    /// yields just `[Target]`.
    #[must_use]
    pub fn path(&self, ancestors_root_to_target: &[NodeId]) -> Vec<DispatchStep> {
        let mut steps = Vec::new();
        let Some((target, ancestors)) = ancestors_root_to_target.split_last() else {
            return steps;
        };
        for &node in ancestors {
            steps.push(DispatchStep::new(node, EventPhase::Capture));
        }
        steps.push(DispatchStep::new(*target, EventPhase::Target));
        for &node in ancestors.iter().rev() {
            steps.push(DispatchStep::new(node, EventPhase::Bubble));
        }
        steps
    }

    /// Walks [`path`](Self::path), advancing `event` through each step.
    ///
    /// This is a pure simulation of routing *without* handlers — dispatching
    /// handlers is the runtime's job. For each step, if the event's propagation
    /// is already stopped, the walk halts: `stopped_at` records the previously
    /// visited node, or, if the event was stopped before any step ran, the node
    /// of the halting step. Otherwise the event's [`phase`](UiEvent::phase) is
    /// advanced to the step's phase. Because there are no handlers, nothing is
    /// consumed, so [`handled`](DispatchResult::handled) stays `false` and the
    /// walk only stops when the caller pre-set propagation control.
    ///
    /// The returned [`DispatchResult`] carries the event's final
    /// default-prevented state.
    #[must_use]
    pub fn dispatch(
        &self,
        ancestors_root_to_target: &[NodeId],
        event: &mut UiEvent,
    ) -> DispatchResult {
        let mut result = DispatchResult::new();
        let mut previous: Option<NodeId> = None;
        for step in self.path(ancestors_root_to_target) {
            if event.propagation.is_stopped() {
                result.stopped_at = Some(previous.unwrap_or(step.node));
                break;
            }
            event.phase = step.phase;
            previous = Some(step.node);
        }
        result.default_prevented = event.is_default_prevented();
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::EventOrigin;

    fn node(raw: u64) -> NodeId {
        NodeId::from_raw(raw)
    }

    #[test]
    fn path_for_three_nodes() {
        let root = node(1);
        let mid = node(2);
        let target = node(3);
        let path = EventDispatcher::new().path(&[root, mid, target]);
        assert_eq!(
            path,
            vec![
                DispatchStep::new(root, EventPhase::Capture),
                DispatchStep::new(mid, EventPhase::Capture),
                DispatchStep::new(target, EventPhase::Target),
                DispatchStep::new(mid, EventPhase::Bubble),
                DispatchStep::new(root, EventPhase::Bubble),
            ]
        );
    }

    #[test]
    fn path_for_single_node_is_target_only() {
        let target = node(9);
        let path = EventDispatcher::new().path(&[target]);
        assert_eq!(path, vec![DispatchStep::new(target, EventPhase::Target)]);
    }

    #[test]
    fn path_for_empty_is_empty() {
        assert!(EventDispatcher::new().path(&[]).is_empty());
    }

    #[test]
    fn dispatch_advances_phase_and_completes() {
        let dispatcher = EventDispatcher::new();
        let mut event = UiEvent::new(EventOrigin::Pointer, node(3));
        let result = dispatcher.dispatch(&[node(1), node(2), node(3)], &mut event);
        assert!(!result.is_handled());
        assert_eq!(result.stopped_at(), None);
        assert!(!result.is_default_prevented());
        // Final step is Bubble(root).
        assert_eq!(event.phase(), EventPhase::Bubble);
    }

    #[test]
    fn dispatch_honors_pre_stopped_event() {
        let dispatcher = EventDispatcher::new();
        let root = node(1);
        let mut event = UiEvent::new(EventOrigin::Pointer, node(3));
        event.stop_propagation();
        let result = dispatcher.dispatch(&[root, node(2), node(3)], &mut event);
        assert_eq!(result.stopped_at(), Some(root));
        // Halted before advancing past the initial phase.
        assert_eq!(event.phase(), EventPhase::Target);
    }

    #[test]
    fn dispatch_reports_prevented_default() {
        let dispatcher = EventDispatcher::new();
        let mut event = UiEvent::new(EventOrigin::Keyboard, node(5));
        event.prevent_default();
        let result = dispatcher.dispatch(&[node(5)], &mut event);
        assert!(result.is_default_prevented());
    }
}
