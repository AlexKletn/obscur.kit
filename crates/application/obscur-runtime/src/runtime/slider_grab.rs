/// Which end of a slider a gesture is moving.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Grab {
    /// The low end — and the only end of a single slider.
    Low,
    /// The high end of a range.
    High,
}
