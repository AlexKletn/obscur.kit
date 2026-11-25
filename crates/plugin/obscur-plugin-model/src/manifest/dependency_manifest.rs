//! A plugin dependency declared in a package manifest.

use super::DependencyKind;

/// One declared plugin dependency.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DependencyManifest {
    /// The provider plugin ID.
    pub id: crate::identity::PluginId,
    /// The supported provider version range.
    pub version: semver::VersionReq,
    /// The reason the dependency participates in validation.
    pub kind: DependencyKind,
}
