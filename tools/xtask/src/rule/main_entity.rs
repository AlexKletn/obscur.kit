#![doc = include_str!("main_entity.doc/_module.md")]

use std::path::Path;

use crate::violation::{Violation, collect_rs};

const RULE: &str = "main-entity";

#[doc = include_str!("main_entity.doc/MARKER.md")]
const MARKER: &str = "xtask:allow(multiple-entities)";

#[doc = include_str!("main_entity.doc/MIN_BEHAVIOUR_LINES.md")]
const MIN_BEHAVIOUR_LINES: usize = 60;

#[doc = include_str!("main_entity.doc/KNOWN.md")]
const KNOWN: &[(&str, &str)] = &[(
    "crates/frontend/obscur-okui-compiler/src/validation/contract_validator.rs",
    "spec §3.11 names `contract_validator.rs` and gives `ComponentContract` \
         no file of its own — the contract is the validator's input grammar \
         (§2.13) and its module doc says so in its first sentence. The pair is \
         colocated deliberately: a contract's meaning IS what the validator does \
         with it, and the two would have to be read together wherever they lived.",
)];

#[doc = include_str!("main_entity.doc/check.md")]
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
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };

        let entities = main_entities(&text);
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
                        "exports {} main entities ({}) — spec §3.16: one file does not \
                         export several main entities without an explicit exception \
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
                    "exports {} main entity but still claims the multiple-entity \
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

#[doc = include_str!("main_entity.doc/claims_exception.md")]
fn claims_exception(text: &str) -> bool {
    text.lines().any(|line| {
        line.trim_start()
            .strip_prefix("//")
            .is_some_and(|rest| rest.trim_start().starts_with(MARKER))
    })
}

#[doc = include_str!("main_entity.doc/main_entities.md")]
fn main_entities(text: &str) -> Vec<String> {
    let lines: Vec<&str> = text.lines().collect();
    let depths = line_depths(&lines);

    let mut candidates: Vec<&str> = Vec::new();
    let mut behaviour: Vec<(String, usize)> = Vec::new();
    for (index, raw) in lines.iter().enumerate() {
        if depths[index] != Some(0) {
            continue;
        }
        let line = raw.trim();
        if let Some((name, is_trait)) = declared_entity(line) {
            candidates.push(name);
            if is_trait {
                let span = block_span(&depths, index);
                add_behaviour(&mut behaviour, name, span);
            }
        } else if let Some(name) = inherent_impl_target(line) {
            let span = block_span(&depths, index);
            add_behaviour(&mut behaviour, name, span);
        }
    }

    candidates
        .into_iter()
        .filter(|name| {
            behaviour
                .iter()
                .find(|(target, _)| target == name)
                .is_some_and(|(_, lines)| *lines >= MIN_BEHAVIOUR_LINES)
        })
        .map(str::to_owned)
        .collect()
}

#[doc = include_str!("main_entity.doc/add_behaviour.md")]
fn add_behaviour(behaviour: &mut Vec<(String, usize)>, name: &str, span: usize) {
    if let Some(entry) = behaviour.iter_mut().find(|(target, _)| target == name) {
        entry.1 += span;
    } else {
        behaviour.push((name.to_owned(), span));
    }
}

#[doc = include_str!("main_entity.doc/declared_entity.md")]
fn declared_entity(line: &str) -> Option<(&str, bool)> {
    let rest = strip_visibility(line)?;
    let rest = strip_word(rest, "unsafe").unwrap_or(rest);
    let rest = strip_word(rest, "auto").unwrap_or(rest);
    for (keyword, is_trait) in [("struct", false), ("enum", false), ("trait", true)] {
        if let Some(rest) = strip_word(rest, keyword) {
            let name = identifier(rest);
            if !name.is_empty() {
                return Some((name, is_trait));
            }
        }
    }
    None
}

#[doc = include_str!("main_entity.doc/inherent_impl_target.md")]
fn inherent_impl_target(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("impl")?;
    if !rest.starts_with(['<', ' ', '\t']) {
        return None;
    }
    let rest = skip_generics(rest.trim_start());
    let header = rest.split('{').next().unwrap_or(rest);
    if header.contains(" for ") {
        return None;
    }
    let name = identifier(rest);
    (!name.is_empty()).then_some(name)
}

#[doc = include_str!("main_entity.doc/strip_visibility.md")]
fn strip_visibility(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("pub")?;
    if let Some(rest) = rest.strip_prefix('(') {
        let close = rest.find(')')?;
        return Some(rest[close + 1..].trim_start());
    }
    rest.starts_with(char::is_whitespace)
        .then(|| rest.trim_start())
}

#[doc = include_str!("main_entity.doc/strip_word.md")]
fn strip_word<'a>(text: &'a str, word: &str) -> Option<&'a str> {
    let rest = text.strip_prefix(word)?;
    rest.starts_with(char::is_whitespace)
        .then(|| rest.trim_start())
}

#[doc = include_str!("main_entity.doc/skip_generics.md")]
fn skip_generics(text: &str) -> &str {
    if !text.starts_with('<') {
        return text;
    }
    let mut depth = 0usize;
    for (index, ch) in text.char_indices() {
        match ch {
            '<' => depth += 1,
            '>' => {
                depth -= 1;
                if depth == 0 {
                    return text[index + 1..].trim_start();
                }
            }
            _ => {}
        }
    }
    text
}

#[doc = include_str!("main_entity.doc/identifier.md")]
fn identifier(text: &str) -> &str {
    let end = text
        .find(|c: char| !c.is_alphanumeric() && c != '_')
        .unwrap_or(text.len());
    &text[..end]
}

#[doc = include_str!("main_entity.doc/block_span.md")]
fn block_span(depths: &[Option<usize>], start: usize) -> usize {
    let base = depths[start].unwrap_or(0);
    let mut opened = false;
    for (offset, depth) in depths.iter().enumerate().skip(start + 1) {
        match depth {
            Some(d) if *d > base => opened = true,
            Some(d) if *d <= base => {
                return if opened { offset - start } else { 1 };
            }
            _ => {}
        }
    }
    if opened { depths.len() - start } else { 1 }
}

#[doc = include_str!("main_entity.doc/line_depths.md")]
fn line_depths(lines: &[&str]) -> Vec<Option<usize>> {
    let mut out = Vec::with_capacity(lines.len());
    let mut depth = 0usize;
    let mut state = Scan::Code;
    for line in lines {
        out.push(matches!(state, Scan::Code).then_some(depth));
        scan_line(line, &mut depth, &mut state);
    }
    out
}

#[doc = include_str!("main_entity.doc/Scan.md")]
enum Scan {
    #[doc = include_str!("main_entity.doc/Code.md")]
    Code,
    #[doc = include_str!("main_entity.doc/BlockComment.md")]
    BlockComment(usize),
    #[doc = include_str!("main_entity.doc/Text.md")]
    Text,
    #[doc = include_str!("main_entity.doc/RawText.md")]
    RawText(usize),
}

#[doc = include_str!("main_entity.doc/scan_line.md")]
fn scan_line(line: &str, depth: &mut usize, state: &mut Scan) {
    let bytes: Vec<char> = line.chars().collect();
    let mut i = 0;
    while i < bytes.len() {
        match state {
            Scan::BlockComment(nesting) => {
                if bytes[i] == '*' && bytes.get(i + 1) == Some(&'/') {
                    *nesting -= 1;
                    if *nesting == 0 {
                        *state = Scan::Code;
                    }
                    i += 2;
                    continue;
                }
                if bytes[i] == '/' && bytes.get(i + 1) == Some(&'*') {
                    *nesting += 1;
                    i += 2;
                    continue;
                }
                i += 1;
            }
            Scan::Text => {
                if bytes[i] == '\\' {
                    i += 2;
                    continue;
                }
                if bytes[i] == '"' {
                    *state = Scan::Code;
                }
                i += 1;
            }
            Scan::RawText(hashes) => {
                if bytes[i] == '"' && closing_hashes(&bytes, i + 1) >= *hashes {
                    i += 1 + *hashes;
                    *state = Scan::Code;
                    continue;
                }
                i += 1;
            }
            Scan::Code => {
                match bytes[i] {
                    '/' if bytes.get(i + 1) == Some(&'/') => return,
                    '/' if bytes.get(i + 1) == Some(&'*') => {
                        *state = Scan::BlockComment(1);
                        i += 2;
                        continue;
                    }
                    '"' => *state = Scan::Text,
                    'r' if matches!(bytes.get(i + 1), Some('"' | '#')) => {
                        let hashes = opening_hashes(&bytes, i + 1);
                        if bytes.get(i + 1 + hashes) == Some(&'"') {
                            *state = Scan::RawText(hashes);
                            i += 2 + hashes;
                            continue;
                        }
                    }
                    '\'' if is_char_literal(&bytes, i) => {
                        i += char_literal_len(&bytes, i);
                        continue;
                    }
                    '{' => *depth += 1,
                    '}' => *depth = depth.saturating_sub(1),
                    _ => {}
                }
                i += 1;
            }
        }
    }
}

#[doc = include_str!("main_entity.doc/opening_hashes.md")]
fn opening_hashes(bytes: &[char], at: usize) -> usize {
    bytes[at..].iter().take_while(|c| **c == '#').count()
}

#[doc = include_str!("main_entity.doc/closing_hashes.md")]
fn closing_hashes(bytes: &[char], at: usize) -> usize {
    bytes.get(at..).map_or(0, opening_hashes_from_start)
}

#[doc = include_str!("main_entity.doc/opening_hashes_from_start.md")]
fn opening_hashes_from_start(bytes: &[char]) -> usize {
    bytes.iter().take_while(|c| **c == '#').count()
}

#[doc = include_str!("main_entity.doc/is_char_literal.md")]
fn is_char_literal(bytes: &[char], at: usize) -> bool {
    match bytes.get(at + 1) {
        Some('\\') => true,
        Some(_) => bytes.get(at + 2) == Some(&'\''),
        None => false,
    }
}

#[doc = include_str!("main_entity.doc/char_literal_len.md")]
fn char_literal_len(bytes: &[char], at: usize) -> usize {
    let mut i = at + 1;
    if bytes.get(i) == Some(&'\\') {
        i += 1;
    }
    while i < bytes.len() {
        if bytes[i] == '\'' {
            return i - at + 1;
        }
        i += 1;
    }
    bytes.len() - at
}

#[cfg(test)]
mod tests {
    use super::{MIN_BEHAVIOUR_LINES, main_entities};

    /// Builds a type with `body` lines of inherent `impl`.
    fn entity(name: &str, body: usize) -> String {
        let filler = "    fn f(&self) {}\n".repeat(body);
        format!("pub struct {name};\n\nimpl {name} {{\n{filler}}}\n")
    }

    #[test]
    fn a_lone_entity_is_not_a_violation() {
        let found = main_entities(&entity("Alpha", MIN_BEHAVIOUR_LINES));
        assert_eq!(found, vec!["Alpha".to_owned()]);
    }

    #[test]
    fn two_behaving_entities_are_both_main() {
        let text = format!(
            "{}{}",
            entity("Alpha", MIN_BEHAVIOUR_LINES),
            entity("Beta", MIN_BEHAVIOUR_LINES)
        );
        assert_eq!(
            main_entities(&text),
            vec!["Alpha".to_owned(), "Beta".to_owned()]
        );
    }

    #[test]
    fn an_entitys_own_vocabulary_is_not_a_second_entity() {
        let text = format!(
            "pub enum Kind {{ A, B }}\n\nimpl Kind {{\n    fn f(&self) {{}}\n}}\n\n{}",
            entity("Alpha", MIN_BEHAVIOUR_LINES)
        );
        assert_eq!(main_entities(&text), vec!["Alpha".to_owned()]);
    }

    #[test]
    fn a_trait_impl_is_not_the_types_own_behaviour() {
        let filler = "    fn f(&self) {}\n".repeat(MIN_BEHAVIOUR_LINES);
        let text = format!(
            "pub struct Stub;\n\nimpl Port for Stub {{\n{filler}}}\n\n{}",
            entity("Alpha", MIN_BEHAVIOUR_LINES)
        );
        assert_eq!(main_entities(&text), vec!["Alpha".to_owned()]);
    }

    #[test]
    fn a_private_type_is_never_a_main_entity() {
        let text = format!(
            "{}{}",
            entity("Alpha", MIN_BEHAVIOUR_LINES),
            entity("Beta", MIN_BEHAVIOUR_LINES).replacen("pub ", "", 1)
        );
        assert_eq!(main_entities(&text), vec!["Alpha".to_owned()]);
    }

    #[test]
    fn a_nested_type_is_not_at_file_scope() {
        let text = format!(
            "{}\n#[cfg(test)]\nmod tests {{\n{}}}\n",
            entity("Alpha", MIN_BEHAVIOUR_LINES),
            entity("Beta", MIN_BEHAVIOUR_LINES)
        );
        assert_eq!(main_entities(&text), vec!["Alpha".to_owned()]);
    }

    #[test]
    fn braces_inside_a_raw_string_do_not_move_file_scope() {
        let text = format!(
            "pub const CSS: &str = r\"body {{ color: red; }}\";\n\n{}",
            entity("Alpha", MIN_BEHAVIOUR_LINES)
        );
        assert_eq!(main_entities(&text), vec!["Alpha".to_owned()]);
    }

    #[test]
    fn generic_impl_headers_bind_to_their_type() {
        let filler = "    fn f(&self) {}\n".repeat(MIN_BEHAVIOUR_LINES);
        let text = format!("pub struct Alpha<T>(T);\n\nimpl<T> Alpha<T> {{\n{filler}}}\n");
        assert_eq!(main_entities(&text), vec!["Alpha".to_owned()]);
    }
}
