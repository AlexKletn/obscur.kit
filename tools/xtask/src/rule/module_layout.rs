//! Rule: `mod.rs` and `lib.rs` contain only structure, not entities (spec §3.1).

use std::ffi::OsStr;
use std::path::Path;

use syn::{Item, Visibility, spanned::Spanned};

use crate::violation::{Violation, collect, read_text};
use crate::workspace_manifests::read_manifest;

const RULE: &str = "module-layout";

#[doc = include_str!("../../docs/api/rule/module_layout.doc/is_proc_macro_crate_root.md")]
fn is_proc_macro_crate_root(file: &Path) -> bool {
    if file.file_name() != Some(OsStr::new("lib.rs")) {
        return false;
    }
    let Some(crate_root) = file.parent().and_then(Path::parent) else {
        return false;
    };
    read_manifest(RULE, &crate_root.join("Cargo.toml")).is_ok_and(|manifest| {
        manifest
            .get("lib")
            .and_then(|table| table.get("proc-macro"))
            .and_then(toml_edit::Item::as_bool)
            == Some(true)
    })
}

/// Checks every `mod.rs`/`lib.rs` under the workspace.
pub(crate) fn check(root: &Path) -> Vec<Violation> {
    let mut files = Vec::new();
    collect(root, "mod.rs", &mut files);
    collect(root, "lib.rs", &mut files);

    let mut out = Vec::new();
    for file in files {
        let allows_proc_macros = is_proc_macro_crate_root(&file);
        let text = match read_text(RULE, &file) {
            Ok(text) => text,
            Err(violation) => {
                out.push(violation);
                continue;
            }
        };
        out.extend(check_source(&file, &text, allows_proc_macros));
    }
    out
}

fn check_source(file: &Path, source: &str, allows_proc_macros: bool) -> Vec<Violation> {
    let syntax = match syn::parse_file(source) {
        Ok(syntax) => syntax,
        Err(error) => {
            return vec![Violation::new(
                RULE,
                file,
                format!("line {}: {error}", error.span().start().line),
            )];
        }
    };
    syntax
        .items
        .iter()
        .filter_map(|item| {
            match item {
                Item::Use(import) if !matches!(import.vis, Visibility::Inherited) => return None,
                Item::Mod(module) if module.content.is_none() => return None,
                Item::Fn(function)
                    if allows_proc_macros
                        && function.attrs.iter().any(|attribute| {
                            let path = attribute.path();
                            path.is_ident("proc_macro")
                                || path.is_ident("proc_macro_derive")
                                || path.is_ident("proc_macro_attribute")
                        }) =>
                {
                    return None;
                }
                _ => {}
            }
            Some(Violation::new(
                RULE,
                file,
                format!(
                    "line {}: only module declarations and re-exports allowed",
                    item.span().start().line
                ),
            ))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proc_macro_exemption_requires_the_library_target_setting() {
        let workspace = tempfile::tempdir().unwrap();
        let source_dir = workspace.path().join("src");
        std::fs::create_dir(&source_dir).unwrap();
        let source = source_dir.join("lib.rs");
        std::fs::write(
            &source,
            "#[proc_macro]\npub fn macro_entry() {}\nfn helper() {}",
        )
        .unwrap();
        let manifest = workspace.path().join("Cargo.toml");

        std::fs::write(&manifest, "[lib]\nproc-macro=true # comment").unwrap();
        let violations = check(workspace.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].detail.contains("line 3:"));

        std::fs::write(&manifest, "[package.metadata]\nproc-macro = true").unwrap();
        let violations = check(workspace.path());
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].file, source);
    }

    #[test]
    fn permits_formatted_attributes_and_nested_reexports() {
        let source = r#"
            #[cfg(any(
                feature = "layer",
                all(feature = "window", target_os = "linux")
            ))]
            mod platform;
            pub(in crate::host) use self::platform::{frame::Frame, window::Window};
        "#;
        assert!(check_source(Path::new("mod.rs"), source, false).is_empty());
    }

    #[test]
    fn rejects_definitions_even_when_they_start_with_an_attribute() {
        let source = "#[cfg(test)] const VALUE: u32 = 1;\nmod inner { fn helper() {} }";
        let violations = check_source(Path::new("mod.rs"), source, false);
        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn rejects_private_imports_in_module_roots() {
        let source = "mod parser;\nuse parser::Token;\npub(super) use parser::Parser;";
        let violations = check_source(Path::new("mod.rs"), source, false);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].detail.contains("line 2:"));
    }
}
