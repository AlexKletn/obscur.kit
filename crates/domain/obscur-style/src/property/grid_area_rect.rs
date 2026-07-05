//! `GridAreaRect`: the line-number rectangle a named grid area occupies.

/// The rectangle one named `grid-template-areas` area spans, in **1-based grid
/// line numbers** (spec §4.2, the named-areas subset).
///
/// A named area is a filled rectangle of cells; this records the four grid lines
/// that bound it. Line numbers are 1-based and the *end* edges are the line
/// *after* the last occupied track, exactly as CSS `grid-row` / `grid-column`
/// line placement reads: an area filling row 1, columns 1-2 (the `"h h"` first
/// row of a two-column grid) is `row_start = 1`, `row_end = 2`,
/// `column_start = 1`, `column_end = 3` — i.e. it can be assigned straight onto a
/// child's `grid-row: 1 / 2` and `grid-column: 1 / 3`.
///
/// # Determinism
///
/// `GridAreaRect` is a plain `Copy` value with no interior mutability, so an
/// area's placement resolves deterministically from the template alone.
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
