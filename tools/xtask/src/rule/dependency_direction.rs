//! Rule: pure crates never depend on platform/adapter crates (spec §5.6).

use std::fs;
use std::path::Path;

use crate::violation::{Violation, collect};

const RULE: &str = "dependency-direction";

const PURE_DIRS: &[&str] = &["crates/foundation", "crates/domain", "crates/application"];

// External implementation crates that must stay behind adapters.
const BANNED_DEPS: &[&str] = &[
    "winit",
    "wgpu",
    "vello",
    "taffy",
    "parley",
    "accesskit",
    "wasmtime",
    "tokio",
    "raw-window-handle",
];

/// Scans pure-crate `Cargo.toml` files for banned dependencies.
pub(crate) fn check(root: &Path) -> Vec<Violation> {
    let mut out = Vec::new();
    for dir in PURE_DIRS {
        let mut manifests = Vec::new();
        collect(&root.join(dir), "Cargo.toml", &mut manifests);
        for manifest in manifests {
            let Ok(text) = fs::read_to_string(&manifest) else {
                continue;
            };
            for line in text.lines() {
                let trimmed = line.trim_start();
                for dep in BANNED_DEPS {
                    if trimmed.starts_with(dep)
                        && trimmed[dep.len()..]
                            .trim_start()
                            .starts_with(['=', '.', ' '])
                    {
                        out.push(Violation::new(
                            RULE,
                            &manifest,
                            format!("pure crate must not depend on '{dep}' (use an adapter)"),
                        ));
                    }
                }
            }
        }
    }
    out
}
