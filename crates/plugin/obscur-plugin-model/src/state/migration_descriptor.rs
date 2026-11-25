//! A declared source range and target schema for one migration.

use super::StateSchema;
use crate::manifest::VersionReq;

/// A declared source range and target schema for one migration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationDescriptor {
    source: VersionReq,
    target: StateSchema,
}

impl MigrationDescriptor {
    /// Declares which source versions can migrate into the target schema.
    #[must_use]
    pub const fn new(source: VersionReq, target: StateSchema) -> Self {
        Self { source, target }
    }

    /// Returns the compatible range of source schema versions.
    #[must_use]
    pub const fn source(&self) -> &VersionReq {
        &self.source
    }

    /// Returns the resulting schema for the candidate.
    #[must_use]
    pub const fn target(&self) -> &StateSchema {
        &self.target
    }
}
