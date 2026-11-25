//! Rule: one file holds one entity, and an entity that outgrows a file is a
//! design fault (spec §3.1).

use std::path::Path;

use crate::violation::{Violation, collect_rs, read_text};

const RULE: &str = "file-size";

#[doc = include_str!("../../docs/api/rule/file_size.doc/MAX_NON_TEST_LINES.md")]
const MAX_NON_TEST_LINES: usize = 1_000;

#[doc = include_str!("../../docs/api/rule/file_size.doc/KNOWN.md")]
const KNOWN: &[(&str, &str)] = &[];

/// Checks that no source file exceeds [`MAX_NON_TEST_LINES`] without being on
/// the ledger, and that no ledger entry has become stale.
pub(crate) fn check(root: &Path) -> Vec<Violation> {
    let mut files = Vec::new();
    collect_rs(root, &mut files);
    files.sort();

    let mut out = Vec::new();
    let mut seen_known = vec![false; KNOWN.len()];
    for file in files {
        let Ok(relative) = file.strip_prefix(root) else {
            continue;
        };
        let relative = relative.to_string_lossy().replace('\\', "/");
        let text = match read_text(RULE, &file) {
            Ok(text) => text,
            Err(violation) => {
                out.push(violation);
                continue;
            }
        };
        let lines = non_test_lines(&text);
        let known = KNOWN.iter().position(|(path, _)| *path == relative);
        if let Some(index) = known {
            seen_known[index] = true;
            // A ledger entry that no longer needs to be one: delete it, so the
            // file cannot grow back through the hole it left behind.
            if lines <= MAX_NON_TEST_LINES {
                out.push(Violation::new(
                    RULE,
                    &file,
                    format!(
                        "is {lines} lines of non-test code, under the \
                         {MAX_NON_TEST_LINES}-line cap — remove its entry from the \
                         `KNOWN` ledger in tools/xtask/src/rule/file_size.rs"
                    ),
                ));
            }
            continue;
        }
        if lines > MAX_NON_TEST_LINES {
            out.push(Violation::new(
                RULE,
                &file,
                format!(
                    "is {lines} lines of non-test code, over the \
                     {MAX_NON_TEST_LINES}-line cap (spec §3.1: one entity per file). \
                     Extract the entity that grew, or add an entry with its reason to \
                     the `KNOWN` ledger in tools/xtask/src/rule/file_size.rs"
                ),
            ));
        }
    }
    // An entry naming a file that no longer exists is a stale claim about the
    // workspace, and this rule reports those the way it reports any other.
    for (index, (path, _)) in KNOWN.iter().enumerate() {
        if !seen_known[index] {
            out.push(Violation::new(
                RULE,
                Path::new(path),
                "is on the `KNOWN` ledger but no such file exists — remove the entry",
            ));
        }
    }
    out
}

#[doc = include_str!("../../docs/api/rule/file_size.doc/non_test_lines.md")]
fn non_test_lines(text: &str) -> usize {
    let lines: Vec<&str> = text.lines().collect();
    let end = lines
        .windows(2)
        .position(|lines| lines[0].trim() == "#[cfg(test)]" && opens_test_module(lines[1].trim()))
        .unwrap_or(lines.len());
    lines[..end]
        .iter()
        .filter(|line| {
            let line = line.trim_start();
            match line.strip_prefix("///") {
                Some(suffix) => suffix.starts_with('/'),
                None => !line.starts_with("//!"),
            }
        })
        .count()
}

/// Whether `line` opens a `mod tests` block, with or without a visibility.
fn opens_test_module(line: &str) -> bool {
    let rest = match line.strip_prefix("pub") {
        Some(rest) => match rest.strip_prefix('(') {
            Some(rest) => match rest.find(')') {
                Some(close) => rest[close + 1..].trim_start(),
                None => return false,
            },
            None => rest.trim_start(),
        },
        None => line,
    };
    rest.strip_prefix("mod ").is_some_and(|rest| {
        let rest = rest.trim_start();
        let end = rest
            .find(|c: char| !c.is_alphanumeric() && c != '_')
            .unwrap_or(rest.len());
        &rest[..end] == "tests"
    })
}

#[cfg(test)]
mod tests {
    use super::non_test_lines;

    #[test]
    fn a_file_without_tests_counts_whole() {
        assert_eq!(non_test_lines("a\nb\nc\n"), 3);
    }

    #[test]
    fn rustdoc_prose_does_not_count_as_code() {
        assert_eq!(non_test_lines("//! Module.\n/// Item.\nfn item() {}\n"), 1);
    }

    #[test]
    fn the_test_module_and_its_attribute_do_not_count() {
        let text = "a\nb\n\n#[cfg(test)]\nmod tests {\n    fn t() {}\n}\n";
        assert_eq!(non_test_lines(text), 3);
    }

    #[test]
    fn a_visible_test_module_is_still_a_test_module() {
        let text = "a\n\n#[cfg(test)]\npub(crate) mod tests {\n}\n";
        assert_eq!(non_test_lines(text), 2);
    }

    #[test]
    fn a_module_merely_named_like_tests_is_not_one() {
        let text = "a\n\n#[cfg(test)]\nmod testing_support {\n}\n";
        assert_eq!(non_test_lines(text), 5);
    }
}
