use std::collections::BTreeSet;

use obscur_plugin_model::manifest::Version;

/// A versioned service and the operations that a handle may invoke.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceScope {
    service: Box<str>,
    version: Version,
    operations: BTreeSet<Box<str>>,
}

impl ServiceScope {
    /// Creates a scope using host-defined service and operation identifiers.
    #[must_use]
    pub fn new(
        service: impl Into<Box<str>>,
        version: Version,
        operations: impl IntoIterator<Item = impl Into<Box<str>>>,
    ) -> Self {
        Self {
            service: service.into(),
            version,
            operations: operations.into_iter().map(Into::into).collect(),
        }
    }

    pub(crate) fn key(&self) -> (&str, &Version) {
        (self.service(), self.version())
    }

    /// Returns the host-defined service identifier.
    #[must_use]
    pub fn service(&self) -> &str {
        &self.service
    }

    /// Returns the version of the service contract.
    #[must_use]
    pub const fn version(&self) -> &Version {
        &self.version
    }

    /// Iterates the operations in this scope in identifier order.
    pub fn operations(&self) -> impl Iterator<Item = &str> {
        self.operations.iter().map(AsRef::as_ref)
    }

    pub(crate) fn includes(&self, scope: &Self) -> bool {
        self.service == scope.service
            && self.version == scope.version
            && scope.operations.is_subset(&self.operations)
    }
    pub(super) fn permits(&self, service: &str, version: &Version, operation: &str) -> bool {
        self.service() == service
            && self.version() == version
            && self.operations.contains(operation)
    }
}
