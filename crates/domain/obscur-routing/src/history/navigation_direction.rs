//! How a navigation moves through the history stack (spec §6.13).

#[doc = include_str!("../../docs/api/history/navigation_direction.doc/NavigationDirection.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NavigationDirection {
    /// Push a new entry, truncating any forward history (standard browser
    /// semantics).
    Push,
    /// Replace the current entry in place, leaving the rest of the stack.
    Replace,
    /// Move the cursor one entry backward toward the start of history.
    Back,
    /// Move the cursor one entry forward toward the end of history.
    Forward,
}

impl NavigationDirection {
    /// Returns whether this direction moves the cursor backward
    /// ([`Back`](NavigationDirection::Back)).
    #[must_use]
    pub const fn is_backward(self) -> bool {
        matches!(self, Self::Back)
    }

    /// Returns whether this direction moves the cursor forward
    /// ([`Forward`](NavigationDirection::Forward)).
    #[must_use]
    pub const fn is_forward(self) -> bool {
        matches!(self, Self::Forward)
    }

    #[doc = include_str!("../../docs/api/history/navigation_direction.doc/mutates_stack.md")]
    #[must_use]
    pub const fn mutates_stack(self) -> bool {
        matches!(self, Self::Push | Self::Replace)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backward_predicate() {
        assert!(NavigationDirection::Back.is_backward());
        assert!(!NavigationDirection::Forward.is_backward());
        assert!(!NavigationDirection::Push.is_backward());
        assert!(!NavigationDirection::Replace.is_backward());
    }

    #[test]
    fn forward_predicate() {
        assert!(NavigationDirection::Forward.is_forward());
        assert!(!NavigationDirection::Back.is_forward());
        assert!(!NavigationDirection::Push.is_forward());
        assert!(!NavigationDirection::Replace.is_forward());
    }

    #[test]
    fn mutates_stack_predicate() {
        assert!(NavigationDirection::Push.mutates_stack());
        assert!(NavigationDirection::Replace.mutates_stack());
        assert!(!NavigationDirection::Back.mutates_stack());
        assert!(!NavigationDirection::Forward.mutates_stack());
    }
}
