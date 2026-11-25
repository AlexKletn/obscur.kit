//! A single workspace crate node: its name, layer, manifest and the `obscur-*`
//! crates it depends on.

use std::path::PathBuf;

use toml_edit::{Item, Table, TableLike};

use super::layer::Layer;
use crate::violation::Violation;
use crate::workspace_manifests::read_manifest;

const MANIFEST_RULE: &str = "dependency-manifest";

/// One node in the workspace dependency graph.
pub(crate) struct CrateNode {
    /// Package name (`[package] name = "…"`).
    pub(crate) name: String,
    /// Architectural layer inferred from the crate's path.
    pub(crate) layer: Layer,
    /// Path to the crate's `Cargo.toml` (used to anchor violations).
    pub(crate) manifest: PathBuf,
    /// Names of the packages this crate depends on (normal and build
    /// dependencies; dev-dependencies are excluded because they are test-only
    /// and may legitimately point up the graph).
    pub(crate) deps: Vec<String>,
    /// Test-only packages, checked for platform dependencies but excluded from graph edges.
    pub(crate) dev_deps: Vec<String>,
}

impl CrateNode {
    /// Parses a crate node or reports why its manifest cannot be used.
    pub(crate) fn parse(
        manifest: PathBuf,
        layer: Layer,
        workspace_deps: Option<&dyn TableLike>,
    ) -> Result<Self, Violation> {
        let document = read_manifest(MANIFEST_RULE, &manifest)?;
        let name = package_name(document.as_table()).ok_or_else(|| {
            Violation::new(MANIFEST_RULE, &manifest, "manifest has no package name")
        })?;
        let invalid_dependency = |detail| Violation::new(MANIFEST_RULE, &manifest, detail);
        let deps = dependency_names(
            document.as_table(),
            workspace_deps,
            &["dependencies", "build-dependencies"],
        )
        .map_err(invalid_dependency)?;
        let dev_deps = dependency_names(document.as_table(), workspace_deps, &["dev-dependencies"])
            .map_err(invalid_dependency)?;
        Ok(Self {
            name,
            layer,
            manifest,
            deps,
            dev_deps,
        })
    }
}

fn package_name(manifest: &Table) -> Option<String> {
    let name = manifest.get("package")?.get("name")?.as_str()?;
    (!name.is_empty()).then(|| name.to_owned())
}

/// Collects resolved package names from the unconditional and target-specific tables.
fn dependency_names(
    manifest: &Table,
    workspace_deps: Option<&dyn TableLike>,
    tables: &[&str],
) -> Result<Vec<String>, String> {
    let mut deps = Vec::new();
    collect_deps(manifest, workspace_deps, tables, &mut deps)?;
    if let Some(targets) = manifest.get("target").and_then(Item::as_table_like) {
        for (_, target) in targets.iter() {
            if let Some(target) = target.as_table_like() {
                collect_deps(target, workspace_deps, tables, &mut deps)?;
            }
        }
    }
    Ok(deps)
}

fn collect_deps(
    scope: &dyn TableLike,
    workspace_deps: Option<&dyn TableLike>,
    tables: &[&str],
    deps: &mut Vec<String>,
) -> Result<(), String> {
    for table in tables {
        let Some(entries) = scope.get(table).and_then(Item::as_table_like) else {
            continue;
        };
        for (key, dependency) in entries.iter() {
            let dependency = if dependency.get("workspace").and_then(Item::as_bool) == Some(true) {
                workspace_deps
                    .and_then(|table| table.get(key))
                    .ok_or_else(|| {
                        format!("dependency '{key}' is missing from [workspace.dependencies]")
                    })?
            } else {
                dependency
            };
            let package = dependency
                .get("package")
                .and_then(Item::as_str)
                .unwrap_or(key);
            if !deps.iter().any(|known| known == package) {
                deps.push(package.to_owned());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{dependency_names, package_name};
    use toml_edit::Document;

    #[test]
    fn reads_package_name() {
        let toml = "[package]\nname = \"obscur-demo\" # comment\nversion = \"0.1.0\"\n";
        assert_eq!(
            package_name(toml.parse::<Document<String>>().unwrap().as_table()).as_deref(),
            Some("obscur-demo")
        );
    }

    #[test]
    fn collects_normal_and_build_but_not_dev_deps() {
        let toml = "\
[package]
name = \"obscur-demo\"

[dependencies]
geometry = { package = 'obscur-geometry', path = '../obscur-geometry' }
obscur-identity = '1'
serde = \"1\"

[build-dependencies]
obscur-buildhelper = '1'

[dev-dependencies]
obscur-headless = '1'
";
        let deps = dependency_names(
            toml.parse::<Document<String>>().unwrap().as_table(),
            None,
            &["dependencies", "build-dependencies"],
        )
        .unwrap();
        assert!(deps.contains(&"obscur-geometry".to_owned()));
        assert!(deps.contains(&"obscur-identity".to_owned()));
        assert!(deps.contains(&"obscur-buildhelper".to_owned()));
        assert!(!deps.contains(&"obscur-headless".to_owned()));
        assert!(deps.iter().any(|d| d == "serde"));
    }

    #[test]
    fn reports_missing_workspace_dependency() {
        let member = "[dependencies]\nmissing.workspace = true"
            .parse::<Document<String>>()
            .unwrap();
        assert_eq!(
            dependency_names(member.as_table(), None, &["dependencies"]),
            Err("dependency 'missing' is missing from [workspace.dependencies]".into())
        );
    }

    #[test]
    fn collects_target_specific_deps() {
        let toml = "\
[package]
name = \"obscur-demo\"

[target.'cfg(unix)'.dependencies]
obscur-system = '1'
";
        assert_eq!(
            dependency_names(
                toml.parse::<Document<String>>().unwrap().as_table(),
                None,
                &["dependencies", "build-dependencies"]
            )
            .unwrap(),
            vec!["obscur-system".to_owned()]
        );
    }
}
