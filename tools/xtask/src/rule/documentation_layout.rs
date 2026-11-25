//! Keeps standalone Markdown documentation outside crate source trees.

use std::path::Path;

use crate::violation::{Violation, collect_matching};
use crate::workspace_manifests::workspace_manifests;

pub(crate) fn check(root: &Path) -> Vec<Violation> {
    let manifests = workspace_manifests(root);
    let mut markdown = Vec::new();
    for crate_root in manifests.iter().filter_map(|manifest| manifest.parent()) {
        collect_matching(
            &crate_root.join("src"),
            &|path| {
                path.extension().is_some_and(|ext| {
                    ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown")
                })
            },
            &mut markdown,
        );
    }
    markdown.sort_unstable();
    markdown.dedup();
    markdown
        .into_iter()
        .map(|file| {
            Violation::new(
                "documentation-layout",
                &file,
                "move standalone Markdown from src/ to the crate's docs/ directory",
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::check;

    #[test]
    fn reports_markdown_in_crate_sources_but_accepts_crate_docs() {
        let workspace = tempfile::tempdir().unwrap();
        let crate_root = workspace.path().join("crates/domain/sample");
        let module = crate_root.join("src/module");
        let docs = crate_root.join("docs");
        fs::create_dir_all(&module).unwrap();
        fs::create_dir_all(&docs).unwrap();
        fs::write(
            crate_root.join("Cargo.toml"),
            "[package]\nname = \"sample\"",
        )
        .unwrap();
        fs::write(module.join("example.rs"), "").unwrap();
        fs::write(docs.join("example.md"), "# Example").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&module, module.join("cycle")).unwrap();

        assert!(check(workspace.path()).is_empty());

        let misplaced = module.join("example.MD");
        fs::write(&misplaced, "# Example").unwrap();
        let alternate_extension = module.join("example.markdown");
        fs::write(&alternate_extension, "# Alternate").unwrap();

        let violations = check(workspace.path());
        assert_eq!(violations.len(), 2);
        assert!(
            violations
                .iter()
                .all(|violation| violation.rule == "documentation-layout")
        );
        assert_eq!(violations[0].file, misplaced);
        assert_eq!(violations[1].file, alternate_extension);
    }
}
