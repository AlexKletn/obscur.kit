//! Rule: pure crates forbid unsafe (spec §1.8, §5.7).

use std::path::Path;

use syn::{Meta, Token, punctuated::Punctuated};

use crate::violation::{Violation, read_text};

use crate::workspace_manifests::pure_manifests;

const RULE: &str = "unsafe-boundary";

/// Checks that every pure crate's `lib.rs` forbids unsafe code.
pub(crate) fn check(root: &Path) -> Vec<Violation> {
    let mut out = Vec::new();
    for lib in
        pure_manifests(root).filter_map(|manifest| Some(manifest.parent()?.join("src/lib.rs")))
    {
        let text = match read_text(RULE, &lib) {
            Ok(text) => text,
            Err(violation) => {
                out.push(violation);
                continue;
            }
        };
        if !forbids_unsafe(&text) {
            out.push(Violation::new(
                RULE,
                &lib,
                "pure crate must declare #![forbid(unsafe_code)]",
            ));
        }
    }
    out
}

fn forbids_unsafe(source: &str) -> bool {
    syn::parse_file(source).is_ok_and(|file| {
        file.attrs.iter().any(|attribute| {
            attribute.path().is_ident("forbid")
                && attribute
                    .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                    .is_ok_and(|arguments| {
                        arguments.iter().any(
                            |argument| matches!(argument, Meta::Path(path) if path.is_ident("unsafe_code")),
                        )
                    })
        })
    })
}

#[cfg(test)]
mod tests {
    use super::{check, forbids_unsafe};
    use std::fs;

    #[test]
    fn requires_an_unconditional_crate_attribute() {
        assert!(forbids_unsafe("#![forbid(unsafe_code, missing_docs)]"));
        assert!(!forbids_unsafe("// #![forbid(unsafe_code)]"));
        assert!(!forbids_unsafe("#![cfg_attr(unix, forbid(unsafe_code))]"));
        assert!(!forbids_unsafe("mod inner { #![forbid(unsafe_code)] }"));
    }

    #[test]
    fn reports_crate_source_that_cannot_be_read_as_rust() {
        let workspace = tempfile::tempdir().unwrap();
        let source_directory = workspace.path().join("crates/domain/sample/src");
        fs::create_dir_all(&source_directory).unwrap();
        fs::write(
            source_directory.join("../Cargo.toml"),
            "[package]\nname = \"sample\"",
        )
        .unwrap();
        let source = source_directory.join("lib.rs");
        fs::write(&source, [0xff]).unwrap();
        let violations = check(workspace.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].file, source);
        assert!(violations[0].detail.contains("cannot read file"));
    }
}
