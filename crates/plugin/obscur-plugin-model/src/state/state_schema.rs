//! An independently versioned format for plugin state.

use crate::manifest::Version;

/// An independently versioned format for plugin state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateSchema(Version);

impl StateSchema {
    /// Identifies an independently versioned plugin state format.
    #[must_use]
    pub const fn new(version: Version) -> Self {
        Self(version)
    }

    /// Returns the version of this state format.
    #[must_use]
    pub const fn version(&self) -> &Version {
        &self.0
    }
}
