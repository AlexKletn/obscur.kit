//! `GridLine`: one edge of a grid item's line-based placement.

/// One placement edge of a grid item along a single axis (spec §4.13, a subset).
///
/// A CSS `grid-column` / `grid-row` is a `start / end` pair; each side is one
/// `GridLine`. This models the three forms this crate supports:
///
/// * [`Auto`](GridLine::Auto) — no explicit line; the item is auto-placed on this
///   edge (the initial value).
/// * [`Line`](GridLine::Line) — a **1-based** grid line index (`grid-column: 2`
///   is line `2`, i.e. the start of the second track). Only positive lines are
///   meaningful here; a non-positive value is clamped to line `1` (named and
///   negative lines are out of scope).
/// * [`Span`](GridLine::Span) — `span N`: the edge is placed `N` tracks away from
///   the opposite edge. A zero span is clamped to `1`.
///
/// # Determinism
///
/// `GridLine` is a plain `Copy` value with no interior mutability, so placement
/// resolves deterministically from the item styles alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridLine {
    /// No explicit line on this edge; the item is auto-placed here.
    Auto,
    /// A 1-based grid line index. Non-positive inputs are clamped to `1`.
    Line(i32),
    /// `span N`: this edge is `N` tracks from the opposite edge (clamped ≥ 1).
    Span(u16),
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

    /// The 0-based track index this line refers to, clamped to `≥ 0`, if it is a
    /// concrete [`Line`](GridLine::Line); `None` for `Auto` / `Span`.
    #[must_use]
    pub fn track_index(self) -> Option<usize> {
        match self {
            GridLine::Line(a) => Some(usize::try_from(a.max(1)).unwrap_or(1) - 1),
            GridLine::Auto | GridLine::Span(_) => None,
        }
    }
}
