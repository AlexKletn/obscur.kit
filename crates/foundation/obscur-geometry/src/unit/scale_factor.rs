//! Fractional display scale factor.

/// A fractional DPI scale factor mapping logical to physical pixels.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct ScaleFactor(f32);

impl ScaleFactor {
    /// The identity scale factor (1.0).
    pub const IDENTITY: Self = Self(1.0);

    /// Creates a scale factor, clamping non-positive inputs to the identity.
    #[must_use]
    pub fn new(value: f32) -> Self {
        if value > 0.0 {
            Self(value)
        } else {
            Self::IDENTITY
        }
    }

    /// Returns the raw scale multiplier.
    #[must_use]
    pub fn get(self) -> f32 {
        self.0
    }
}

impl Default for ScaleFactor {
    fn default() -> Self {
        Self::IDENTITY
    }
}
