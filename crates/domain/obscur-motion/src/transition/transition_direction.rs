#![doc = include_str!("transition_direction.doc/_module.md")]

#[doc = include_str!("transition_direction.doc/TransitionDirection.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TransitionDirection {
    #[doc = include_str!("transition_direction.doc/Forward.md")]
    #[default]
    Forward,
    #[doc = include_str!("transition_direction.doc/Reverse.md")]
    Reverse,
}

impl TransitionDirection {
    #[doc = include_str!("transition_direction.doc/is_forward.md")]
    #[must_use]
    pub const fn is_forward(self) -> bool {
        matches!(self, Self::Forward)
    }

    #[doc = include_str!("transition_direction.doc/is_reverse.md")]
    #[must_use]
    pub const fn is_reverse(self) -> bool {
        matches!(self, Self::Reverse)
    }

    #[doc = include_str!("transition_direction.doc/flipped.md")]
    #[must_use]
    pub const fn flipped(self) -> Self {
        match self {
            Self::Forward => Self::Reverse,
            Self::Reverse => Self::Forward,
        }
    }

    #[doc = include_str!("transition_direction.doc/orient.md")]
    #[must_use]
    pub fn orient(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Forward => t,
            Self::Reverse => 1.0 - t,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_forward() {
        assert_eq!(TransitionDirection::default(), TransitionDirection::Forward);
    }

    #[test]
    fn predicates_are_exclusive() {
        assert!(TransitionDirection::Forward.is_forward());
        assert!(!TransitionDirection::Forward.is_reverse());
        assert!(TransitionDirection::Reverse.is_reverse());
        assert!(!TransitionDirection::Reverse.is_forward());
    }

    #[test]
    fn flipped_swaps_direction() {
        assert_eq!(
            TransitionDirection::Forward.flipped(),
            TransitionDirection::Reverse
        );
        assert_eq!(
            TransitionDirection::Reverse.flipped(),
            TransitionDirection::Forward
        );
    }

    #[test]
    fn orient_forward_is_identity() {
        assert_eq!(TransitionDirection::Forward.orient(0.0), 0.0);
        assert_eq!(TransitionDirection::Forward.orient(0.25), 0.25);
        assert_eq!(TransitionDirection::Forward.orient(1.0), 1.0);
    }

    #[test]
    fn orient_reverse_mirrors_progress() {
        assert_eq!(TransitionDirection::Reverse.orient(0.0), 1.0);
        assert_eq!(TransitionDirection::Reverse.orient(0.25), 0.75);
        assert_eq!(TransitionDirection::Reverse.orient(1.0), 0.0);
    }

    #[test]
    fn orient_clamps_out_of_range_input() {
        assert_eq!(TransitionDirection::Forward.orient(-0.5), 0.0);
        assert_eq!(TransitionDirection::Forward.orient(2.0), 1.0);
        assert_eq!(TransitionDirection::Reverse.orient(-0.5), 1.0);
        assert_eq!(TransitionDirection::Reverse.orient(2.0), 0.0);
    }
}
