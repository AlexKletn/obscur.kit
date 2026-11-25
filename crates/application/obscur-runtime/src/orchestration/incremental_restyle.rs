use super::ComputedStyles;

/// What [`StyleCoordinator::resolve_incremental`](super::StyleCoordinator::resolve_incremental) did, so the frame pipeline can
/// classify the phases downstream of it (spec §3.4 phase 5, §4.3).
#[derive(Debug, Clone, Default)]
pub struct IncrementalRestyle {
    /// How many nodes were re-resolved — the frame's real style cost, and the
    /// number the `style.resolved` counter reports. Not the size of the resulting
    /// map, and not `previous.len()` either.
    pub resolved: usize,
    #[doc = include_str!("../../docs/api/orchestration/style_coordinator.doc/previous.md")]
    pub previous: ComputedStyles,
    #[doc = include_str!("../../docs/api/orchestration/style_coordinator.doc/introduced.md")]
    pub introduced: bool,
}
