#![doc = include_str!("grid_line.doc/_module.md")]

#[doc = include_str!("grid_line.doc/GridLine.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridLine {
    #[doc = include_str!("grid_line.doc/Auto.md")]
    Auto,
    #[doc = include_str!("grid_line.doc/Line.md")]
    Line(i32),
    #[doc = include_str!("grid_line.doc/Span.md")]
    Span(u16),
}

impl Default for GridLine {
    #[doc = include_str!("grid_line.doc/default.md")]
    fn default() -> Self {
        GridLine::Auto
    }
}

impl GridLine {
    #[doc = include_str!("grid_line.doc/line.md")]
    #[must_use]
    pub fn line(index: i32) -> Self {
        GridLine::Line(index.max(1))
    }

    #[doc = include_str!("grid_line.doc/span.md")]
    #[must_use]
    pub fn span(n: u16) -> Self {
        GridLine::Span(n.max(1))
    }

    #[doc = include_str!("grid_line.doc/track_index.md")]
    #[must_use]
    pub fn track_index(self) -> Option<usize> {
        match self {
            GridLine::Line(a) => Some(usize::try_from(a.max(1)).unwrap_or(1) - 1),
            GridLine::Auto | GridLine::Span(_) => None,
        }
    }
}
