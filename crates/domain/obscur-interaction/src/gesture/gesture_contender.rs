#![doc = include_str!("gesture_contender.doc/_module.md")]

use obscur_identity::NodeId;

use super::RecognizerId;

#[doc = include_str!("gesture_contender.doc/GestureContender.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GestureContender {
    recognizer: RecognizerId,
    node: NodeId,
    priority: i32,
}

impl GestureContender {
    #[doc = include_str!("gesture_contender.doc/new.md")]
    #[must_use]
    pub const fn new(recognizer: RecognizerId, node: NodeId) -> Self {
        Self {
            recognizer,
            node,
            priority: 0,
        }
    }

    #[doc = include_str!("gesture_contender.doc/with_priority.md")]
    #[must_use]
    pub const fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    #[doc = include_str!("gesture_contender.doc/recognizer.md")]
    #[must_use]
    pub const fn recognizer(self) -> RecognizerId {
        self.recognizer
    }

    #[doc = include_str!("gesture_contender.doc/node.md")]
    #[must_use]
    pub const fn node(self) -> NodeId {
        self.node
    }

    #[doc = include_str!("gesture_contender.doc/priority.md")]
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
