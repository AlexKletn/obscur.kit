/// Tiling mode on a single background axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackgroundRepeatAxis {
    /// Tile at the authored size.
    Repeat,
    /// Keep only the positioned tile.
    NoRepeat,
    /// Distribute the remaining space evenly between whole tiles.
    Space,
    /// Resize tiles so a whole number fits.
    Round,
}
