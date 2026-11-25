/// The dependency kinds admitted by the package manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyKind {
    /// A required provider.
    Required,
    /// A provider that may be absent.
    Optional,
    /// A provider used through a service contract.
    ServiceOnly,
}
