#![doc = include_str!("navigation_direction.doc/_module.md")]

#[doc = include_str!("navigation_direction.doc/NavigationDirection.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NavigationDirection {
    #[doc = include_str!("navigation_direction.doc/Up.md")]
    Up,
    #[doc = include_str!("navigation_direction.doc/Down.md")]
    Down,
    #[doc = include_str!("navigation_direction.doc/Left.md")]
    Left,
    #[doc = include_str!("navigation_direction.doc/Right.md")]
    Right,
}

impl NavigationDirection {
    #[doc = include_str!("navigation_direction.doc/is_horizontal.md")]
    #[must_use]
    pub const fn is_horizontal(self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }

    #[doc = include_str!("navigation_direction.doc/is_vertical.md")]
    #[must_use]
    pub const fn is_vertical(self) -> bool {
        matches!(self, Self::Up | Self::Down)
    }

    #[doc = include_str!("navigation_direction.doc/opposite.md")]
    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_predicates() {
        assert!(NavigationDirection::Left.is_horizontal());
        assert!(NavigationDirection::Right.is_horizontal());
        assert!(!NavigationDirection::Up.is_horizontal());
        assert!(!NavigationDirection::Down.is_horizontal());

        assert!(NavigationDirection::Up.is_vertical());
        assert!(NavigationDirection::Down.is_vertical());
        assert!(!NavigationDirection::Left.is_vertical());
        assert!(!NavigationDirection::Right.is_vertical());
    }

    #[test]
    fn opposite_flips_each_direction() {
        assert_eq!(
            NavigationDirection::Up.opposite(),
            NavigationDirection::Down
        );
        assert_eq!(
            NavigationDirection::Down.opposite(),
            NavigationDirection::Up
        );
        assert_eq!(
            NavigationDirection::Left.opposite(),
            NavigationDirection::Right
        );
        assert_eq!(
            NavigationDirection::Right.opposite(),
            NavigationDirection::Left
        );
    }

    #[test]
    fn opposite_is_involutive() {
        for d in [
            NavigationDirection::Up,
            NavigationDirection::Down,
            NavigationDirection::Left,
            NavigationDirection::Right,
        ] {
            assert_eq!(d.opposite().opposite(), d);
        }
    }
}
