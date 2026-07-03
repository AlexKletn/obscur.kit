//! How a navigation moves through the history stack (spec §6.13).

/// The way a navigation moves through the history stack (spec §6.13
/// back/forward).
///
/// Every navigation is one of four moves. Two of them *mutate* the stack by
/// adding or replacing an entry ([`Push`](NavigationDirection::Push),
/// [`Replace`](NavigationDirection::Replace)), and two of them merely move the
/// history cursor over entries that already exist
/// ([`Back`](NavigationDirection::Back),
/// [`Forward`](NavigationDirection::Forward)). The
/// [`mutates_stack`](NavigationDirection::mutates_stack),
/// [`is_backward`](NavigationDirection::is_backward), and
/// [`is_forward`](NavigationDirection::is_forward) predicates classify a
/// direction without the caller matching on it.
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

    /// Returns whether this direction adds or replaces a history entry
    /// ([`Push`](NavigationDirection::Push) or
    /// [`Replace`](NavigationDirection::Replace)).
    ///
    /// [`Back`](NavigationDirection::Back) and
    /// [`Forward`](NavigationDirection::Forward) only move the cursor over
    /// existing entries and therefore return `false`.
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
