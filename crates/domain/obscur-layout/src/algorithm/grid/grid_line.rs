//! `GridLine`: one edge of a grid item's line-based placement.

/// The number of implicit tracks an authored anchor or span may extend beyond either edge of the explicit grid.
pub(super) const MAX_IMPLICIT_LINE_DISTANCE: u32 = 1024;

#[doc = include_str!("../../../docs/api/algorithm/grid/grid_line.doc/GridLine.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridLine {
    /// No explicit line on this edge; the item is auto-placed here.
    Auto,
    /// A 1-based grid line index; a negative index counts back from the end of the
    /// explicit grid (`-1` is the last line), resolved against the track count.
    Line(i32),
    /// `span N`: this edge is `N` tracks from the opposite edge (clamped ≥ 1).
    Span(u16),
    /// Signed distance in lines beyond the explicit grid.
    OutsideExplicit(i16),
}

impl Default for GridLine {
    /// The initial value: [`Auto`](GridLine::Auto) (pure auto-placement).
    fn default() -> Self {
        GridLine::Auto
    }
}

impl GridLine {
    /// A 1-based line, clamped so a non-positive index becomes line `1`.
    #[must_use]
    pub fn line(index: i32) -> Self {
        GridLine::Line(index.max(1))
    }

    /// A `span N`, clamped so a zero span becomes `1`.
    #[must_use]
    pub fn span(n: u16) -> Self {
        GridLine::Span(n.max(1))
    }

    /// Creates a line offset beyond the nearest explicit edge. Placement applies the track limit.
    #[must_use]
    pub fn outside_explicit(distance: u32, before: bool) -> Self {
        let distance = distance.clamp(1, i16::MAX as u32) as i16;
        Self::OutsideExplicit(if before { -distance } else { distance })
    }

    /// The 0-based track index this line refers to, clamped to `≥ 0`, if it is a
    /// concrete [`Line`](GridLine::Line); `None` for `Auto` / `Span`.
    #[must_use]
    pub fn track_index(self) -> Option<usize> {
        match self {
            GridLine::Line(a) => Some(usize::try_from(a.max(1)).unwrap_or(1) - 1),
            GridLine::Auto | GridLine::Span(_) | GridLine::OutsideExplicit(_) => None,
        }
    }
}
