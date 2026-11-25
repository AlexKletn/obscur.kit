use obscur_ui_ir::document::UiIrDocument;
use obscur_ui_ir::source::SourceMap;

use crate::compile_diagnostic::CompileDiagnostic;

#[doc = include_str!("../docs/api/compile.doc/CompileResult.md")]
#[derive(Debug, Clone)]
pub struct CompileResult {
    document: Option<UiIrDocument>,
    diagnostics: Vec<CompileDiagnostic>,
    source_map: SourceMap,
}

impl CompileResult {
    pub(crate) fn new(document: Option<UiIrDocument>, diagnostics: Vec<CompileDiagnostic>) -> Self {
        let source_map = document
            .as_ref()
            .map_or_else(SourceMap::new, |document| document.source_map().clone());
        Self {
            document,
            diagnostics,
            source_map,
        }
    }

    /// Returns the lowered UI IR document, if one was produced.
    #[must_use]
    pub fn document(&self) -> Option<&UiIrDocument> {
        self.document.as_ref()
    }

    /// Returns the accumulated diagnostics from every pass.
    #[must_use]
    pub fn diagnostics(&self) -> &[CompileDiagnostic] {
        &self.diagnostics
    }

    /// Returns the source map linking IR node ids back to source spans.
    #[must_use]
    pub fn source_map(&self) -> &SourceMap {
        &self.source_map
    }

    /// Returns whether any pass reported a diagnostic.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.diagnostics.is_empty()
    }

    /// Consumes the result, returning just the lowered document, if any.
    #[must_use]
    pub fn into_document(self) -> Option<UiIrDocument> {
        self.document
    }
}
