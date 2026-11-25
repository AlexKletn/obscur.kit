#[doc = include_str!("../../docs/api/property/linear_gradient.doc/Corner.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Corner {
    /// `to top left`.
    TopLeft,
    /// `to top right`.
    TopRight,
    /// `to bottom right`.
    BottomRight,
    /// `to bottom left`.
    BottomLeft,
}
