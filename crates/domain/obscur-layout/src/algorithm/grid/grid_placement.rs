#![doc = include_str!("grid_placement.doc/_module.md")]

use super::GridLine;

#[doc = include_str!("grid_placement.doc/GridPlacement.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GridPlacement {
    #[doc = include_str!("grid_placement.doc/column_start.md")]
    pub column_start: GridLine,
    #[doc = include_str!("grid_placement.doc/column_end.md")]
    pub column_end: GridLine,
    #[doc = include_str!("grid_placement.doc/row_start.md")]
    pub row_start: GridLine,
    #[doc = include_str!("grid_placement.doc/row_end.md")]
    pub row_end: GridLine,
}

impl GridPlacement {
    #[doc = include_str!("grid_placement.doc/new.md")]
    #[must_use]
    pub fn new(
        column_start: GridLine,
        column_end: GridLine,
        row_start: GridLine,
        row_end: GridLine,
    ) -> Self {
        Self {
            column_start,
            column_end,
            row_start,
            row_end,
        }
    }

    #[doc = include_str!("grid_placement.doc/column.md")]
    #[must_use]
    pub fn column(start: GridLine, end: GridLine) -> Self {
        Self {
            column_start: start,
            column_end: end,
            ..Self::default()
        }
    }

    #[doc = include_str!("grid_placement.doc/row.md")]
    #[must_use]
    pub fn row(start: GridLine, end: GridLine) -> Self {
        Self {
            row_start: start,
            row_end: end,
            ..Self::default()
        }
    }

    #[doc = include_str!("grid_placement.doc/column_start_2.md")]
    #[must_use]
    pub const fn column_start(&self) -> GridLine {
        self.column_start
    }

    #[doc = include_str!("grid_placement.doc/column_end_2.md")]
    #[must_use]
    pub const fn column_end(&self) -> GridLine {
        self.column_end
    }

    #[doc = include_str!("grid_placement.doc/row_start_2.md")]
    #[must_use]
    pub const fn row_start(&self) -> GridLine {
        self.row_start
    }

    #[doc = include_str!("grid_placement.doc/row_end_2.md")]
    #[must_use]
    pub const fn row_end(&self) -> GridLine {
        self.row_end
    }

    #[doc = include_str!("grid_placement.doc/resolve_column.md")]
    #[must_use]
    pub fn resolve_column(&self, num_tracks: usize) -> (Option<usize>, usize) {
        resolve_axis(self.column_start, self.column_end, num_tracks)
    }

    #[doc = include_str!("grid_placement.doc/resolve_row.md")]
    #[must_use]
    pub fn resolve_row(&self, num_tracks: usize) -> (Option<usize>, usize) {
        resolve_axis(self.row_start, self.row_end, num_tracks)
    }
}

#[doc = include_str!("grid_placement.doc/resolve_axis.md")]
fn resolve_axis(start: GridLine, end: GridLine, num_tracks: usize) -> (Option<usize>, usize) {
    let tracks_i32 = i32::try_from(num_tracks).unwrap_or(i32::MAX);
    let has_negative =
        matches!(start, GridLine::Line(a) if a < 0) || matches!(end, GridLine::Line(b) if b < 0);
    // Normalise a negative (from-end) line to a positive 1-based line: `-1` is the
    // last line (`num_tracks + 1`), i.e. `num_tracks + 2 + a` for a negative `a`;
    // clamped to a valid `≥ 1` line. Positive/`Auto`/`Span` edges pass through.
    let norm = |line: GridLine| -> GridLine {
        match line {
            GridLine::Line(a) if a < 0 => GridLine::Line((tracks_i32 + 2 + a).max(1)),
            other => other,
        }
    };
    let start = norm(start);
    let end = norm(end);
    // A clamped-positive line index (`≥ 1`) as a 0-based track index.
    let track = |line: i32| usize::try_from(line.max(1)).unwrap_or(1) - 1;
    let (anchor, span) = match (start, end) {
        // Definite start line — end decides the span.
        (GridLine::Line(a), end) => {
            let span = match end {
                GridLine::Line(b) => usize::try_from((b.max(1) - a.max(1)).max(1)).unwrap_or(1),
                GridLine::Span(n) => usize::from(n.max(1)),
                GridLine::Auto => 1,
            };
            (Some(track(a)), span)
        }
        // Span start, definite end line — anchor the item to the end line.
        (GridLine::Span(n), GridLine::Line(b)) => {
            let span = usize::from(n.max(1));
            (Some(track(b).saturating_sub(span)), span)
        }
        // `auto / b`: item ends at line b, span 1 → starts at track b - 2.
        (GridLine::Auto, GridLine::Line(b)) => (Some(track(b).saturating_sub(1)), 1),
        // Fully auto-flowed start; only the span (if any) is definite.
        (GridLine::Span(n), _) | (GridLine::Auto, GridLine::Span(n)) => {
            (None, usize::from(n.max(1)))
        }
        (GridLine::Auto, GridLine::Auto) => (None, 1),
    };
    // A from-end (negative) edge resolves against the real track grid: keep the
    // block inside it. Positive-only placements are never touched here.
    match anchor {
        Some(a) if has_negative && num_tracks > 0 => {
            (Some(a.min(num_tracks.saturating_sub(span))), span)
        }
        other => (other, span),
    }
}

#[cfg(test)]
mod tests {
    use super::{GridLine, GridPlacement};

    #[test]
    fn default_is_all_auto() {
        let p = GridPlacement::default();
        assert_eq!(p.resolve_column(3), (None, 1));
        assert_eq!(p.resolve_row(3), (None, 1));
    }

    #[test]
    fn line_to_line_spans_the_difference() {
        // grid-column: 1 / 3 → start col 0, span 2.
        let p = GridPlacement::column(GridLine::Line(1), GridLine::Line(3));
        assert_eq!(p.resolve_column(3), (Some(0), 2));
    }

    #[test]
    fn line_to_span_uses_span() {
        let p = GridPlacement::column(GridLine::Line(2), GridLine::Span(2));
        assert_eq!(p.resolve_column(3), (Some(1), 2));
    }

    #[test]
    fn auto_to_span_is_auto_flowed() {
        // grid-column: span 2 → auto-placed, span 2.
        let p = GridPlacement::column(GridLine::Auto, GridLine::Span(2));
        assert_eq!(p.resolve_column(3), (None, 2));
    }

    #[test]
    fn line_to_auto_is_single_track() {
        let p = GridPlacement::row(GridLine::Line(2), GridLine::Auto);
        assert_eq!(p.resolve_row(3), (Some(1), 1));
    }

    #[test]
    fn span_to_line_anchors_to_end() {
        // grid-column: span 2 / 5 → ends at line 5 (track 4), span 2 → start 2.
        let p = GridPlacement::column(GridLine::Span(2), GridLine::Line(5));
        assert_eq!(p.resolve_column(4), (Some(2), 2));
    }

    #[test]
    fn reversed_lines_clamp_to_span_one() {
        let p = GridPlacement::column(GridLine::Line(3), GridLine::Line(1));
        assert_eq!(p.resolve_column(3), (Some(2), 1));
    }

    #[test]
    fn negative_end_line_spans_to_last_line() {
        // grid-column: 1 / -1 in a 3-track grid → whole row: start 0, span 3.
        let p = GridPlacement::column(GridLine::Line(1), GridLine::Line(-1));
        assert_eq!(p.resolve_column(3), (Some(0), 3));
    }

    #[test]
    fn negative_start_line_lands_on_last_track() {
        // grid-column: -1 → last column: start 2, span 1 in a 3-track grid.
        let p = GridPlacement::column(GridLine::Line(-1), GridLine::Auto);
        assert_eq!(p.resolve_column(3), (Some(2), 1));
    }

    #[test]
    fn negative_second_from_end_line() {
        // grid-row: -2 → line before last (1-based line 3) → track 2 in a 3-track grid.
        let p = GridPlacement::row(GridLine::Line(-2), GridLine::Auto);
        assert_eq!(p.resolve_row(3), (Some(2), 1));
    }

    #[test]
    fn negative_range_spans_interior() {
        // grid-column: 2 / -1 in a 4-track grid → start 1, up to last line 5 → span 3.
        let p = GridPlacement::column(GridLine::Line(2), GridLine::Line(-1));
        assert_eq!(p.resolve_column(4), (Some(1), 3));
    }

    #[test]
    fn positive_lines_unaffected_by_track_count() {
        // A positive line past the template still opens implicit tracks (no clamp).
        let p = GridPlacement::column(GridLine::Line(5), GridLine::Auto);
        assert_eq!(p.resolve_column(3), (Some(4), 1));
    }
}
