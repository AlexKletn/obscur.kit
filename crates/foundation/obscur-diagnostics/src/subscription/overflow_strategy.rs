/// How a subscription's buffer behaves when it is full and a new diagnostic
/// arrives (spec §7.8 backpressure).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverflowStrategy {
    /// Evict the oldest buffered diagnostic to make room for the new one. The
    /// newest diagnostics are always retained.
    DropOldest,
    /// Reject the incoming diagnostic, keeping the existing buffer intact. The
    /// oldest diagnostics are always retained.
    DropNewest,
    /// Merge the incoming diagnostic with an existing one that shares the same
    /// code and subsystem (the retained copy is updated to the newest
    /// occurrence); only fall back to dropping the oldest if no match exists.
    Coalesce,
}
