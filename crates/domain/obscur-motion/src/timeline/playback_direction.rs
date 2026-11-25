/// The direction a [`Timeline`](super::Timeline) plays its iterations (spec §4.17
/// "reversal / alternate").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlaybackDirection {
    /// Plays from the start of the iteration (progress `0`) toward its end
    /// (progress `1`).
    Forward,
    /// Plays from the end of the iteration (progress `1`) back toward its start
    /// (progress `0`).
    Reverse,
}

impl PlaybackDirection {
    /// Returns the opposite direction (used when a timeline alternates).
    #[must_use]
    pub const fn flipped(self) -> Self {
        match self {
            Self::Forward => Self::Reverse,
            Self::Reverse => Self::Forward,
        }
    }
}
