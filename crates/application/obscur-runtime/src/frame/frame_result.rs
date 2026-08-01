#![doc = include_str!("frame_result.doc/_module.md")]

use obscur_scene::scene::SceneSnapshot;

use super::FrameId;

#[doc = include_str!("frame_result.doc/FrameResult.md")]
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
    #[doc = include_str!("frame_result.doc/new.md")]
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

    #[doc = include_str!("frame_result.doc/id.md")]
    #[must_use]
    pub const fn id(&self) -> FrameId {
        self.id
    }

    #[doc = include_str!("frame_result.doc/snapshot.md")]
    #[must_use]
    pub const fn snapshot(&self) -> &SceneSnapshot {
        &self.snapshot
    }

    #[doc = include_str!("frame_result.doc/reconciled.md")]
    #[must_use]
    pub const fn reconciled(&self) -> bool {
        self.reconciled
    }

    #[doc = include_str!("frame_result.doc/restyled.md")]
    #[must_use]
    pub const fn restyled(&self) -> bool {
        self.restyled
    }

    #[doc = include_str!("frame_result.doc/relaid_out.md")]
    #[must_use]
    pub const fn relaid_out(&self) -> bool {
        self.relaid_out
    }

    #[doc = include_str!("frame_result.doc/composed.md")]
    #[must_use]
    pub const fn composed(&self) -> bool {
        self.composed
    }

    #[doc = include_str!("frame_result.doc/is_balanced.md")]
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
