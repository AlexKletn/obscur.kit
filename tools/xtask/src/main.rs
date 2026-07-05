//! Workspace architecture checks (spec §3.16, §5.8).
//!
//! Usage: `cargo run -p xtask -- check`.
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

mod rule;
mod violation;

use std::path::Path;
use std::process::ExitCode;

use rule::{
    dependency_direction, dependency_graph, forbidden_file_name, module_layout, unsafe_boundary,
};
use violation::Violation;

fn main() -> ExitCode {
    let cmd = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "check".to_owned());
    if cmd != "check" {
        eprintln!("usage: cargo run -p xtask -- check");
        return ExitCode::FAILURE;
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root")
        .to_path_buf();

    let mut violations: Vec<Violation> = Vec::new();
    violations.extend(module_layout::check(&root));
    violations.extend(forbidden_file_name::check(&root));
    violations.extend(unsafe_boundary::check(&root));
    violations.extend(dependency_direction::check(&root));
    violations.extend(dependency_graph::check(&root));

    if violations.is_empty() {
        println!("architecture checks passed");
        ExitCode::SUCCESS
    } else {
        for v in &violations {
            println!("{v}");
        }
        eprintln!("\n{} architecture violation(s)", violations.len());
        ExitCode::FAILURE
    }
}
