/// CSS box used to bound a background.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackgroundBox {
    /// Outer border edge.
    Border,
    /// Inner border edge.
    Padding,
    /// Inner padding edge.
    Content,
}
