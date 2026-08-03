#![doc = include_str!("module_layout.doc/_module.md")]

use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use crate::violation::{Violation, collect};

const RULE: &str = "module-layout";

#[doc = include_str!("module_layout.doc/is_proc_macro_crate_root.md")]
fn is_proc_macro_crate_root(file: &Path) -> bool {
    if file.file_name() != Some(OsStr::new("lib.rs")) {
        return false;
    }
    let Some(crate_root) = file.parent().and_then(Path::parent) else {
        return false;
    };
    fs::read_to_string(crate_root.join("Cargo.toml"))
        .is_ok_and(|text| text.lines().any(|line| line.trim() == "proc-macro = true"))
}

#[doc = include_str!("module_layout.doc/check.md")]
pub(crate) fn check(root: &Path) -> Vec<Violation> {
    let mut files = Vec::new();
    collect(root, "mod.rs", &mut files);
    collect(root, "lib.rs", &mut files);

    let mut out = Vec::new();
    for file in files {
        if is_proc_macro_crate_root(&file) {
            continue;
        }
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
