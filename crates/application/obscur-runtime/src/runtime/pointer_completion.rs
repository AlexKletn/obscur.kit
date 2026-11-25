/// How a pointer's sequence ended (spec §2.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PointerCompletion {
    /// An `Up`: the contact lifted. A drag still travelling is thrown.
    Released,
    /// A `Cancel`: the system took the sequence away. Nothing is thrown and
    /// nothing activates.
    Cancelled,
}
