#![doc = include_str!("resource_resolution.doc/_module.md")]

use crate::frame::FrameReason;
use crate::orchestration::ComputedStyles;
use crate::runtime::ResourceReporting;

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("resource_resolution.doc/poll_resource_tasks.md")]
    pub fn poll_resource_tasks(&mut self) -> usize {
        // Two sources of finished work land here: loads the frame phase DEFERRED
        // because no executor is installed (run now, on this thread, after the
        // frame phase — never during `render_now`), and off-thread loads whose
        // typed results are waiting on the channel. Both commit through the same
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

    #[doc = include_str!("resource_resolution.doc/commit_resource_results.md")]
    pub(super) fn commit_resource_results(&mut self) -> usize {
        self.resources.commit_results(&ResourceReporting {
            sink: &*self.diagnostics,
            runtime: self.id,
            locale: &self.locale,
        })
    }

    #[doc = include_str!("resource_resolution.doc/mark_image_visibility.md")]
    pub(super) fn mark_image_visibility(&mut self, computed: &ComputedStyles) {
        let viewport = self.surface.document_viewport();
        self.resources
            .mark_image_visibility(computed, &self.tree, &self.boxes, viewport);
    }

    #[doc = include_str!("resource_resolution.doc/resolve_background_images.md")]
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

    #[doc = include_str!("resource_resolution.doc/resolve_element_images.md")]
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

    #[doc = include_str!("resource_resolution.doc/resolve_element_vectors.md")]
    pub(super) fn resolve_element_vectors(&mut self) {
        self.resources.resolve_element_vectors(&self.tree);
    }
}
