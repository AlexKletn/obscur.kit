/// How a CSS rectangle specifies its four coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipRectangleKind {
    /// Insets from the four reference box edges.
    Inset,
    /// Four edge coordinates measured from the top-left origin.
    Edges,
    /// Origin, followed by width and height.
    OriginAndSize,
}
