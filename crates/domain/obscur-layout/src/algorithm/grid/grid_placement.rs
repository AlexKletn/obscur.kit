//! `GridPlacement`: an item's line-based placement on both grid axes.

use super::GridLine;

/// A grid item's explicit line-based placement (spec §4.13, a subset).
///
/// Each axis is a `start / end` [`GridLine`] pair — the CSS `grid-column`
/// (`column_start / column_end`) and `grid-row` (`row_start / row_end`). The
/// [`Default`] is all [`GridLine::Auto`], i.e. pure row-major auto-placement (the
/// grid's historical behaviour), so a default `GridPlacement` never perturbs an
/// existing layout.
///
/// # Resolution
///
/// [`resolve_axis`](GridPlacement::resolve_column) turns one axis pair into a
/// concrete `(start, span)`:
///
/// * `Line(a) / Line(b)` → definite start `a - 1`, span `max(b - a, 1)`.
/// * `Line(a) / Span(n)` → definite start `a - 1`, span `n`.
/// * `Line(a) / Auto`    → definite start `a - 1`, span `1`.
/// * `Span(n) / Line(b)` → definite start `(b - 1) - n`, span `n` (end-anchored).
/// * `Auto / Line(b)`    → definite start `b - 2`, span `1` (end-anchored).
/// * `Auto / Span(n)`    → auto-placed (start `None`), span `n`.
/// * `Auto / Auto`       → auto-placed (start `None`), span `1`.
///
/// A `None` start means the axis is auto-flowed; a `Some(i)` start is a definite
/// 0-based track index. Spans are always `≥ 1`.
///
/// # Determinism
///
/// `GridPlacement` is a `Copy` value with no interior mutability; placement
/// resolves deterministically from the item styles alone.
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

    /// Resolves the **column** axis to `(start, span)` — see the type docs.
    ///
    /// `num_tracks` is the number of explicit tracks on the axis (reserved for
    /// forward-compatible clamping; the current positive-line subset resolves
    /// without needing it).
    #[must_use]
    pub fn resolve_column(&self, num_tracks: usize) -> (Option<usize>, usize) {
        resolve_axis(self.column_start, self.column_end, num_tracks)
    }

    /// Resolves the **row** axis to `(start, span)` — see the type docs.
    #[must_use]
    pub fn resolve_row(&self, num_tracks: usize) -> (Option<usize>, usize) {
        resolve_axis(self.row_start, self.row_end, num_tracks)
    }
}

/// Resolves one `start / end` axis pair into a concrete `(start, span)`.
///
/// `start` is `None` when the axis must be auto-flowed, `Some(i)` for a definite
/// 0-based track index; `span` is always `≥ 1`.
fn resolve_axis(start: GridLine, end: GridLine, num_tracks: usize) -> (Option<usize>, usize) {
    let _ = num_tracks;
    // A clamped-positive line index (`≥ 1`) as a 0-based track index.
    let track = |line: i32| usize::try_from(line.max(1)).unwrap_or(1) - 1;
    match (start, end) {
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
}
