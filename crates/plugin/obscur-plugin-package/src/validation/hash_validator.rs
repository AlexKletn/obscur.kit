//! HashValidator (skeleton stub for the spec-defined entity; not yet implemented).

use std::collections::{BTreeMap, BTreeSet};

use obscur_plugin_model::manifest::PluginManifest;
use sha2::{Digest, Sha256};

use super::PackageError;
use crate::package::PackageEntry;
use crate::path::PackagePath;

/// Checks full payload coverage and SHA-256 equality against a validated manifest.
pub struct HashValidator;

impl HashValidator {
    pub(crate) fn validate(
        manifest: &PluginManifest,
        files: &BTreeMap<PackagePath, PackageEntry>,
    ) -> Result<(), PackageError> {
        let mut seen = BTreeSet::new();
        for file in &manifest.files {
            let path = PackagePath::parse(&file.path)
                .ok_or_else(|| PackageError::InvalidPath(file.path.to_string()))?;
            if !seen.insert(path.clone()) {
                return Err(PackageError::DuplicatePath(path.as_str().into()));
            }
            if file.sha256.len() != 64 || !file.sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
            {
                return Err(PackageError::Manifest(
                    format!("invalid SHA-256 for {:?}", path.as_str()).into(),
                ));
            }
            let entry = files
                .get(&path)
                .ok_or_else(|| PackageError::MissingFile(path.as_str().into()))?;
            let actual = format!("{:x}", Sha256::digest(entry.bytes()));
            if !actual.eq_ignore_ascii_case(&file.sha256) {
                return Err(PackageError::HashMismatch(path.as_str().into()));
            }
        }
        if let Some(unlisted) = files.keys().find(|path| !seen.contains(*path)) {
            return Err(PackageError::UnlistedFile(unlisted.as_str().into()));
        }
        Ok(())
    }
}
