//! The workspace dependency graph, built by scanning member manifests, plus
//! DFS cycle detection.

use std::collections::HashMap;
use std::path::Path;
use toml_edit::Item;

use super::crate_node::CrateNode;
use super::layer::Layer;
use super::visit_mark::VisitMark;
use crate::violation::Violation;
use crate::workspace_manifests::{read_manifest, workspace_manifests};

/// The full set of workspace crate nodes and their `obscur-*` edges.
pub(crate) struct WorkspaceGraph {
    /// All member crates discovered under `crates/*/*` and `tools/*`.
    pub(crate) nodes: Vec<CrateNode>,
}

impl WorkspaceGraph {
    /// Builds the graph by scanning every workspace member manifest under
    /// `crates/<layer>/<crate>/Cargo.toml` and `tools/<crate>/Cargo.toml`.
    pub(crate) fn from_root(root: &Path) -> (Self, Vec<Violation>) {
        let mut graph = Self { nodes: Vec::new() };
        let mut violations = Vec::new();
        let workspace_manifest = root.join("Cargo.toml");
        let workspace = if workspace_manifest.is_file() {
            match read_manifest("dependency-manifest", &workspace_manifest) {
                Ok(document) => Some(document),
                Err(violation) => {
                    violations.push(violation);
                    None
                }
            }
        } else {
            None
        };
        let workspace_deps = workspace
            .as_ref()
            .and_then(|document| document.get("workspace"))
            .and_then(|table| table.get("dependencies"))
            .and_then(Item::as_table_like);

        for manifest in workspace_manifests(root) {
            let layer_dir = if manifest.starts_with(root.join("tools")) {
                Some("tools")
            } else {
                manifest
                    .ancestors()
                    .nth(2)
                    .and_then(Path::file_name)
                    .and_then(|name| name.to_str())
            };
            let Some(layer) = layer_dir.and_then(Layer::from_dir) else {
                violations.push(Violation::new(
                    "dependency-layering",
                    &manifest,
                    "crate has no recognized architecture layer",
                ));
                continue;
            };
            match CrateNode::parse(manifest, layer, workspace_deps) {
                Ok(node) => {
                    if let Some(first) = graph.node(&node.name) {
                        violations.push(Violation::new(
                            "dependency-manifest",
                            &node.manifest,
                            format!(
                                "duplicate package '{}' also declared in {}",
                                node.name,
                                first.manifest.display()
                            ),
                        ));
                    } else {
                        graph.nodes.push(node);
                    }
                }
                Err(violation) => violations.push(violation),
            }
        }

        (graph, violations)
    }

    /// Looks up a node by crate name.
    pub(crate) fn node(&self, name: &str) -> Option<&CrateNode> {
        self.nodes.iter().find(|n| n.name == name)
    }

    /// Returns the first dependency cycle found (as an ordered path of crate
    /// names, closing back on the first), or `None` if the graph is acyclic.
    /// Only edges between known workspace nodes are followed.
    pub(crate) fn find_cycle(&self) -> Option<Vec<String>> {
        let mut color: HashMap<&str, VisitMark> = self
            .nodes
            .iter()
            .map(|n| (n.name.as_str(), VisitMark::White))
            .collect();
        let mut path: Vec<&str> = Vec::new();
        for start in &self.nodes {
            if color[start.name.as_str()] == VisitMark::White
                && let Some(cycle) = self.visit(start.name.as_str(), &mut color, &mut path)
            {
                return Some(cycle);
            }
        }
        None
    }

    /// Recursive DFS helper for [`find_cycle`](Self::find_cycle).
    fn visit<'a>(
        &'a self,
        name: &'a str,
        color: &mut HashMap<&'a str, VisitMark>,
        path: &mut Vec<&'a str>,
    ) -> Option<Vec<String>> {
        color.insert(name, VisitMark::Gray);
        path.push(name);
        if let Some(node) = self.node(name) {
            for dep in &node.deps {
                match color.get(dep.as_str()) {
                    Some(VisitMark::Gray) => {
                        let start = path.iter().position(|&n| n == dep).unwrap_or(0);
                        let mut cycle: Vec<String> =
                            path[start..].iter().map(|&n| n.to_owned()).collect();
                        cycle.push(dep.clone());
                        return Some(cycle);
                    }
                    Some(VisitMark::White) => {
                        if let Some(cycle) = self.visit(dep.as_str(), color, path) {
                            return Some(cycle);
                        }
                    }
                    _ => {}
                }
            }
        }
        path.pop();
        color.insert(name, VisitMark::Black);
        None
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::WorkspaceGraph;
    use crate::rule::dependency_graph::crate_node::CrateNode;
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
    fn acyclic_graph_has_no_cycle() {
        let graph = WorkspaceGraph {
            nodes: vec![
                node("a", Layer::Domain, &["b"]),
                node("b", Layer::Foundation, &[]),
            ],
        };
        assert!(graph.find_cycle().is_none());
    }

    #[test]
    fn direct_cycle_is_detected() {
        let graph = WorkspaceGraph {
            nodes: vec![
                node("a", Layer::Domain, &["b"]),
                node("b", Layer::Domain, &["a"]),
            ],
        };
        let cycle = graph.find_cycle().expect("cycle expected");
        assert!(cycle.contains(&"a".to_owned()));
        assert!(cycle.contains(&"b".to_owned()));
    }

    #[test]
    fn indirect_cycle_is_detected() {
        let graph = WorkspaceGraph {
            nodes: vec![
                node("a", Layer::Domain, &["b"]),
                node("b", Layer::Domain, &["c"]),
                node("c", Layer::Domain, &["a"]),
            ],
        };
        assert!(graph.find_cycle().is_some());
    }
}
