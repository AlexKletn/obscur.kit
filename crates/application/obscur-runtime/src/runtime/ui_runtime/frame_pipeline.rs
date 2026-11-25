#![doc = include_str!("../../../docs/api/runtime/ui_runtime/frame_pipeline.doc/_module.md")]

use obscur_identity::NodeId;
use obscur_interaction::event::{EventOrigin, UiEvent};
use obscur_style::matching::Stylesheet;

use crate::frame::{FrameContext, FrameReason, FrameResult};
use crate::orchestration::{
    ComputedStyles, GeneratedContent, establishes_scroll_container, scroll_containers,
};
use crate::runtime::{
    ArrangePhase, AuthorSheets, ComposePhase, DesiredView, FoldInputs, FramePhaseClock,
    LayoutClassifier, LayoutInputs, LayoutPhase, PointerEventBindings, ReconcilePhase, StyleDelta,
    StylePhase, TextEditSession,
};

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/frame_pipeline.doc/run_frame.md")]
    pub(super) fn run_frame(
        &mut self,
        ctx: FrameContext,
        desired: DesiredView<'_>,
        sheets: &[Stylesheet],
    ) -> FrameResult {
        // The frame's clock starts HERE, at the first statement, and stops at the
        // last — not in the middle of the pipeline, where it used to. Started
        // where it was, everything below it (sheet retention with its two deep
        // clones, input dispatch with the whole of hit-testing, the resource
        // commit, choosing the plan) lay outside the measured interval, and the
        // four phase marks tiled exactly what remained — so `other` was zero by
        // arithmetic no matter how much of the frame was spent outside the four
        // phases (§3.0 M1). A subtotal that can only ever be zero is not a
        // measurement. An idle frame still returns before any mark, and is still
        // not recorded: it runs no phase, and counting it would move every mean.
        let phase_clock = FramePhaseClock::start();

        // Author-sheet retention: a frame given no sheets reuses the last non-empty
        // set, so a pure runtime-state change (scroll, hover) re-rendered with `&[]`
        // keeps the styled tree instead of restyling to the UA default (see
        // [`CascadeSheets::author_for_frame`]). Resolved HERE and not down in the
        // style phase, because a frame that skips styling must still remember the
        // sheets it was handed — the next bare frame reads them.
        let author_sheets = self.sheets.author_for_frame(sheets);

        // Phase 3 (§3.4): dispatch the input queued since the last frame — before
        // anything commits, and before the plan is chosen, so what dispatch
        // changes is what this frame renders rather than what the next one does.
        let dispatched = self.dispatch_queued_input();

        // §1.5: this is the deterministic synchronization point at which results
        // that came back from another thread are committed. Every off-thread load
        // returns as a typed message on `resource_rx`; draining here — before the
        // plan is chosen, so a landed image can dirty this frame rather than the
        // next — is what makes an installed `BlockingExecutor` self-servicing: a
        // host that renders frames needs no `poll_resource_tasks` discipline to
        // see its loads. With no executor installed nothing is ever sent, so this
        // is an empty `try_recv` and the frame is unchanged.
        let landed = self.commit_resource_results() > 0;

        // Phase 5 (§3.4): classify what this frame must recalculate, out of the
        // reason it was asked for, a staged transaction plan, what dispatch just
        // invalidated and what landed. The first frame over an empty tree is
        // always full so every retained output exists before later frames can
        // reuse it.
        let first_frame = self.tree.root().is_none();
        let plan = self.plan_for_frame(&ctx, first_frame, dispatched, landed);

        // Open the frame telemetry span (spec §7.9). A `NullTelemetrySink`
        // (the default) makes this and every per-phase record below a no-op, so
        // an unconfigured runtime pays essentially nothing.
        self.telemetry.open_frame();

        // An idle frame (e.g. a diagnostics capture) recalculates nothing: it
        // re-presents the retained scene without touching tree, styles or boxes.
        if plan.is_idle() {
            let snapshot = self.scene.snapshot();
            // Record that this frame ran zero phases so an observer still sees it.
            self.telemetry.close_frame(0);
            self.next_frame = self.next_frame.next();
            return FrameResult::new(ctx.id(), snapshot, false, false, false, true);
        }

        let scroll_anchors = self.capture_tracked_scroll_anchors();

        // 1. Reconcile the desired view into the retained tree (structural
        //    reasons only; input/animation/window frames leave structure alone,
        //    for which reconcile would be an idempotent no-op).
        //
        //    The mutation flows through the transaction commit boundary (spec
        //    §3.2): the enactor stages the reconcile as a validated
        //    `UiTransaction`, commits it atomically, and advances the commit
        //    version when the tree actually changed. `reconciled` is derived from
        //    whether the commit carried any operation — identical to the previous
        //    `!plan.is_empty()`.
        // Everything from `start()` to here belongs to no phase — it is the
        // prologue, and closing it without billing it to anything is what puts
        // it into `other` instead of hiding it inside `reconcile`.
        phase_clock.unattributed();
        let mut animating_canvas = false;
        let (reconciled, reconcile_roots) = if plan.reconcile() {
            // `tree`, `reconciler`, `enactor`, `components`, `virtual_lists`,
            // `scroll`, `telemetry` and `diagnostics` are disjoint fields: a legal
            // split borrow, and the whole of what settling the tree touches.
            let viewport_scroll_y = self.surface.viewport_scroll().1;
            // A stylesheet or environment change makes the later style scope fall back to a full pass.
            let previous_sheets = self.last_style.as_ref().map(|last| last.sheets.as_slice());
            ReconcilePhase::new(
                &mut self.tree,
                &self.reconciler,
                &mut self.enactor,
                &mut self.components,
                &mut self.virtual_lists,
                &mut self.scroll,
                &mut self.telemetry,
                self.diagnostics.as_ref(),
            )
            .run(
                &ctx,
                desired,
                viewport_scroll_y,
                previous_sheets,
                phase_clock.detailed(),
            )
        } else {
            (false, Some(Vec::new()))
        };

        // 1a. Controlled-input reseed (WHATWG HTML §4.10.5.5 *value reflection*): a
        //     structural frame may have rewritten the focused field's retained
        //     value from the app model (a "Clear" button, or a normalizing
        //     `on-input` that writes back a trimmed/uppercased value). If the new
        //     retained value is an EXTERNAL change — not the session's own edit
        //     mirrored back — reseed the editing session from it now, BEFORE the
        //     measure/compose phases read the session, so the field shows the model
        //     value THIS frame instead of the stale buffer until blur (Defect A).
        //     Gated on `reconcile()` because only a structural frame can change the
        //     retained value; on a plain keystroke (`Input`/`TextEdit`) frame this
        //     never runs, so the user's in-flight edit is never fought.
        if plan.reconcile() {
            self.reseed_editing_from_retained();
        }

        // 1b. Re-answer "does anything bind a raw pointer event?" — the gate that
        //     keeps `pointermove` free. A binding can only appear or disappear on
        //     a commit that carried an operation, so this is the one moment the
        //     answer can change, and it costs one pass over the live nodes on a
        //     frame that already walks the tree several times. Asking per pointer
        //     event instead would mean an ancestor walk — and its allocation — at
        //     the pointer's report rate for an application that binds none of the
        //     four.
        if reconciled {
            self.refresh_portals();
            self.arrange_cache.forget_topology();
            self.pointer_bindings = PointerEventBindings::scan(&self.tree);
            // 1c. …and re-answer "is the focused node still there?". A structural
            //     frame can remove it, and nothing used to notice: `focused` went
            //     on naming a dead id. It degraded quietly wherever a dispatch
            //     looked the node up and found nothing, but the accessibility
            //     projection is built from this field and published the stale id
            //     to the platform — a screen reader following focus into a node
            //     that no longer exists. Cleared through the same one door every
            //     other focus move goes through, so the editing session is
            //     released with it.
            self.drop_focus_if_removed();
            // 1d. …and the same question about every other map keyed by a node.
            //     Focus was the one field anybody had thought to ask it of; the
            //     computed styles, the cascade inputs, the pseudo-class states
            //     and the scroll offsets all outlived their nodes, and the style
            //     maps are walked whole every composing frame.
            self.drop_state_of_removed_nodes();
        }

        // 2-3. Match + style: rebuild cascade inputs and resolve computed styles.
        //    When the style phase is skipped, reuse the retained computed styles
        //    so a downstream layout/compose still has values to work from.
        // `tree`, `pseudo`, `scene_composer`, `matcher`, `style`, `inputs`,
        // `overlaid`, `computed`, `last_style`, `generated_content`, `sheets`,
        // `motion`, `telemetry` and `diagnostics` are disjoint fields: a legal
        // split borrow, and the whole of what styling this frame touches.
        let (mut computed, mut restyled, delta) = self.style_for_frame(
            &ctx,
            plan.style(),
            author_sheets,
            reconcile_roots.as_deref(),
            reconciled,
            &phase_clock,
        );
        // The style phase ends HERE, where styling ends — not after block 3c
        // below, which is compose's work and used to be billed to style (§3.0
        // M2). The work is cheap on a scene with no images, so this moves a
        // boundary rather than any real time; a boundary that lies is still a
        // boundary that lies.
        phase_clock.mark_style();

        // 3c. Resolve every image this frame references — `background-image`
        //     URLs and element `src`s — into the decoded-image/vector caches
        //     BEFORE layout. Two things need it here rather than at compose: a
        //     replaced element with no authored size is measured against its
        //     content's intrinsic size (see `size_replaced_elements`), which the
        //     arrange pass below reads; and the §5.19 cache is then touched
        //     exactly once per frame, so its hit/miss/eviction metrics count one
        //     lookup per URL. `mark_image_visibility` runs first (against the
        //     retained boxes) so an eviction the loads force drops what this frame
        //     stopped showing, not what it is about to paint. Gated on `compose()`
        //     because a frame that will not paint has no reason to load; the later
        //     compose phase then finds every one cached.
        if plan.compose() {
            self.mark_image_visibility(&computed);
            self.resolve_background_images(&computed);
            self.resolve_element_images();
            self.resolve_element_vectors();
        }
        // Billed to compose, which is whose work it is, and added to — not
        // replacing — what the display-list build below will bill.
        phase_clock.mark_images();

        // 4. Layout: map to layout styles, arrange, and retain the boxes.
        //    The document root's `font-size` supplies the `rem` base; the
        //    viewport supplies `vw`/`vh` (spec §4.5). When skipped, the retained
        //    boxes and hit-test index carry over unchanged.
        //
        //    The plan's flag is a conservative upper bound derived from the frame
        //    REASON; now that the reconcile result and the real computed styles
        //    exist, refine it into what ACTUALLY changed (spec §3.4 phase 5 + step
        //    8, §4.3) — a hover that only shifts a fill reuses every retained box,
        //    even on the reactive path where every frame is classified structural,
        //    and a hover that DOES move one box re-measures that box's contexts
        //    rather than the tree.
        let current_inputs = self.current_layout_inputs(&ctx);
        let mut layout_phase = LayoutClassifier::new(
            self.last_layout_inputs.as_ref(),
            &current_inputs,
            &self.computed,
            &self.overlaid,
        )
        .classify(plan, &ctx, first_frame, reconciled, &computed, &delta);
        if restyled
            && (self.generated_boxes.is_some()
                || self
                    .generated_content
                    .values()
                    .any(GeneratedContent::has_boxes))
            && matches!(layout_phase, LayoutPhase::Skip)
        {
            layout_phase = LayoutPhase::Full;
        }
        // Thread the runtime-owned editing session into the measure pass so the
        // focused editable is sized from the live session buffer — see
        // [`ArrangePhase::run`]. `editing`, `layout`, `tree`, `scene_composer`,
        // `resources`, the two caches, `boxes`, `hits`, `last_layout_inputs` and
        // `telemetry` are disjoint fields: a legal split borrow.
        let mut relaid_out = self.arrange_for_frame(
            &ctx,
            &mut computed,
            &layout_phase,
            current_inputs,
            &phase_clock,
        );
        // Correct hover before composing. At most one correction can affect the
        // current frame, even if :hover itself changes the hit geometry.
        let correcting_hover = std::mem::take(&mut self.hover_correction_pending);
        let probe_hover =
            plan.compose() && self.hover_pointer.is_some() && (!correcting_hover || reconciled);
        let hover_corrected = probe_hover && self.refresh_hover_with_styles(&mut computed);
        if hover_corrected {
            self.computed = computed;
            let sheets = self.sheets.author_for_frame(sheets);
            let (mut corrected, changed, delta) =
                self.style_for_frame(&ctx, true, sheets, Some(&[]), false, &phase_clock);
            restyled |= changed;
            self.resolve_background_images(&corrected);
            let inputs = self.current_layout_inputs(&ctx);
            let mut phase = LayoutClassifier::new(
                self.last_layout_inputs.as_ref(),
                &inputs,
                &self.computed,
                &self.overlaid,
            )
            .classify(plan, &ctx, false, false, &corrected, &delta);
            if changed
                && (self.generated_boxes.is_some()
                    || self
                        .generated_content
                        .values()
                        .any(GeneratedContent::has_boxes))
                && matches!(phase, LayoutPhase::Skip)
            {
                phase = LayoutPhase::Full;
            }
            relaid_out |=
                self.arrange_for_frame(&ctx, &mut corrected, &phase, inputs, &phase_clock);
            computed = corrected;
        }
        if restyled || relaid_out || reconciled {
            self.scroll.retain_nodes(|node| {
                self.boxes.contains_key(&node) && establishes_scroll_container(&computed, node)
            });
        }
        phase_clock.mark_layout();
        self.restore_tracked_scroll_anchors(scroll_anchors, &computed);
        if plan.reconcile() {
            self.start_shared_elements(&computed);
        }
        self.virtual_lists
            .dump_geometry(&self.tree, &self.boxes, &self.scroll);
        // Diagnostics, not layout and not compose: closed unbilled so it lands
        // in `other` rather than inflating the phase it happens to sit next to.
        phase_clock.unattributed();

        // 5. Compose: build the display list, install it, publish a snapshot.
        //    When skipped, the retained scene's current snapshot is republished.
        if plan.compose() {
            if relaid_out
                && let Some(session) = self.editing.as_mut()
                && let Some(scroll_left) = crate::runtime::CaretScroll::new(
                    &self.tree,
                    &computed,
                    &self.boxes,
                    &self.scene_composer,
                )
                .horizontal(session)
            {
                session.set_scroll_left(scroll_left);
            }
            // Thread the runtime-owned editing session into compose so the focused
            // editable paints its live session text plus a caret + selection
            // highlight — see [`ComposePhase::run`]. `editing`, `scene`,
            // `scene_composer`, `tree`, `boxes`, `resources`, `scroll`,
            // `generated_content`, `motion`, `inspector`, `surface` and `telemetry`
            // are disjoint fields: a legal split borrow.
            // §5.3: every canvas's painter runs HERE — after layout, so it knows
            // its own box, and before compose, because it is application code
            // that needs `&mut` while the compose walk takes `&self`. What it
            // draws is spliced into its node's own paint unit. Run BEFORE the
            // borrows below, because painting is the one step in this block that
            // needs `&mut self`.
            let (canvases, canvas_animating) = self.canvas_painters.paint(
                &self.tree,
                &self.boxes,
                &computed,
                self.logical_time.as_secs_f32(),
                self.diagnostics.as_ref(),
            );
            // §4.13: what each painter declared as its own hit region, handed to
            // the tester that will be asked about it on every pointer report.
            self.hits
                .set_canvas_regions(self.canvas_painters.hits().clone());
            let editing = self.editing.as_ref().map(TextEditSession::view);
            let hovered = self.hover.topmost();
            ComposePhase::new(
                &mut self.scene,
                &self.scene_composer,
                &self.tree,
                &self.boxes,
                &mut self.hits,
                &self.resources,
                &self.scroll,
                &self.generated_content,
                self.generated_boxes.as_ref(),
                &canvases,
                &self.motion,
                &self.inspector,
                &self.surface,
                &mut self.telemetry,
                &*self.diagnostics,
            )
            .run(
                &ctx,
                &mut computed,
                editing,
                hovered,
                probe_hover && !hover_corrected,
                &phase_clock,
            );
            // A painter that says it is still moving must be asked again, or it
            // draws one frame and freezes. One that says it is not must not be,
            // or an idle application renders forever. Requested after the block
            // so the borrows above are done with.
            animating_canvas = canvas_animating;
        }
        self.scroll_snaps
            .retain(|node, _| self.tree.contains(*node));
        // The tick's animation request was coalesced into this frame. Keep a
        // request queued for the next tick as long as a timeline is running.
        if animating_canvas
            || self.motion.active_count() > 0
            || self.scroll.has_motion()
            || self.scroll_snaps.values().any(|snap| snap.is_pending())
            || self.pointers.has_flings()
            || (self.drag.is_active() && self.drag_scroll_point.is_some())
            || (self.pointers.text_drag().is_some() && self.selection_scroll_point.is_some())
        {
            self.request_timeline_frame();
            self.timeline_waiting_for_tick = true;
        }
        let snapshot = self.scene.snapshot();

        // A component that called `request_update` during its own render is still
        // stale: the request is deliberately not serviced inside the frame that
        // raised it (that would let a component spin the frame forever), so ask
        // the scheduler for the next one. Coalescing is the scheduler's job, so
        // many requesting components still cost one frame.
        if self.components.has_stale() {
            // …and say how many, because "the loop turns and nothing changes on
            // screen" and "the loop turns because there is work" are the same
            // picture from outside and opposite faults. A frame that ends with
            // components still stale has asked for the next one; a run where
            // every frame ends that way is a loop feeding itself. The count is
            // only taken when someone is watching — see `note_stale`.
            phase_clock.note_stale(|| self.components.stale_count());
            self.scheduler.request_reason(FrameReason::UiCommit);
        }

        // Close the frame span with a counter of how many phases actually ran,
        // so an observer can attribute work to this frame (spec §7.9).
        let phases_run = u64::from(plan.reconcile())
            + u64::from(plan.style())
            + u64::from(plan.layout())
            + u64::from(plan.compose());
        self.telemetry.close_frame(phases_run);

        // Publish the accessibility projection of what this frame committed (spec
        // §7.1). It runs here, after reconcile has settled the tree and dispatch
        // has settled focus, so what a screen reader is told is what this frame
        // actually produced — not the previous frame's tree. A no-op unless a host
        // installed a bridge.
        self.computed = computed;
        if self.focused.is_some_and(|node| self.is_css_hidden(node)) {
            self.move_focus_to(None, UiEvent::detached(EventOrigin::Synthetic));
            self.request_frame(FrameReason::Input);
        }
        self.publish_accessibility(reconciled || plan.compose());

        // Refresh query containers, then advance the frame counter.
        if restyled || relaid_out || reconciled {
            self.update_query_containers(ctx.viewport());
        }
        self.next_frame = self.next_frame.next();

        // Refresh the viewport-relative projection the public `layout_boxes`
        // accessor exposes now that this frame's boxes, styles and scroll state
        // have settled (spec §4.4). A no-op reference swap when nothing overflows.
        self.refresh_scroll_folded_boxes();
        if plan.compose()
            && !hover_corrected
            && (!correcting_hover || reconciled)
            && self.refresh_hover_at_cursor()
        {
            self.hover_correction_pending = true;
            self.request_frame(FrameReason::Input);
        }
        self.publish_geometry_observations();

        // The frame is over. Everything since `start()` that no phase claimed —
        // the prologue above, the snapshot, the accessibility publish, the
        // computed install, the scroll refresh — is `other`, and it is now a
        // number that can be non-zero.
        phase_clock.finish();

        FrameResult::new(ctx.id(), snapshot, reconciled, restyled, relaid_out, true)
    }

    /// Matches, cascades and applies the frame’s motion overlay.
    fn style_for_frame(
        &mut self,
        ctx: &FrameContext,
        restyle: bool,
        author_sheets: AuthorSheets,
        reconcile_roots: Option<&[NodeId]>,
        reconciled: bool,
        phase_clock: &FramePhaseClock,
    ) -> (ComputedStyles, bool, StyleDelta) {
        StylePhase::new(
            &self.tree,
            &self.pseudo,
            self.environment,
            self.media_preferences,
            self.media_input(),
            self.surface.device_scale(),
            &self.container_queries,
            self.scene_composer.paragraph(),
            &mut self.matcher,
            &mut self.style,
            &mut self.inputs,
            &mut self.overlaid,
            &mut self.inline_styles,
            &mut self.computed,
            &mut self.last_style,
            &mut self.generated_content,
            &self.sheets,
            &mut self.motion,
            &mut self.telemetry,
            self.diagnostics.as_ref(),
        )
        .run(
            ctx,
            restyle,
            author_sheets,
            reconcile_roots,
            reconciled,
            phase_clock,
        )
    }

    /// Arranges a set of computed styles and refreshes the hit index if needed.
    fn arrange_for_frame(
        &mut self,
        ctx: &FrameContext,
        computed: &mut ComputedStyles,
        layout_phase: &LayoutPhase,
        current_inputs: LayoutInputs,
        phase_clock: &FramePhaseClock,
    ) -> bool {
        let editing = self.editing.as_ref().map(TextEditSession::view);
        let viewport_scroll = self.surface.viewport_scroll();
        let relaid_out = ArrangePhase::new(
            &self.layout,
            &mut self.layout_styles,
            &self.container_queries,
            &self.tree,
            &self.scene_composer,
            &self.resources,
            &self.measure_cache,
            &self.arrange_cache,
            &mut self.boxes,
            &mut self.hits,
            &mut self.last_layout_inputs,
            &mut self.telemetry,
            &*self.diagnostics,
        )
        .run(
            ctx,
            computed,
            editing,
            layout_phase,
            viewport_scroll,
            current_inputs,
            phase_clock.detailed(),
            &self.generated_content,
            &mut self.generated_boxes,
        );
        if !matches!(layout_phase, LayoutPhase::Skip)
            && let Some(node) = self.measure_cache.depth_limit_node()
        {
            let mut diagnostic = obscur_diagnostics::Diagnostic::new(
                obscur_diagnostics::DiagnosticCode::new("layout.depth_limit"),
                obscur_diagnostics::DiagnosticSeverity::Warning,
                obscur_diagnostics::DiagnosticSubsystem::Layout,
                "the layout recursion limit was reached; deeper nodes have no layout box",
            );
            diagnostic.context.node = Some(node);
            self.report_diagnostic(&diagnostic);
        }
        if !matches!(layout_phase, LayoutPhase::Skip)
            && let Some(node) = self.measure_cache.grid_limited_item()
        {
            let mut diagnostic = obscur_diagnostics::Diagnostic::new(
                obscur_diagnostics::DiagnosticCode::new("layout.grid_track_limit"),
                obscur_diagnostics::DiagnosticSeverity::Warning,
                obscur_diagnostics::DiagnosticSubsystem::Layout,
                "grid placement was clamped to the implicit track limit",
            )
            .with_recovery(obscur_diagnostics::RecoveryAction::Fallback);
            diagnostic.context.node = Some(node);
            self.report_diagnostic(&diagnostic);
        }
        relaid_out
    }

    /// Snapshots the non-style inputs the layout phase reads (see
    /// [`LayoutInputs`]), so a later frame can tell whether the retained boxes
    /// are still valid.
    fn current_layout_inputs(&self, ctx: &FrameContext) -> LayoutInputs {
        LayoutInputs {
            viewport: ctx.viewport(),
            scroll: self.surface.viewport_scroll(),
            editing: self.editing_view().map(|view| {
                (
                    view.node,
                    view.text.to_owned(),
                    view.preedit.map(|preedit| preedit.text.to_owned()),
                )
            }),
            language: self.locale.language().into(),
        }
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/frame_pipeline.doc/refresh_scroll_folded_boxes.md")]
    fn refresh_scroll_folded_boxes(&mut self) {
        let containers = scroll_containers(&self.tree, &self.computed, &self.boxes);
        self.scroll.refold(FoldInputs {
            tree: &self.tree,
            boxes: &self.boxes,
            containers: &containers,
        });
    }
}
