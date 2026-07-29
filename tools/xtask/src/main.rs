//! Workspace architecture checks (spec §3.16, §5.8).
//!
//! Usage: `cargo run -p xtask -- check` (source rules, fast) or
//! `cargo run -p xtask -- consumer` (builds the out-of-workspace facade
//! consumer, so it costs a real compile and stays out of `check`).
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

mod consumer;
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
    if !matches!(cmd.as_str(), "check" | "consumer") {
        eprintln!("usage: cargo run -p xtask -- [check|consumer]");
        return ExitCode::FAILURE;
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root")
        .to_path_buf();

    let mut violations: Vec<Violation> = Vec::new();
    if cmd == "consumer" {
        violations.extend(consumer::check(&root));
        return report(&violations);
    }
    violations.extend(module_layout::check(&root));
    violations.extend(forbidden_file_name::check(&root));
    violations.extend(unsafe_boundary::check(&root));
    violations.extend(dependency_direction::check(&root));
    violations.extend(dependency_graph::check(&root));

    report(&violations)
}

/// Prints `violations` and turns them into the process's exit status.
fn report(violations: &[Violation]) -> ExitCode {
    if violations.is_empty() {
        println!("architecture checks passed");
        ExitCode::SUCCESS
    } else {
        for v in violations {
            println!("{v}");
        }
        eprintln!("\n{} architecture violation(s)", violations.len());
        ExitCode::FAILURE
    }
}
