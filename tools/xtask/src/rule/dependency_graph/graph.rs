#![doc = include_str!("graph.doc/_module.md")]

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::crate_node::CrateNode;
use super::layer::Layer;

#[doc = include_str!("graph.doc/WorkspaceGraph.md")]
pub(crate) struct WorkspaceGraph {
    #[doc = include_str!("graph.doc/nodes.md")]
    pub(crate) nodes: Vec<CrateNode>,
}

impl WorkspaceGraph {
    #[doc = include_str!("graph.doc/from_root.md")]
    pub(crate) fn from_root(root: &Path) -> Self {
        let mut nodes = Vec::new();

        if let Ok(layers) = fs::read_dir(root.join("crates")) {
            for layer_entry in layers.flatten() {
                let layer_path = layer_entry.path();
                let Some(layer) = layer_entry.file_name().to_str().and_then(Layer::from_dir) else {
                    continue;
                };
                if let Ok(crate_dirs) = fs::read_dir(&layer_path) {
                    for crate_dir in crate_dirs.flatten() {
                        let manifest = crate_dir.path().join("Cargo.toml");
                        if manifest.is_file()
                            && let Some(node) = CrateNode::parse(manifest, layer)
                        {
                            nodes.push(node);
                        }
                    }
                }
            }
        }

        if let Ok(tool_dirs) = fs::read_dir(root.join("tools")) {
            for tool_dir in tool_dirs.flatten() {
                let manifest = tool_dir.path().join("Cargo.toml");
                if manifest.is_file()
                    && let Some(node) = CrateNode::parse(manifest, Layer::Tools)
                {
                    nodes.push(node);
                }
            }
        }

        Self { nodes }
    }

    #[doc = include_str!("graph.doc/node.md")]
    pub(crate) fn node(&self, name: &str) -> Option<&CrateNode> {
        self.nodes.iter().find(|n| n.name == name)
    }

    #[doc = include_str!("graph.doc/find_cycle.md")]
    pub(crate) fn find_cycle(&self) -> Option<Vec<String>> {
        let mut color: HashMap<&str, Mark> = self
            .nodes
            .iter()
            .map(|n| (n.name.as_str(), Mark::White))
            .collect();
        let mut path: Vec<&str> = Vec::new();
        for start in &self.nodes {
            if color[start.name.as_str()] == Mark::White
                && let Some(cycle) = self.visit(start.name.as_str(), &mut color, &mut path)
            {
                return Some(cycle);
            }
        }
        None
    }

    #[doc = include_str!("graph.doc/visit.md")]
    fn visit<'a>(
        &'a self,
        name: &'a str,
        color: &mut HashMap<&'a str, Mark>,
        path: &mut Vec<&'a str>,
    ) -> Option<Vec<String>> {
        color.insert(name, Mark::Gray);
        path.push(name);
        if let Some(node) = self.node(name) {
            for dep in &node.deps {
                match color.get(dep.as_str()) {
                    Some(Mark::Gray) => {
                        let start = path.iter().position(|&n| n == dep).unwrap_or(0);
                        let mut cycle: Vec<String> =
                            path[start..].iter().map(|&n| n.to_owned()).collect();
                        cycle.push(dep.clone());
                        return Some(cycle);
                    }
                    Some(Mark::White) => {
                        if let Some(cycle) = self.visit(dep.as_str(), color, path) {
                            return Some(cycle);
                        }
                    }
                    _ => {}
                }
            }
        }
        path.pop();
        color.insert(name, Mark::Black);
        None
    }
}

#[doc = include_str!("graph.doc/Mark.md")]
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mark {
    #[doc = include_str!("graph.doc/White.md")]
    White,
    #[doc = include_str!("graph.doc/Gray.md")]
    Gray,
    #[doc = include_str!("graph.doc/Black.md")]
    Black,
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
