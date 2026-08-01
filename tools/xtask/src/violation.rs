#![doc = include_str!("violation.doc/_module.md")]

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

#[doc = include_str!("violation.doc/Violation.md")]
pub(crate) struct Violation {
    #[doc = include_str!("violation.doc/rule.md")]
    pub(crate) rule: &'static str,
    #[doc = include_str!("violation.doc/file.md")]
    pub(crate) file: PathBuf,
    #[doc = include_str!("violation.doc/detail.md")]
    pub(crate) detail: String,
}

impl Violation {
    #[doc = include_str!("violation.doc/new.md")]
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

#[doc = include_str!("violation.doc/collect.md")]
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

#[doc = include_str!("violation.doc/collect_rs.md")]
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
