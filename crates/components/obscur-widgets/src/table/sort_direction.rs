/// The direction of a [`Table`](super::Table)'s active sort.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    /// Ascending order (A→Z).
    Ascending,
    /// Descending order (Z→A).
    Descending,
}

impl SortDirection {
    /// Returns the opposite direction.
    #[must_use]
    pub const fn reversed(self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }
}
