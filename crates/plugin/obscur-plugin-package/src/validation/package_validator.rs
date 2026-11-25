//! PackageValidator (skeleton stub for the spec-defined entity; not yet implemented).

use std::collections::{BTreeMap, BTreeSet};
use std::{fs::File, io, path::Path};

use obscur_plugin_model::{
    contribution::ExtensionPoint,
    identity::PluginId,
    manifest::{DependencyKind, PluginManifest},
};
use semver::Version;

use crate::archive::{TarPackageReader, ZipPackageReader};
use crate::package::{PackageEntry, PackageFormat, PackageReader, PluginPackage};
use crate::path::PackagePath;

use super::{HashValidator, PackageError, PackageLimits};

/// Validates decoded plugin packages against an explicit host policy.
#[derive(Debug)]
pub struct PackageValidator {
    plugin_api: Version,
    ui_ir: Version,
    os: Box<str>,
    arch: Box<str>,
    capabilities: BTreeSet<Box<str>>,
    providers: BTreeMap<PluginId, Version>,
    application: Option<(Box<str>, Version)>,
    host_services: BTreeMap<Box<str>, BTreeSet<Version>>,
    host_features: BTreeSet<Box<str>>,
    limits: PackageLimits,
}

impl PackageValidator {
    /// Loads and verifies a TAR, compressed TAR, or ZIP archive selected by its file name.
    pub fn load_path(&self, path: &Path) -> Result<PluginPackage, PackageError> {
        let format = PackageFormat::from_path(path).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "expected a .zip, .tar, .tar.gz, or .tar.zst archive",
            )
        })?;
        let file = File::open(path)?;
        match format {
            PackageFormat::Zip => self.load(ZipPackageReader::new(file)),
            format => self.load(TarPackageReader::new(file, format)),
        }
    }
    /// Uses the current platform, default budgets, and an empty capability and provider allowlist.
    #[must_use]
    pub fn new(plugin_api: Version, ui_ir: Version) -> Self {
        Self {
            plugin_api,
            ui_ir,
            os: std::env::consts::OS.into(),
            arch: std::env::consts::ARCH.into(),
            capabilities: BTreeSet::new(),
            providers: BTreeMap::new(),
            application: None,
            host_services: BTreeMap::new(),
            host_features: BTreeSet::new(),
            limits: PackageLimits::default(),
        }
    }

    /// Overrides the operating system and architecture checked by the policy.
    #[must_use]
    pub fn with_platform(mut self, os: impl Into<Box<str>>, arch: impl Into<Box<str>>) -> Self {
        self.os = os.into();
        self.arch = arch.into();
        self
    }

    /// Configures archive, expanded, entry, and manifest limits.
    #[must_use]
    pub const fn with_limits(mut self, limits: PackageLimits) -> Self {
        self.limits = limits;
        self
    }

    /// Allows a manifest to request the named capability.
    #[must_use]
    pub fn allow_capability(mut self, capability: impl Into<Box<str>>) -> Self {
        self.capabilities.insert(capability.into());
        self
    }

    /// Adds an available dependency version to the host inventory.
    #[must_use]
    pub fn with_provider(mut self, id: PluginId, version: Version) -> Self {
        self.providers.insert(id, version);
        self
    }

    /// Identifies the host application and its version for compatibility checks.
    #[must_use]
    pub fn with_application(mut self, id: impl Into<Box<str>>, version: Version) -> Self {
        self.application = Some((id.into(), version));
        self
    }

    /// Declares a service contract version available from the host application.
    #[must_use]
    pub fn with_host_service(mut self, id: impl Into<Box<str>>, version: Version) -> Self {
        self.host_services
            .entry(id.into())
            .or_default()
            .insert(version);
        self
    }

    /// Declares one optional feature supplied by the host application.
    #[must_use]
    pub fn with_host_feature(mut self, id: impl Into<Box<str>>) -> Self {
        self.host_features.insert(id.into());
        self
    }

    /// Decodes and verifies a package in memory before returning its payload.
    pub fn load(&self, reader: impl PackageReader) -> Result<PluginPackage, PackageError> {
        let mut files = index_files(reader.read_entries(self.limits)?, self.limits)?;
        let manifest_path = PackagePath::parse("manifest.json").expect("fixed manifest path");
        let manifest_file = files
            .remove(&manifest_path)
            .ok_or_else(|| PackageError::MissingFile("manifest.json".into()))?;
        if manifest_file.bytes().len() > self.limits.manifest_bytes {
            return Err(PackageError::Limit("manifest size"));
        }
        let manifest: PluginManifest = serde_json::from_slice(manifest_file.bytes())?;
        self.validate_manifest(&manifest)?;
        HashValidator::validate(&manifest, &files)?;
        let entrypoint = present_path(&manifest.entrypoint.path, &files)?;
        for path in manifest
            .resources
            .iter()
            .chain(&manifest.css)
            .chain(&manifest.themes)
            .chain(&manifest.localization)
        {
            present_path(path, &files)?;
        }
        let host_features = manifest
            .compatibility
            .optional_features
            .intersection(&self.host_features)
            .cloned()
            .collect();
        Ok(PluginPackage::validated(
            manifest,
            manifest_file,
            files,
            entrypoint,
            host_features,
        ))
    }

    fn validate_manifest(&self, manifest: &PluginManifest) -> Result<(), PackageError> {
        if manifest.schema_version != 1 {
            return Err(PackageError::Manifest("unsupported schema version".into()));
        }
        if manifest.name.trim().is_empty() || manifest.author.trim().is_empty() {
            return Err(PackageError::Manifest(
                "name and author must be nonempty".into(),
            ));
        }
        if !manifest.compatibility.plugin_api.matches(&self.plugin_api) {
            return Err(PackageError::Incompatible("Plugin API version"));
        }
        if !manifest.compatibility.ui_ir.matches(&self.ui_ir) {
            return Err(PackageError::Incompatible("UI IR version"));
        }
        if !manifest.compatibility.applications.is_empty()
            && !self.application.as_ref().is_some_and(|(id, version)| {
                manifest
                    .compatibility
                    .applications
                    .get(id)
                    .is_some_and(|accepted| accepted.matches(version))
            })
        {
            return Err(PackageError::Incompatible("host application ID or version"));
        }
        for (id, accepted) in &manifest.compatibility.required_services {
            if !self
                .host_services
                .get(id)
                .is_some_and(|versions| versions.iter().any(|version| accepted.matches(version)))
            {
                return Err(PackageError::Incompatible("required host service"));
            }
        }
        if manifest
            .compatibility
            .extension_contracts
            .keys()
            .any(|id| ExtensionPoint::new(id).is_none())
        {
            return Err(PackageError::Manifest(
                "invalid extension point contract ID".into(),
            ));
        }
        if !manifest.platforms.is_empty()
            && !manifest
                .platforms
                .iter()
                .any(|target| target.os == self.os && target.arch == self.arch)
        {
            return Err(PackageError::Incompatible("platform or architecture"));
        }
        for capability in &manifest.capabilities {
            if !self.capabilities.contains(capability) {
                return Err(PackageError::DeniedCapability(capability.to_string()));
            }
        }
        let mut dependencies = BTreeSet::new();
        for dependency in &manifest.dependencies {
            let id = dependency.id.as_str();
            if dependency.id == manifest.id || !dependencies.insert(&dependency.id) {
                return Err(PackageError::Dependency(format!(
                    "invalid or repeated provider {id}"
                )));
            }
            match self.providers.get(&dependency.id) {
                Some(version) if dependency.version.matches(version) => {}
                None if dependency.kind == DependencyKind::Optional => {}
                _ => {
                    return Err(PackageError::Dependency(format!(
                        "provider {id} must satisfy {}",
                        dependency.version
                    )));
                }
            }
        }
        Ok(())
    }
}

fn present_path(
    value: &str,
    files: &BTreeMap<PackagePath, PackageEntry>,
) -> Result<PackagePath, PackageError> {
    let path = PackagePath::parse(value).ok_or_else(|| PackageError::InvalidPath(value.into()))?;
    if !files.contains_key(&path) {
        return Err(PackageError::MissingFile(path.as_str().into()));
    }
    Ok(path)
}

fn index_files(
    entries: Vec<PackageEntry>,
    limits: PackageLimits,
) -> Result<BTreeMap<PackagePath, PackageEntry>, PackageError> {
    if entries.len() > limits.entries {
        return Err(PackageError::Limit("entry count"));
    }
    let mut total = 0_u64;
    let mut files = BTreeMap::new();
    for entry in entries {
        let size =
            u64::try_from(entry.bytes().len()).map_err(|_| PackageError::Limit("entry size"))?;
        if size > limits.entry_bytes {
            return Err(PackageError::Limit("entry size"));
        }
        total = total
            .checked_add(size)
            .ok_or(PackageError::Limit("expanded size"))?;
        if total > limits.expanded_bytes {
            return Err(PackageError::Limit("expanded size"));
        }
        let path = entry.path().clone();
        match files.entry(path) {
            std::collections::btree_map::Entry::Vacant(slot) => {
                slot.insert(entry);
            }
            std::collections::btree_map::Entry::Occupied(slot) => {
                return Err(PackageError::DuplicatePath(slot.key().as_str().into()));
            }
        }
    }
    for path in files.keys() {
        for (index, _) in path.as_str().match_indices('/') {
            let prefix =
                PackagePath::parse(&path.as_str()[..index]).expect("canonical path prefix");
            if files.contains_key(&prefix) {
                return Err(PackageError::DuplicatePath(path.as_str().into()));
            }
        }
    }
    Ok(files)
}
