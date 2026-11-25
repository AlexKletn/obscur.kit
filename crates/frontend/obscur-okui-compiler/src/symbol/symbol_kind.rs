#[doc = include_str!("../../docs/api/symbol/symbol.doc/SymbolKind.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolKind {
    /// A component input prop bound by the caller (spec §2.9).
    Prop,
    /// A piece of component-local reactive state (spec §2.9).
    State,
    /// A loop-local binding introduced by a `<For>` body — `item` or `index`
    /// (spec §2.11).
    LoopLocal,
    /// A component declaration referenced by name (spec §2.9).
    Component,
    /// A callable function declaration (spec §2.10).
    Function,
    /// A named content slot (spec §2.9).
    Slot,
}

impl SymbolKind {
    #[doc = include_str!("../../docs/api/symbol/symbol.doc/is_reference_target.md")]
    #[must_use]
    pub const fn is_reference_target(&self) -> bool {
        matches!(self, Self::Prop | Self::State | Self::LoopLocal)
    }
}
