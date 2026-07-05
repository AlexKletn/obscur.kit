//! A single workspace crate node: its name, layer, manifest and the `obscur-*`
//! crates it depends on.

use std::fs;
use std::path::PathBuf;

use super::layer::Layer;

/// One node in the workspace dependency graph.
pub(crate) struct CrateNode {
    /// Package name (`[package] name = "…"`).
    pub(crate) name: String,
    /// Architectural layer inferred from the crate's path.
    pub(crate) layer: Layer,
    /// Path to the crate's `Cargo.toml` (used to anchor violations).
    pub(crate) manifest: PathBuf,
    /// Names of the `obscur-*` crates this crate depends on (normal and build
    /// dependencies; dev-dependencies are excluded because they are test-only
    /// and may legitimately point up the graph).
    pub(crate) deps: Vec<String>,
}

impl CrateNode {
    /// Parses a crate node from its manifest, returning `None` if the manifest
    /// is unreadable or has no package name.
    pub(crate) fn parse(manifest: PathBuf, layer: Layer) -> Option<Self> {
        let text = fs::read_to_string(&manifest).ok()?;
        let name = package_name(&text)?;
        let deps = obscur_deps(&text);
        Some(Self {
            name,
            layer,
            manifest,
            deps,
        })
    }
}

/// Extracts `[package] name = "…"` from a manifest.
fn package_name(text: &str) -> Option<String> {
    let mut in_package = false;
    for raw in text.lines() {
        let line = raw.trim();
        if line.starts_with('[') {
            in_package = line == "[package]";
            continue;
        }
        if in_package
            && line.starts_with("name")
            && let Some((_, rhs)) = line.split_once('=')
        {
            let value = rhs.trim().trim_matches('"');
            if !value.is_empty() {
                return Some(value.to_owned());
            }
        }
    }
    None
}

/// Collects `obscur-*` dependency keys from every `[dependencies]` /
/// `[build-dependencies]` (including `[target.'…'.dependencies]`) table,
/// skipping `dev-dependencies`.
fn obscur_deps(text: &str) -> Vec<String> {
    let mut deps = Vec::new();
    let mut in_deps = false;
    for raw in text.lines() {
        let line = raw.trim();
        if line.starts_with('[') {
            in_deps = line.ends_with("dependencies]") && !line.contains("dev-dependencies");
            continue;
        }
        if !in_deps {
            continue;
        }
        if let Some(name) = obscur_dep_key(line)
            && !deps.contains(&name)
        {
            deps.push(name);
        }
    }
    deps
}

/// If `line` declares an `obscur-*` dependency, returns its crate name.
///
/// Handles both `obscur-foo = { … }` and `obscur-foo.workspace = true` forms.
fn obscur_dep_key(line: &str) -> Option<String> {
    if !line.starts_with("obscur-") {
        return None;
    }
    let end = line
        .find(|c: char| c == '=' || c == '.' || c.is_whitespace())
        .unwrap_or(line.len());
    let key = line[..end].trim();
    if key.len() > "obscur-".len() {
        Some(key.to_owned())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{obscur_deps, package_name};

    #[test]
    fn reads_package_name() {
        let toml = "[package]\nname = \"obscur-demo\"\nversion = \"0.1.0\"\n";
        assert_eq!(package_name(toml).as_deref(), Some("obscur-demo"));
    }

    #[test]
    fn collects_normal_and_build_but_not_dev_deps() {
        let toml = "\
[package]
name = \"obscur-demo\"

[dependencies]
obscur-geometry = { workspace = true }
obscur-identity.workspace = true
serde = \"1\"

[build-dependencies]
obscur-buildhelper = { workspace = true }

[dev-dependencies]
obscur-headless = { workspace = true }
";
        let deps = obscur_deps(toml);
        assert!(deps.contains(&"obscur-geometry".to_owned()));
        assert!(deps.contains(&"obscur-identity".to_owned()));
        assert!(deps.contains(&"obscur-buildhelper".to_owned()));
        assert!(!deps.contains(&"obscur-headless".to_owned()));
        assert!(!deps.iter().any(|d| d == "serde"));
    }

    #[test]
    fn collects_target_specific_deps() {
        let toml = "\
[package]
name = \"obscur-demo\"

[target.'cfg(unix)'.dependencies]
obscur-system = { workspace = true }
";
        assert_eq!(obscur_deps(toml), vec!["obscur-system".to_owned()]);
    }
}
