//! What the style phase leaves for the layout classifier: [`StyleDelta`].

use crate::orchestration::ComputedStyles;

#[doc = include_str!("../../docs/api/runtime/style_delta.doc/StyleDelta.md")]
pub(crate) enum StyleDelta {
    /// Every node was re-resolved from scratch. The previous styles are still in
    /// the runtime's retained styles (the fresh map is a separate allocation), and any node
    /// may have changed.
    Full,
    /// Only the invalidated nodes were re-resolved, in place. Every node absent
    /// from `replaced` kept a style this frame did not recompute and therefore
    /// provably did not change, so those are the only candidates — and `replaced`
    /// holds the style each of them had before.
    Incremental {
        /// The styles the re-resolved nodes had before, as
        /// [`IncrementalRestyle::previous`] returned them, plus the pre-overlay
        /// restorations the phase made first.
        replaced: ComputedStyles,
        /// Whether the walk styled a node that had no retained style, in which case
        /// `replaced` is *not* the complete list of candidates and nothing may be
        /// concluded from a node's absence. See
        /// [`IncrementalRestyle::introduced`].
        introduced: bool,
    },
}
