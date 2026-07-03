//! Rule: pure crates forbid unsafe (spec §1.8, §5.7).

use std::fs;
use std::path::Path;

use crate::violation::{Violation, collect};

const RULE: &str = "unsafe-boundary";

// Directories whose crates are "pure" and must declare `#![forbid(unsafe_code)]`.
const PURE_DIRS: &[&str] = &["crates/foundation", "crates/domain", "crates/application"];

/// Checks that every pure crate's `lib.rs` forbids unsafe code.
pub(crate) fn check(root: &Path) -> Vec<Violation> {
    let mut out = Vec::new();
    for dir in PURE_DIRS {
        let mut libs = Vec::new();
        collect(&root.join(dir), "lib.rs", &mut libs);
        for lib in libs {
            let Ok(text) = fs::read_to_string(&lib) else {
                continue;
            };
            if !text.contains("#![forbid(unsafe_code)]") {
                out.push(Violation::new(
                    RULE,
                    &lib,
                    "pure crate must declare #![forbid(unsafe_code)]",
                ));
            }
        }
    }
    out
}
