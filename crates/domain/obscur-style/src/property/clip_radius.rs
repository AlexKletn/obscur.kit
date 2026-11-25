use super::LengthExpr;

/// A radius in a CSS circle or ellipse.
#[derive(Debug, Clone, PartialEq)]
pub enum ClipRadius {
    /// Distance to the nearer side.
    ClosestSide,
    /// Distance to the farther side.
    FarthestSide,
    /// Explicit radius relative to the reference box.
    Length(LengthExpr),
}
