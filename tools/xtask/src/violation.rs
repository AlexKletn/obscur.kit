//! A single architecture-rule violation and file-walking helpers.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

/// One violation of an architecture rule, anchored to a file.
pub(crate) struct Violation {
    /// Rule that was violated.
    pub(crate) rule: &'static str,
    /// Offending file.
    pub(crate) file: PathBuf,
    /// Explanation.
    pub(crate) detail: String,
}

impl Violation {
    /// Builds a violation.
    pub(crate) fn new(rule: &'static str, file: &Path, detail: impl Into<String>) -> Self {
        Self {
            rule,
            file: file.to_path_buf(),
            detail: detail.into(),
        }
    }
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {}: {}",
            self.rule,
            self.file.display(),
            self.detail
        )
    }
}

/// Reads a rule input and retains the path and I/O error in its diagnostic.
pub(crate) fn read_text(rule: &'static str, path: &Path) -> Result<String, Violation> {
    fs::read_to_string(path)
        .map_err(|error| Violation::new(rule, path, format!("cannot read file: {error}")))
}

/// Recursively collects files under `dir` whose name matches `wanted`, skipping
/// any `target` directories.
pub(crate) fn collect(dir: &Path, wanted: &str, out: &mut Vec<PathBuf>) {
    collect_matching(
        dir,
        &|path| path.file_name().is_some_and(|n| n == wanted),
        out,
    );
}

/// Recursively collects all `.rs` files under `dir`, skipping `target`.
pub(crate) fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    collect_matching(
        dir,
        &|path| path.extension().is_some_and(|ext| ext == "rs"),
        out,
    );
}

/// Walks real directories only, so links cannot lead outside the tree or form cycles.
pub(crate) fn collect_matching(
    dir: &Path,
    matches: &impl Fn(&Path) -> bool,
    out: &mut Vec<PathBuf>,
) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_dir() {
            if path.file_name().is_some_and(|n| n == "target") {
                continue;
            }
            collect_matching(&path, matches, out);
        } else if matches(&path) {
            out.push(path);
        }
    }
}
