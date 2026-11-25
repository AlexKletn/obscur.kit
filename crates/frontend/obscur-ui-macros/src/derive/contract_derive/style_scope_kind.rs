/// Which `StyleScope` variant a `#[contract(style_scope = ...)]` selects
/// (spec §4.15).
#[derive(Debug, Clone, Copy)]
pub(super) enum StyleScopeKind {
    /// The default: the document's ordinary selectors stop at the host.
    Scoped,
    /// "Explicitly open/global scopes are possible": the component's rendered
    /// elements are styled by the document's sheets like authored ones.
    Open,
}
