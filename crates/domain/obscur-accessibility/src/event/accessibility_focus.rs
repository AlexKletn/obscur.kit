//! An accessibility focus change (spec §7.1 focus).

use obscur_identity::NodeId;

/// An accessibility focus change (spec §7.1 focus).
///
/// Reports that accessibility focus moved from [`previous`](Self::previous) to
/// [`focused`](Self::focused). A `None` [`focused`](Self::focused) means focus
/// was cleared; a `None` [`previous`](Self::previous) means nothing was focused
/// before. Adapters reflect the new focus to the platform so assistive tech can
/// follow it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AccessibilityFocus {
    /// The newly focused node, or `None` if focus was cleared.
    focused: Option<NodeId>,
    /// The previously focused node, or `None` if nothing was focused.
    previous: Option<NodeId>,
}

impl AccessibilityFocus {
    /// Creates a focus change from `previous` to `focused`.
    #[must_use]
    pub const fn new(focused: Option<NodeId>, previous: Option<NodeId>) -> Self {
        Self { focused, previous }
    }

    /// Creates a focus change that moves focus to `node` from `previous`.
    #[must_use]
    pub const fn moved_to(node: NodeId, previous: Option<NodeId>) -> Self {
        Self {
            focused: Some(node),
            previous,
        }
    }

    /// Creates a focus change that clears focus, from `previous`.
    #[must_use]
    pub const fn cleared(previous: Option<NodeId>) -> Self {
        Self {
            focused: None,
            previous,
        }
    }

    /// Returns the newly focused node, or `None` if focus was cleared.
    #[must_use]
    pub const fn focused(&self) -> Option<NodeId> {
        self.focused
    }

    /// Returns the previously focused node, or `None` if nothing was focused.
    #[must_use]
    pub const fn previous(&self) -> Option<NodeId> {
        self.previous
    }

    /// Returns whether this change cleared focus (its
    /// [`focused`](Self::focused) is `None`).
    #[must_use]
    pub const fn is_cleared(&self) -> bool {
        self.focused.is_none()
    }

    /// Returns whether focus actually changed — the newly focused node differs
    /// from the previous one.
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
