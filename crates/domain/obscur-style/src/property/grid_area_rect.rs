#![doc = include_str!("grid_area_rect.doc/_module.md")]

#[doc = include_str!("grid_area_rect.doc/GridAreaRect.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridAreaRect {
    #[doc = include_str!("grid_area_rect.doc/row_start.md")]
    pub row_start: u32,
    #[doc = include_str!("grid_area_rect.doc/row_end.md")]
    pub row_end: u32,
    #[doc = include_str!("grid_area_rect.doc/column_start.md")]
    pub column_start: u32,
    #[doc = include_str!("grid_area_rect.doc/column_end.md")]
    pub column_end: u32,
}

impl GridAreaRect {
    #[doc = include_str!("grid_area_rect.doc/new.md")]
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
