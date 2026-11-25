//! `GridAreaRect`: the line-number rectangle a named grid area occupies.

#[doc = include_str!("../../docs/api/property/grid_area_rect.doc/GridAreaRect.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridAreaRect {
    /// The 1-based grid line the area's first row starts on.
    pub row_start: u32,
    /// The 1-based grid line the area's last row ends on (last row + 1).
    pub row_end: u32,
    /// The 1-based grid line the area's first column starts on.
    pub column_start: u32,
    /// The 1-based grid line the area's last column ends on (last column + 1).
    pub column_end: u32,
}

impl GridAreaRect {
    /// Builds a rectangle from its four bounding grid lines (1-based).
    #[must_use]
    pub const fn new(row_start: u32, row_end: u32, column_start: u32, column_end: u32) -> Self {
        Self {
            row_start,
            row_end,
            column_start,
            column_end,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GridAreaRect;

    #[test]
    fn carries_its_four_lines() {
        let rect = GridAreaRect::new(1, 2, 1, 3);
        assert_eq!(rect.row_start, 1);
        assert_eq!(rect.row_end, 2);
        assert_eq!(rect.column_start, 1);
        assert_eq!(rect.column_end, 3);
    }
}
