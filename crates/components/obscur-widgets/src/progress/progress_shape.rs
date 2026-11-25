#[doc = include_str!("../../docs/api/progress/progress.doc/ProgressShape.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProgressShape {
    /// A straight bar across the box.
    #[default]
    Line,
    /// A circle inscribed in the box, filling clockwise from twelve.
    Ring,
}

impl ProgressShape {
    /// The shape as an integer, because a `ViewValue` carries no enum.
    #[must_use]
    pub const fn as_int(self) -> i64 {
        match self {
            Self::Line => 0,
            Self::Ring => 1,
        }
    }

    #[doc = include_str!("../../docs/api/progress/progress.doc/from_int.md")]
    #[must_use]
    pub const fn from_int(raw: i64) -> Self {
        match raw {
            1 => Self::Ring,
            // Anything else is a line, which is the shape that needs no room
            // and no policy — a control asked for a shape nobody modelled is
            // better as a plain bar than as nothing at all.
            _ => Self::Line,
        }
    }
}
