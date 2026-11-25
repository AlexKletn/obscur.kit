//! Materializes a validated plugin payload in a fresh host-managed directory.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{DirBuilder, OpenOptions};
use std::io::Write;
use std::path::Path;

use super::{ExtractedEntry, IsolatedDirectory};
use crate::package::PluginPackage;
use crate::validation::PackageError;

/// Materializes a validated plugin payload in a fresh host-managed directory.
#[derive(Debug)]
pub struct ExtractionPlan<'a> {
    package: &'a PluginPackage,
}

impl<'a> ExtractionPlan<'a> {
    /// Prepares to extract the verified payload of a package.
    #[must_use]
    pub const fn new(package: &'a PluginPackage) -> Self {
        Self { package }
    }

    /// Creates a fresh directory within an existing host-managed parent.
    ///
    /// The host must keep the parent and its ancestors protected against
    /// replacement by untrusted users. On platforms with inherited directory
    /// permissions, the parent must grant access only to the intended users.
    pub fn extract_in(&self, parent: &Path) -> Result<IsolatedDirectory, PackageError> {
        for file in self.package.files() {
            if !file.path().as_str().split('/').all(portable_component) {
                return Err(PackageError::InvalidPath(file.path().as_str().into()));
            }
        }

        let mut temp = tempfile::Builder::new();
        temp.prefix("obscur-plugin-");
        let directories = DirBuilder::new();
        let mut files = OpenOptions::new();
        files.write(true).create_new(true);
        #[cfg(unix)]
        let directories = {
            use std::fs::Permissions;
            use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};

            temp.permissions(Permissions::from_mode(0o700));
            let mut directories = directories;
            directories.mode(0o700);
            files.mode(0o600);
            directories
        };

        let root = temp.tempdir_in(parent.canonicalize()?)?;
        let mut created = BTreeSet::new();
        let mut extracted = BTreeMap::new();
        for file in self.package.files() {
            let relative = Path::new(file.path().as_str());
            // Create each logical parent exactly once. If another spelling
            // resolves to the same directory on disk, create fails.
            let ancestors: Vec<_> = relative
                .ancestors()
                .skip(1)
                .take_while(|path| !path.as_os_str().is_empty())
                .collect();
            for ancestor in ancestors.into_iter().rev() {
                if created.insert(ancestor) {
                    directories.create(root.path().join(ancestor))?;
                }
            }
            let path = root.path().join(relative);
            files.open(&path)?.write_all(file.bytes())?;
            extracted.insert(file.path().clone(), ExtractedEntry { path });
        }
        Ok(IsolatedDirectory {
            root,
            files: extracted,
            entrypoint: self.package.entrypoint().path().clone(),
        })
    }
}

fn portable_component(component: &str) -> bool {
    if component.ends_with(['.', ' ']) || component.contains(['<', '>', '"', '|', '?', '*']) {
        return false;
    }

    // Windows recognizes these device names even with an extension, and
    // recognizes superscript 1–3 in the COM and LPT name families.
    let stem = component
        .split('.')
        .next()
        .unwrap_or_default()
        .trim_end_matches(' ')
        .to_ascii_uppercase();
    !matches!(
        stem.as_str(),
        "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
    ) && !stem
        .strip_prefix("COM")
        .or_else(|| stem.strip_prefix("LPT"))
        .is_some_and(|suffix| {
            matches!(
                suffix,
                "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
            )
        })
}
