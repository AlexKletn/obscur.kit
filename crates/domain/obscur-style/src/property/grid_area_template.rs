//! `GridAreaTemplate`: a parsed `grid-template-areas` grid of named cells.

use crate::property::GridAreaRect;

#[doc = include_str!("../../docs/api/property/grid_area_template.doc/GridAreaTemplate.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridAreaTemplate {
    /// The number of rows (strings) in the template.
    rows: u32,
    /// The number of columns (cells per row) in the template.
    columns: u32,
    /// The resolved named areas, sorted by name for determinism.
    areas: Box<[(Box<str>, GridAreaRect)]>,
}

impl GridAreaTemplate {
    #[doc = include_str!("../../docs/api/property/grid_area_template.doc/from_rows.md")]
    #[must_use]
    pub fn from_rows(rows: &[Vec<Option<Box<str>>>]) -> Option<Self> {
        let row_count = rows.len();
        if row_count == 0 {
            return None;
        }
        let column_count = rows[0].len();
        if column_count == 0 {
            return None;
        }
        if rows.iter().any(|row| row.len() != column_count) {
            return None;
        }

        // Accumulate each name's bounding box (min/max row and column, 0-based) and
        // its filled-cell count, so a non-rectangular area is caught below.
        let mut acc: Vec<(Box<str>, usize, usize, usize, usize, usize)> = Vec::new();
        for (r, row) in rows.iter().enumerate() {
            for (c, cell) in row.iter().enumerate() {
                let Some(name) = cell else { continue };
                match acc.iter_mut().find(|(n, ..)| n.as_ref() == name.as_ref()) {
                    Some((_, min_r, max_r, min_c, max_c, count)) => {
                        *min_r = (*min_r).min(r);
                        *max_r = (*max_r).max(r);
                        *min_c = (*min_c).min(c);
                        *max_c = (*max_c).max(c);
                        *count += 1;
                    }
                    None => acc.push((name.clone(), r, r, c, c, 1)),
                }
            }
        }

        let mut areas: Vec<(Box<str>, GridAreaRect)> = Vec::with_capacity(acc.len());
        for (name, min_r, max_r, min_c, max_c, count) in acc {
            // A filled rectangle's cell count equals its bounding-box area; any hole
            // or a second disjoint block of the same name makes it smaller.
            let width = max_c - min_c + 1;
            let height = max_r - min_r + 1;
            if count != width * height {
                return None;
            }
            // 0-based track indices → 1-based grid lines: start line = index + 1,
            // end line = last index + 2 (the line after the last occupied track).
            let rect = GridAreaRect::new(
                to_line(min_r),
                to_line(max_r) + 1,
                to_line(min_c),
                to_line(max_c) + 1,
            );
            areas.push((name, rect));
        }
        areas.sort_by(|a, b| a.0.cmp(&b.0));

        Some(Self {
            rows: to_line(row_count) - 1,
            columns: to_line(column_count) - 1,
            areas: areas.into_boxed_slice(),
        })
    }

    /// Returns the line-number [`GridAreaRect`] of the area named `name`, or
    /// [`None`] when no such area exists in the template.
    #[must_use]
    pub fn area(&self, name: &str) -> Option<GridAreaRect> {
        self.areas
            .iter()
            .find(|(n, _)| &**n == name)
            .map(|(_, rect)| *rect)
    }

    /// The number of rows in the template.
    #[must_use]
    pub const fn rows(&self) -> u32 {
        self.rows
    }

    /// The number of columns in the template.
    #[must_use]
    pub const fn columns(&self) -> u32 {
        self.columns
    }
}

/// Narrows a 0-based `usize` index into the 1-based `u32` line number `index + 1`,
/// saturating so a pathologically large grid cannot overflow.
fn to_line(index: usize) -> u32 {
    u32::try_from(index)
        .unwrap_or(u32::MAX - 1)
        .saturating_add(1)
}

#[cfg(test)]
mod tests {
    use super::GridAreaTemplate;

    /// Builds the authored rows from `&str` cells, mapping `"."` to a null cell.
    fn rows(spec: &[&[&str]]) -> Vec<Vec<Option<Box<str>>>> {
        spec.iter()
            .map(|row| {
                row.iter()
                    .map(|&cell| if cell == "." { None } else { Some(cell.into()) })
                    .collect()
            })
            .collect()
    }

    #[test]
    fn single_row_area_spans_its_columns() {
        // `"h h"`: area `h` fills row 1, columns 1-2.
        let template = GridAreaTemplate::from_rows(&rows(&[&["h", "h"]])).expect("template");
        let h = template.area("h").expect("h");
        // row 1..2, col 1..3 in 1-based line numbers.
        assert_eq!(h.row_start, 1);
        assert_eq!(h.row_end, 2);
        assert_eq!(h.column_start, 1);
        assert_eq!(h.column_end, 3);
        assert_eq!(template.rows(), 1);
        assert_eq!(template.columns(), 2);
    }

    #[test]
    fn multi_row_column_area_spans_both_rows() {
        // "h h" / "s m" / "f f": `s` is (row 2, col 1); `h`/`f` each span the row.
        let template = GridAreaTemplate::from_rows(&rows(&[&["h", "h"], &["s", "m"], &["f", "f"]]))
            .expect("template");
        let s = template.area("s").expect("s");
        assert_eq!(
            (s.row_start, s.row_end, s.column_start, s.column_end),
            (2, 3, 1, 2)
        );
        let f = template.area("f").expect("f");
        assert_eq!(
            (f.row_start, f.row_end, f.column_start, f.column_end),
            (3, 4, 1, 3)
        );
        assert_eq!(template.rows(), 3);
        assert_eq!(template.columns(), 2);
    }

    #[test]
    fn tall_area_spans_both_rows_of_one_column() {
        // "a b" / "a c": `a` occupies column 1 of both rows (a 1x2 vertical block).
        let template =
            GridAreaTemplate::from_rows(&rows(&[&["a", "b"], &["a", "c"]])).expect("template");
        let a = template.area("a").expect("a");
        assert_eq!(
            (a.row_start, a.row_end, a.column_start, a.column_end),
            (1, 3, 1, 2)
        );
    }

    #[test]
    fn null_cells_are_ignored() {
        let template =
            GridAreaTemplate::from_rows(&rows(&[&["a", "."], &[".", "b"]])).expect("template");
        assert!(template.area("a").is_some());
        assert!(template.area("b").is_some());
        assert!(template.area(".").is_none());
    }

    #[test]
    fn non_rectangular_area_is_rejected() {
        // `a` forms an L-shape (a hole in its bounding box) — not a filled rectangle.
        assert!(GridAreaTemplate::from_rows(&rows(&[&["a", "a"], &["a", "."]])).is_none());
    }

    #[test]
    fn disjoint_same_name_blocks_are_rejected() {
        // Two separate `a` cells at opposite corners: bounding box has holes.
        assert!(GridAreaTemplate::from_rows(&rows(&[&["a", "."], &[".", "a"]])).is_none());
    }

    #[test]
    fn ragged_rows_are_rejected() {
        assert!(GridAreaTemplate::from_rows(&rows(&[&["a", "b"], &["a"]])).is_none());
    }

    #[test]
    fn empty_template_is_rejected() {
        assert!(GridAreaTemplate::from_rows(&[]).is_none());
        assert!(GridAreaTemplate::from_rows(&rows(&[&[]])).is_none());
    }
}
