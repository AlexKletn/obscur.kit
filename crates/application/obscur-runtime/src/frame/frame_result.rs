//! The outcome of a produced frame: [`FrameResult`].

use obscur_scene::scene::SceneSnapshot;

use super::FrameId;

/// The outcome of one [`FramePipeline`](super::FramePipeline) run.
///
/// A `FrameResult` pairs the produced frame's identity with the immutable
/// [`SceneSnapshot`] a renderer consumes, plus a set of flags recording which
/// pipeline phases did observable work. The snapshot is the hand-off contract:
/// a renderer reads [`FrameResult::snapshot`] and honours its display list; it
/// never touches the mutable scene the pipeline built it from (spec §5.2).
///
/// The flags are diagnostic — they let the runtime and its instrumentation see
/// what a frame actually did (whether the tree changed, styles resolved, layout
/// ran, and a scene was composed) without re-deriving it.
#[derive(Debug, Clone, PartialEq)]
pub struct FrameResult {
    id: FrameId,
    snapshot: SceneSnapshot,
    reconciled: bool,
    restyled: bool,
    relaid_out: bool,
    composed: bool,
}

impl FrameResult {
    /// Creates a result for frame `id` carrying `snapshot` and the phase flags.
    ///
    /// * `reconciled` — the retained tree changed during reconciliation.
    /// * `restyled` — at least one node resolved a computed style.
    /// * `relaid_out` — at least one node was arranged into a box.
    /// * `composed` — a scene was composed and published as `snapshot`.
    // The four phase flags are the spec-mandated per-phase outcome of a frame;
    // they are distinct booleans by design, not a flags struct.
    #[allow(clippy::fn_params_excessive_bools)]
    #[must_use]
    pub const fn new(
        id: FrameId,
        snapshot: SceneSnapshot,
        reconciled: bool,
        restyled: bool,
        relaid_out: bool,
        composed: bool,
    ) -> Self {
        Self {
            id,
            snapshot,
            reconciled,
            restyled,
            relaid_out,
            composed,
        }
    }

    /// Returns the identity of the produced frame.
    #[must_use]
    pub const fn id(&self) -> FrameId {
        self.id
    }

    /// Returns the immutable scene snapshot a renderer consumes.
    #[must_use]
    pub const fn snapshot(&self) -> &SceneSnapshot {
        &self.snapshot
    }

    /// Whether reconciliation changed the retained tree.
    #[must_use]
    pub const fn reconciled(&self) -> bool {
        self.reconciled
    }

    /// Whether the style phase resolved any computed styles.
    #[must_use]
    pub const fn restyled(&self) -> bool {
        self.restyled
    }

    /// Whether the layout phase arranged any boxes.
    #[must_use]
    pub const fn relaid_out(&self) -> bool {
        self.relaid_out
    }

    /// Whether a scene was composed for this frame.
    #[must_use]
    pub const fn composed(&self) -> bool {
        self.composed
    }

    /// Whether the published snapshot's display list is well-formed (balanced).
    ///
    /// Delegates to [`SceneSnapshot::is_balanced`]; a balanced list has every
    /// push paired with a matching pop and is safe for a renderer to replay.
    #[must_use]
    pub fn is_balanced(&self) -> bool {
        self.snapshot.is_balanced()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_scene::command::DisplayList;
    use obscur_scene::scene::SceneGeneration;

    fn empty_snapshot() -> SceneSnapshot {
        SceneSnapshot::new(DisplayList::new(), SceneGeneration::FIRST)
    }

    #[test]
    fn accessors_return_constructor_inputs() {
        let result = FrameResult::new(
            FrameId::from_raw(5),
            empty_snapshot(),
            true,
            false,
            true,
            true,
        );
        assert_eq!(result.id(), FrameId::from_raw(5));
        assert!(result.reconciled());
        assert!(!result.restyled());
        assert!(result.relaid_out());
        assert!(result.composed());
    }

    #[test]
    fn is_balanced_delegates_to_snapshot() {
        let result = FrameResult::new(FrameId::FIRST, empty_snapshot(), false, false, false, true);
        // An empty display list is trivially balanced.
        assert_eq!(result.is_balanced(), result.snapshot().is_balanced());
        assert!(result.is_balanced());
    }
}
