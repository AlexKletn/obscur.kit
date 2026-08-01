#![doc = include_str!("check.doc/_module.md")]

use std::path::Path;

use super::graph::WorkspaceGraph;
use super::layer::Layer;
use crate::violation::Violation;

const RULE_LAYERING: &str = "dependency-layering";
const RULE_FACADE: &str = "facade-dependency";
const RULE_CYCLE: &str = "dependency-cycle";

#[doc = include_str!("check.doc/FACADE_CRATE.md")]
const FACADE_CRATE: &str = "obscur-kit";

#[doc = include_str!("check.doc/check.md")]
pub(crate) fn check(root: &Path) -> Vec<Violation> {
    let graph = WorkspaceGraph::from_root(root);
    let mut out = Vec::new();
    out.extend(layering_violations(&graph));
    out.extend(facade_violations(&graph));
    out.extend(cycle_violations(&graph));
    out
}

#[doc = include_str!("check.doc/layering_violations.md")]
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

#[doc = include_str!("check.doc/facade_violations.md")]
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

#[doc = include_str!("check.doc/cycle_violations.md")]
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
