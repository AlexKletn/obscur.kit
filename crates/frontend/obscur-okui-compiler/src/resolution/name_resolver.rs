//! `NameResolver`: resolves the identifiers referenced by a document's
//! expressions against a [`SymbolTable`] (spec §2.10).

use obscur_okui_syntax::ast::{Document, Element};

use crate::resolution::ResolutionError;
use crate::symbol::{Symbol, SymbolTable};

/// Resolves the identifiers referenced by the expressions in a [`Document`]
/// against a [`SymbolTable`] (spec §2.10).
///
/// The caller pre-seeds the table with the component's props and state (via
/// [`SymbolTable::define`]); [`resolve`](Self::resolve) then walks the element
/// tree, checking that every attribute expression's leading reference resolves
/// to a defined symbol.
///
/// ## What is not a value reference
///
/// An `on:<event>` binding is skipped. Its value is a **handler name** that the
/// lowering hashes into a command symbol, not an expression anything evaluates, so
/// a symbol table of props and state has no opinion worth reporting about it — see
/// the note in `resolve_element`. Those names are checked by
/// [`CommandValidator`](crate::validation::CommandValidator) against the commands
/// the caller declared (spec §2.13).
///
/// ## Scoping conventions
///
/// A `<For>` loop opens a child scope for its body and binds two loop-local
/// symbols — `item` (the current element) and `index` (its position) — so
/// expressions inside the body may read them (spec §2.11). A `<For>` must carry
/// a `key` attribute for stable per-item identity; a missing key is reported as
/// [`ResolutionError::missing_loop_key`]. Conditionals (`<If>`/`<ElseIf>`/
/// `<Else>`) do not open a new scope — their body resolves in the enclosing
/// scope.
#[derive(Debug, Clone, Default)]
pub struct NameResolver;

impl NameResolver {
    /// Creates a name resolver.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Resolves every reference in `document` against `table`, returning the
    /// accumulated errors (empty when the document is fully resolved).
    ///
    /// The table is used as a mutable cursor: loop bodies push and pop child
    /// scopes as they are entered and left, so on return the cursor is back at
    /// the scope it started in.
    #[must_use]
    pub fn resolve(&self, document: &Document, table: &mut SymbolTable) -> Vec<ResolutionError> {
        let mut errors = Vec::new();
        for element in document.elements() {
            self.resolve_element(&element, table, &mut errors);
        }
        errors
    }

    /// Resolves a single element: checks its attribute references, then recurses
    /// into its children, opening a loop-local scope for a `<For>`.
    fn resolve_element(
        &self,
        element: &Element,
        table: &mut SymbolTable,
        errors: &mut Vec<ResolutionError>,
    ) {
        // Check the leading reference of every dynamic attribute expression. A
        // literal string attribute has no `value_expression`, so no reference
        // check is performed for it. This covers `each`/`key`/`cond` alongside
        // ordinary attributes, all resolved in the current (enclosing) scope.
        for attribute in element.attributes() {
            // An `on:<event>` value is NOT a value expression: the lowering takes
            // its raw text as the handler *name* and hashes it into a command
            // symbol, so resolving it against the props/state table asks the wrong
            // question and answers it misleadingly — `on:click={increment}` was
            // reported as an unknown *name*, whose only remedy is to declare a state
            // value called `increment`, which no handler ever reads. It also split
            // the two spellings of one reference: `on:click=increment` was checked by
            // nothing while `on:click={increment}` had to be a declared value.
            // Handler names belong to
            // [`CommandValidator`](crate::validation::CommandValidator), which judges
            // both spellings against the commands the caller declared (§2.13).
            if attribute.is_event() {
                continue;
            }
            let Some(expression) = attribute.value_expression() else {
                continue;
            };
            let text = expression.text();
            if let Some(root) = Self::root_reference(&text)
                && !table.is_defined(root)
            {
                let offset = expression.syntax().offset().unwrap_or(0);
                errors.push(ResolutionError::unknown_name(root, offset));
            }
        }

        if element.is_loop() {
            // A `<For>` requires a `key` attribute for stable item identity.
            if element.attribute("key").is_none() {
                let offset = element.syntax().offset().unwrap_or(0);
                errors.push(ResolutionError::missing_loop_key(offset));
            }
            // The body sees the loop-local bindings `item` and `index`.
            table.push_scope();
            table.define(Symbol::loop_local("item"));
            table.define(Symbol::loop_local("index"));
            self.resolve_children(element, table, errors);
            table.pop_scope();
        } else {
            // Plain and conditional elements resolve their body in the same
            // scope.
            self.resolve_children(element, table, errors);
        }
    }

    /// Recurses into the element-shaped content children of `element`; non-element
    /// children (text, bare expressions) are skipped.
    fn resolve_children(
        &self,
        element: &Element,
        table: &mut SymbolTable,
        errors: &mut Vec<ResolutionError>,
    ) {
        for child in element.children() {
            if let Some(child_element) = Element::cast(child) {
                self.resolve_element(&child_element, table, errors);
            }
        }
    }

    /// Extracts the root reference of an expression: its leading identifier.
    ///
    /// Leading whitespace is skipped, then characters are taken while they are
    /// alphanumeric or `_`, stopping at the first `.`, space or operator. Returns
    /// `None` when the identifier is empty or starts with a digit (so it is not a
    /// valid reference, e.g. a numeric literal like `42`).
    fn root_reference(expression_text: &str) -> Option<&str> {
        let text = expression_text.trim_start();
        let mut end = 0;
        for (index, ch) in text.char_indices() {
            if ch.is_alphanumeric() || ch == '_' {
                end = index + ch.len_utf8();
            } else {
                break;
            }
        }
        let identifier = &text[..end];
        if identifier.is_empty() {
            return None;
        }
        if identifier.starts_with(|c: char| c.is_ascii_digit()) {
            return None;
        }
        Some(identifier)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_okui_syntax::parser::Parser;

    use crate::resolution::ResolutionErrorKind;

    fn document(src: &str) -> Document {
        Document::cast(Parser::new().parse(src).tree().root().clone()).expect("Document root")
    }

    #[test]
    fn root_reference_extracts_leading_identifier() {
        assert_eq!(NameResolver::root_reference("count"), Some("count"));
        assert_eq!(NameResolver::root_reference("a.b.c"), Some("a"));
        assert_eq!(NameResolver::root_reference("a + b"), Some("a"));
        assert_eq!(NameResolver::root_reference("  spaced"), Some("spaced"));
        assert_eq!(NameResolver::root_reference(""), None);
        assert_eq!(NameResolver::root_reference("42"), None);
        assert_eq!(NameResolver::root_reference("(x)"), None);
    }

    #[test]
    fn defined_reference_resolves_without_error() {
        let mut table = SymbolTable::new();
        assert!(table.define(Symbol::prop("count")));
        let doc = document("<Text value={count}/>");
        let errors = NameResolver::new().resolve(&doc, &mut table);
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    }

    #[test]
    fn unknown_reference_reports_one_error() {
        let mut table = SymbolTable::new();
        let doc = document("<Text value={missing}/>");
        let errors = NameResolver::new().resolve(&doc, &mut table);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].kind(), &ResolutionErrorKind::UnknownName);
        assert_eq!(errors[0].name(), "missing");
    }

    #[test]
    fn loop_with_key_and_seeded_collection_resolves() {
        let mut table = SymbolTable::new();
        assert!(table.define(Symbol::prop("items")));
        assert!(table.define(Symbol::prop("id")));
        let doc = document("<For each={items} key={id}><Row/></For>");
        let errors = NameResolver::new().resolve(&doc, &mut table);
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    }

    #[test]
    fn loop_body_can_reference_loop_local_item() {
        let mut table = SymbolTable::new();
        assert!(table.define(Symbol::prop("items")));
        assert!(table.define(Symbol::prop("id")));
        let doc = document("<For each={items} key={id}><Row v={item}/></For>");
        let errors = NameResolver::new().resolve(&doc, &mut table);
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
        // The loop-local is no longer visible after the walk pops its scope.
        assert!(!table.is_defined("item"));
    }

    #[test]
    fn an_event_handler_is_not_resolved_as_a_value_reference() {
        // The two spellings of one handler reference must be judged alike, and by
        // the pass that knows what a command is. Before, the bare word was checked
        // by nothing and the braced form had to name a declared *value* — so the
        // advice for `on:click={increment}` was "declare a state called increment",
        // which no handler ever reads.
        let mut table = SymbolTable::new();
        let doc = document("<button on:click={increment}/><button on:click=increment/>");
        let errors = NameResolver::new().resolve(&doc, &mut table);
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    }

    #[test]
    fn loop_without_key_reports_missing_loop_key() {
        let mut table = SymbolTable::new();
        assert!(table.define(Symbol::prop("items")));
        let doc = document("<For each={items}><Row/></For>");
        let errors = NameResolver::new().resolve(&doc, &mut table);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].kind(), &ResolutionErrorKind::MissingLoopKey);
    }
}
