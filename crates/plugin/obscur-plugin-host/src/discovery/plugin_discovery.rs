//! Finds and validates plugin archives without executing them.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use obscur_plugin_package::{
    package::PackageFormat,
    validation::{PackageError, PackageValidator},
};

use super::DiscoveredPlugin;

/// Discovers plugin archive metadata in a host-selected directory.
#[derive(Debug)]
pub struct PluginDiscovery;

impl PluginDiscovery {
    /// Scans regular archives in one directory, returning each validation result in path order.
    /// A broken archive does not hide valid siblings. No plugin entry points are executed.
    pub fn scan(
        directory: &Path,
        validator: &PackageValidator,
    ) -> io::Result<Vec<Result<DiscoveredPlugin, (PathBuf, PackageError)>>> {
        let mut paths = Vec::new();
        for entry in fs::read_dir(directory.canonicalize()?)? {
            let entry = entry?;
            if entry.file_type()?.is_file() && PackageFormat::from_path(&entry.path()).is_some() {
                paths.push(entry.path());
            }
        }
        paths.sort_unstable();
        Ok(paths
            .into_iter()
            .map(|path| match validator.load_path(&path) {
                Ok(package) => Ok(DiscoveredPlugin {
                    path,
                    manifest: package.manifest().clone(),
                }),
                Err(error) => Err((path, error)),
            })
            .collect())
    }
}
