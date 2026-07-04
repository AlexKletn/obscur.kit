//! The [`NavigationDirection`] for directional (spatial) focus movement
//! (spec §4.7).

/// A cardinal direction for spatial focus navigation (spec §4.7).
///
/// Spatial navigation moves focus geometrically — pressing an arrow key moves
/// focus to the nearest element in that direction rather than to the next
/// element in tab order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NavigationDirection {
    /// Toward smaller `y` (upward).
    Up,
    /// Toward larger `y` (downward).
    Down,
    /// Toward smaller `x` (leftward).
    Left,
    /// Toward larger `x` (rightward).
    Right,
}

impl NavigationDirection {
    /// Returns whether this direction moves along the horizontal axis
    /// ([`Left`](Self::Left) or [`Right`](Self::Right)).
    #[must_use]
    pub const fn is_horizontal(self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }

    /// Returns whether this direction moves along the vertical axis
    /// ([`Up`](Self::Up) or [`Down`](Self::Down)).
    #[must_use]
    pub const fn is_vertical(self) -> bool {
        matches!(self, Self::Up | Self::Down)
    }

    /// Returns the direction pointing the opposite way.
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
