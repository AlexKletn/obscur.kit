//! Rule: pure crates never depend on platform/adapter crates (spec §5.6).

use super::dependency_graph::{graph::WorkspaceGraph, layer::Layer};
use crate::violation::Violation;

const RULE: &str = "dependency-direction";

// Platform/GPU/IO implementation crates that must stay behind adapters (spec
// §5.6). NOTE: `taffy` is deliberately NOT here — it is a pure-Rust layout
// ALGORITHM library, and spec §3.6 mandates its integration live *inside*
// `obscur-layout` (`algorithm/taffy/`) with `taffy::Style`/`taffy::NodeId` kept
// out of the public API. It is a domain algorithm dependency, not a platform
// adapter.
const BANNED_DEPS: &[&str] = &[
    "winit",
    "wgpu",
    "vello",
    "parley",
    "accesskit",
    "wasmtime",
    "tokio",
    "raw-window-handle",
];

/// Checks resolved dependencies of the workspace's pure layers.
pub(crate) fn check(graph: &WorkspaceGraph) -> Vec<Violation> {
    let mut violations = Vec::new();
    for krate in &graph.nodes {
        if !matches!(
            krate.layer,
            Layer::Foundation | Layer::Domain | Layer::Application
        ) {
            continue;
        }
        for dependency in krate.deps.iter().chain(&krate.dev_deps) {
            if BANNED_DEPS.contains(&dependency.as_str()) {
                violations.push(Violation::new(
                    RULE,
                    &krate.manifest,
                    format!("pure crate must not depend on '{dependency}' (use an adapter)"),
                ));
            }
        }
    }
    violations
}

#[cfg(test)]
mod tests {
    #[test]
    fn rejects_an_inherited_platform_package_in_target_dev_dependencies() {
        let workspace = tempfile::tempdir().unwrap();
        let root = workspace.path();
        let member = root.join("crates/domain/model");
        std::fs::create_dir_all(&member).unwrap();
        std::fs::write(
            root.join("Cargo.toml"),
            "[workspace.dependencies]\ngraphics = { package = 'wgpu', version = '29' }",
        )
        .unwrap();
        let manifest = member.join("Cargo.toml");
        std::fs::write(
            &manifest,
            "[package]\nname = 'obscur-model'\n[target.'cfg(unix)'.dev-dependencies]\ngraphics.workspace = true",
        )
        .unwrap();

        let violations = crate::rule::dependency_graph::check(root);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, super::RULE);
        assert_eq!(violations[0].file, manifest);
        assert!(violations[0].detail.contains("'wgpu'"));
    }
}
