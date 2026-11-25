//! `GridPlacement`: an item's line-based placement on both grid axes.

use super::GridLine;
use super::grid_line::MAX_IMPLICIT_LINE_DISTANCE;

#[doc = include_str!("../../../docs/api/algorithm/grid/grid_placement.doc/GridPlacement.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GridPlacement {
    /// The `grid-column` start edge.
    pub column_start: GridLine,
    /// The `grid-column` end edge.
    pub column_end: GridLine,
    /// The `grid-row` start edge.
    pub row_start: GridLine,
    /// The `grid-row` end edge.
    pub row_end: GridLine,
}

impl GridPlacement {
    /// Builds a placement from the four edges.
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

    /// A column-only placement (`grid-column: start / end`), rows auto.
    #[must_use]
    pub fn column(start: GridLine, end: GridLine) -> Self {
        Self {
            column_start: start,
            column_end: end,
            ..Self::default()
        }
    }

    /// A row-only placement (`grid-row: start / end`), columns auto.
    #[must_use]
    pub fn row(start: GridLine, end: GridLine) -> Self {
        Self {
            row_start: start,
            row_end: end,
            ..Self::default()
        }
    }

    /// The `grid-column` start edge.
    #[must_use]
    pub const fn column_start(&self) -> GridLine {
        self.column_start
    }

    /// The `grid-column` end edge.
    #[must_use]
    pub const fn column_end(&self) -> GridLine {
        self.column_end
    }

    /// The `grid-row` start edge.
    #[must_use]
    pub const fn row_start(&self) -> GridLine {
        self.row_start
    }

    /// The `grid-row` end edge.
    #[must_use]
    pub const fn row_end(&self) -> GridLine {
        self.row_end
    }

    /// Resolves the column axis to `(start, span)`. The start is relative to
    /// the first track after accounting for any implicit tracks required by this
    /// placement before the explicit grid. Negative indices count from the end
    /// of the `num_tracks` explicit tracks.
    #[must_use]
    pub fn resolve_column(&self, num_tracks: usize) -> (Option<usize>, usize) {
        resolve_axis(
            self.column_start,
            self.column_end,
            num_tracks,
            preceding_tracks(self.column_start, self.column_end, num_tracks),
        )
        .0
    }

    /// Resolves the row axis to `(start, span)`. The start is relative to
    /// the first track after accounting for any implicit tracks required by this
    /// placement before the explicit grid. Negative indices count from the end
    /// of the `num_tracks` explicit tracks.
    #[must_use]
    pub fn resolve_row(&self, num_tracks: usize) -> (Option<usize>, usize) {
        resolve_axis(
            self.row_start,
            self.row_end,
            num_tracks,
            preceding_tracks(self.row_start, self.row_end, num_tracks),
        )
        .0
    }
}

/// Resolves a line to an offset from the first explicit boundary.
fn line_position(line: GridLine, explicit: usize) -> Option<i64> {
    let end = i64::try_from(explicit).unwrap_or(i64::MAX);
    let offset = match line {
        GridLine::Line(index) if index < 0 => end.saturating_add(i64::from(index) + 1),
        GridLine::Line(index) => i64::from(index.max(1)) - 1,
        GridLine::OutsideExplicit(distance) if distance < 0 => i64::from(distance),
        GridLine::OutsideExplicit(distance) => end.saturating_add(i64::from(distance)),
        _ => return None,
    };
    Some(offset)
}

/// Number of implicit tracks needed before the explicit start.
pub(in crate::algorithm) fn preceding_tracks(
    start: GridLine,
    end: GridLine,
    explicit: usize,
) -> usize {
    let limit = i64::from(MAX_IMPLICIT_LINE_DISTANCE);
    let end_limit = i64::try_from(explicit)
        .unwrap_or(i64::MAX)
        .saturating_add(limit);
    let bounded =
        |line| line_position(line, explicit).map(|offset| offset.clamp(-limit, end_limit));
    let start_position = bounded(start);
    let end_position = bounded(end);
    let earliest = match (start_position, end_position, start) {
        (Some(a), Some(b), _) => a.min(b),
        (Some(a), None, _) => a,
        (None, Some(b), GridLine::Span(span)) => b - i64::from(span.max(1)),
        (None, Some(b), _) => b - 1,
        (None, None, _) => 0,
    };
    usize::try_from((-earliest).clamp(0, i64::from(MAX_IMPLICIT_LINE_DISTANCE))).unwrap_or(0)
}

#[doc = include_str!("../../../docs/api/algorithm/grid/grid_placement.doc/resolve_axis.md")]
pub(in crate::algorithm) fn resolve_axis(
    start: GridLine,
    end: GridLine,
    num_tracks: usize,
    prefix: usize,
) -> ((Option<usize>, usize), bool) {
    let trailing = MAX_IMPLICIT_LINE_DISTANCE as usize;
    let track_limit = prefix.saturating_add(num_tracks).saturating_add(trailing);
    let mut limited = false;
    let mut placed_line = |line| {
        line_position(line, num_tracks).map(|offset| {
            let shifted = offset.saturating_add(i64::try_from(prefix).unwrap_or(i64::MAX));
            let upper = i64::try_from(track_limit).unwrap_or(i64::MAX);
            let bounded = shifted.clamp(0, upper);
            limited |= shifted != bounded;
            usize::try_from(bounded).unwrap_or(track_limit)
        })
    };
    let edges = (placed_line(start), placed_line(end));
    let span = |line| match line {
        GridLine::Span(span) => usize::from(span.max(1)),
        _ => 1,
    };
    let (anchor, length) = match edges {
        (Some(a), Some(b)) => (Some(a.min(b)), a.abs_diff(b).max(1)),
        (Some(a), None) => (Some(a), span(end)),
        (None, Some(b)) => {
            let requested = span(start);
            let length = requested.min(b).max(1);
            limited |= requested != length || b == 0;
            (Some(b.saturating_sub(requested)), length)
        }
        (None, None) if matches!(start, GridLine::Span(_)) => (None, span(start)),
        (None, None) => (None, span(end)),
    };
    if let Some(anchor) = anchor {
        let start = anchor.min(track_limit - 1);
        let bounded_length = length.min(track_limit - start);
        (
            (Some(start), bounded_length),
            limited || anchor != start || length != bounded_length,
        )
    } else {
        (
            (None, length.min(track_limit)),
            limited || length > track_limit,
        )
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
    fn reversed_lines_use_the_same_span_as_forward_lines() {
        let p = GridPlacement::column(GridLine::Line(3), GridLine::Line(1));
        assert_eq!(p.resolve_column(3), (Some(0), 2));
        let negative = GridPlacement::column(GridLine::Line(-1), GridLine::Line(2));
        assert_eq!(negative.resolve_column(4), (Some(1), 3));
    }

    #[test]
    fn equal_lines_use_one_track() {
        let p = GridPlacement::column(GridLine::Line(2), GridLine::Line(2));
        assert_eq!(p.resolve_column(3), (Some(1), 1));
    }

    #[test]
    fn negative_end_line_spans_to_last_line() {
        // grid-column: 1 / -1 in a 3-track grid → whole row: start 0, span 3.
        let p = GridPlacement::column(GridLine::Line(1), GridLine::Line(-1));
        assert_eq!(p.resolve_column(3), (Some(0), 3));
    }

    #[test]
    fn last_explicit_line_as_start_adds_an_implicit_track() {
        // grid-column: -1 starts after the three explicit tracks.
        let p = GridPlacement::column(GridLine::Line(-1), GridLine::Auto);
        assert_eq!(p.resolve_column(3), (Some(3), 1));
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
    fn placement_is_clamped_at_the_implicit_edges() {
        let limit = super::MAX_IMPLICIT_LINE_DISTANCE;
        let first_outside = 3 + limit as i32 + 1;
        let trailing = GridPlacement::row(
            GridLine::Line(first_outside),
            GridLine::Line(first_outside + 2),
        );
        assert_eq!(trailing.resolve_row(3), (Some(3 + limit as usize - 1), 1));

        let leading = GridPlacement::column(
            GridLine::OutsideExplicit(-(limit as i16) - 1),
            GridLine::OutsideExplicit(-(limit as i16) - 2),
        );
        assert_eq!(leading.resolve_column(3), (Some(0), 1));
    }

    #[test]
    fn anchored_spans_cannot_cross_the_implicit_limit() {
        let limit = super::MAX_IMPLICIT_LINE_DISTANCE;
        let span = GridLine::Span(limit as u16 + 5);
        let trailing = GridPlacement::row(GridLine::Line(-1), span);
        assert_eq!(trailing.resolve_row(3), (Some(3), limit as usize));
        let leading = GridPlacement::column(span, GridLine::Line(1));
        assert_eq!(leading.resolve_column(3), (Some(0), limit as usize));
    }

    #[test]
    fn positive_lines_unaffected_by_track_count() {
        // A positive line past the template can open implicit tracks within the limit.
        let p = GridPlacement::column(GridLine::Line(5), GridLine::Auto);
        assert_eq!(p.resolve_column(3), (Some(4), 1));
    }
}
