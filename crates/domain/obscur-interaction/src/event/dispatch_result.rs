#![doc = include_str!("dispatch_result.doc/_module.md")]

use obscur_identity::NodeId;

#[doc = include_str!("dispatch_result.doc/DispatchResult.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DispatchResult {
    #[doc = include_str!("dispatch_result.doc/handled.md")]
    pub handled: bool,
    #[doc = include_str!("dispatch_result.doc/default_prevented.md")]
    pub default_prevented: bool,
    #[doc = include_str!("dispatch_result.doc/stopped_at.md")]
    pub stopped_at: Option<NodeId>,
}

impl DispatchResult {
    #[doc = include_str!("dispatch_result.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            handled: false,
            default_prevented: false,
            stopped_at: None,
        }
    }

    #[doc = include_str!("dispatch_result.doc/handled_2.md")]
    #[must_use]
    pub const fn handled(mut self) -> Self {
        self.handled = true;
        self
    }

    #[doc = include_str!("dispatch_result.doc/mark_handled.md")]
    pub const fn mark_handled(&mut self) {
        self.handled = true;
    }

    #[doc = include_str!("dispatch_result.doc/is_handled.md")]
    #[must_use]
    pub const fn is_handled(&self) -> bool {
        self.handled
    }

    #[doc = include_str!("dispatch_result.doc/is_default_prevented.md")]
    #[must_use]
    pub const fn is_default_prevented(&self) -> bool {
        self.default_prevented
    }

    #[doc = include_str!("dispatch_result.doc/stopped_at_2.md")]
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
