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

/// Recursively collects files under `dir` whose name matches `wanted`, skipping
/// any `target` directories.
pub(crate) fn collect(dir: &Path, wanted: &str, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|n| n == "target") {
                continue;
            }
            collect(&path, wanted, out);
        } else if path.file_name().is_some_and(|n| n == wanted) {
            out.push(path);
        }
    }
}

/// Recursively collects all `.rs` files under `dir`, skipping `target`.
pub(crate) fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|n| n == "target") {
                continue;
            }
            collect_rs(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}
