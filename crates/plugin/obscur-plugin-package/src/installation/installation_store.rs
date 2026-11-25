use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

use obscur_plugin_model::identity::PluginId;

use super::generation_name::GenerationName;
use super::{InstalledPackage, StagedInstallation};
use crate::extraction::ExtractionPlan;
use crate::package::PluginPackage;
use crate::validation::PackageError;

/// Publishes one installed generation per plugin ID in a host-owned directory.
#[derive(Debug)]
pub struct InstallationStore {
    pub(super) root: PathBuf,
}

impl InstallationStore {
    /// Opens an existing directory dedicated to this store.
    ///
    /// The host must protect the directory and its ancestors against
    /// replacement or modification by untrusted users.
    pub fn open(root: &Path) -> io::Result<Self> {
        let root = root.canonicalize()?;
        if !root.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::NotADirectory,
                "installation store",
            ));
        }
        Ok(Self { root })
    }

    /// Reads the currently published package. The previous generation remains
    /// accessible to callers holding its paths after a subsequent commit.
    pub fn current(&self, id: &PluginId) -> Result<Option<InstalledPackage>, PackageError> {
        let generation = match fs::read_to_string(self.marker(id)) {
            Ok(generation) => generation,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        let directory = self.root.join(GenerationName::parse(&generation)?.as_str());
        let manifest = serde_json::from_slice(&fs::read(directory.join("manifest.json"))?)?;
        let package = InstalledPackage::new(directory, manifest)?;
        if &package.manifest().id != id {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "installation ID does not match its manifest",
            )
            .into());
        }
        Ok(Some(package))
    }

    /// Prepares a validated package beside the current generation.
    /// The candidate remains unpublished until the host commits it.
    pub fn stage(&self, package: &PluginPackage) -> Result<StagedInstallation<'_>, PackageError> {
        let previous = self
            .current(&package.manifest().id)?
            .map(|installed| installed.directory);
        let generation = GenerationName::temporary_directory_in(&self.root)?;
        let extracted = ExtractionPlan::new(package).extract_in(generation.path())?;
        extracted.move_to(&generation.path().join("payload"))?;
        fs::write(
            generation.path().join("manifest.json"),
            package.manifest_bytes(),
        )?;
        let candidate =
            InstalledPackage::new(generation.path().to_owned(), package.manifest().clone())?;
        Ok(StagedInstallation {
            store: self,
            generation,
            candidate,
            previous,
        })
    }

    pub(super) fn marker(&self, id: &PluginId) -> PathBuf {
        self.root.join(format!("plugin-{}.current", id.as_str()))
    }

    pub(super) fn lock(&self) -> io::Result<File> {
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(self.root.join(".installation-lock"))?;
        lock.lock()?;
        Ok(lock)
    }
}
