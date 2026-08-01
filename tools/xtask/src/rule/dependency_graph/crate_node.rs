#![doc = include_str!("crate_node.doc/_module.md")]

use std::fs;
use std::path::PathBuf;

use super::layer::Layer;

#[doc = include_str!("crate_node.doc/CrateNode.md")]
pub(crate) struct CrateNode {
    #[doc = include_str!("crate_node.doc/name.md")]
    pub(crate) name: String,
    #[doc = include_str!("crate_node.doc/layer.md")]
    pub(crate) layer: Layer,
    #[doc = include_str!("crate_node.doc/manifest.md")]
    pub(crate) manifest: PathBuf,
    #[doc = include_str!("crate_node.doc/deps.md")]
    pub(crate) deps: Vec<String>,
}

impl CrateNode {
    #[doc = include_str!("crate_node.doc/parse.md")]
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

#[doc = include_str!("crate_node.doc/package_name.md")]
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

#[doc = include_str!("crate_node.doc/obscur_deps.md")]
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

#[doc = include_str!("crate_node.doc/obscur_dep_key.md")]
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
