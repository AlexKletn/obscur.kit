//! The outcome of dispatching an event through a propagation path (spec §4.7).

use obscur_identity::NodeId;

#[doc = include_str!("../../docs/api/event/dispatch_result.doc/DispatchResult.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DispatchResult {
    /// Whether any node along the path consumed the event.
    pub handled: bool,
    /// Whether the host's default action was suppressed.
    pub default_prevented: bool,
    /// The node at which propagation was stopped, if it was stopped at all.
    pub stopped_at: Option<NodeId>,
}

impl DispatchResult {
    /// Creates a result recording that nothing handled the event, no default was
    /// prevented, and propagation ran to completion.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            handled: false,
            default_prevented: false,
            stopped_at: None,
        }
    }

    /// Returns this result with [`handled`](Self::handled) set to `true`
    /// (fluent form of [`mark_handled`](Self::mark_handled)).
    #[must_use]
    pub const fn handled(mut self) -> Self {
        self.handled = true;
        self
    }

    /// Marks this result as handled in place.
    pub const fn mark_handled(&mut self) {
        self.handled = true;
    }

    /// Returns whether any node consumed the event.
    #[must_use]
    pub const fn is_handled(&self) -> bool {
        self.handled
    }

    /// Returns whether the host's default action was suppressed.
    #[must_use]
    pub const fn is_default_prevented(&self) -> bool {
        self.default_prevented
    }

    /// Returns the node at which propagation was stopped, if any.
    #[must_use]
    pub const fn stopped_at(&self) -> Option<NodeId> {
        self.stopped_at
    }
}

impl Default for DispatchResult {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty() {
        let r = DispatchResult::new();
        assert!(!r.is_handled());
        assert!(!r.is_default_prevented());
        assert_eq!(r.stopped_at(), None);
        assert_eq!(r, DispatchResult::default());
    }

    #[test]
    fn fluent_handled() {
        assert!(DispatchResult::new().handled().is_handled());
    }

    #[test]
    fn mark_handled_mutates() {
        let mut r = DispatchResult::new();
        r.mark_handled();
        assert!(r.is_handled());
    }

    #[test]
    fn accessors_reflect_fields() {
        let r = DispatchResult {
            handled: true,
            default_prevented: true,
            stopped_at: Some(NodeId::from_raw(7)),
        };
        assert!(r.is_handled());
        assert!(r.is_default_prevented());
        assert_eq!(r.stopped_at(), Some(NodeId::from_raw(7)));
    }
}
