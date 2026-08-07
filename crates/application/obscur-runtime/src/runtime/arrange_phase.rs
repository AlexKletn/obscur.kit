#![doc = include_str!("arrange_phase.doc/_module.md")]

use obscur_tree::tree::RetainedTree;

use super::{
    FramePhase, FrameTelemetry, HitTesting, LayoutInputs, LayoutPhase, ResourceBridge, VirtualLists,
};
use crate::frame::FrameContext;
use crate::orchestration::{
    ArrangeCache, ComputedStyles, EditingView, LayoutBoxes, LayoutCoordinator, MeasureCache,
    SceneCoordinator, root_font_size,
};

#[doc = include_str!("arrange_phase.doc/ArrangePhase.md")]
pub(crate) struct ArrangePhase<'a> {
    #[doc = include_str!("arrange_phase.doc/layout.md")]
    layout: &'a LayoutCoordinator,
    #[doc = include_str!("arrange_phase.doc/tree.md")]
    tree: &'a RetainedTree,
    #[doc = include_str!("arrange_phase.doc/scene_composer.md")]
    scene_composer: &'a SceneCoordinator,
    #[doc = include_str!("arrange_phase.doc/resources.md")]
    resources: &'a ResourceBridge,
    #[doc = include_str!("arrange_phase.doc/measure_cache.md")]
    measure_cache: &'a MeasureCache,
    #[doc = include_str!("arrange_phase.doc/arrange_cache.md")]
    arrange_cache: &'a ArrangeCache,
    #[doc = include_str!("arrange_phase.doc/boxes.md")]
    boxes: &'a mut LayoutBoxes,
    #[doc = include_str!("arrange_phase.doc/hits.md")]
    hits: &'a mut HitTesting,
    #[doc = include_str!("arrange_phase.doc/last_layout_inputs.md")]
    last_layout_inputs: &'a mut Option<LayoutInputs>,
    #[doc = include_str!("arrange_phase.doc/telemetry.md")]
    telemetry: &'a mut FrameTelemetry,
}

impl<'a> ArrangePhase<'a> {
    #[doc = include_str!("arrange_phase.doc/new.md")]
    // A borrow bundle's constructor is exactly as wide as the state the phase
    // touches; folding these into a struct would only move the same ten names.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        layout: &'a LayoutCoordinator,
        tree: &'a RetainedTree,
        scene_composer: &'a SceneCoordinator,
        resources: &'a ResourceBridge,
        measure_cache: &'a MeasureCache,
        arrange_cache: &'a ArrangeCache,
        boxes: &'a mut LayoutBoxes,
        hits: &'a mut HitTesting,
        last_layout_inputs: &'a mut Option<LayoutInputs>,
        telemetry: &'a mut FrameTelemetry,
    ) -> Self {
        Self {
            layout,
            tree,
            scene_composer,
            resources,
            measure_cache,
            arrange_cache,
            boxes,
            hits,
            last_layout_inputs,
            telemetry,
        }
    }

    #[doc = include_str!("arrange_phase.doc/run.md")]
    pub(crate) fn run(
        &mut self,
        ctx: &FrameContext,
        computed: &ComputedStyles,
        editing: Option<EditingView<'_>>,
        phase: &LayoutPhase,
        viewport_scroll: (f32, f32),
        inputs: LayoutInputs,
    ) -> bool {
        if matches!(phase, LayoutPhase::Skip) {
            return false;
        }
        let root_font = root_font_size(computed, self.tree.root());
        // `ch` is the one unit that cannot be folded from numbers the
        // resolver already holds: it is the `0` glyph's advance, so it must be
        // shaped. The scene's paragraph orchestrator does that here — only for
        // a node that actually writes `ch`, which is almost none of them — and
        // it is the SAME shaper that measures this node's text below, so a
        // `20ch` box and the twenty characters in it agree on the font.
        let mut layout_styles = self.layout.layout_styles_from(
            computed,
            ctx.viewport(),
            root_font,
            Some(self.scene_composer.paragraph()),
        );
        // Named-area placement (`grid-area: <name>` against the container's
        // `grid-template-areas`) needs the tree to resolve, so it patches the
        // freshly built styles here; items with no named area are untouched.
        self.layout
            .resolve_named_grid_areas(&mut layout_styles, self.tree, computed);
        // Named grid *lines* (`grid-column: <name>` against the container's
        // `[name]` track-list lines) resolve the same way — after the styles
        // table is built and against the parent's template; items with no named
        // line are left untouched.
        self.layout
            .resolve_named_grid_lines(&mut layout_styles, self.tree, computed);
        // Replaced elements (`<img>`) with neither dimension authored take
        // their content's own intrinsic logical size (CSS Images 3 §5.4
        // default sizing; spec §5.16). The image bytes were decoded into the
        // bridge's caches by phase 3c — synchronously, before arrange — so this
        // only patches the just built styles from what is already resident.
        self.resources
            .size_replaced_elements(self.tree, &mut layout_styles);
        VirtualLists::size_spacers(self.tree, &mut layout_styles);
        // Authored grid templates (`grid-template-columns`/`-rows`) become a
        // per-node side-table so a grid container dispatches the real
        // track-sizing algorithm; a grid node with no template falls back to a
        // single implicit column.
        let grid_templates = self.layout.grid_templates_from(
            computed,
            ctx.viewport(),
            root_font,
            Some(self.scene_composer.paragraph()),
        );
        // Text nodes are sized inside the constraint-aware measure/arrange
        // pass: `layout` shapes and wraps each text leaf to its available
        // width through the scene's paragraph orchestrator, so a long string
        // wraps to more lines (a taller box) instead of collapsing to one wide
        // line. It is the same orchestrator the scene phase later re-wraps and
        // draws with — so height and painted line count agree.
        //
        // §3.4 step 8: **incremental** layout, on both halves. The classifier
        // named the contexts whose inputs moved; hand that set — and the
        // two caches carried over from the last laid-out frame — to the
        // coordinator, which drops exactly the entries the set can have
        // invalidated, serves every other node's size without re-measuring, and
        // serves every unchanged subtree's absolute boxes without re-running its
        // layout algorithms. `Full` passes `None`, which drops both caches whole
        // and reproduces the historical all-or-nothing pass byte-for-byte.
        let dirty = match phase {
            LayoutPhase::Invalidated(dirty) => Some(dirty),
            LayoutPhase::Skip | LayoutPhase::Full => None,
        };
        let boxes = self.layout.layout_invalidated(
            self.tree,
            &layout_styles,
            computed,
            &grid_templates,
            ctx.viewport(),
            viewport_scroll,
            self.scene_composer.paragraph(),
            self.scene_composer.hyphenator(),
            editing,
            self.measure_cache,
            self.arrange_cache,
            dirty,
        );
        let relaid_out = !boxes.is_empty();
        *self.boxes = boxes;
        // Remember the non-style inputs these boxes were arranged under, so a
        // later frame can only reuse them while the viewport and the sticky
        // scroll offset are unchanged.
        *self.last_layout_inputs = Some(inputs);
        // Rebuild the hit-test index from the freshly laid-out boxes so the
        // next pointer event tests against this frame's geometry.
        self.hits.rebuild(self.tree, computed, self.boxes);
        // Telemetry: mark the layout phase and count arranged boxes (§7.9).
        let count = self.boxes.len() as u64;
        self.telemetry
            .record_phase(FramePhase::Layout, count, ctx.id());
        relaid_out
    }
}
