//! The frame's layout phase: [`ArrangePhase`].

use obscur_diagnostics::DiagnosticSink;
use obscur_identity::{FastMap, NodeId};
use obscur_style::computed::ResolutionContext;
use obscur_tree::tree::RetainedTree;

use super::layout_style_cache::LayoutStyleCache;
use super::{
    FramePhase, FrameTelemetry, HitTesting, LayoutInputs, LayoutPhase, ResourceBridge, VirtualLists,
};
use crate::frame::FrameContext;
use crate::orchestration::{
    ArrangeCache, ComputedStyles, EditingView, GeneratedBoxes, GeneratedContent, LayoutBoxes,
    LayoutCoordinator, MeasureCache, SceneCoordinator, ShapedChMetrics, layout_style_from,
    root_font_size,
};

#[doc = include_str!("../../docs/api/runtime/arrange_phase.doc/ArrangePhase.md")]
pub(crate) struct ArrangePhase<'a> {
    /// Drives layout over the retained tree.
    layout: &'a LayoutCoordinator,
    layout_styles: &'a mut LayoutStyleCache,
    query_containers: &'a obscur_style::matching::ContainerQueryEnvironment,
    /// The retained tree the boxes are arranged over.
    tree: &'a RetainedTree,
    /// The paragraph orchestrator that folds `ch`, shapes text leaves and
    /// hyphenates them — the same one compose later re-wraps and paints with, so
    /// box height and painted line count agree.
    scene_composer: &'a SceneCoordinator,
    /// The decoded-image / parsed-vector caches a replaced element with no
    /// authored size is measured against (CSS Images 3 §5.4).
    resources: &'a ResourceBridge,
    /// The `measure(node, constraints)` results carried over from the last
    /// laid-out frame (§4.2).
    measure_cache: &'a MeasureCache,
    /// The `arrange` results carried over from the last laid-out frame — the
    /// other half of §4.2's cached layout, and a matched pair with
    /// [`measure_cache`](Self::measure_cache).
    arrange_cache: &'a ArrangeCache,
    /// The absolute boxes this phase produces and the runtime retains.
    boxes: &'a mut LayoutBoxes,
    /// The z-order-aware hit-test index rebuilt from those boxes (§4.7).
    hits: &'a mut HitTesting,
    /// The non-style inputs the retained boxes were arranged under, recorded so a
    /// later frame can only reuse them while those inputs hold still.
    last_layout_inputs: &'a mut Option<LayoutInputs>,
    /// Where the layout phase's span and box count land (spec §7.9).
    telemetry: &'a mut FrameTelemetry,
    diagnostics: &'a dyn DiagnosticSink,
}

impl<'a> ArrangePhase<'a> {
    /// Bundles the state one layout phase reads and writes.
    // A borrow bundle's constructor is exactly as wide as the state the phase
    // touches; folding these into a struct would only move the same ten names.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        layout: &'a LayoutCoordinator,
        layout_styles: &'a mut LayoutStyleCache,
        query_containers: &'a obscur_style::matching::ContainerQueryEnvironment,
        tree: &'a RetainedTree,
        scene_composer: &'a SceneCoordinator,
        resources: &'a ResourceBridge,
        measure_cache: &'a MeasureCache,
        arrange_cache: &'a ArrangeCache,
        boxes: &'a mut LayoutBoxes,
        hits: &'a mut HitTesting,
        last_layout_inputs: &'a mut Option<LayoutInputs>,
        telemetry: &'a mut FrameTelemetry,
        diagnostics: &'a dyn DiagnosticSink,
    ) -> Self {
        Self {
            layout,
            layout_styles,
            query_containers,
            tree,
            scene_composer,
            resources,
            measure_cache,
            arrange_cache,
            boxes,
            hits,
            last_layout_inputs,
            telemetry,
            diagnostics,
        }
    }

    #[doc = include_str!("../../docs/api/runtime/arrange_phase.doc/run.md")]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn run(
        &mut self,
        ctx: &FrameContext,
        computed: &mut ComputedStyles,
        editing: Option<EditingView<'_>>,
        phase: &LayoutPhase,
        viewport_scroll: (f32, f32),
        inputs: LayoutInputs,
        detailed: bool,
        generated: &FastMap<NodeId, GeneratedContent>,
        generated_boxes: &mut Option<GeneratedBoxes>,
    ) -> bool {
        if matches!(phase, LayoutPhase::Skip) {
            return false;
        }
        let prep_started = detailed.then(std::time::Instant::now);
        let root_font = root_font_size(computed, self.tree.root());
        // `ch` is the one unit that cannot be folded from numbers the
        // resolver already holds: it is the `0` glyph's advance, so it must be
        // shaped. The scene's paragraph orchestrator does that here — only for
        // a node that actually writes `ch`, which is almost none of them — and
        // it is the SAME shaper that measures this node's text below, so a
        // `20ch` box and the twenty characters in it agree on the font.
        let mut layout_styles = self.layout_styles.prepare(
            self.layout,
            computed,
            ctx.viewport(),
            root_font,
            self.scene_composer.paragraph(),
            phase,
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
        let mut grid_templates = self.layout.grid_templates_from(
            computed,
            ctx.viewport(),
            root_font,
            Some(self.scene_composer.paragraph()),
        );
        crate::orchestration::defer_repeated_grid_lines(&mut grid_templates, self.tree, computed);
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
        let solve_started = detailed.then(std::time::Instant::now);
        if detailed {
            self.measure_cache.begin_text_timing();
        }
        if dirty.is_some_and(|dirty| !dirty.is_empty()) {
            self.arrange_cache.retain_topology(self.tree);
        }
        let metrics = ShapedChMetrics::new(self.scene_composer.paragraph());
        let context =
            ResolutionContext::new(root_font, ctx.viewport().width, ctx.viewport().height)
                .with_metrics(&metrics)
                .with_environment(ctx.environment());
        let mut projection = GeneratedBoxes::project(
            self.tree,
            computed,
            generated,
            context,
            self.query_containers,
        );
        if let Some(projection) = &projection {
            projection.lay_out_text_rows(&mut layout_styles);
            for (&node, style) in &projection.styles {
                layout_styles.insert(
                    node,
                    layout_style_from(
                        style,
                        ctx.viewport(),
                        root_font,
                        Some(self.scene_composer.paragraph()),
                    ),
                );
            }
            computed.extend(
                projection
                    .styles
                    .iter()
                    .map(|(&node, style)| (node, style.clone())),
            );
        }
        let mut boxes = self.layout.layout_invalidated(
            projection
                .as_ref()
                .map_or(self.tree, |projection| &projection.tree),
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
            dirty.filter(|_| projection.is_none() && generated_boxes.is_none()),
        );
        if let Some(projection) = &mut projection {
            for node in projection.styles.keys() {
                computed.remove(node);
            }
            projection.boxes = boxes.clone();
            boxes.retain(|node, _| !projection.styles.contains_key(node));
        }
        *generated_boxes = projection;
        let index_started = detailed.then(std::time::Instant::now);
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
            .record_phase(FramePhase::Layout, count, ctx.id(), self.diagnostics);
        if let (Some(prep), Some(solve), Some(index)) = (prep_started, solve_started, index_started)
        {
            let (text_count, text_time) = self.measure_cache.end_text_timing();
            let ms = |d: std::time::Duration| d.as_secs_f64() * 1000.0;
            let mode = if dirty.is_some() {
                "invalidated"
            } else {
                "full"
            };
            println!(
                "[layout-work] mode={mode} boxes={count} measures={} arranges={} text_measures={text_count} styles_ms={:.3} solve_ms={:.3} text_ms={:.3} index_ms={:.3}",
                self.measure_cache.measured_count(),
                self.arrange_cache.arranged_count(),
                ms(solve - prep),
                ms(index - solve),
                ms(text_time),
                ms(index.elapsed()),
            );
        }
        relaid_out
    }
}
