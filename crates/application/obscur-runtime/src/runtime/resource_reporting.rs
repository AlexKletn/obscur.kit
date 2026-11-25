use obscur_diagnostics::DiagnosticSink;
use obscur_identity::RuntimeId;
use obscur_localization::locale::LanguageTag;

#[doc = include_str!("../../docs/api/runtime/resource_bridge.doc/ResourceReporting.md")]
pub(crate) struct ResourceReporting<'a> {
    /// Where the diagnostic goes (spec §1.6).
    pub(crate) sink: &'a dyn DiagnosticSink,
    /// The runtime whose surface the fault happened in — reported as the
    /// diagnostic's `window` context.
    pub(crate) runtime: RuntimeId,
    /// The locale the human message is resolved in. Only the message localizes;
    /// the diagnostic *code* stays locale-invariant (spec §7.2).
    pub(crate) locale: &'a LanguageTag,
}
