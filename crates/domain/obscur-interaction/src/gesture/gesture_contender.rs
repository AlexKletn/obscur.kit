//! The [`GestureContender`] — a recognizer entered into the arena (spec §6.16
//! mobile gestures).

use obscur_identity::NodeId;

use super::RecognizerId;

#[doc = include_str!("../../docs/api/gesture/gesture_contender.doc/GestureContender.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GestureContender {
    recognizer: RecognizerId,
    node: NodeId,
    priority: i32,
}

impl GestureContender {
    /// Creates a contender binding `recognizer` to `node` with priority `0`.
    #[must_use]
    pub const fn new(recognizer: RecognizerId, node: NodeId) -> Self {
        Self {
            recognizer,
            node,
            priority: 0,
        }
    }

    /// Returns this contender with its priority set to `priority` (higher wins).
    #[must_use]
    pub const fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Returns the recognizer this contender represents.
    #[must_use]
    pub const fn recognizer(self) -> RecognizerId {
        self.recognizer
    }

    /// Returns the node this contender is bound to.
    #[must_use]
    pub const fn node(self) -> NodeId {
        self.node
    }

    /// Returns the contender's priority (higher is preferred).
    #[must_use]
    pub const fn priority(self) -> i32 {
        self.priority
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn recognizer(raw: u64) -> RecognizerId {
        RecognizerId::from_raw(raw)
    }

    fn node(raw: u64) -> NodeId {
        NodeId::from_raw(raw)
    }

    #[test]
    fn new_defaults_priority_zero() {
        let c = GestureContender::new(recognizer(1), node(2));
        assert_eq!(c.recognizer(), recognizer(1));
        assert_eq!(c.node(), node(2));
        assert_eq!(c.priority(), 0);
    }

    #[test]
    fn with_priority_builder() {
        let c = GestureContender::new(recognizer(1), node(2)).with_priority(100);
        assert_eq!(c.priority(), 100);
        assert_eq!(c.recognizer(), recognizer(1));
        assert_eq!(c.node(), node(2));
    }
}
