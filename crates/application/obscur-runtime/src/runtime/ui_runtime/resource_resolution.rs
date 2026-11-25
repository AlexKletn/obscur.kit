#![doc = include_str!("../../../docs/api/runtime/ui_runtime/resource_resolution.doc/_module.md")]

use crate::frame::FrameReason;
use crate::orchestration::ComputedStyles;
use crate::runtime::ResourceReporting;

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/resource_resolution.doc/poll_resource_tasks.md")]
    pub fn poll_resource_tasks(&mut self) -> usize {
        // Two sources of finished work land here: loads the frame phase DEFERRED
        // because no executor is installed (run now, on this thread, after the
        // frame phase — never during `render_now`), and off-thread loads whose
        // typed results are waiting on completion channels. Both commit through the same
        // `apply_outcome` path inside the bridge.
        let mut applied = self.resources.run_deferred_loads(&ResourceReporting {
            sink: &*self.diagnostics,
            runtime: self.id,
            locale: &self.locale,
        });
        // A newly decoded image/vector can be the intrinsic size of a replaced
        // element with `auto`/`auto` dimensions (§5.16). The frame that referenced
        // it laid out before it existed, and the next `render_now` (a `UiCommit`
        // frame) would refine layout to a Skip because no computed style changed —
        // leaving the element sized from the missing image. Invalidate the retained
        // layout inputs so that next frame re-lays out fully; the executor path
        // gets the same effect from the `ResourceCompletion` frame its landing
        // requests. (No effect when nothing landed — e.g. a failed load.)
        if applied > 0 {
            self.last_layout_inputs = None;
        }
        applied += self.commit_resource_results();
        if applied > 0 {
            self.scheduler
                .request_reason(FrameReason::ResourceCompletion);
        }
        applied
    }

    /// Commits every available off-thread load result, **without**
    /// asking for a frame — the §1.5 synchronization point the frame path takes.
    pub(super) fn commit_resource_results(&mut self) -> usize {
        self.resources.commit_results(&ResourceReporting {
            sink: &*self.diagnostics,
            runtime: self.id,
            locale: &self.locale,
        })
    }

    /// Records each resident image's on-screen priority for this frame, so
    /// eviction drops what nothing is showing first (spec §5.19). The viewport is
    /// built here, in document coordinates, because that is the space the arranged
    /// `boxes` are expressed in.
    pub(super) fn mark_image_visibility(&mut self, computed: &ComputedStyles) {
        let viewport = self.surface.document_viewport();
        self.resources
            .mark_image_visibility(computed, &self.tree, &self.boxes, viewport);
    }

    /// Resolves every not-yet-cached `background-image: url(...)` in `computed`
    /// (spec §5.15 load → §5.16 decode).
    pub(super) fn resolve_background_images(&mut self, computed: &ComputedStyles) {
        self.resources.resolve_background_images(
            computed,
            self.surface.device_scale(),
            &ResourceReporting {
                sink: &*self.diagnostics,
                runtime: self.id,
                locale: &self.locale,
            },
        );
    }

    /// Resolves every not-yet-cached element image `src` in the retained tree.
    pub(super) fn resolve_element_images(&mut self) {
        self.resources.resolve_element_images(
            &self.tree,
            self.surface.device_scale(),
            &ResourceReporting {
                sink: &*self.diagnostics,
                runtime: self.id,
                locale: &self.locale,
            },
        );
    }

    /// Resolves every not-yet-parsed element `src` naming a vector document
    /// (spec §5.17).
    pub(super) fn resolve_element_vectors(&mut self) {
        self.resources.resolve_element_vectors(
            &self.tree,
            &ResourceReporting {
                sink: &*self.diagnostics,
                runtime: self.id,
                locale: &self.locale,
            },
        );
    }
}
