#[doc = include_str!("../../docs/api/primitive/corner_radii.doc/CornerRadius.md")]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CornerRadius {
    /// The horizontal radius (along the top/bottom edges).
    pub x: f32,
    /// The vertical radius (along the left/right edges).
    pub y: f32,
}

impl CornerRadius {
    /// A circular corner: the same radius on both axes.
    #[must_use]
    pub const fn circular(radius: f32) -> Self {
        Self {
            x: radius,
            y: radius,
        }
    }

    /// An elliptical corner with a horizontal radius `x` and a vertical radius `y`.
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[doc = include_str!("../../docs/api/primitive/corner_radii.doc/is_rounded.md")]
    #[must_use]
    pub fn is_rounded(self) -> bool {
        self.x > 0.0 && self.y > 0.0
    }

    #[doc = include_str!("../../docs/api/primitive/corner_radii.doc/non_negative.md")]
    #[must_use]
    pub fn non_negative(self) -> Self {
        Self {
            x: self.x.max(0.0),
            y: self.y.max(0.0),
        }
    }

    /// Scales both radii by `f`.
    #[must_use]
    pub fn scaled(self, f: f32) -> Self {
        Self {
            x: self.x * f,
            y: self.y * f,
        }
    }

    #[doc = include_str!("../../docs/api/primitive/corner_radii.doc/circular_approximation.md")]
    #[must_use]
    pub fn circular_approximation(self) -> f32 {
        (self.x + self.y) / 2.0
    }
}
