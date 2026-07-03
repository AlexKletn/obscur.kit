//! Rule: no dumping-ground file names (spec §3.1).

use std::path::Path;

use crate::violation::{Violation, collect_rs};

const RULE: &str = "forbidden-file-name";

const FORBIDDEN: &[&str] = &[
    "utils.rs",
    "helpers.rs",
    "common.rs",
    "misc.rs",
    "types.rs",
    "models.rs",
    "manager.rs",
];

/// Checks for banned generic file names anywhere in the workspace.
pub(crate) fn check(root: &Path) -> Vec<Violation> {
    let mut files = Vec::new();
    collect_rs(root, &mut files);

    let mut out = Vec::new();
    for file in files {
        if let Some(name) = file.file_name().and_then(|n| n.to_str())
            && FORBIDDEN.contains(&name)
        {
            out.push(Violation::new(
                RULE,
                &file,
                format!("banned dumping-ground name '{name}'"),
            ));
        }
    }
    out
}
