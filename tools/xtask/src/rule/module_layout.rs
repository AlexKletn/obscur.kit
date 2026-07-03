//! Rule: `mod.rs` and `lib.rs` contain only structure, not entities (spec §3.1).

use std::fs;
use std::path::Path;

use crate::violation::{Violation, collect};

const RULE: &str = "module-layout";

/// Checks every `mod.rs`/`lib.rs` under the workspace.
pub(crate) fn check(root: &Path) -> Vec<Violation> {
    let mut files = Vec::new();
    collect(root, "mod.rs", &mut files);
    collect(root, "lib.rs", &mut files);

    let mut out = Vec::new();
    for file in files {
        let Ok(text) = fs::read_to_string(&file) else {
            continue;
        };
        for (i, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if is_structural(line) {
                continue;
            }
            out.push(Violation::new(
                RULE,
                &file,
                format!(
                    "line {}: only module declarations and re-exports allowed, found: {line}",
                    i + 1
                ),
            ));
        }
    }
    out
}

fn is_structural(line: &str) -> bool {
    line.is_empty()
        || line.starts_with("//")
        || line.starts_with("/*")
        || line.starts_with('*')
        || line.starts_with("#!")
        || line.starts_with("#[")
        || line.starts_with("mod ")
        || line.starts_with("pub mod ")
        || line.starts_with("pub(crate) mod ")
        || line.starts_with("use ")
        || line.starts_with("pub use ")
        || line.starts_with("pub(crate) use ")
        || line == "}"
        || line.starts_with("};")
        || is_reexport_continuation(line)
}

// A `pub use path::{ ... }` block may wrap across lines; its inner lines are
// bare identifiers or paths ending in a comma. Accept those conservatively.
fn is_reexport_continuation(line: &str) -> bool {
    line.chars()
        .all(|c| c.is_alphanumeric() || matches!(c, '_' | ':' | ',' | ' '))
        && line.ends_with(',')
}
