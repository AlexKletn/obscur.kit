#![doc = include_str!("../../docs/api/rule/main_entity.doc/_module.md")]

use std::path::Path;

use syn::Item;

use crate::violation::{Violation, collect_rs, read_text};
use crate::workspace_manifests::workspace_manifests;

const RULE: &str = "main-entity";

#[doc = include_str!("../../docs/api/rule/main_entity.doc/MARKER.md")]
const MARKER: &str = "xtask:allow(multiple-entities)";

#[doc = include_str!("../../docs/api/rule/main_entity.doc/KNOWN.md")]
const KNOWN: &[(&str, &str)] = &[];

/// Checks that no source file defines several main entities without a marker, and that
/// no marker or ledger entry has gone stale.
pub(crate) fn check(root: &Path) -> Vec<Violation> {
    let mut files = Vec::new();
    for manifest in workspace_manifests(root) {
        if let Some(member) = manifest.parent() {
            collect_rs(&member.join("src"), &mut files);
        }
    }
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

        let entities = match main_entities(&text) {
            Ok(entities) => entities,
            Err(error) => {
                out.push(Violation::new(
                    RULE,
                    &file,
                    format!("line {}: {error}", error.span().start().line),
                ));
                continue;
            }
        };
        let marked = claims_exception(&text);
        let known = KNOWN.iter().position(|(path, _)| *path == relative);
        if let Some(index) = known {
            seen_known[index] = true;
        }

        // The exception is claimed in two places on purpose, so state the two
        // halves separately rather than reporting "not marked" for a file that
        // plainly is.
        if marked && known.is_none() {
            out.push(Violation::new(
                RULE,
                &file,
                format!(
                    "carries the `{MARKER}` marker but has no entry in the `KNOWN` \
                     ledger in tools/xtask/src/rule/main_entity.rs — add the reason \
                     there, or remove the marker"
                ),
            ));
        }
        if known.is_some() && !marked {
            out.push(Violation::new(
                RULE,
                &file,
                format!(
                    "is on the `KNOWN` ledger but carries no `{MARKER}` marker — a \
                     reader of the file must be able to see the exception without \
                     reading xtask"
                ),
            ));
        }

        if entities.len() > 1 {
            if known.is_none() && !marked {
                out.push(Violation::new(
                    RULE,
                    &file,
                    format!(
                        "defines {} file-scope entities ({}) — spec §3.16: one file does not \
                         define several entities without an explicit exception \
                         marker. Give each its own file beside this one, or claim the \
                         exception with a `{MARKER}` comment and an entry in the \
                         `KNOWN` ledger in tools/xtask/src/rule/main_entity.rs",
                        entities.len(),
                        entities.join(", ")
                    ),
                ));
            }
        } else if known.is_some() || marked {
            // The exception has been earned away: withdraw it, so the file
            // cannot grow a second entity back through the hole it left.
            out.push(Violation::new(
                RULE,
                &file,
                format!(
                    "defines {} file-scope entities but still claims the multiple-entity \
                     exception — remove the `{MARKER}` marker and its `KNOWN` entry in \
                     tools/xtask/src/rule/main_entity.rs",
                    entities.len()
                ),
            ));
        }
    }

    // An entry naming a file that no longer exists is a stale claim about the
    // workspace, reported the way any other violation is.
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

#[doc = include_str!("../../docs/api/rule/main_entity.doc/claims_exception.md")]
fn claims_exception(text: &str) -> bool {
    text.lines().any(|line| {
        line.trim_start()
            .strip_prefix("//")
            .is_some_and(|rest| rest.trim_start().starts_with(MARKER))
    })
}

/// File-scope types are candidates regardless of visibility or method count.
fn main_entities(source: &str) -> syn::Result<Vec<String>> {
    Ok(syn::parse_file(source)?
        .items
        .into_iter()
        .filter_map(|item| match item {
            Item::Struct(item) => Some(item.ident),
            Item::Enum(item) => Some(item.ident),
            Item::Trait(item) => Some(item.ident),
            Item::Union(item) => Some(item.ident),
            _ => None,
        })
        .map(|name| name.to_string())
        .collect())
}

#[cfg(test)]
mod tests {
    use super::main_entities;

    #[test]
    fn private_and_restricted_types_count_without_inherent_methods() {
        let source = "pub(super) struct State; enum Mode { Active }";
        assert_eq!(main_entities(source).unwrap(), ["State", "Mode"]);
    }

    #[test]
    fn related_implementations_and_test_doubles_stay_with_the_entity() {
        let source = r#"
            struct State;
            impl State {
                fn label() -> &'static str { r"state { active }" }
            }
            impl Default for State {
                fn default() -> Self { Self }
            }
            #[cfg(test)]
            mod tests { struct Fixture; }
        "#;
        assert_eq!(main_entities(source).unwrap(), ["State"]);
    }

    #[test]
    fn multiline_declarations_and_unsafe_traits_count() {
        let source = "pub(crate)
struct State<T>(T); unsafe trait Port {}";
        assert_eq!(main_entities(source).unwrap(), ["State", "Port"]);
    }
}
