#![doc = include_str!("accessibility_focus.doc/_module.md")]

use obscur_identity::NodeId;

#[doc = include_str!("accessibility_focus.doc/AccessibilityFocus.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AccessibilityFocus {
    #[doc = include_str!("accessibility_focus.doc/focused.md")]
    focused: Option<NodeId>,
    #[doc = include_str!("accessibility_focus.doc/previous.md")]
    previous: Option<NodeId>,
}

impl AccessibilityFocus {
    #[doc = include_str!("accessibility_focus.doc/new.md")]
    #[must_use]
    pub const fn new(focused: Option<NodeId>, previous: Option<NodeId>) -> Self {
        Self { focused, previous }
    }

    #[doc = include_str!("accessibility_focus.doc/moved_to.md")]
    #[must_use]
    pub const fn moved_to(node: NodeId, previous: Option<NodeId>) -> Self {
        Self {
            focused: Some(node),
            previous,
        }
    }

    #[doc = include_str!("accessibility_focus.doc/cleared.md")]
    #[must_use]
    pub const fn cleared(previous: Option<NodeId>) -> Self {
        Self {
            focused: None,
            previous,
        }
    }

    #[doc = include_str!("accessibility_focus.doc/focused_2.md")]
    #[must_use]
    pub const fn focused(&self) -> Option<NodeId> {
        self.focused
    }

    #[doc = include_str!("accessibility_focus.doc/previous_2.md")]
    #[must_use]
    pub const fn previous(&self) -> Option<NodeId> {
        self.previous
    }

    #[doc = include_str!("accessibility_focus.doc/is_cleared.md")]
    #[must_use]
    pub const fn is_cleared(&self) -> bool {
        self.focused.is_none()
    }

    #[doc = include_str!("accessibility_focus.doc/changed.md")]
    #[must_use]
    pub fn changed(&self) -> bool {
        self.focused != self.previous
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: NodeId = NodeId::from_raw(1);
    const B: NodeId = NodeId::from_raw(2);

    #[test]
    fn moved_to_sets_focused_and_previous() {
        let f = AccessibilityFocus::moved_to(B, Some(A));
        assert_eq!(f.focused(), Some(B));
        assert_eq!(f.previous(), Some(A));
        assert!(!f.is_cleared());
        assert!(f.changed());
    }

    #[test]
    fn cleared_has_no_focus() {
        let f = AccessibilityFocus::cleared(Some(A));
        assert_eq!(f.focused(), None);
        assert_eq!(f.previous(), Some(A));
        assert!(f.is_cleared());
        assert!(f.changed());
    }

    #[test]
    fn changed_is_false_when_same() {
        let f = AccessibilityFocus::new(Some(A), Some(A));
        assert!(!f.changed());
        assert!(AccessibilityFocus::cleared(None).is_cleared());
        assert!(!AccessibilityFocus::cleared(None).changed());
    }
}
