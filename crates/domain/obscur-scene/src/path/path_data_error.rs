use super::PathDataFault;

/// What was wrong with some path data, and where.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathDataError {
    /// The byte offset the fault was found at.
    pub at: usize,
    /// What was wrong.
    pub what: PathDataFault,
}
