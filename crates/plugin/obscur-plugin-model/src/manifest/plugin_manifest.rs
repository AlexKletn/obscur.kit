//! The manifest read from a plugin package.

use super::{
    ContributionDeclarations, DependencyManifest, Entrypoint, HostCompatibility, ManifestFile,
    PluginKind, PluginPlatform,
};
use crate::identity::PluginId;

/// The versioned manifest of a plugin package.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PluginManifest {
    /// The version of this manifest format.
    pub schema_version: u32,
    /// The validated plugin identity.
    pub id: PluginId,
    /// The user-facing plugin name.
    pub name: Box<str>,
    /// The package version.
    pub version: semver::Version,
    /// The author named by the package.
    pub author: Box<str>,
    /// The declared execution class.
    pub kind: PluginKind,
    /// The host interface requirements.
    pub compatibility: HostCompatibility,
    /// The only entrypoint selected by the package.
    pub entrypoint: Entrypoint,
    /// Declarations of contributions for future activation.
    #[serde(default)]
    pub contributions: ContributionDeclarations,
    /// Paths to declared resources.
    #[serde(default)]
    pub resources: Vec<Box<str>>,
    /// Paths to CSS assets.
    #[serde(default)]
    pub css: Vec<Box<str>>,
    /// Paths to theme assets.
    #[serde(default)]
    pub themes: Vec<Box<str>>,
    /// Paths to localization assets.
    #[serde(default)]
    pub localization: Vec<Box<str>>,
    /// Capabilities that must be explicitly allowed by the host.
    #[serde(default)]
    pub capabilities: Vec<Box<str>>,
    /// Permitted OS and architecture pairs. An empty list is unrestricted.
    #[serde(default)]
    pub platforms: Vec<PluginPlatform>,
    /// The plugin providers referenced by the package.
    #[serde(default)]
    pub dependencies: Vec<DependencyManifest>,
    /// SHA-256 hashes of every package payload file.
    pub files: Vec<ManifestFile>,
}
