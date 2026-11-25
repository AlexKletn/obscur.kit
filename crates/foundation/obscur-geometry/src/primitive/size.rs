//! A 2D size in logical pixels.

/// A width/height pair in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Size {
    /// Width in logical pixels.
    pub width: f32,
    /// Height in logical pixels.
    pub height: f32,
}

impl Size {
    /// The zero size.
    pub const ZERO: Self = Self {
        width: 0.0,
        height: 0.0,
    };

    /// Constructs a size.
    #[must_use]
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}
