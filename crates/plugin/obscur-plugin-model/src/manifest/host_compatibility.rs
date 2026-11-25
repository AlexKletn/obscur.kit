//! `HostCompatibility` (skeleton stub for the spec-defined entity; not yet implemented).

use std::collections::{BTreeMap, BTreeSet};

/// Version requirements for the independent plugin and UI IR interfaces.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostCompatibility {
    /// The supported Plugin API range.
    pub plugin_api: semver::VersionReq,
    /// The supported UI IR range.
    pub ui_ir: semver::VersionReq,
    /// Supported host application identifiers and version ranges. An empty map accepts any host application.
    #[serde(default)]
    pub applications: BTreeMap<Box<str>, semver::VersionReq>,
    /// Required application service identifiers and accepted contract versions.
    #[serde(default)]
    pub required_services: BTreeMap<Box<str>, semver::VersionReq>,
    /// Optional host features the package can use when available.
    #[serde(default)]
    pub optional_features: BTreeSet<Box<str>>,
    /// Required contract version ranges keyed by application extension point ID. Only contributions targeting a mismatched point are skipped.
    #[serde(default)]
    pub extension_contracts: BTreeMap<Box<str>, semver::VersionReq>,
}
