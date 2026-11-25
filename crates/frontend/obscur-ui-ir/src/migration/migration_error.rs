use crate::version::UiIrVersion;

/// The reason a [`MigrationRegistry`](super::MigrationRegistry) could not migrate a document (spec §2.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationError {
    /// No chain of registered migrations leads from `from` to `to`. Raised when
    /// some intermediate version has no outgoing migration, or when the search
    /// exceeds its iteration bound (a cyclic or non-advancing chain).
    NoPath {
        /// The version the chain stalled at.
        from: UiIrVersion,
        /// The requested target version.
        to: UiIrVersion,
    },
}

impl std::fmt::Display for MigrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoPath { from, to } => write!(
                f,
                "no migration path from {}.{} to {}.{}",
                from.major(),
                from.minor(),
                to.major(),
                to.minor()
            ),
        }
    }
}

impl std::error::Error for MigrationError {}
