//! The outcome of a produced frame: [`FrameResult`].

use obscur_scene::scene::SceneSnapshot;

use super::FrameId;

#[doc = include_str!("../../docs/api/frame/frame_result.doc/FrameResult.md")]
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
    #[doc = include_str!("../../docs/api/frame/frame_result.doc/new.md")]
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

    #[doc = include_str!("../../docs/api/frame/frame_result.doc/is_balanced.md")]
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
