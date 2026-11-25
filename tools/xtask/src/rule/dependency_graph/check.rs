//! Graph-based architecture checks: layering direction, the facade-dependency
//! ban, and workspace cycles (spec §3.16, §5.9).

use std::path::Path;

use super::graph::WorkspaceGraph;
use super::layer::Layer;
use crate::violation::Violation;

const RULE_LAYERING: &str = "dependency-layering";
const RULE_FACADE: &str = "facade-dependency";
const RULE_CYCLE: &str = "dependency-cycle";

/// The umbrella crate; no internal crate may depend on it (spec §2.10, §5.6).
const FACADE_CRATE: &str = "obscur-kit";

/// Builds the workspace dependency graph from member manifests and runs the
/// layering, facade-ban and cycle checks.
pub(crate) fn check(root: &Path) -> Vec<Violation> {
    let (graph, mut out) = WorkspaceGraph::from_root(root);
    out.extend(super::super::dependency_direction::check(&graph));
    out.extend(layering_violations(&graph));
    out.extend(facade_violations(&graph));
    out.extend(cycle_violations(&graph));
    out
}

/// Flags any `obscur-*` edge that crosses an allowed dependency direction.
fn layering_violations(graph: &WorkspaceGraph) -> Vec<Violation> {
    let mut out = Vec::new();
    for source in &graph.nodes {
        for dep in &source.deps {
            // Facade edges are covered by the dedicated facade-ban check.
            if dep == FACADE_CRATE {
                continue;
            }
            let Some(target) = graph.node(dep) else {
                continue;
            };
            if !source.layer.allows(target.layer) {
                out.push(Violation::new(
                    RULE_LAYERING,
                    &source.manifest,
                    format!(
                        "{} ({}) must not depend on {} ({}): {} may not depend on {}",
                        source.name,
                        source.layer.label(),
                        target.name,
                        target.layer.label(),
                        source.layer.label(),
                        target.layer.label(),
                    ),
                ));
            }
        }
    }
    out
}

/// Flags any non-facade crate that depends on the umbrella `obscur-kit`.
fn facade_violations(graph: &WorkspaceGraph) -> Vec<Violation> {
    let mut out = Vec::new();
    for source in &graph.nodes {
        if source.layer == Layer::Facade {
            continue;
        }
        if source.deps.iter().any(|d| d == FACADE_CRATE) {
            out.push(Violation::new(
                RULE_FACADE,
                &source.manifest,
                format!(
                    "internal crate '{}' must not depend on the facade '{FACADE_CRATE}'",
                    source.name,
                ),
            ));
        }
    }
    out
}

/// Flags the first dependency cycle among workspace crates, if any.
fn cycle_violations(graph: &WorkspaceGraph) -> Vec<Violation> {
    let Some(cycle) = graph.find_cycle() else {
        return Vec::new();
    };
    let anchor = graph.node(&cycle[0]).map_or_else(
        || Path::new("Cargo.toml").to_path_buf(),
        |n| n.manifest.clone(),
    );
    vec![Violation::new(
        RULE_CYCLE,
        &anchor,
        format!("workspace dependency cycle: {}", cycle.join(" -> ")),
    )]
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{check, facade_violations, layering_violations};
    use crate::rule::dependency_graph::crate_node::CrateNode;
    use crate::rule::dependency_graph::graph::WorkspaceGraph;
    use crate::rule::dependency_graph::layer::Layer;

    fn node(name: &str, layer: Layer, deps: &[&str]) -> CrateNode {
        CrateNode {
            name: name.to_owned(),
            layer,
            manifest: PathBuf::from(format!("{name}/Cargo.toml")),
            deps: deps.iter().map(|d| (*d).to_owned()).collect(),
            dev_deps: Vec::new(),
        }
    }

    #[test]
    fn foundation_depending_on_domain_is_flagged() {
        let graph = WorkspaceGraph {
            nodes: vec![
                node("obscur-foundy", Layer::Foundation, &["obscur-domy"]),
                node("obscur-domy", Layer::Domain, &[]),
            ],
        };
        let v = layering_violations(&graph);
        assert_eq!(v.len(), 1);
    }

    #[test]
    fn allowed_domain_to_foundation_is_not_flagged() {
        let graph = WorkspaceGraph {
            nodes: vec![
                node("obscur-domy", Layer::Domain, &["obscur-foundy"]),
                node("obscur-foundy", Layer::Foundation, &[]),
            ],
        };
        assert!(layering_violations(&graph).is_empty());
    }

    #[test]
    fn non_facade_depending_on_facade_is_flagged() {
        let graph = WorkspaceGraph {
            nodes: vec![
                node("obscur-domy", Layer::Domain, &["obscur-kit"]),
                node("obscur-kit", Layer::Facade, &["obscur-domy"]),
            ],
        };
        let v = facade_violations(&graph);
        assert_eq!(v.len(), 1);
        // The facade edge must not also be reported as a layering violation.
        assert!(layering_violations(&graph).is_empty());
    }

    #[test]
    fn facade_may_depend_on_anything() {
        let graph = WorkspaceGraph {
            nodes: vec![
                node(
                    "obscur-kit",
                    Layer::Facade,
                    &["obscur-domy", "obscur-adapt"],
                ),
                node("obscur-domy", Layer::Domain, &[]),
                node("obscur-adapt", Layer::Adapters, &["obscur-domy"]),
            ],
        };
        assert!(facade_violations(&graph).is_empty());
        assert!(layering_violations(&graph).is_empty());
    }

    #[test]
    fn duplicate_package_names_report_both_manifests() {
        let workspace = tempfile::tempdir().unwrap();
        let first = workspace.path().join("crates/domain/first");
        let second = workspace.path().join("tools/second");
        for member in [&first, &second] {
            std::fs::create_dir_all(member).unwrap();
            std::fs::write(member.join("Cargo.toml"), "[package]\nname = \"same\"").unwrap();
        }
        let violations = check(workspace.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].file, second.join("Cargo.toml"));
        assert!(
            violations[0]
                .detail
                .contains("crates/domain/first/Cargo.toml")
                || violations[0]
                    .detail
                    .contains(r"crates\domain\first\Cargo.toml")
        );
    }

    #[test]
    fn workspace_aliases_are_checked_against_the_package_layer() {
        let workspace = tempfile::tempdir().unwrap();
        std::fs::write(
            workspace.path().join("Cargo.toml"),
            "[workspace.dependencies]\nwindow = { package = 'obscur-window', path = 'crates/adapters/window' }",
        )
        .unwrap();
        for (member, manifest) in [
            (
                "crates/domain/model",
                "[package]\nname = 'obscur-model'\n[dependencies]\nwindow.workspace = true",
            ),
            (
                "crates/adapters/window",
                "[package]\nname = 'obscur-window'",
            ),
        ] {
            let member = workspace.path().join(member);
            std::fs::create_dir_all(&member).unwrap();
            std::fs::write(member.join("Cargo.toml"), manifest).unwrap();
        }
        let violations = check(workspace.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, "dependency-layering");
        assert!(violations[0].detail.contains("obscur-window"));
    }

    #[test]
    fn real_workspace_passes_all_graph_checks() {
        // Reads real member manifests (no cargo invocation) and asserts the
        // current clean tree has no graph violations.
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .expect("workspace root")
            .to_path_buf();
        let violations = check(&root);
        assert!(
            violations.is_empty(),
            "unexpected graph violations: {}",
            violations
                .iter()
                .map(|v| format!("{v}"))
                .collect::<Vec<_>>()
                .join("; ")
        );
    }
}
