#![doc = include_str!("navigation_direction.doc/_module.md")]

#[doc = include_str!("navigation_direction.doc/NavigationDirection.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NavigationDirection {
    #[doc = include_str!("navigation_direction.doc/Push.md")]
    Push,
    #[doc = include_str!("navigation_direction.doc/Replace.md")]
    Replace,
    #[doc = include_str!("navigation_direction.doc/Back.md")]
    Back,
    #[doc = include_str!("navigation_direction.doc/Forward.md")]
    Forward,
}

impl NavigationDirection {
    #[doc = include_str!("navigation_direction.doc/is_backward.md")]
    #[must_use]
    pub const fn is_backward(self) -> bool {
        matches!(self, Self::Back)
    }

    #[doc = include_str!("navigation_direction.doc/is_forward.md")]
    #[must_use]
    pub const fn is_forward(self) -> bool {
        matches!(self, Self::Forward)
    }

    #[doc = include_str!("navigation_direction.doc/mutates_stack.md")]
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
