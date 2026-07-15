//! `Analyzer`: the language-server core that turns open documents into LSP
//! diagnostics, document symbols and hover results.

use obscur_okui_compiler::compile::{CompileDiagnostic, Compiler, ContractRegistry};
use obscur_okui_compiler::lowering::LoweringError;
use obscur_okui_syntax::ast::{Document, Element};
use obscur_okui_syntax::parser::Parser;
use obscur_okui_syntax::syntax::SyntaxNode;

use crate::analysis::{DocumentSymbol, Hover, SymbolKind};
use crate::diagnostics::{LspDiagnostic, LspSeverity, Position, Range};
use crate::document::{DocumentStore, LineIndex};

/// Converts a `usize` length to `u32`, saturating at [`u32::MAX`].
fn to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

/// The analysis core of the `.okui` language server.
///
/// The `Analyzer` owns a [`DocumentStore`] (mutate it via [`store_mut`](Self::store_mut)
/// to drive the `didOpen`/`didChange`/`didClose` lifecycle) and a
/// [`ContractRegistry`] used by the compiler's §2.13 contract checks. It is
/// transport-agnostic: a JSON-RPC/stdio wrapper feeds documents into the store
/// and calls [`analyze`](Self::analyze), [`document_symbols`](Self::document_symbols)
/// and [`hover`](Self::hover), then serializes the results onto the wire.
///
/// # Diagnostics
///
/// [`analyze`](Self::analyze) parses the document with the okui
/// [`Parser`], mapping each parse error (a byte offset) to a one-character
/// [`LspDiagnostic`] via a [`LineIndex`]. When the source parses cleanly it is
/// cast to a [`Document`] and run through the okui [`Compiler`]; every
/// [`CompileDiagnostic`] is likewise mapped from its byte offset to an
/// [`LspDiagnostic`].
#[derive(Debug, Clone, Default)]
pub struct Analyzer {
    store: DocumentStore,
    registry: ContractRegistry,
}

impl Analyzer {
    /// Creates an analyzer with an empty document store and no contracts.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the contract registry the compiler consults (fluent).
    #[must_use]
    pub fn with_registry(mut self, registry: ContractRegistry) -> Self {
        self.registry = registry;
        self
    }

    /// Returns a shared reference to the document store.
    #[must_use]
    pub const fn store(&self) -> &DocumentStore {
        &self.store
    }

    /// Returns a mutable reference to the document store, for driving the
    /// `didOpen`/`didChange`/`didClose` lifecycle.
    pub const fn store_mut(&mut self) -> &mut DocumentStore {
        &mut self.store
    }

    /// Analyzes the document at `uri`, returning every diagnostic in LSP shape.
    ///
    /// Returns an empty vector if no document is open at `uri`. Parse errors are
    /// reported first; only when the source parses cleanly is it compiled and
    /// the compiler's diagnostics appended.
    #[must_use]
    pub fn analyze(&self, uri: &str) -> Vec<LspDiagnostic> {
        let Some(document) = self.store.get(uri) else {
            return Vec::new();
        };
        let index = LineIndex::new(&document.text);
        let mut diagnostics = Vec::new();

        let parsed = Parser::new().parse(&document.text);
        for error in parsed.errors() {
            diagnostics.push(LspDiagnostic::new(
                point_range(&index, error.offset()),
                LspSeverity::Error,
                error.message(),
                "okui-parse",
            ));
        }

        if parsed.is_ok()
            && let Some(ast) = Document::cast(parsed.tree().root().clone())
        {
            let result = Compiler::new(uri.to_owned())
                .with_registry(self.registry.clone())
                .compile(&ast);
            for diagnostic in result.diagnostics() {
                let (offset, message, severity) = map_compile_diagnostic(diagnostic);
                diagnostics.push(LspDiagnostic::new(
                    point_range(&index, offset),
                    severity,
                    message,
                    "okui-compile",
                ));
            }
        }

        diagnostics
    }

    /// Returns the hierarchical outline of elements/components in the document at
    /// `uri` (empty if no document is open).
    #[must_use]
    pub fn document_symbols(&self, uri: &str) -> Vec<DocumentSymbol> {
        let Some(document) = self.store.get(uri) else {
            return Vec::new();
        };
        let index = LineIndex::new(&document.text);
        let parsed = Parser::new().parse(&document.text);
        let Some(ast) = Document::cast(parsed.tree().root().clone()) else {
            return Vec::new();
        };
        ast.elements()
            .filter_map(|element| symbol_of(&index, &element))
            .collect()
    }

    /// Returns hover information for the innermost element/component containing
    /// `position`, if any.
    #[must_use]
    pub fn hover(&self, uri: &str, position: Position) -> Option<Hover> {
        let symbols = self.document_symbols(uri);
        let symbol = deepest_containing(&symbols, position)?;
        Some(Hover::new(
            format!("{} {}", symbol.kind.label(), symbol.name),
            symbol.range,
        ))
    }
}

/// Maps a [`CompileDiagnostic`] to its byte offset, message and LSP severity.
///
/// Validation diagnostics (unknown attribute/event, missing prop — §2.13) are
/// surfaced as warnings; resolution, type and lowering failures are errors.
fn map_compile_diagnostic(diagnostic: &CompileDiagnostic) -> (u32, String, LspSeverity) {
    match diagnostic {
        CompileDiagnostic::Resolution(error) => {
            (error.offset(), error.to_string(), LspSeverity::Error)
        }
        CompileDiagnostic::Type(error) => (error.offset(), error.to_string(), LspSeverity::Error),
        CompileDiagnostic::Validation(validation) => (
            validation.offset(),
            validation.message().to_owned(),
            LspSeverity::Warning,
        ),
        CompileDiagnostic::Lowering(error) => {
            // A missing `<For>` key is non-fatal (spec §2.11) — report it as a
            // warning; structural lowering errors stay errors.
            let (offset, severity) = match error {
                LoweringError::MissingTagName(offset) => (*offset, LspSeverity::Error),
                LoweringError::EmptyDocument => (0, LspSeverity::Error),
                LoweringError::MissingLoopKey(offset) => (*offset, LspSeverity::Warning),
            };
            (offset, error.to_string(), severity)
        }
    }
}

/// Builds a one-character [`Range`] anchored at `offset` (an empty range at the
/// end of the document), mapping byte offsets through `index`.
fn point_range(index: &LineIndex, offset: u32) -> Range {
    let start = index.offset_to_position(offset);
    let end = index.offset_to_position(offset.saturating_add(1));
    Range::new(start, end)
}

/// Builds a [`Range`] covering the full source span of `node`.
fn node_range(index: &LineIndex, node: &SyntaxNode) -> Range {
    let start_offset = node.offset().unwrap_or(0);
    let end_offset = start_offset.saturating_add(to_u32(node.text().len()));
    Range::new(
        index.offset_to_position(start_offset),
        index.offset_to_position(end_offset),
    )
}

/// Builds a [`DocumentSymbol`] for `element` and its element-shaped children,
/// or `None` for a nameless element.
fn symbol_of(index: &LineIndex, element: &Element) -> Option<DocumentSymbol> {
    let name = element.tag_name()?;
    let kind = SymbolKind::of_tag(&name);
    let range = node_range(index, element.syntax());
    let children = element
        .children()
        .filter_map(Element::cast)
        .filter_map(|child| symbol_of(index, &child))
        .collect();
    Some(DocumentSymbol::new(name, kind, range).with_children(children))
}

/// Finds the deepest symbol whose range contains `position`.
fn deepest_containing(symbols: &[DocumentSymbol], position: Position) -> Option<&DocumentSymbol> {
    for symbol in symbols {
        if symbol.range.contains(position) {
            return Some(deepest_containing(&symbol.children, position).unwrap_or(symbol));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_okui_compiler::validation::ComponentContract;

    const URI: &str = "file:///app.okui";

    fn button_registry() -> ContractRegistry {
        ContractRegistry::new().with(
            ComponentContract::new("Button")
                .requiring_prop("label")
                .allowing_event("click"),
        )
    }

    fn analyzer_with(text: &str) -> Analyzer {
        let mut analyzer = Analyzer::new().with_registry(button_registry());
        analyzer.store_mut().open(URI, 1, text);
        analyzer
    }

    #[test]
    fn well_formed_document_has_no_diagnostics() {
        let analyzer = analyzer_with("<Button label=\"Save\" on:click=submit/>");
        assert!(
            analyzer.analyze(URI).is_empty(),
            "diagnostics: {:?}",
            analyzer.analyze(URI)
        );
    }

    #[test]
    fn missing_document_analyzes_to_nothing() {
        let analyzer = Analyzer::new();
        assert!(analyzer.analyze("file:///nope.okui").is_empty());
        assert!(analyzer.document_symbols("file:///nope.okui").is_empty());
    }

    #[test]
    fn parse_error_yields_an_error_diagnostic_with_a_range() {
        // A missing close tag is a recoverable parse error.
        let analyzer = analyzer_with("<Button>oops");
        let diagnostics = analyzer.analyze(URI);
        assert!(!diagnostics.is_empty());
        let parse = diagnostics
            .iter()
            .find(|d| d.source == "okui-parse")
            .expect("a parse diagnostic");
        assert_eq!(parse.severity, LspSeverity::Error);
        // The error is anchored on the first line at/after the tag.
        assert_eq!(parse.range.start.line, 0);
        assert!(parse.range.end >= parse.range.start);
    }

    #[test]
    fn semantic_error_yields_a_warning_at_the_attribute() {
        // Parses cleanly but violates the Button contract (unknown attribute).
        let analyzer = analyzer_with("<Button label=\"Save\" colr=\"red\"/>");
        let diagnostics = analyzer.analyze(URI);
        let semantic = diagnostics
            .iter()
            .find(|d| d.source == "okui-compile")
            .expect("a compile diagnostic");
        assert_eq!(semantic.severity, LspSeverity::Warning);
        assert!(
            semantic.message.contains("unknown attribute 'colr'"),
            "message: {}",
            semantic.message
        );
        // `colr` starts at byte 21 on line 0.
        assert_eq!(semantic.range.start, Position::new(0, 21));
    }

    #[test]
    fn resolution_error_is_reported_as_error() {
        // `{missing}` is an unknown reference — a resolution error.
        let mut analyzer = Analyzer::new();
        analyzer.store_mut().open(URI, 1, "<Text value={missing}/>");
        let diagnostics = analyzer.analyze(URI);
        assert!(
            diagnostics
                .iter()
                .any(|d| d.source == "okui-compile" && d.severity == LspSeverity::Error),
            "diagnostics: {diagnostics:?}"
        );
    }

    #[test]
    fn document_symbols_list_top_level_and_nested_elements() {
        let mut analyzer = Analyzer::new();
        analyzer
            .store_mut()
            .open(URI, 1, "<Col>\n  <Row/>\n  <Icon/>\n</Col>");
        let symbols = analyzer.document_symbols(URI);
        assert_eq!(symbols.len(), 1);
        assert_eq!(&*symbols[0].name, "Col");
        assert_eq!(symbols[0].kind, SymbolKind::Component);
        assert_eq!(symbols[0].children.len(), 2);
        let names: Vec<_> = symbols[0]
            .children
            .iter()
            .map(|s| s.name.to_string())
            .collect();
        assert_eq!(names, ["Row", "Icon"]);
    }

    #[test]
    fn document_symbol_range_covers_the_element() {
        let mut analyzer = Analyzer::new();
        analyzer.store_mut().open(URI, 1, "<App/>");
        let symbols = analyzer.document_symbols(URI);
        assert_eq!(symbols[0].range.start, Position::new(0, 0));
        assert_eq!(symbols[0].range.end, Position::new(0, 6));
    }

    #[test]
    fn hover_returns_the_innermost_element_name() {
        let mut analyzer = Analyzer::new();
        analyzer.store_mut().open(URI, 1, "<Col><Row/></Col>");
        // Byte 6 is inside `<Row/>` (which spans bytes 5..11).
        let hover = analyzer
            .hover(URI, Position::new(0, 6))
            .expect("a hover result");
        assert!(
            hover.contents.contains("Row"),
            "contents: {}",
            hover.contents
        );
    }

    #[test]
    fn hover_outside_any_element_is_none() {
        let mut analyzer = Analyzer::new();
        analyzer.store_mut().open(URI, 1, "  <A/>");
        assert!(analyzer.hover(URI, Position::new(0, 0)).is_none());
    }
}
