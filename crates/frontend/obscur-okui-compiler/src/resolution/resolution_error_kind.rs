/// The kind of failure a [`ResolutionError`](super::ResolutionError) reports (spec §2.11).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolutionErrorKind {
    /// An expression referenced an identifier that is not defined in any
    /// enclosing scope (spec §2.10).
    UnknownName,
    /// A name was defined twice in the same scope (spec §2.10).
    DuplicateDefinition,
    /// A `<For>` loop lacks the mandatory `key` attribute that gives each item
    /// a stable identity (spec §2.11).
    MissingLoopKey,
}
