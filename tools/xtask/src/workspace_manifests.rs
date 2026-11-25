//! Discovers manifests at the member paths declared in the workspace.

use std::fs;
use std::path::{Path, PathBuf};

use toml_edit::Document;

use crate::violation::{Violation, read_text};

pub(crate) fn read_manifest(
    rule: &'static str,
    path: &Path,
) -> Result<Document<String>, Violation> {
    read_text(rule, path)?
        .parse()
        .map_err(|error: toml_edit::TomlError| Violation::new(rule, path, error.to_string()))
}

pub(crate) fn workspace_manifests(root: &Path) -> Vec<PathBuf> {
    let mut manifests: Vec<_> = directories(&root.join("crates"))
        .flat_map(|layer| directories(&layer))
        .chain(directories(&root.join("tools")))
        .map(|member| member.join("Cargo.toml"))
        .filter(|manifest| manifest.is_file())
        .collect();
    manifests.sort_unstable();
    manifests
}

fn directories(path: &Path) -> impl Iterator<Item = PathBuf> + use<> {
    fs::read_dir(path)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .map(|entry| entry.path())
}

/// Selects workspace members whose source and dependencies must remain pure.
pub(crate) fn pure_manifests(root: &Path) -> impl Iterator<Item = PathBuf> + use<> {
    let layers = ["foundation", "domain", "application"].map(|name| root.join("crates").join(name));
    workspace_manifests(root)
        .into_iter()
        .filter(move |manifest| layers.iter().any(|layer| manifest.starts_with(layer)))
}
