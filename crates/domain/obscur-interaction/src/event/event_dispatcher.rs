#![doc = include_str!("event_dispatcher.doc/_module.md")]

use obscur_identity::NodeId;

use super::{DispatchResult, EventPhase, UiEvent};

#[doc = include_str!("event_dispatcher.doc/DispatchStep.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DispatchStep {
    #[doc = include_str!("event_dispatcher.doc/node.md")]
    pub node: NodeId,
    #[doc = include_str!("event_dispatcher.doc/phase.md")]
    pub phase: EventPhase,
}

impl DispatchStep {
    #[doc = include_str!("event_dispatcher.doc/new.md")]
    #[must_use]
    pub const fn new(node: NodeId, phase: EventPhase) -> Self {
        Self { node, phase }
    }
}

#[doc = include_str!("event_dispatcher.doc/EventDispatcher.md")]
#[derive(Debug, Clone, Copy, Default)]
pub struct EventDispatcher;

impl EventDispatcher {
    #[doc = include_str!("event_dispatcher.doc/new_2.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[doc = include_str!("event_dispatcher.doc/path.md")]
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

    #[doc = include_str!("event_dispatcher.doc/dispatch.md")]
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
