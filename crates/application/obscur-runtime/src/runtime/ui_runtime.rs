//! The embeddable UI runtime and application-core integrator: [`UiRuntime`].

use std::fmt;

use obscur_geometry::Size;
use obscur_identity::{NodeId, RuntimeId};
use obscur_interaction::command::PhysicalKey;
use obscur_interaction::hover::{HoverChain, HoverTransition};
use obscur_interaction::pointer::{PointerEvent, PointerEventKind};
use obscur_interaction::spatial::{HitTest, SpatialIndex};
use obscur_scene::scene::{Scene, SceneSnapshot};
use obscur_style::matching::Stylesheet;
use obscur_style::selector::PseudoState;
use obscur_tree::reconciliation::Reconciler;
use obscur_tree::tree::RetainedTree;
use obscur_view::view::View;

use super::PointerDispatch;
use crate::frame::{FrameContext, FrameId, FrameReason, FrameResult};
use crate::orchestration::{
    LayoutBoxes, LayoutCoordinator, PseudoStates, SceneCoordinator, StyleCoordinator, StyleInputs,
    StyleMatchingCoordinator,
};
use crate::scheduler::{BackpressureDecision, FrameScheduler};

/// The embeddable UI runtime: the application-core **integrator** the catalog
/// mandates (spec §1.4, §3.4–§3.7).
///
/// Unlike the bare skeleton it replaces, this `UiRuntime` *owns and integrates*
/// the moving parts of a live UI:
///
/// - the retained [`RetainedTree`] and the mutable [`Scene`] carried across
///   frames;
/// - the orchestration coordinators (reconcile → match → style → layout → scene)
///   it drives in a fixed, deterministic order — reimplementing no domain logic;
/// - the event-driven [`FrameScheduler`] that gates when a frame actually runs
///   (§3.5); and
/// - the per-node [`PseudoStates`] interaction map that finally lets authored
///   `:hover`/`:focus` rules match (§3.10, Phase 3 input → styling).
///
/// It never owns the process or an event loop: a host drives it by
/// [`request_frame`](Self::request_frame)-ing as events arrive and calling
/// [`run_due_frame`](Self::run_due_frame) when it is ready to produce one.
///
/// It retains the last frame's absolute [`LayoutBoxes`] (see
/// [`layout_boxes`](Self::layout_boxes)) — unlike [`FramePipeline`], which
/// discards them — because hit-testing (Phase 3) needs them.
///
/// # Relationship to `HeadlessHost`
///
/// [`HeadlessHost`](../../obscur_headless/host/struct.HeadlessHost.html) remains
/// the deterministic, windowless **test host** that wraps
/// [`FramePipeline`](crate::frame::FramePipeline) and force-renders every call.
/// `UiRuntime` is the *production* integrator: same deterministic core, but
/// scheduler-gated and stateful (retained boxes, live pseudo-state). The two are
/// intentionally distinct entry points.
///
/// [`FramePipeline`]: crate::frame::FramePipeline
pub struct UiRuntime {
    /// Identity of this runtime.
    id: RuntimeId,
    /// The retained tree carried across frames.
    tree: RetainedTree,
    /// The mutable scene composed into each frame.
    scene: Scene,
    /// Turns the desired view tree into retained-tree structure.
    reconciler: Reconciler,
    /// Matches authored stylesheets (pseudo-state aware) into cascade inputs.
    matcher: StyleMatchingCoordinator,
    /// Resolves computed styles over the retained tree.
    style: StyleCoordinator,
    /// Drives layout over the retained tree.
    layout: LayoutCoordinator,
    /// Composes the paint-ordered display list.
    scene_composer: SceneCoordinator,
    /// The event-driven scheduler gating [`run_due_frame`](Self::run_due_frame).
    scheduler: FrameScheduler,
    /// The runtime-owned per-node interaction state feeding the matcher.
    pseudo: PseudoStates,
    /// The absolute layout boxes retained from the last produced frame.
    boxes: LayoutBoxes,
    /// The cascade inputs computed for the last produced frame.
    inputs: StyleInputs,
    /// The z-order-aware hit-test index, rebuilt from `boxes` after each frame.
    spatial: SpatialIndex,
    /// The nodes currently under the pointer, topmost-first (hover target at
    /// index `0`).
    hover: HoverChain,
    /// The node that currently holds focus, if any.
    ///
    /// Tracked directly rather than via
    /// [`FocusTree`](obscur_interaction::focus::FocusTree): the focus tree
    /// requires nodes to be pre-registered as [`FocusNode`]s, which this runtime
    /// does not yet do, so a plain `Option<NodeId>` is the simpler correct model
    /// for pointer-press focus (documented fallback).
    focused: Option<NodeId>,
    /// The id assigned to the next produced frame.
    next_frame: FrameId,
    /// The viewport frames are laid out under.
    viewport: Size,
}

impl UiRuntime {
    /// Creates a runtime with the given identity and viewport.
    ///
    /// The runtime starts with an empty retained tree and scene, fresh
    /// coordinators, an idle scheduler, no recorded interaction state, and the
    /// first produced frame at [`FrameId::FIRST`].
    #[must_use]
    pub fn new(id: RuntimeId, viewport: Size) -> Self {
        Self {
            id,
            tree: RetainedTree::new(),
            scene: Scene::new(),
            reconciler: Reconciler::new(),
            matcher: StyleMatchingCoordinator::new(),
            style: StyleCoordinator::new(),
            layout: LayoutCoordinator::new(),
            scene_composer: SceneCoordinator::new(),
            scheduler: FrameScheduler::new(),
            pseudo: PseudoStates::new(),
            boxes: LayoutBoxes::new(),
            inputs: StyleInputs::new(),
            spatial: SpatialIndex::new(),
            hover: HoverChain::new(),
            focused: None,
            next_frame: FrameId::FIRST,
            viewport,
        }
    }

    /// Requests a frame for `reason`, returning the scheduler's admission
    /// [`BackpressureDecision`] (§3.5).
    ///
    /// This is the event ingress: a host calls it as input, animation ticks,
    /// resource completions, resizes, etc. arrive. The request is coalesced and
    /// prioritised by the [`FrameScheduler`]; nothing runs until
    /// [`run_due_frame`](Self::run_due_frame) is called.
    pub fn request_frame(&mut self, reason: FrameReason) -> BackpressureDecision {
        self.scheduler.request_reason(reason)
    }

    /// Returns whether any frame request is pending in the scheduler.
    #[must_use]
    pub fn has_pending_frame(&self) -> bool {
        self.scheduler.has_pending()
    }

    /// Returns the number of pending (coalesced) frame requests.
    #[must_use]
    pub fn pending_frames(&self) -> usize {
        self.scheduler.pending_count()
    }

    /// Runs **one** frame if the scheduler has a request pending, returning its
    /// [`FrameResult`]; returns [`None`] when nothing is due.
    ///
    /// This is the scheduler-wired frame loop that closes Phase 2: it pulls at
    /// most one most-urgent request via
    /// [`FrameScheduler::next_frame`](crate::scheduler::FrameScheduler::next_frame),
    /// runs the full deterministic core for it (reconcile → pseudo-aware match →
    /// style → layout with retained boxes → scene), and advances the frame
    /// counter. Because requests coalesce, a burst of same-reason events produces
    /// exactly one frame. When idle it returns [`None`] without side effects — the
    /// runtime never busy-loops (§3.5).
    pub fn run_due_frame(&mut self, desired: &View, sheets: &[Stylesheet]) -> Option<FrameResult> {
        let request = self.scheduler.next_frame()?;
        let ctx = FrameContext::new(self.next_frame, request.reason(), self.viewport);
        Some(self.run_frame(ctx, desired, sheets))
    }

    /// Force-renders one frame **ignoring the scheduler**, returning its
    /// [`FrameResult`].
    ///
    /// This is the unconditional path for first paint and tests: it runs the same
    /// deterministic core as [`run_due_frame`](Self::run_due_frame) with reason
    /// [`FrameReason::UiCommit`], regardless of whether any request is pending.
    /// Two `render_now` calls over the same view + sheets + pseudo-state yield
    /// identical display lists.
    pub fn render_now(&mut self, desired: &View, sheets: &[Stylesheet]) -> FrameResult {
        let ctx = FrameContext::new(self.next_frame, FrameReason::UiCommit, self.viewport);
        self.run_frame(ctx, desired, sheets)
    }

    /// Runs the deterministic core over the runtime-owned state, retaining the
    /// laid-out boxes and cascade inputs, and advancing the frame counter.
    ///
    /// Mirrors [`FramePipeline::run_matched`](crate::frame::FramePipeline::run_matched)
    /// stage-for-stage, differing only in that it (a) sources cascade inputs
    /// through the **pseudo-state-aware** matcher and (b) *keeps* the layout
    /// boxes instead of discarding them.
    fn run_frame(&mut self, ctx: FrameContext, desired: &View, sheets: &[Stylesheet]) -> FrameResult {
        // 1. Reconcile the desired view into the retained tree.
        let plan = self.reconciler.reconcile_root(&mut self.tree, desired);
        let reconciled = !plan.is_empty();

        // 2. Match authored sheets against the fresh tree, honouring live
        //    interaction state so `:hover`/`:focus` participate.
        let inputs = self
            .matcher
            .build_inputs_with_pseudo(&self.tree, sheets, &self.pseudo);

        // 3. Style: resolve computed styles over the retained tree.
        let computed = self.style.resolve(&self.tree, &inputs);
        let restyled = !computed.is_empty();

        // 4. Layout: map to layout styles, arrange, and retain the boxes.
        let layout_styles = self.layout.layout_styles_from(&computed);
        let boxes = self.layout.layout(&self.tree, &layout_styles, ctx.viewport());
        let relaid_out = !boxes.is_empty();

        // 5. Compose: build the display list, install it, publish a snapshot.
        let display_list = self.scene_composer.compose(&self.tree, &computed, &boxes);
        self.scene.set_root(display_list);
        let snapshot = self.scene.snapshot();

        // Retain cross-frame state for hit-testing and diagnostics.
        self.boxes = boxes;
        self.inputs = inputs;
        // Rebuild the hit-test index from the freshly laid-out boxes so the next
        // pointer event tests against this frame's geometry (both `render_now`
        // and `run_due_frame` route through here, so both stay consistent).
        self.rebuild_spatial_index();
        self.next_frame = self.next_frame.next();

        FrameResult::new(ctx.id(), snapshot, reconciled, restyled, relaid_out, true)
    }

    /// Rebuilds the spatial hit-test index from the retained [`LayoutBoxes`].
    ///
    /// Every node's absolute border-box `bounds` is inserted with a `z_order`
    /// equal to its **logical depth** — the number of logical-parent links from
    /// the tree root. Deeper (more nested) nodes therefore carry a higher
    /// z-order, so [`SpatialIndex::hits_at`] returns the innermost element first
    /// and a pointer over overlapping ancestor/descendant boxes resolves to the
    /// topmost (innermost) node, matching paint order. Ties at equal depth break
    /// deterministically by [`NodeId`] inside the index.
    fn rebuild_spatial_index(&mut self) {
        self.spatial.clear();
        for (&node, layout_box) in &self.boxes {
            let z_order = self.logical_depth(node);
            self.spatial.insert(node, layout_box.bounds, z_order);
        }
    }

    /// Returns the logical depth of `node`: the number of logical-parent links
    /// between it and the tree root (root itself is depth `0`).
    ///
    /// The walk is bounded by the node count so a malformed (cyclic) parent
    /// chain can never loop forever.
    fn logical_depth(&self, node: NodeId) -> i32 {
        let mut depth: i32 = 0;
        let mut current = node;
        // The tree is a DAG in practice; cap the walk at the node count so a
        // pathological cycle terminates deterministically.
        let cap = self.tree.len();
        for _ in 0..cap {
            match self
                .tree
                .get(current)
                .and_then(|n| n.logical_parent().parent())
            {
                Some(parent) => {
                    depth = depth.saturating_add(1);
                    current = parent;
                }
                None => break,
            }
        }
        depth
    }

    /// Records `state` as `node`'s interaction state for subsequent frames.
    ///
    /// This does **not** itself schedule a frame: the caller (the input step) is
    /// responsible for calling [`request_frame`](Self::request_frame) — typically
    /// with [`FrameReason::Input`] — so the state change actually reaches paint.
    /// Keeping the two explicit avoids hidden frame requests.
    pub fn set_pseudo_state(&mut self, node: NodeId, state: PseudoState) {
        self.pseudo.set(node, state);
    }

    /// Clears every recorded interaction state; subsequent frames match as if no
    /// node were hovered/focused/active.
    ///
    /// Like [`set_pseudo_state`](Self::set_pseudo_state), this does not schedule a
    /// frame; the caller requests one explicitly.
    pub fn clear_pseudo_states(&mut self) {
        self.pseudo.clear();
    }

    /// Returns the interaction state currently recorded for `node`, or
    /// [`PseudoState::empty`] when none.
    #[must_use]
    pub fn pseudo_state(&self, node: NodeId) -> PseudoState {
        self.pseudo.get(node)
    }

    /// Dispatches a hit-tested pointer event: updates the hover chain and focus,
    /// recomputes the per-node [`PseudoStates`], and schedules a frame when
    /// anything changed (§3.10, §4.7).
    ///
    /// The flow is:
    ///
    /// 1. **Hit-test** `event.position` against the retained spatial index; the
    ///    topmost hit (highest z-order = innermost element) is the `target`.
    /// 2. **Hover** — build the new hover chain as `target` plus its element
    ///    ancestors (topmost-first toward root) and diff it against the old chain
    ///    via [`HoverTransition`]. A non-empty transition marks `hover_changed`.
    /// 3. **Focus** — a [`PointerEventKind::Down`] press moves focus to `target`;
    ///    `focus_changed` records whether that differed from the prior focus.
    /// 4. **Recompute pseudo-states** — when hover or focus changed, clear the
    ///    interaction map, set [`PseudoState::HOVER`] on every node in the hover
    ///    chain, then merge [`PseudoState::FOCUS`] into the focused node's state.
    /// 5. **Schedule** — if anything changed, request a
    ///    [`FrameReason::Input`] frame so the next `run_due_frame`/`render_now`
    ///    restyles with the updated `:hover`/`:focus` state.
    ///
    /// It never itself renders; it only mutates interaction state and requests a
    /// frame. Returns a [`PointerDispatch`] describing the target and what moved.
    pub fn handle_pointer(&mut self, event: PointerEvent) -> PointerDispatch {
        // 1. Hit-test: the first hit is the topmost (highest z-order) node.
        let target = self
            .spatial
            .hits_at(event.position())
            .first()
            .map(HitTest::node);

        // 2. Hover: chain = target + its element ancestors, topmost-first.
        let new_hover = HoverChain::from_nodes(self.hover_chain_for(target));
        let transition = HoverTransition::between(&self.hover, &new_hover);
        let hover_changed = transition.has_changes();
        self.hover = new_hover;

        // 3. Focus: only a press moves focus.
        let mut focus_changed = false;
        if event.kind() == PointerEventKind::Down && self.focused != target {
            self.focused = target;
            focus_changed = true;
        }

        // 4. Recompute the interaction pseudo-states from the new hover/focus.
        if hover_changed || focus_changed {
            self.recompute_pseudo_states();
            // 5. Schedule a restyle for the next frame.
            self.request_frame(FrameReason::Input);
        }

        PointerDispatch::new(target, hover_changed, focus_changed)
    }

    /// Builds the hover chain for `target`: the target followed by its element
    /// (and root) ancestors, topmost-first toward the root.
    ///
    /// Returns an empty vector when `target` is `None` (the pointer is over
    /// nothing). The walk follows logical parents and is bounded by the node
    /// count so a malformed parent chain terminates.
    fn hover_chain_for(&self, target: Option<NodeId>) -> Vec<NodeId> {
        let Some(target) = target else {
            return Vec::new();
        };
        let mut chain = vec![target];
        let mut current = target;
        let cap = self.tree.len();
        for _ in 0..cap {
            match self
                .tree
                .get(current)
                .and_then(|n| n.logical_parent().parent())
            {
                Some(parent) => {
                    chain.push(parent);
                    current = parent;
                }
                None => break,
            }
        }
        chain
    }

    /// Recomputes the per-node [`PseudoStates`] from the current hover chain and
    /// focus: [`PseudoState::HOVER`] on every hovered node, with
    /// [`PseudoState::FOCUS`] merged into the focused node.
    fn recompute_pseudo_states(&mut self) {
        self.pseudo.clear();
        for &node in self.hover.nodes() {
            self.pseudo.set(node, PseudoState::HOVER);
        }
        if let Some(focused) = self.focused {
            let merged = self.pseudo.get(focused).union(PseudoState::FOCUS);
            self.pseudo.set(focused, merged);
        }
    }

    /// Routes a physical key event to the currently focused node and schedules a
    /// frame.
    ///
    /// This is intentionally minimal: it resolves the routing *target* (the
    /// focused node) and requests a frame so any focus-driven visual can update,
    /// but it does **not** yet translate the key into a command or run any
    /// handler — there are no component handlers in this runtime. Full command
    /// routing (shortcut matching, capture/bubble dispatch) is a later Phase 3
    /// step. Returns the node the event was routed to, if any.
    pub fn handle_key(&mut self, _key: PhysicalKey, _pressed: bool) -> Option<NodeId> {
        let target = self.focused;
        if target.is_some() {
            self.request_frame(FrameReason::Input);
        }
        target
    }

    /// Returns the node currently under the pointer (the hover target), if any.
    #[must_use]
    pub fn hovered(&self) -> Option<NodeId> {
        self.hover.topmost()
    }

    /// Returns the node that currently holds focus, if any.
    #[must_use]
    pub fn focused(&self) -> Option<NodeId> {
        self.focused
    }

    /// Returns the retained tree carried across frames.
    #[must_use]
    pub fn tree(&self) -> &RetainedTree {
        &self.tree
    }

    /// Returns the mutable scene's current state (as a shared reference).
    #[must_use]
    pub fn scene(&self) -> &Scene {
        &self.scene
    }

    /// Returns an immutable snapshot of the current scene — the hand-off a
    /// renderer consumes.
    #[must_use]
    pub fn snapshot(&self) -> SceneSnapshot {
        self.scene.snapshot()
    }

    /// Returns the absolute layout boxes retained from the last produced frame.
    ///
    /// Empty until the first frame runs. This is the map hit-testing walks to map
    /// a pointer position back to a [`NodeId`].
    #[must_use]
    pub fn layout_boxes(&self) -> &LayoutBoxes {
        &self.boxes
    }

    /// Returns this runtime's identity.
    #[must_use]
    pub fn runtime_id(&self) -> RuntimeId {
        self.id
    }

    /// Returns this runtime's identity (alias of [`runtime_id`](Self::runtime_id)).
    #[must_use]
    pub fn id(&self) -> RuntimeId {
        self.id
    }

    /// Returns the viewport frames are laid out under.
    #[must_use]
    pub fn viewport(&self) -> Size {
        self.viewport
    }

    /// Sets the viewport subsequent frames are laid out under.
    ///
    /// Does not itself schedule a frame; a host typically pairs this with
    /// [`request_frame`](Self::request_frame)`(`[`FrameReason::WindowChange`]`)`.
    pub fn set_viewport(&mut self, viewport: Size) {
        self.viewport = viewport;
    }
}

impl fmt::Debug for UiRuntime {
    /// Reports the stable identity fields — runtime id, viewport, next frame, and
    /// pending frame count — without touching the opaque tree, scene, or
    /// coordinator internals.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UiRuntime")
            .field("id", &self.id)
            .field("viewport", &self.viewport)
            .field("next_frame", &self.next_frame)
            .field("pending_frames", &self.scheduler.pending_count())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::ComponentId;
    use obscur_scene::command::DisplayList;
    use obscur_style::cascade::CascadeOrigin;
    use obscur_style::matching::{BlockDeclaration, StyleRule};
    use obscur_style::property::{Length, PropertyId, PropertyValue, StyleColor};
    use obscur_style::selector::{ComplexSelector, CompoundSelector, SelectorList, SimpleSelector};
    use obscur_view::view::{CLASS, ViewFragment, ViewNode, ViewProperty, ViewValue};

    fn runtime_id() -> RuntimeId {
        RuntimeId::from_raw(1)
    }

    fn viewport() -> Size {
        Size::new(300.0, 300.0)
    }

    /// A view whose single top-level element carries the `counter` class.
    fn counter_view() -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(10))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("counter".into()))),
        ]))
    }

    /// `.counter { background-color: red; width: 100px; height: 50px }`.
    fn counter_stylesheet() -> Stylesheet {
        let selectors = SelectorList::new(vec![ComplexSelector::new(
            CompoundSelector::new(vec![SimpleSelector::Class("counter".into())]),
            Vec::new(),
        )]);
        Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![StyleRule::new(
                selectors,
                vec![
                    BlockDeclaration::normal(
                        PropertyId::BackgroundColor,
                        PropertyValue::Color(StyleColor::rgba(1.0, 0.0, 0.0, 1.0)),
                    ),
                    BlockDeclaration::normal(
                        PropertyId::Width,
                        PropertyValue::Length(Length::px(100.0)),
                    ),
                    BlockDeclaration::normal(
                        PropertyId::Height,
                        PropertyValue::Length(Length::px(50.0)),
                    ),
                ],
            )],
        )
    }

    fn counter_child(runtime: &UiRuntime) -> NodeId {
        let root = runtime.tree().root().expect("tree has a root after a frame");
        runtime.tree().get(root).unwrap().children().to_vec()[0]
    }

    #[test]
    fn new_starts_idle_with_identity_and_viewport() {
        let runtime = UiRuntime::new(runtime_id(), viewport());
        assert_eq!(runtime.runtime_id(), runtime_id());
        assert_eq!(runtime.id(), runtime_id());
        assert_eq!(runtime.viewport(), viewport());
        assert!(!runtime.has_pending_frame());
        assert_eq!(runtime.pending_frames(), 0);
        assert!(runtime.layout_boxes().is_empty());
    }

    #[test]
    fn render_now_paints_matched_background() {
        // S6 acceptance mirror: authored CSS reaches paint through the runtime.
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let view = counter_view();
        let sheet = counter_stylesheet();

        let result = runtime.render_now(&view, std::slice::from_ref(&sheet));
        assert!(result.reconciled());
        assert!(result.restyled(), "the matched node resolved styles");
        assert!(result.relaid_out(), "the sized node arranged a box");
        let list: &DisplayList = result.snapshot().display_list();
        assert!(!list.is_empty(), "matched background produced a fill");
        assert!(result.is_balanced());
        assert_eq!(result.id(), FrameId::FIRST);
    }

    #[test]
    fn layout_boxes_are_retained_after_a_frame() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let view = counter_view();
        let sheet = counter_stylesheet();

        let _ = runtime.render_now(&view, std::slice::from_ref(&sheet));

        let child = counter_child(&runtime);
        let boxes = runtime.layout_boxes();
        assert!(!boxes.is_empty(), "boxes retained from the frame");
        let child_box = boxes.get(&child).expect("the sized node has a box");
        // `.counter` is 100x50 per the stylesheet, arranged at the origin.
        assert_eq!(child_box.size(), Size::new(100.0, 50.0));
        assert_eq!(child_box.origin(), obscur_geometry::Point::new(0.0, 0.0));
    }

    #[test]
    fn render_now_is_deterministic() {
        let mut a = UiRuntime::new(runtime_id(), viewport());
        let mut b = UiRuntime::new(runtime_id(), viewport());
        let view = counter_view();
        let sheet = counter_stylesheet();

        let first = a.render_now(&view, std::slice::from_ref(&sheet));
        let second = b.render_now(&view, std::slice::from_ref(&sheet));
        let first_commands: Vec<_> = first.snapshot().display_list().commands().to_vec();
        let second_commands: Vec<_> = second.snapshot().display_list().commands().to_vec();
        assert_eq!(first_commands, second_commands);
    }

    #[test]
    fn run_due_frame_is_gated_by_the_scheduler() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let view = counter_view();
        let sheet = counter_stylesheet();
        let sheets = std::slice::from_ref(&sheet);

        // Nothing requested → nothing due.
        assert!(runtime.run_due_frame(&view, sheets).is_none());

        // One request → exactly one frame, then idle again (coalesced).
        runtime.request_frame(FrameReason::Input);
        assert!(runtime.has_pending_frame());
        let produced = runtime
            .run_due_frame(&view, sheets)
            .expect("a requested frame is due");
        assert_eq!(produced.id(), FrameId::FIRST);
        assert!(!runtime.has_pending_frame());
        assert!(runtime.run_due_frame(&view, sheets).is_none());

        // A second request advances the frame id (proves one frame per request).
        runtime.request_frame(FrameReason::Input);
        let second = runtime
            .run_due_frame(&view, sheets)
            .expect("the next request is due");
        assert_eq!(second.id(), FrameId::FIRST.next());
    }

    #[test]
    fn same_reason_requests_coalesce_into_one_frame() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let view = counter_view();

        runtime.request_frame(FrameReason::Animation);
        runtime.request_frame(FrameReason::Animation);
        assert_eq!(runtime.pending_frames(), 1, "same reason coalesces");
        assert!(runtime.run_due_frame(&view, &[]).is_some());
        assert!(runtime.run_due_frame(&view, &[]).is_none());
    }

    #[test]
    fn hover_state_reaches_paint_through_the_runtime() {
        use obscur_style::selector::PseudoClassSelector;

        // `.counter:hover { background-color: red; width: 100px; height: 50px }`.
        let hover_sheet = {
            let selectors = SelectorList::new(vec![ComplexSelector::new(
                CompoundSelector::new(vec![
                    SimpleSelector::Class("counter".into()),
                    SimpleSelector::PseudoClass(PseudoClassSelector::Hover),
                ]),
                Vec::new(),
            )]);
            Stylesheet::new(
                CascadeOrigin::GlobalApp,
                vec![StyleRule::new(
                    selectors,
                    vec![
                        BlockDeclaration::normal(
                            PropertyId::BackgroundColor,
                            PropertyValue::Color(StyleColor::rgba(1.0, 0.0, 0.0, 1.0)),
                        ),
                        BlockDeclaration::normal(
                            PropertyId::Width,
                            PropertyValue::Length(Length::px(100.0)),
                        ),
                        BlockDeclaration::normal(
                            PropertyId::Height,
                            PropertyValue::Length(Length::px(50.0)),
                        ),
                    ],
                )],
            )
        };
        let view = counter_view();
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let sheets = std::slice::from_ref(&hover_sheet);

        // First frame: no hover → the hover-gated rule paints nothing.
        let cold = runtime.render_now(&view, sheets);
        assert!(
            cold.snapshot().display_list().is_empty(),
            ":hover paints nothing until hover is recorded"
        );

        // Record hover on the node and re-render → the rule now paints.
        let child = counter_child(&runtime);
        assert_eq!(runtime.pseudo_state(child), PseudoState::empty());
        runtime.set_pseudo_state(child, PseudoState::HOVER);
        assert_eq!(runtime.pseudo_state(child), PseudoState::HOVER);

        let hot = runtime.render_now(&view, sheets);
        assert!(
            !hot.snapshot().display_list().is_empty(),
            ":hover reaches paint once recorded"
        );

        // Clearing state reverts to the cold behaviour.
        runtime.clear_pseudo_states();
        let cleared = runtime.render_now(&view, sheets);
        assert!(cleared.snapshot().display_list().is_empty());
    }

    #[test]
    fn set_viewport_changes_layout_bounds() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        assert_eq!(runtime.viewport(), viewport());
        runtime.set_viewport(Size::new(640.0, 480.0));
        assert_eq!(runtime.viewport(), Size::new(640.0, 480.0));
    }

    #[test]
    fn debug_reports_identity_fields() {
        let runtime = UiRuntime::new(runtime_id(), viewport());
        let rendered = format!("{runtime:?}");
        assert!(rendered.contains("UiRuntime"));
        assert!(rendered.contains("next_frame"));
        assert!(rendered.contains("pending_frames"));
    }

    // ---- Phase 3 step 2: input dispatch (hit-test → hover/focus → pseudo) ----

    use obscur_geometry::Point;
    use obscur_interaction::command::PhysicalKey;
    use obscur_interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};
    use obscur_style::selector::PseudoClassSelector;

    fn class_rule(class: &str, decls: Vec<BlockDeclaration>) -> StyleRule {
        let selectors = SelectorList::new(vec![ComplexSelector::new(
            CompoundSelector::new(vec![SimpleSelector::Class(class.into())]),
            Vec::new(),
        )]);
        StyleRule::new(selectors, decls)
    }

    fn class_hover_rule(class: &str, decls: Vec<BlockDeclaration>) -> StyleRule {
        let selectors = SelectorList::new(vec![ComplexSelector::new(
            CompoundSelector::new(vec![
                SimpleSelector::Class(class.into()),
                SimpleSelector::PseudoClass(PseudoClassSelector::Hover),
            ]),
            Vec::new(),
        )]);
        StyleRule::new(selectors, decls)
    }

    fn class_focus_rule(class: &str, decls: Vec<BlockDeclaration>) -> StyleRule {
        let selectors = SelectorList::new(vec![ComplexSelector::new(
            CompoundSelector::new(vec![
                SimpleSelector::Class(class.into()),
                SimpleSelector::PseudoClass(PseudoClassSelector::Focus),
            ]),
            Vec::new(),
        )]);
        StyleRule::new(selectors, decls)
    }

    fn size(width: f32, height: f32) -> Vec<BlockDeclaration> {
        vec![
            BlockDeclaration::normal(PropertyId::Width, PropertyValue::Length(Length::px(width))),
            BlockDeclaration::normal(PropertyId::Height, PropertyValue::Length(Length::px(height))),
        ]
    }

    fn background(r: f32, g: f32, b: f32) -> Vec<BlockDeclaration> {
        vec![BlockDeclaration::normal(
            PropertyId::BackgroundColor,
            PropertyValue::Color(StyleColor::rgba(r, g, b, 1.0)),
        )]
    }

    /// A `panel` container (100x100) holding two stacked children `a` and `b`
    /// (each 100x50, so `a` occupies y∈[0,50) and `b` occupies y∈[50,100)).
    fn panel_view() -> View {
        let child = |component: u64, class: &str| {
            ViewNode::element(ComponentId::from_raw(component))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
        };
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(100))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into())))
                .with_children(ViewFragment::from_nodes(vec![
                    child(101, "a"),
                    child(102, "b"),
                ])),
        ]))
    }

    /// Sheet: `a`/`b` are *sized but unpainted*; only `:hover`/`:focus` add a
    /// background — so a paint command for `a`/`b` appears **iff** the
    /// interaction pseudo-class fired end-to-end.
    fn interaction_sheet() -> Stylesheet {
        Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![
                class_rule("panel", size(100.0, 100.0)),
                class_rule("a", size(100.0, 50.0)),
                class_rule("b", size(100.0, 50.0)),
                class_hover_rule("b", background(1.0, 0.0, 0.0)),
                class_focus_rule("a", background(1.0, 1.0, 0.0)),
            ],
        )
    }

    /// Returns `(panel, a, b)` node ids after a frame has populated the tree.
    fn panel_nodes(runtime: &UiRuntime) -> (NodeId, NodeId, NodeId) {
        let root = runtime.tree().root().expect("root after a frame");
        let panel = runtime.tree().get(root).unwrap().children()[0];
        let kids = runtime.tree().get(panel).unwrap().children().to_vec();
        (panel, kids[0], kids[1])
    }

    fn mouse(kind: PointerEventKind, x: f32, y: f32) -> PointerEvent {
        PointerEvent::new(
            PointerId::MOUSE,
            PointerDevice::Mouse,
            kind,
            Point::new(x, y),
        )
    }

    #[test]
    fn pointer_over_child_b_makes_it_the_hover_target() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&panel_view(), sheets);
        let (panel, a, b) = panel_nodes(&runtime);

        // (50, 75) lands inside `b` (y∈[50,100)); the innermost node wins.
        let dispatch = runtime.handle_pointer(mouse(PointerEventKind::Move, 50.0, 75.0));
        assert_eq!(dispatch.target(), Some(b));
        assert!(dispatch.hover_changed());
        assert!(!dispatch.focus_changed());
        assert_eq!(runtime.hovered(), Some(b));

        // The hover chain records HOVER on b and its ancestors, not on a.
        assert!(runtime.pseudo_state(b).contains(PseudoState::HOVER));
        assert!(runtime.pseudo_state(panel).contains(PseudoState::HOVER));
        assert_eq!(runtime.pseudo_state(a), PseudoState::empty());
    }

    #[test]
    fn hover_reaches_paint_end_to_end() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);

        // Cold frame: nothing is hovered, so no `:hover` background paints.
        let cold = runtime.render_now(&panel_view(), sheets);
        let cold_fills = cold.snapshot().display_list().len();

        // Point at b, which schedules an Input frame; render it.
        let dispatch = runtime.handle_pointer(mouse(PointerEventKind::Move, 50.0, 75.0));
        assert!(dispatch.hover_changed());
        assert!(runtime.has_pending_frame(), "hover change requested a frame");
        let hot = runtime
            .run_due_frame(&panel_view(), sheets)
            .expect("the input-requested frame is due");

        // `.b:hover { background-color }` now contributes a fill that was absent
        // before hover — proving `:hover` matched through the runtime.
        assert!(
            hot.snapshot().display_list().len() > cold_fills,
            ":hover added a paint command once the pointer entered b"
        );
    }

    #[test]
    fn moving_the_pointer_between_children_transitions_hover() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&panel_view(), sheets);
        let (_panel, a, b) = panel_nodes(&runtime);

        // Enter b, then move to a: b leaves, a enters.
        let to_b = runtime.handle_pointer(mouse(PointerEventKind::Move, 50.0, 75.0));
        assert_eq!(to_b.target(), Some(b));
        assert!(runtime.pseudo_state(b).contains(PseudoState::HOVER));

        let to_a = runtime.handle_pointer(mouse(PointerEventKind::Move, 50.0, 25.0));
        assert_eq!(to_a.target(), Some(a));
        assert!(to_a.hover_changed());
        assert_eq!(runtime.hovered(), Some(a));
        // Hover moved: a is now HOVER, b is cleared.
        assert!(runtime.pseudo_state(a).contains(PseudoState::HOVER));
        assert_eq!(runtime.pseudo_state(b), PseudoState::empty());
    }

    #[test]
    fn press_focuses_the_target_and_focus_reaches_paint() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let cold = runtime.render_now(&panel_view(), sheets);
        let cold_fills = cold.snapshot().display_list().len();
        let (_panel, a, _b) = panel_nodes(&runtime);

        // Press inside a → a takes focus.
        let dispatch = runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
        assert_eq!(dispatch.target(), Some(a));
        assert!(dispatch.focus_changed());
        assert_eq!(runtime.focused(), Some(a));
        assert!(runtime.pseudo_state(a).contains(PseudoState::FOCUS));

        // `.a:focus { background-color }` now paints where it did not before.
        let focused = runtime
            .run_due_frame(&panel_view(), sheets)
            .expect("the input-requested frame is due");
        assert!(
            focused.snapshot().display_list().len() > cold_fills,
            ":focus added a paint command once a was pressed"
        );
    }

    #[test]
    fn hover_and_focus_bits_combine_on_the_same_node() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&panel_view(), sheets);
        let (_panel, a, _b) = panel_nodes(&runtime);

        // Hover a, then press a: it must carry HOVER *and* FOCUS.
        runtime.handle_pointer(mouse(PointerEventKind::Move, 50.0, 25.0));
        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
        let state = runtime.pseudo_state(a);
        assert!(state.contains(PseudoState::HOVER));
        assert!(state.contains(PseudoState::FOCUS));
    }

    #[test]
    fn pointer_outside_every_box_clears_hover() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&panel_view(), sheets);
        let (_panel, _a, b) = panel_nodes(&runtime);

        // Hover b first.
        runtime.handle_pointer(mouse(PointerEventKind::Move, 50.0, 75.0));
        assert_eq!(runtime.hovered(), Some(b));

        // Move outside the viewport (root box is only 300x300) → no hits.
        let dispatch = runtime.handle_pointer(mouse(PointerEventKind::Move, 1000.0, 1000.0));
        assert_eq!(dispatch.target(), None);
        assert!(dispatch.hover_changed(), "leaving b is a hover change");
        assert_eq!(runtime.hovered(), None);
        assert_eq!(runtime.pseudo_state(b), PseudoState::empty());
    }

    #[test]
    fn unchanged_hover_does_not_request_a_frame() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&panel_view(), sheets);

        let first = runtime.handle_pointer(mouse(PointerEventKind::Move, 50.0, 75.0));
        assert!(first.hover_changed());
        // Drain the frame the first move scheduled.
        let _ = runtime.run_due_frame(&panel_view(), sheets);
        assert!(!runtime.has_pending_frame());

        // A second move within the same node changes nothing → no frame.
        let second = runtime.handle_pointer(mouse(PointerEventKind::Move, 55.0, 80.0));
        assert!(!second.changed());
        assert!(!runtime.has_pending_frame());
    }

    #[test]
    fn handle_pointer_is_safe_before_any_frame() {
        // No frame has run → the spatial index is empty; hit-testing must not
        // panic and must report nothing.
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let dispatch = runtime.handle_pointer(mouse(PointerEventKind::Move, 10.0, 10.0));
        assert_eq!(dispatch.target(), None);
        assert!(!dispatch.changed());
        assert_eq!(runtime.hovered(), None);
        assert_eq!(runtime.focused(), None);
    }

    #[test]
    fn handle_key_routes_to_focused_node() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&panel_view(), sheets);
        let (_panel, a, _b) = panel_nodes(&runtime);

        // With nothing focused, a key routes nowhere and schedules no frame.
        assert_eq!(runtime.handle_key(PhysicalKey::KeyA, true), None);
        assert!(!runtime.has_pending_frame());

        // Focus a via a press, drain its frame, then a key routes to a.
        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
        let _ = runtime.run_due_frame(&panel_view(), sheets);
        assert_eq!(runtime.handle_key(PhysicalKey::KeyA, true), Some(a));
        assert!(runtime.has_pending_frame());
    }
}
