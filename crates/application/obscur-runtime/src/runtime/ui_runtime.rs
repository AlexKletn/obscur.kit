//! The embeddable UI runtime and application-core integrator: [`UiRuntime`].

use std::collections::{HashMap, HashSet};
use std::fmt;

use obscur_accessibility::port::AccessibilityBridge;
use obscur_accessibility::tree::{SemanticSnapshot, SemanticTree};
use obscur_diagnostics::{
    BudgetViolation, DiagnosticSubsystem, NullTelemetrySink, PerformanceCounter, PerformanceSpan,
    TelemetrySink,
};
use obscur_geometry::Size;
use obscur_identity::{NodeId, RuntimeId};
use obscur_interaction::command::{CommandRouter, KeyEvent, Modifiers, Shortcut};
use obscur_interaction::hover::{HoverChain, HoverTransition};
use obscur_interaction::pointer::{PointerEvent, PointerEventKind};
use obscur_interaction::spatial::{HitTest, SpatialIndex};
use obscur_motion::scheduler::MotionScheduler;
use obscur_motion::timeline::{Timeline, TimelineId};
use obscur_motion::track::{Keyframe, KeyframeTrack};
use obscur_motion::transition::TransitionInterruption;
use obscur_motion::value::MotionValue;
use obscur_scene::scene::{Scene, SceneSnapshot};
use obscur_style::computed::{ComputedStyle, ComputedValue};
use obscur_style::matching::{MediaEnvironment, Stylesheet};
use obscur_style::property::{Length, PropertyId, PropertyValue};
use obscur_style::selector::{PseudoState, SelectorMatcher};
use obscur_text::font::FontResolver;
use obscur_text::layout::Hyphenator;
use obscur_text::shaping::{TextItemizer, TextShaper};
use obscur_tree::reconciliation::Reconciler;
use obscur_tree::tree::RetainedTree;
use obscur_view::view::View;

use super::{FrameBudget, FramePlan, KeyDispatch, PointerDispatch, SemanticTreeBuilder};
use crate::frame::{FrameContext, FrameId, FrameReason, FrameResult};
use crate::orchestration::{
    root_font_size, AnimatedKind, AnimationBinding, ComputedStyles, CssMotionRules, LayoutBoxes,
    LayoutCoordinator, MotionCoordinator, PseudoStates, SceneCoordinator, StyleCoordinator,
    StyleInputs, StyleMatchingCoordinator, TreeElement,
};
use crate::scheduler::{BackpressureDecision, FrameScheduler};
use crate::transaction::ReconcileEnactor;

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
    /// Enacts each frame's root reconcile through the atomic transaction commit
    /// boundary (spec §3.2): it stages the reconcile as a validated
    /// [`UiTransaction`](crate::transaction::UiTransaction), commits it, and
    /// advances the commit [`VersionStamp`](obscur_identity::VersionStamp) when
    /// the tree actually changed — so the live tree mutation flows through the
    /// transaction layer rather than calling `reconcile_root` directly.
    enactor: ReconcileEnactor,
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
    /// The computed styles resolved for the last produced frame, retained so a
    /// frame that skips the style phase (e.g. an idle diagnostics capture, or a
    /// future paint-only class) can reuse them instead of re-resolving (§3.4
    /// phase 5 incremental recalculation).
    computed: ComputedStyles,
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
    /// The shortcut→command-id binding table keyboard input resolves against
    /// (spec §6.7). Populated by [`bind_command`](Self::bind_command); consulted
    /// by [`handle_key`](Self::handle_key) on each key press.
    commands: CommandRouter,
    /// The modifier keys currently held, folded from physical modifier-key
    /// press/release transitions so shortcut matching sees the live chord.
    modifiers: Modifiers,
    /// The id assigned to the next produced frame.
    next_frame: FrameId,
    /// The viewport frames are laid out under.
    viewport: Size,
    /// The domain motion scheduler owning every active timeline (spec §4.17).
    ///
    /// The runtime *owns* the scheduler and its timelines; each frame it samples
    /// the active timelines and overlays the result onto computed style before
    /// layout and paint, closing the style → motion → paint pipeline.
    motion: MotionScheduler,
    /// The animation binding registry: which timeline animates what
    /// (node/property/track/kind), parallel to the timelines in `motion`.
    ///
    /// A binding is retained after its timeline finishes so the settled endpoint
    /// value keeps being overlaid (the animation *holds* its final value); a new
    /// [`animate`](Self::animate) for the same node+property supersedes it.
    animations: Vec<AnimationBinding>,
    /// The id minted for the next timeline started by
    /// [`animate`](Self::animate); monotonically increasing so ids are unique.
    next_timeline: TimelineId,
    /// The CSS-authored transitions and `@keyframes` animations, keyed by
    /// selector, that the runtime drives through the [`MotionScheduler`] each
    /// style frame (spec §4.17). Empty until a host installs a compiled
    /// stylesheet's motion via [`set_css_motion`](Self::set_css_motion).
    css_motion: CssMotionRules,
    /// The last computed value observed for each `(node, property)` a CSS
    /// `transition` watches — the baseline a transition's change-detection
    /// compares against. A transition starts only when the freshly resolved
    /// value *differs* from this retained target (spec §4.17 "a transition starts
    /// when the property changes").
    transition_targets: HashMap<(NodeId, PropertyId), f32>,
    /// The `(node, property)` pairs a CSS `@keyframes` animation has already been
    /// started for, so a matching element registers its keyframe timeline once
    /// rather than re-registering it every style frame.
    css_animations_started: HashSet<(NodeId, PropertyId)>,
    /// The performance-telemetry destination each frame's phases record into
    /// (spec §7.9). Defaults to a [`NullTelemetrySink`], so an unconfigured
    /// (production) runtime pays ~zero cost: every `record_*` call is a no-op.
    /// A host installs a real sink via
    /// [`set_telemetry_sink`](Self::set_telemetry_sink) to observe spans,
    /// counters and budget violations.
    telemetry: Box<dyn TelemetrySink>,
    /// The per-phase cost budgets checked after each frame phase (spec §7.9).
    /// Unbounded by default, so no [`BudgetViolation`] is produced until a host
    /// installs one via [`set_frame_budget`](Self::set_frame_budget). A phase
    /// whose reported count exceeds its bound records a violation into
    /// `telemetry` — a diagnostic, never a crash.
    budget: FrameBudget,
    /// The number of pending background task-work units (spec §3.6 band 5:
    /// background shaping / decode / cache work).
    ///
    /// Background work is a **signal, not a frame**: it is separate subsystem
    /// task work driven by budgeting and starvation prevention rather than by a
    /// [`FrameReason`]. A host enqueues a unit via
    /// [`request_background`](Self::request_background); the scheduler is told
    /// work is waiting ([`FrameScheduler::note_background_pending`]), and every
    /// higher-priority frame drained afterwards counts as a preemption. When the
    /// scheduler reports the work starved
    /// ([`FrameScheduler::background_starved`]) the drain loop services one unit
    /// on the live path, honouring §3.6 "starvation prevention for background
    /// work".
    background_work: usize,
    /// How many background task-work units have been serviced through the drain
    /// path, so a host (and the wiring tests) can observe that background work
    /// made progress even under a stream of higher-priority frames.
    serviced_background: u64,
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
            enactor: ReconcileEnactor::new(),
            matcher: StyleMatchingCoordinator::new(),
            style: StyleCoordinator::new(),
            layout: LayoutCoordinator::new(),
            scene_composer: SceneCoordinator::new(),
            scheduler: FrameScheduler::new(),
            pseudo: PseudoStates::new(),
            boxes: LayoutBoxes::new(),
            inputs: StyleInputs::new(),
            computed: ComputedStyles::new(),
            spatial: SpatialIndex::new(),
            hover: HoverChain::new(),
            focused: None,
            commands: CommandRouter::new(),
            modifiers: Modifiers::new(),
            next_frame: FrameId::FIRST,
            viewport,
            motion: MotionScheduler::new(),
            animations: Vec::new(),
            next_timeline: TimelineId::from_raw(0),
            css_motion: CssMotionRules::new(),
            transition_targets: HashMap::new(),
            css_animations_started: HashSet::new(),
            telemetry: Box::new(NullTelemetrySink),
            budget: FrameBudget::unbounded(),
            background_work: 0,
            serviced_background: 0,
        }
    }

    /// Installs the performance-telemetry `sink` each frame's phases record into
    /// (spec §7.9), replacing any previously installed sink.
    ///
    /// Until a sink is installed the runtime records into a
    /// [`NullTelemetrySink`] — telemetry is *produced* every frame but discarded,
    /// so deep tracing stays opt-in and production overhead is negligible. A host
    /// or DevTools backend installs a real
    /// [`TelemetrySink`](obscur_diagnostics::TelemetrySink) (e.g. a
    /// [`CollectingTelemetrySink`](obscur_diagnostics::CollectingTelemetrySink))
    /// to receive per-phase spans/counters and budget violations as frames run.
    pub fn set_telemetry_sink(&mut self, sink: Box<dyn TelemetrySink>) {
        self.telemetry = sink;
    }

    /// Installs the per-phase cost [`FrameBudget`] checked after each frame phase
    /// (spec §7.9), replacing any previously installed budget.
    ///
    /// A phase whose deterministic cost count exceeds its bound records a
    /// [`BudgetViolation`] into the installed telemetry sink on the next frame —
    /// a warning-level diagnostic, not a crash.
    pub fn set_frame_budget(&mut self, budget: FrameBudget) {
        self.budget = budget;
    }

    /// Returns the per-phase cost budget checked after each frame phase.
    #[must_use]
    pub fn frame_budget(&self) -> FrameBudget {
        self.budget
    }

    /// Installs the CSS-authored motion (`transition`/`@keyframes`) the runtime
    /// drives, replacing any previously installed set (spec §4.17 S4).
    ///
    /// A host builds a [`CssMotionRules`] from a compiled stylesheet and calls
    /// this once the stylesheet is (re)compiled. Each style frame the runtime
    /// then matches these selector-bound rules against the retained tree and
    /// registers the corresponding timelines with the
    /// [`MotionScheduler`](obscur_motion::scheduler::MotionScheduler): a
    /// `transition` fires when a matched element's watched property *changes*
    /// value, and a `@keyframes` animation starts when an element first matches
    /// the rule that references it. Installing a new set resets the per-node
    /// change-detection baselines so the new sheet re-observes from scratch.
    pub fn set_css_motion(&mut self, rules: CssMotionRules) {
        self.css_motion = rules;
        self.transition_targets.clear();
        self.css_animations_started.clear();
    }

    /// Installs a real text `shaper` on the owned scene coordinator, replacing the
    /// default deterministic [`MockTextShaper`](obscur_text::shaping::MockTextShaper).
    ///
    /// Text shaping is a hexagonal port: the runtime only knows the
    /// `Box<dyn TextShaper>` — it never depends on a concrete engine. A host (or
    /// the facade) builds a real shaper (e.g. the HarfRust adapter, wired to the
    /// same font bytes handed to the renderer so glyph indices align) and installs
    /// it here. From the next composed frame on, each `DrawText` run carries that
    /// shaper's glyphs (real, font-specific glyph **indices** rather than the mock's
    /// Unicode-scalar placeholders). Leaving it unset preserves headless
    /// determinism, so all default-path tests are unchanged.
    pub fn set_text_shaper(&mut self, shaper: Box<dyn TextShaper>) {
        self.scene_composer.set_text_shaper(shaper);
    }

    /// Installs a text `itemizer` on the owned scene coordinator, replacing the
    /// default [`SingleRunItemizer`](obscur_text::shaping::SingleRunItemizer).
    ///
    /// Text itemization is a hexagonal port: the runtime only knows the
    /// `Box<dyn TextItemizer>`. Installing a real Unicode script/bidi itemizer
    /// makes a text node split into multiple script runs, which — paired with a
    /// covering [`set_font_resolver`](Self::set_font_resolver) — the fallback
    /// orchestrator shapes and paints as several font-tagged segments. Leaving it
    /// unset keeps the single-run default, preserving headless determinism.
    pub fn set_text_itemizer(&mut self, itemizer: Box<dyn TextItemizer>) {
        self.scene_composer.set_itemizer(itemizer);
    }

    /// Installs a font `resolver` on the owned scene coordinator, replacing the
    /// default empty [`MockFontResolver`](obscur_text::font::MockFontResolver).
    ///
    /// Font resolution is a hexagonal port: the runtime only knows the
    /// `Box<dyn FontResolver>`. Installing a coverage registry lets each itemized
    /// script run resolve to a covering font, so the fallback orchestrator tags
    /// runs with the right face indices for the renderer to pick per run. Leaving
    /// it unset falls every run back to the primary face (font `0`).
    pub fn set_font_resolver(&mut self, resolver: Box<dyn FontResolver>) {
        self.scene_composer.set_font_resolver(resolver);
    }

    /// Installs a `hyphenator` on the owned scene coordinator, replacing the
    /// default no-op [`NullHyphenator`](obscur_text::layout::NullHyphenator).
    ///
    /// Hyphenation is a hexagonal port: the runtime only knows the
    /// `Box<dyn Hyphenator>` — it never depends on a concrete pattern set. A host
    /// installs a real adapter (e.g. Knuth-Liang English patterns) here; from the
    /// next laid-out frame on, an over-wide word breaks at a valid point and (when
    /// a hyphen glyph is shaped) renders a trailing hyphen. The **same** port
    /// sizes text in the layout pass and paints it in the compose pass, so the
    /// measured box height and the painted line count agree. Leaving it unset
    /// preserves headless determinism, so all default-path tests are unchanged.
    pub fn set_hyphenator(&mut self, hyphenator: Box<dyn Hyphenator>) {
        self.scene_composer.set_hyphenator(hyphenator);
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

    /// Enqueues one unit of **background task work** (spec §3.6 band 5:
    /// background shaping, decoding and cache work) for the runtime to service.
    ///
    /// Background work is not a frame — it is separate, lower-priority subsystem
    /// work — so it is signalled to the [`FrameScheduler`] rather than pushed as
    /// a [`FrameRequest`]. This records the pending unit and, on the first
    /// outstanding unit, tells the scheduler background work is now waiting
    /// ([`FrameScheduler::note_background_pending`]). From then on every
    /// higher-priority frame drained by [`run_due_frame`](Self::run_due_frame)
    /// counts as a preemption of this work; once the scheduler's fairness
    /// threshold is crossed the drain loop promotes and services a unit so the
    /// background band cannot be starved (§3.6 "starvation prevention for
    /// background work").
    pub fn request_background(&mut self) {
        self.background_work = self.background_work.saturating_add(1);
        if self.background_work == 1 {
            self.scheduler.note_background_pending();
        }
    }

    /// Returns the number of background task-work units still waiting to be
    /// serviced.
    #[must_use]
    pub const fn pending_background(&self) -> usize {
        self.background_work
    }

    /// Returns how many background task-work units have been serviced through
    /// the drain path so far.
    ///
    /// This advances only when [`run_due_frame`](Self::run_due_frame) promotes
    /// and runs a starved background unit, so it is the observable proof that
    /// background work made progress despite higher-priority frames (§3.6).
    #[must_use]
    pub const fn serviced_background(&self) -> u64 {
        self.serviced_background
    }

    /// Returns whether pending background work has been preempted past the
    /// scheduler's fairness threshold and must be serviced next (§3.6).
    #[must_use]
    pub const fn is_background_starved(&self) -> bool {
        self.scheduler.background_starved()
    }

    /// Services exactly one pending background task-work unit, resetting the
    /// scheduler's fairness count and clearing the pending signal when the last
    /// unit drains (spec §3.6 band 5).
    ///
    /// The background unit's effect is deliberately minimal here — it advances
    /// the serviced counter — but it flows through the same scheduler
    /// bookkeeping a real shaping/decode/cache task would: it calls
    /// [`FrameScheduler::service_background`] (resetting fairness so the next
    /// wait starts over) and, when no further units remain,
    /// [`FrameScheduler::note_background_idle`] so subsequent frames stop
    /// counting as preemptions.
    fn service_background_unit(&mut self) {
        if self.background_work == 0 {
            return;
        }
        self.background_work -= 1;
        self.serviced_background = self.serviced_background.saturating_add(1);
        // Reset the starvation fairness count: the waiting work just ran.
        self.scheduler.service_background();
        // With no work left, stop treating later frames as preemptions.
        if self.background_work == 0 {
            self.scheduler.note_background_idle();
        }
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
        // Starvation prevention (§3.6): if a stream of higher-priority frames has
        // preempted pending background work past the scheduler's fairness
        // threshold, promote and service one background unit *before* pulling the
        // next frame. This drives the scheduler's background band on the live
        // path — `background_starved`/`service_background` — so background work
        // cannot be indefinitely starved by higher-priority frames. The frame
        // priority order itself is untouched: a due frame still runs this call.
        if self.scheduler.background_starved() {
            self.service_background_unit();
        }
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
    /// laid-out boxes, computed styles and cascade inputs, and advancing the
    /// frame counter.
    ///
    /// Unlike [`FramePipeline::run_matched`](crate::frame::FramePipeline::run_matched),
    /// which re-runs reconcile → match → style → layout → compose every call,
    /// this classifies the frame's [`FrameReason`] into a [`FramePlan`] (spec
    /// §3.4 phase 5) and **skips phases whose inputs are unchanged**, reusing the
    /// previous frame's retained output. It still (a) sources cascade inputs
    /// through the **pseudo-state-aware** matcher and (b) *keeps* the layout
    /// boxes for hit-testing.
    ///
    /// The classification is conservative: the first frame (empty tree) and any
    /// structural reason run every phase, and a phase that cannot be proven
    /// unnecessary is run. The concrete wins are that a hover/focus (`Input`) or
    /// motion (`Animation`) frame does **not** re-reconcile, and an idle
    /// diagnostics capture recalculates nothing and simply re-presents the
    /// retained scene.
    fn run_frame(&mut self, ctx: FrameContext, desired: &View, sheets: &[Stylesheet]) -> FrameResult {
        // Phase 5 (§3.4): classify what this frame must recalculate. The first
        // frame over an empty tree is always full so every retained output
        // exists before later frames can reuse it.
        let first_frame = self.tree.root().is_none();
        let plan = FramePlan::for_frame(ctx.reason(), first_frame);

        // Open the frame telemetry span (spec §7.9). A `NullTelemetrySink`
        // (the default) makes this and every per-phase record below a no-op, so
        // an unconfigured runtime pays essentially nothing.
        self.telemetry
            .record_span(&PerformanceSpan::new("frame", DiagnosticSubsystem::Runtime));

        // An idle frame (e.g. a diagnostics capture) recalculates nothing: it
        // re-presents the retained scene without touching tree, styles or boxes.
        if plan.is_idle() {
            let snapshot = self.scene.snapshot();
            // Record that this frame ran zero phases so an observer still sees it.
            self.telemetry.record_counter(&PerformanceCounter::new(
                DiagnosticSubsystem::Runtime,
                "frame.phases_run",
            ));
            self.next_frame = self.next_frame.next();
            return FrameResult::new(ctx.id(), snapshot, false, false, false, true);
        }

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
        let reconciled = if plan.reconcile() {
            let commit = self
                .enactor
                .commit_root(&self.reconciler, &mut self.tree, desired)
                .expect("a root reconcile is a single, structurally-valid ReplaceRoot");
            let changed = !commit.is_empty();
            // Telemetry: mark the reconcile phase and count the resulting nodes,
            // budgeting the retained-tree size (spec §7.9).
            let nodes = self.tree.len() as u64;
            record_phase(
                &mut *self.telemetry,
                "reconcile",
                DiagnosticSubsystem::Tree,
                "reconcile.nodes",
                nodes,
                self.budget.check_reconcile(nodes),
            );
            changed
        } else {
            false
        };

        // 2-3. Match + style: rebuild cascade inputs and resolve computed styles.
        //    When the style phase is skipped, reuse the retained computed styles
        //    so a downstream layout/compose still has values to work from.
        let (computed, restyled) = if plan.style() {
            // Match authored sheets against the tree, honouring live interaction
            // state so `:hover`/`:focus` participate and gating `@media`/
            // `@container` rules by this frame's viewport (§4.13).
            let environment =
                MediaEnvironment::viewport(ctx.viewport().width, ctx.viewport().height);
            let inputs = self.matcher.build_inputs_with_pseudo_env(
                &self.tree,
                sheets,
                &self.pseudo,
                &environment,
            );
            let mut computed = self.style.resolve(&self.tree, &inputs);
            // 3a. CSS motion: match selector-bound `transition`/`@keyframes`
            //     rules against the tree over the *resolved* (pre-overlay) values,
            //     starting a timeline for any transitioned property that changed
            //     or any newly-matched animation (spec §4.17 S4).
            self.drive_css_motion(&computed);
            // 3b. Motion: overlay every active animation's sampled value onto the
            //     computed style *before* layout/paint (spec §4.17 style →
            //     motion → paint). This now includes the CSS-started timelines.
            self.apply_motion_overlay(&mut computed);
            let restyled = !computed.is_empty();
            self.inputs = inputs;
            // Telemetry: mark the style phase and count resolved styles (§7.9).
            let count = computed.len() as u64;
            record_phase(
                &mut *self.telemetry,
                "style",
                DiagnosticSubsystem::Style,
                "style.resolved",
                count,
                self.budget.check_style(count),
            );
            (computed, restyled)
        } else {
            // Reuse the retained computed styles; re-apply the motion overlay so
            // a motion-only frame still advances its animated values.
            let mut computed = self.computed.clone();
            self.apply_motion_overlay(&mut computed);
            (computed, false)
        };

        // 4. Layout: map to layout styles, arrange, and retain the boxes.
        //    The document root's `font-size` supplies the `rem` base; the
        //    viewport supplies `vw`/`vh` (spec §4.5). When skipped, the retained
        //    boxes and hit-test index carry over unchanged.
        let relaid_out = if plan.layout() {
            let root_font = root_font_size(&computed, self.tree.root());
            let mut layout_styles = self
                .layout
                .layout_styles_from(&computed, ctx.viewport(), root_font);
            // Named-area placement (`grid-area: <name>` against the container's
            // `grid-template-areas`) needs the tree to resolve, so it patches the
            // freshly built styles here; items with no named area are untouched.
            self.layout
                .resolve_named_grid_areas(&mut layout_styles, &self.tree, &computed);
            // Named grid *lines* (`grid-column: <name>` against the container's
            // `[name]` track-list lines) resolve the same way — after the styles
            // table is built and against the parent's template; items with no named
            // line are left untouched.
            self.layout
                .resolve_named_grid_lines(&mut layout_styles, &self.tree, &computed);
            // Authored grid templates (`grid-template-columns`/`-rows`) become a
            // per-node side-table so a grid container dispatches the real
            // track-sizing algorithm; a grid node with no template falls back to a
            // single implicit column.
            let grid_templates = self
                .layout
                .grid_templates_from(&computed, ctx.viewport(), root_font);
            // Text nodes are sized inside the constraint-aware measure/arrange
            // pass: `layout` shapes and wraps each text leaf to its available
            // width through the scene's paragraph orchestrator, so a long string
            // wraps to more lines (a taller box) instead of collapsing to one wide
            // line. `layout` and `scene_composer` are disjoint fields of the
            // runtime, so borrowing the scene's orchestrator here is a legal split
            // borrow, and the same orchestrator sizes the boxes that the scene
            // phase later re-wraps and draws with — so height and painted line
            // count agree.
            let boxes = self.layout.layout_with_hyphenator(
                &self.tree,
                &layout_styles,
                &computed,
                &grid_templates,
                ctx.viewport(),
                self.scene_composer.paragraph(),
                self.scene_composer.hyphenator(),
            );
            let relaid_out = !boxes.is_empty();
            self.boxes = boxes;
            // Rebuild the hit-test index from the freshly laid-out boxes so the
            // next pointer event tests against this frame's geometry.
            self.rebuild_spatial_index();
            // Telemetry: mark the layout phase and count arranged boxes (§7.9).
            let count = self.boxes.len() as u64;
            record_phase(
                &mut *self.telemetry,
                "layout",
                DiagnosticSubsystem::Layout,
                "layout.boxes",
                count,
                self.budget.check_layout(count),
            );
            relaid_out
        } else {
            false
        };

        // 5. Compose: build the display list, install it, publish a snapshot.
        //    When skipped, the retained scene's current snapshot is republished.
        if plan.compose() {
            let display_list = self.scene_composer.compose(&self.tree, &computed, &self.boxes);
            // Telemetry: mark the compose phase and count paint commands (§7.9).
            let count = display_list.len() as u64;
            record_phase(
                &mut *self.telemetry,
                "compose",
                DiagnosticSubsystem::Render,
                "compose.commands",
                count,
                self.budget.check_compose(count),
            );
            self.scene.set_root(display_list);
        }
        let snapshot = self.scene.snapshot();

        // Close the frame span with a counter of how many phases actually ran,
        // so an observer can attribute work to this frame (spec §7.9).
        let phases_run = u64::from(plan.reconcile())
            + u64::from(plan.style())
            + u64::from(plan.layout())
            + u64::from(plan.compose());
        let mut phases = PerformanceCounter::new(DiagnosticSubsystem::Runtime, "frame.phases_run");
        phases.add(phases_run);
        self.telemetry.record_counter(&phases);

        // Retain the computed styles for a later frame that skips the style
        // phase, then advance the frame counter.
        self.computed = computed;
        self.next_frame = self.next_frame.next();

        FrameResult::new(ctx.id(), snapshot, reconciled, restyled, relaid_out, true)
    }

    /// Starts an animation of `property` on `node`, sampling `track` over
    /// `duration` logical seconds and lowering samples per `kind`, reconciling any
    /// in-flight animation of the same target per `interruption`; returns the
    /// [`TimelineId`] that drives the resulting motion (spec §4.17
    /// "interruption / reversal / blend").
    ///
    /// A fresh, already-*started* [`Timeline`] is registered with the owned
    /// [`MotionScheduler`] and a matching [`AnimationBinding`] is recorded, so the
    /// next frame's overlay pass samples the track at the timeline's progress and
    /// writes the result onto `node`'s computed style before layout/paint.
    ///
    /// When a prior binding for the **same** `node`+`property` is still present,
    /// the [`TransitionInterruption`] policy — not a blind remove — decides what
    /// happens (§4.17):
    ///
    /// - [`Blend`](TransitionInterruption::Blend) (the default): the new track is
    ///   re-based to begin at the element's **current sampled value**, so the
    ///   redirect is seamless with no visible snap.
    /// - [`Reverse`](TransitionInterruption::Reverse): the new track plays from the
    ///   current value back toward the interrupted transition's origin.
    /// - [`Restart`](TransitionInterruption::Restart): the incoming track runs from
    ///   its authored start (the element may visibly jump).
    /// - [`Ignore`](TransitionInterruption::Ignore): the in-flight animation is
    ///   kept and the incoming request dropped; the existing [`TimelineId`] is
    ///   returned and no new frame is scheduled.
    ///
    /// For every case except [`Ignore`](TransitionInterruption::Ignore) the prior
    /// timeline is removed, a new one is started, and a [`FrameReason::Animation`]
    /// frame is requested so the animation reaches paint.
    pub fn animate(
        &mut self,
        node: NodeId,
        property: PropertyId,
        track: KeyframeTrack,
        duration: f32,
        kind: AnimatedKind,
        interruption: TransitionInterruption,
    ) -> TimelineId {
        // Resolve the track to actually run against any animation already in
        // flight for this exact target (spec §4.17 interruption).
        let existing = self
            .animations
            .iter()
            .position(|binding| binding.node() == node && binding.property() == property);

        let resolved_track = if let Some(index) = existing {
            let prior = &self.animations[index];
            // The current on-screen value: the prior track sampled at its
            // timeline's progress (a finished/pruned timeline holds t == 1.0).
            let progress = self
                .motion
                .active()
                .get(prior.timeline())
                .map_or(1.0, Timeline::progress);
            let current = MotionCoordinator::sample_keyframes(prior.track(), progress);
            let origin = MotionCoordinator::sample_keyframes(prior.track(), 0.0);
            match (current, origin) {
                // Apply the interruption policy from the prior track's sampled
                // values. `None` means `Ignore` — keep the in-flight animation.
                (Some(current), Some(origin)) => {
                    match interruption.resolve_track(&track, current, origin) {
                        Some(resolved) => resolved,
                        None => return prior.timeline(),
                    }
                }
                // A degenerate (empty) prior track cannot be sampled; fall back to
                // the incoming track verbatim.
                _ => track,
            }
        } else {
            track
        };

        // Supersede any existing animation of the same target: pull its timeline
        // out of the scheduler, then drop its binding.
        while let Some(index) = self
            .animations
            .iter()
            .position(|binding| binding.node() == node && binding.property() == property)
        {
            let stale = self.animations.remove(index);
            self.motion.remove(stale.timeline());
        }

        let id = self.next_timeline;
        self.next_timeline = TimelineId::from_raw(id.raw().saturating_add(1));

        let mut timeline = Timeline::new(id, duration);
        timeline.start();
        self.motion.register(timeline);
        self.animations
            .push(AnimationBinding::new(id, node, property, resolved_track, kind));

        self.request_frame(FrameReason::Animation);
        id
    }

    /// Advances every active timeline by `delta` logical seconds and, while any
    /// motion remains, requests an [`FrameReason::Animation`] frame so the
    /// animation keeps reaching paint (spec §4.17 "active animations keep
    /// requesting frames").
    ///
    /// With no animations this is a no-op: the scheduler is empty, nothing
    /// advances, and no frame is requested. A finished timeline is pruned from the
    /// scheduler by [`MotionScheduler::advance`]; its binding is retained so the
    /// overlay keeps applying the settled endpoint value (the animation *holds*
    /// its final value) until superseded.
    pub fn advance_motion(&mut self, delta: f32) {
        self.motion.advance(delta);
        if self.motion.needs_frame() {
            self.request_frame(FrameReason::Animation);
        }
    }

    /// Returns the number of **running** animations — the count of active
    /// timelines still driving motion.
    ///
    /// This falls to zero once every timeline finishes (the scheduler prunes
    /// finished timelines), even though bindings may linger to hold their final
    /// value. It is the signal a host uses to decide whether to keep ticking
    /// [`advance_motion`](Self::advance_motion).
    #[must_use]
    pub fn active_animations(&self) -> usize {
        self.motion.active_count()
    }

    /// Overlays every animation binding's sampled value onto `computed` before
    /// layout and paint (spec §4.17 style → motion → paint).
    ///
    /// For each binding: if its timeline is still active in the scheduler the
    /// track is sampled at that timeline's progress; if the timeline has finished
    /// (and was pruned) the track is sampled at its endpoint `t == 1.0`, so the
    /// settled value keeps being applied. The sampled scalar is lowered to a
    /// [`PropertyValue`] per the binding's [`AnimatedKind`], wrapped as a
    /// [`ComputedValue`], and written onto the target node's [`ComputedStyle`]. A
    /// node with no computed entry (e.g. `display: none`) is skipped, and a
    /// non-scalar sample or an empty track contributes nothing.
    ///
    /// [`PropertyValue`]: obscur_style::property::PropertyValue
    /// [`ComputedStyle`]: obscur_style::computed::ComputedStyle
    fn apply_motion_overlay(&mut self, computed: &mut ComputedStyles) {
        for binding in &self.animations {
            // Read progress from the scheduler; a finished (pruned) timeline holds
            // its endpoint at t == 1.0.
            let progress = self
                .motion
                .active()
                .get(binding.timeline())
                .map_or(1.0, Timeline::progress);

            let Some(sampled) = MotionCoordinator::sample_keyframes(binding.track(), progress)
            else {
                continue;
            };
            let Some(scalar) = sampled.as_scalar() else {
                continue;
            };
            let property_value = binding.kind().to_property_value(scalar);
            let Some(value) = ComputedValue::from_concrete(property_value) else {
                continue;
            };
            if let Some(style) = computed.get_mut(&binding.node()) {
                style.set(binding.property(), value);
            }
        }
    }

    /// Drives the installed CSS `transition`/`@keyframes` rules against the tree,
    /// registering timelines with the [`MotionScheduler`] for any transitioned
    /// property that changed value or any newly-matched animation (spec §4.17 S4).
    ///
    /// `computed` is this frame's *resolved* style, sampled **before** the motion
    /// overlay, so a transition's change-detection compares the settled cascade
    /// target — not a mid-animation value — against the retained
    /// [`transition_targets`](Self::transition_targets) baseline. The pass runs in
    /// two stages to keep the borrow of `self.css_motion` disjoint from the
    /// `&mut self` [`animate`](Self::animate) calls: it first *collects* the
    /// actions to take (matching each rule's selector against every element node
    /// via [`SelectorMatcher`], honouring recorded pseudo-state), then *applies*
    /// them — updating the baselines and starting the timelines.
    fn drive_css_motion(&mut self, computed: &ComputedStyles) {
        if self.css_motion.is_empty() {
            return;
        }

        // Stage 1: collect. Pure reads of `self.css_motion`, `self.tree`,
        // `self.pseudo`, and the change-detection baselines.
        let mut target_updates: Vec<((NodeId, PropertyId), f32)> = Vec::new();
        let mut transitions_to_start: Vec<TransitionStart> = Vec::new();
        let mut animations_to_start: Vec<AnimationStart> = Vec::new();

        for rule in self.css_motion.transitions() {
            let Some(kind) = animated_kind_for(rule.property()) else {
                continue;
            };
            for (id, node) in self.tree.iter() {
                if !node.kind().is_element() {
                    continue;
                }
                let element = TreeElement::with_pseudo(&self.tree, id, &self.pseudo);
                if SelectorMatcher::matches_list(rule.selectors(), &element).is_none() {
                    continue;
                }
                let Some(new_scalar) = computed
                    .get(&id)
                    .and_then(|style| scalar_of(style, rule.property(), kind))
                else {
                    continue;
                };
                let key = (id, rule.property());
                match self.transition_targets.get(&key) {
                    // A real change of a watched property: start a transition from
                    // the previous target to the new one, unless the duration is
                    // non-positive (then it would just snap).
                    Some(&previous) if previous != new_scalar => {
                        target_updates.push((key, new_scalar));
                        if rule.duration_secs() > 0.0 {
                            transitions_to_start.push(TransitionStart {
                                node: id,
                                property: rule.property(),
                                from: previous,
                                to: new_scalar,
                                duration: rule.duration_secs(),
                                easing: rule.easing(),
                                kind,
                            });
                        }
                    }
                    // Unchanged: nothing to do.
                    Some(_) => {}
                    // First observation: record the baseline, do not animate.
                    None => target_updates.push((key, new_scalar)),
                }
            }
        }

        for rule in self.css_motion.keyframes() {
            for (id, node) in self.tree.iter() {
                if !node.kind().is_element() {
                    continue;
                }
                let element = TreeElement::with_pseudo(&self.tree, id, &self.pseudo);
                if SelectorMatcher::matches_list(rule.selectors(), &element).is_none() {
                    continue;
                }
                for (property, track) in rule.tracks() {
                    let Some(kind) = animated_kind_for(*property) else {
                        continue;
                    };
                    if self.css_animations_started.contains(&(id, *property)) {
                        continue;
                    }
                    animations_to_start.push(AnimationStart {
                        node: id,
                        property: *property,
                        track: track.clone(),
                        duration: rule.duration_secs(),
                        kind,
                    });
                }
            }
        }

        // Stage 2: apply. Mutating `self` (baselines + `animate`).
        for (key, value) in target_updates {
            self.transition_targets.insert(key, value);
        }
        for start in transitions_to_start {
            let track = KeyframeTrack::from_keyframes(vec![
                Keyframe::new(0.0, MotionValue::scalar(start.from)).with_easing(start.easing),
                Keyframe::new(1.0, MotionValue::scalar(start.to)),
            ]);
            self.animate(
                start.node,
                start.property,
                track,
                start.duration,
                start.kind,
                TransitionInterruption::Blend,
            );
        }
        for start in animations_to_start {
            self.css_animations_started
                .insert((start.node, start.property));
            self.animate(
                start.node,
                start.property,
                start.track,
                start.duration,
                start.kind,
                TransitionInterruption::Restart,
            );
        }
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

    /// Binds `shortcut` to `command_id` in the runtime's command router, so a
    /// matching key press at [`handle_key`](Self::handle_key) resolves to that
    /// command (spec §6.7).
    ///
    /// Registration order is priority: an earlier binding shadows a later one
    /// with the same shortcut (see [`CommandRouter::bind`]). Binding does not
    /// itself schedule a frame.
    pub fn bind_command(&mut self, shortcut: Shortcut, command_id: impl Into<Box<str>>) {
        self.commands.bind(shortcut, command_id);
    }

    /// Returns the shortcut→command-id binding table keyboard input resolves
    /// against.
    #[must_use]
    pub const fn command_router(&self) -> &CommandRouter {
        &self.commands
    }

    /// Returns the modifier keys currently held (folded from physical modifier
    /// key transitions seen by [`handle_key`](Self::handle_key)).
    #[must_use]
    pub const fn held_modifiers(&self) -> Modifiers {
        self.modifiers
    }

    /// Routes a keyboard [`KeyEvent`] to the currently focused node, resolves it
    /// against the bound command shortcuts, and schedules a frame — the keyboard
    /// analogue of [`handle_pointer`](Self::handle_pointer) (spec §6.7, §3.10).
    ///
    /// The flow mirrors the pointer path:
    ///
    /// 1. **Modifier tracking** — a physical modifier key
    ///    (`Shift`/`Control`/`Alt`/`Meta`) folds into the runtime's held
    ///    [`Modifiers`] set (a press adds, a release removes) so a later shortcut
    ///    sees the live chord. Modifier keys still route like any other key.
    /// 2. **Route** — the event is aimed at the [`focused`](Self::focused) node.
    ///    With nothing focused the key routes nowhere: the method is a no-op that
    ///    returns [`KeyDispatch::unhandled`] and schedules no frame.
    /// 3. **Resolve** — on a **press**, the event's layout-dependent
    ///    [`LogicalKey`](obscur_interaction::command::LogicalKey) plus the held
    ///    modifiers are looked up in the [`CommandRouter`]; a match yields the
    ///    bound command id. A release, an event with no logical key, or an
    ///    unbound chord resolves to no command.
    /// 4. **Schedule** — because the key reached a focused node, a
    ///    [`FrameReason::Input`] frame is requested so any focus/command-driven
    ///    visual reaches paint on the next `run_due_frame`/`render_now`.
    ///
    /// Returns a [`KeyDispatch`] describing the routed target and any resolved
    /// command.
    pub fn handle_key(&mut self, event: KeyEvent) -> KeyDispatch {
        // 1. Fold a physical modifier transition into the held modifier set.
        if let Some(modifier) = event.physical().modifier() {
            self.modifiers = if event.pressed() {
                self.modifiers.with(modifier)
            } else {
                self.modifiers.without(modifier)
            };
        }

        // 2. Route to the focused node; with no focus the key goes nowhere.
        let Some(target) = self.focused else {
            return KeyDispatch::unhandled();
        };

        // 3. Resolve a bound command from the produced logical key + modifiers,
        //    but only on a press (a release never triggers a shortcut).
        let command = if event.pressed() {
            event
                .logical()
                .and_then(|key| self.commands.resolve(key, self.modifiers))
                .map(Box::<str>::from)
        } else {
            None
        };

        // 4. The key reached a focused node: schedule a restyle for the next
        //    frame, mirroring the pointer path's request on a change.
        self.request_frame(FrameReason::Input);
        KeyDispatch::new(Some(target), command)
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

    /// Returns the version stamped at the most recent frame that committed a real
    /// tree change through the transaction boundary (spec §3.2).
    ///
    /// It advances by one each time a frame's reconcile actually mutates the
    /// retained tree and is unchanged by an idempotent (no-op) reconcile or a
    /// frame that skips the reconcile phase — so a host can observe that the live
    /// tree mutation went through a validated, atomic [`UiTransaction`] commit
    /// rather than a bare `reconcile_root`.
    ///
    /// [`UiTransaction`]: crate::transaction::UiTransaction
    #[must_use]
    pub fn commit_version(&self) -> obscur_identity::VersionStamp {
        self.enactor.version()
    }

    /// Builds the semantic accessibility tree from the live retained tree
    /// (spec §7.1).
    ///
    /// This is the missing link §7.1 mandates — "accessibility uses a semantic
    /// tree linked to the retained UI tree". Each retained node becomes a
    /// [`SemanticNode`](obscur_accessibility::semantic::SemanticNode) keyed by the
    /// same [`NodeId`], with its role/name/state derived from the node's kind,
    /// CSS type/local name, text content, and flags; the runtime's current
    /// [`focused`](Self::focused) node is reflected as the focused semantic node.
    /// The retained tree is left untouched.
    #[must_use]
    pub fn semantic_tree(&self) -> SemanticTree {
        SemanticTreeBuilder::build(&self.tree, self.focused)
    }

    /// Captures the semantic accessibility tree as an immutable, versioned
    /// [`SemanticSnapshot`] — the hand-off a platform
    /// [`AccessibilityBridge`] consumes (spec §7.1).
    #[must_use]
    pub fn semantic_snapshot(&self) -> SemanticSnapshot {
        SemanticTreeBuilder::snapshot(&self.tree, self.focused)
    }

    /// Pushes the current semantic snapshot to a platform
    /// [`AccessibilityBridge`], feeding the retained tree's accessibility
    /// projection into UI Automation / AX / AT-SPI / web a11y (spec §7.1).
    pub fn push_accessibility(&self, bridge: &mut dyn AccessibilityBridge) {
        bridge.update_tree(&self.semantic_snapshot());
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

/// A pending CSS-transition start collected during
/// [`UiRuntime::drive_css_motion`]'s read stage.
struct TransitionStart {
    node: NodeId,
    property: PropertyId,
    from: f32,
    to: f32,
    duration: f32,
    easing: obscur_motion::track::Easing,
    kind: AnimatedKind,
}

/// A pending CSS-`@keyframes` animation start collected during
/// [`UiRuntime::drive_css_motion`]'s read stage.
struct AnimationStart {
    node: NodeId,
    property: PropertyId,
    track: KeyframeTrack,
    duration: f32,
    kind: AnimatedKind,
}

/// Records one frame phase's telemetry into `sink` (spec §7.9): a
/// [`PerformanceSpan`] marker for the phase, a [`PerformanceCounter`] carrying
/// the phase's deterministic cost `metric`, and — when the phase's optional
/// budget was exceeded — the resulting [`BudgetViolation`].
///
/// Taking `&mut dyn TelemetrySink` (rather than `&mut self`) keeps the borrow
/// disjoint from the caller's other `self` fields, and makes every call a no-op
/// under the default [`NullTelemetrySink`].
fn record_phase(
    sink: &mut dyn TelemetrySink,
    name: &'static str,
    subsystem: DiagnosticSubsystem,
    label: &'static str,
    metric: u64,
    violation: Option<BudgetViolation>,
) {
    sink.record_span(&PerformanceSpan::new(name, subsystem));
    let mut counter = PerformanceCounter::new(subsystem, label);
    counter.add(metric);
    sink.record_counter(&counter);
    if let Some(violation) = violation {
        sink.record_violation(&violation);
    }
}

/// Returns how a scalar animates into `property`'s computed value, or [`None`]
/// for a property the motion overlay cannot express as a scalar.
///
/// This mirrors [`AnimationBinding`]'s lowering: `opacity`/`flex-grow`/
/// `flex-shrink` are bare numbers; box-model lengths are pixel lengths. A
/// property outside this set (e.g. a colour or keyword) is not scalar-animatable
/// here and drives no CSS transition/animation.
fn animated_kind_for(property: PropertyId) -> Option<AnimatedKind> {
    match property {
        PropertyId::Opacity | PropertyId::FlexGrow | PropertyId::FlexShrink => {
            Some(AnimatedKind::Number)
        }
        PropertyId::Width
        | PropertyId::Height
        | PropertyId::MinWidth
        | PropertyId::MinHeight
        | PropertyId::MaxWidth
        | PropertyId::MaxHeight
        | PropertyId::MarginTop
        | PropertyId::MarginRight
        | PropertyId::MarginBottom
        | PropertyId::MarginLeft
        | PropertyId::PaddingTop
        | PropertyId::PaddingRight
        | PropertyId::PaddingBottom
        | PropertyId::PaddingLeft
        | PropertyId::Gap
        | PropertyId::FlexBasis
        | PropertyId::FontSize => Some(AnimatedKind::LengthPx),
        _ => None,
    }
}

/// Extracts `property`'s computed value from `style` as the scalar `kind`
/// animates, or [`None`] when the property is absent or not that scalar shape.
///
/// A [`AnimatedKind::Number`] reads a bare [`PropertyValue::Number`]; a
/// [`AnimatedKind::LengthPx`] reads an absolute pixel length (a percentage or
/// `auto` length has no fixed scalar and yields [`None`]).
fn scalar_of(style: &ComputedStyle, property: PropertyId, kind: AnimatedKind) -> Option<f32> {
    let value = style.get(property)?;
    match kind {
        AnimatedKind::Number => match value.value() {
            PropertyValue::Number(n) => Some(*n),
            _ => None,
        },
        AnimatedKind::LengthPx => value.as_length().and_then(Length::as_px).map(|px| px.0),
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
    fn tree_change_commits_through_the_transaction_boundary() {
        // S2: the live frame's tree mutation flows through the atomic transaction
        // commit boundary (spec §3.2), observable via the commit version stamp —
        // while the produced frame output stays identical.
        use obscur_identity::VersionStamp;

        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let view = counter_view();
        let sheet = counter_stylesheet();
        let sheets = std::slice::from_ref(&sheet);

        // No frame has committed yet.
        assert_eq!(runtime.commit_version(), VersionStamp::from_raw(0));

        // First frame populates the tree: the reconcile is committed, so the
        // version stamp advances — proving the mutation went through the commit.
        let first = runtime.render_now(&view, sheets);
        assert!(first.reconciled());
        assert_eq!(runtime.commit_version(), VersionStamp::from_raw(1));

        // A second frame over the same view is an idempotent reconcile: it commits
        // empty and does not advance the version, yet still produces the exact same
        // display list as the first frame (behavior is unchanged by the boundary).
        let second = runtime.render_now(&view, sheets);
        assert_eq!(
            runtime.commit_version(),
            VersionStamp::from_raw(1),
            "an idempotent reconcile does not advance the commit version"
        );
        let first_commands: Vec<_> = first.snapshot().display_list().commands().to_vec();
        let second_commands: Vec<_> = second.snapshot().display_list().commands().to_vec();
        assert_eq!(
            first_commands, second_commands,
            "the committed path yields the same display list as before"
        );
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
    fn requesting_background_marks_it_pending_without_a_frame() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        // Background work is a signal, not a frame: it must not enqueue a frame.
        assert_eq!(runtime.pending_background(), 0);
        assert!(!runtime.has_pending_frame());
        runtime.request_background();
        assert_eq!(runtime.pending_background(), 1);
        assert!(
            !runtime.has_pending_frame(),
            "background work is not a frame request"
        );
        assert!(!runtime.is_background_starved());
        // A second unit accumulates but does not re-signal a frame either.
        runtime.request_background();
        assert_eq!(runtime.pending_background(), 2);
        assert!(!runtime.has_pending_frame());
    }

    #[test]
    fn background_work_is_serviced_through_the_drain_path_despite_higher_priority_frames() {
        // Drive the *runtime* (not the bare scheduler): note background work, then
        // pound it with a stream of higher-priority Input frames. §3.6 starvation
        // prevention must eventually promote and service the background unit
        // through `run_due_frame` — proving `background_starved`/`service_background`
        // are live on the real path.
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let view = counter_view();
        let sheet = counter_stylesheet();
        let sheets = std::slice::from_ref(&sheet);

        runtime.request_background();
        assert_eq!(runtime.serviced_background(), 0);

        // The default fairness threshold before background work is starved.
        let threshold = crate::scheduler::StarvationGuard::DEFAULT_THRESHOLD;

        // Serve `threshold` higher-priority (Input) frames while the background
        // unit waits. None of these service the background work yet.
        for _ in 0..threshold {
            runtime.request_frame(FrameReason::Input);
            assert!(runtime.run_due_frame(&view, sheets).is_some());
        }
        assert_eq!(
            runtime.serviced_background(),
            0,
            "background work is not serviced before the fairness threshold"
        );
        assert!(
            runtime.is_background_starved(),
            "after `threshold` preempting frames the background band is starved"
        );

        // The next drain call promotes and services the starved background unit.
        runtime.request_frame(FrameReason::Input);
        assert!(runtime.run_due_frame(&view, sheets).is_some());
        assert_eq!(
            runtime.serviced_background(),
            1,
            "the starved background unit is serviced through the drain path"
        );
        assert_eq!(runtime.pending_background(), 0);
        // Servicing reset fairness and cleared the pending signal.
        assert!(!runtime.is_background_starved());
    }

    #[test]
    fn serviced_background_stays_pending_until_every_unit_drains() {
        // Two units: the first promotion services one; the band stays pending and
        // is starved again by another burst, servicing the second.
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let view = counter_view();
        let sheet = counter_stylesheet();
        let sheets = std::slice::from_ref(&sheet);
        let threshold = crate::scheduler::StarvationGuard::DEFAULT_THRESHOLD;

        runtime.request_background();
        runtime.request_background();
        assert_eq!(runtime.pending_background(), 2);

        // First starvation cycle services one unit.
        for _ in 0..threshold {
            runtime.request_frame(FrameReason::Input);
            let _ = runtime.run_due_frame(&view, sheets);
        }
        runtime.request_frame(FrameReason::Input);
        let _ = runtime.run_due_frame(&view, sheets);
        assert_eq!(runtime.serviced_background(), 1);
        assert_eq!(runtime.pending_background(), 1);
        assert!(
            !runtime.is_background_starved(),
            "fairness reset, so the remaining unit is not immediately starved"
        );

        // Second starvation cycle services the last unit and clears the band.
        for _ in 0..threshold {
            runtime.request_frame(FrameReason::Input);
            let _ = runtime.run_due_frame(&view, sheets);
        }
        runtime.request_frame(FrameReason::Input);
        let _ = runtime.run_due_frame(&view, sheets);
        assert_eq!(runtime.serviced_background(), 2);
        assert_eq!(runtime.pending_background(), 0);
        assert!(!runtime.is_background_starved());
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
    fn semantic_snapshot_reflects_the_live_rendered_tree() {
        use obscur_accessibility::semantic::SemanticRole;
        use obscur_view::view::TYPE;

        // A `button` element (CSS type) carrying a "Save" text child.
        let view = View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(20))
                .with_prop(ViewProperty::new(TYPE, ViewValue::Text("button".into())))
                .with_children(ViewFragment::from_nodes(vec![ViewNode::text("Save")])),
        ]));

        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let _ = runtime.render_now(&view, &[]);

        let snapshot = runtime.semantic_snapshot();
        // root + button element + text child.
        assert_eq!(snapshot.len(), 3);
        let root = runtime.tree().root().expect("root after a frame");
        assert_eq!(snapshot.root(), Some(root));

        let button = runtime.tree().get(root).unwrap().children()[0];
        let button_node = snapshot.get(button).expect("button in the snapshot");
        assert_eq!(button_node.role(), SemanticRole::Button);
        assert_eq!(button_node.name(), Some("Save"));

        // The text child maps to a static-text node named by its content.
        let text_child = runtime.tree().get(button).unwrap().children()[0];
        let text_node = snapshot.get(text_child).expect("text in the snapshot");
        assert_eq!(text_node.role(), SemanticRole::Text);
        assert_eq!(text_node.name(), Some("Save"));

        // Determinism: the same live tree yields an equal snapshot.
        assert_eq!(runtime.semantic_snapshot(), snapshot);
    }

    #[test]
    fn push_accessibility_feeds_the_bridge() {
        use obscur_accessibility::port::RecordingAccessibilityBridge;

        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let _ = runtime.render_now(&counter_view(), &[]);

        let mut bridge = RecordingAccessibilityBridge::new();
        runtime.push_accessibility(&mut bridge);
        assert_eq!(bridge.tree_updates, 1, "the snapshot reached the bridge");
    }

    // ---- §4.17 S4: CSS transitions / @keyframes driven through motion --------

    use crate::orchestration::{CssKeyframeRule, CssMotionRules, CssTransitionRule};
    use obscur_motion::track::Easing;

    /// The selector list `.box`.
    fn box_selectors() -> SelectorList {
        SelectorList::new(vec![ComplexSelector::new(
            CompoundSelector::new(vec![SimpleSelector::Class("box".into())]),
            Vec::new(),
        )])
    }

    /// A single `.box` element.
    fn box_view() -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(10))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("box".into()))),
        ]))
    }

    /// `.box { width: 100px; height: 50px } .box:hover { width: 200px }`.
    fn box_hover_sheet() -> Stylesheet {
        Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![
                class_rule("box", size(100.0, 50.0)),
                class_hover_rule(
                    "box",
                    vec![BlockDeclaration::normal(
                        PropertyId::Width,
                        PropertyValue::Length(Length::px(200.0)),
                    )],
                ),
            ],
        )
    }

    #[test]
    fn css_transition_registers_and_interpolates_on_change() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let view = box_view();
        let sheet = box_hover_sheet();
        let sheets = std::slice::from_ref(&sheet);
        runtime.set_css_motion(CssMotionRules::from_parts(
            vec![CssTransitionRule::new(
                box_selectors(),
                PropertyId::Width,
                1.0,
                Easing::Linear,
            )],
            Vec::new(),
        ));

        // Frame 1 (no hover): the transition records its baseline (width 100) but
        // starts nothing — no change yet.
        let _ = runtime.render_now(&view, sheets);
        let child = counter_child(&runtime);
        assert_eq!(runtime.active_animations(), 0, "no change => no timeline");
        assert_eq!(box_width(&runtime, child), 100.0);

        // Hover changes the resolved width 100 -> 200: a transition timeline is
        // registered, and at t == 0 the overlaid width is still the old value.
        runtime.set_pseudo_state(child, PseudoState::HOVER);
        let _ = runtime.render_now(&view, sheets);
        assert_eq!(
            runtime.active_animations(),
            1,
            "a changed transitioned property registers a running timeline"
        );
        assert_eq!(box_width(&runtime, child), 100.0, "starts from the old value");

        // Advance halfway: the sampled width interpolates linearly to 150.
        runtime.advance_motion(0.5);
        let _ = runtime.render_now(&view, sheets);
        assert_eq!(box_width(&runtime, child), 150.0, "interpolates old->new");
        assert_eq!(runtime.active_animations(), 1, "still running at t = 0.5");

        // Advance to the end: the timeline finishes and the width settles at 200.
        runtime.advance_motion(0.5);
        let _ = runtime.render_now(&view, sheets);
        assert_eq!(box_width(&runtime, child), 200.0, "settles at the new value");
        assert_eq!(runtime.active_animations(), 0, "finished timeline is pruned");
    }

    #[test]
    fn css_transition_does_not_fire_without_a_change() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let view = box_view();
        // Only the base rule: the width never changes across frames.
        let sheet = Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![class_rule("box", size(100.0, 50.0))],
        );
        let sheets = std::slice::from_ref(&sheet);
        runtime.set_css_motion(CssMotionRules::from_parts(
            vec![CssTransitionRule::new(
                box_selectors(),
                PropertyId::Width,
                1.0,
                Easing::Linear,
            )],
            Vec::new(),
        ));

        let _ = runtime.render_now(&view, sheets);
        let _ = runtime.render_now(&view, sheets);
        assert_eq!(
            runtime.active_animations(),
            0,
            "an unchanged property registers no timeline"
        );
    }

    #[test]
    fn css_keyframes_rule_registers_a_keyframe_timeline() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let view = box_view();
        // The base rule sizes the box so it lays out; the animation targets opacity.
        let sheet = Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![class_rule("box", size(100.0, 50.0))],
        );
        let sheets = std::slice::from_ref(&sheet);
        let track = KeyframeTrack::from_keyframes(vec![
            Keyframe::new(0.0, MotionValue::scalar(0.0)),
            Keyframe::new(1.0, MotionValue::scalar(1.0)),
        ]);
        runtime.set_css_motion(CssMotionRules::from_parts(
            Vec::new(),
            vec![CssKeyframeRule::new(
                box_selectors(),
                vec![(PropertyId::Opacity, track)],
                1.0,
            )],
        ));

        // Matching `.box` registers the keyframe timeline once.
        let _ = runtime.render_now(&view, sheets);
        assert_eq!(
            runtime.active_animations(),
            1,
            "an @keyframes-referencing rule registers a keyframe timeline"
        );

        // A second frame must not re-register it (started-set guards it).
        let _ = runtime.render_now(&view, sheets);
        assert_eq!(runtime.active_animations(), 1, "registered once, not per frame");
    }

    #[test]
    fn debug_reports_identity_fields() {
        let runtime = UiRuntime::new(runtime_id(), viewport());
        let rendered = format!("{runtime:?}");
        assert!(rendered.contains("UiRuntime"));
        assert!(rendered.contains("next_frame"));
        assert!(rendered.contains("pending_frames"));
    }

    // ---- §7.9 (audit S3): frame-phase telemetry through a TelemetrySink -------

    use obscur_diagnostics::{CollectingTelemetrySink, DiagnosticSeverity};
    use std::cell::RefCell;
    use std::rc::Rc;

    /// A telemetry sink that shares a [`CollectingTelemetrySink`] behind an
    /// `Rc<RefCell<_>>`, so a test can install it into the runtime (which takes
    /// ownership of a `Box<dyn TelemetrySink>`) and still read what was recorded
    /// afterwards.
    #[derive(Clone, Default)]
    struct SharedSink(Rc<RefCell<CollectingTelemetrySink>>);

    impl TelemetrySink for SharedSink {
        fn record_span(&mut self, span: &PerformanceSpan) {
            self.0.borrow_mut().record_span(span);
        }
        fn record_counter(&mut self, counter: &PerformanceCounter) {
            self.0.borrow_mut().record_counter(counter);
        }
        fn record_violation(&mut self, violation: &obscur_diagnostics::BudgetViolation) {
            self.0.borrow_mut().record_violation(violation);
        }
    }

    #[test]
    fn a_frame_records_every_phase_span_and_counter_into_the_sink() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let shared = SharedSink::default();
        runtime.set_telemetry_sink(Box::new(shared.clone()));

        let view = counter_view();
        let sheet = counter_stylesheet();
        let _ = runtime.render_now(&view, std::slice::from_ref(&sheet));

        let sink = shared.0.borrow();
        // Every phase of a full frame recorded its span, in pipeline order.
        let span_names: Vec<&str> = sink.spans.iter().map(|s| s.name).collect();
        assert_eq!(
            span_names,
            vec!["frame", "reconcile", "style", "layout", "compose"]
        );
        // Each phase recorded a cost counter, plus the closing phases_run tally.
        let counter_labels: Vec<&str> =
            sink.counters.iter().map(PerformanceCounter::label).collect();
        assert_eq!(
            counter_labels,
            vec![
                "reconcile.nodes",
                "style.resolved",
                "layout.boxes",
                "compose.commands",
                "frame.phases_run",
            ]
        );
        // The full frame ran all four phases and produced real work.
        let phases_run = sink
            .counters
            .iter()
            .find(|c| c.label() == "frame.phases_run")
            .expect("phases_run counter");
        assert_eq!(phases_run.value(), 4);
        assert_eq!(phases_run.subsystem(), DiagnosticSubsystem::Runtime);
        let commands = sink
            .counters
            .iter()
            .find(|c| c.label() == "compose.commands")
            .expect("compose counter");
        assert!(commands.value() > 0, "the matched fill produced a command");
        // No budget configured => no violations.
        assert!(sink.violations.is_empty());
    }

    #[test]
    fn multiple_frames_accumulate_telemetry() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let shared = SharedSink::default();
        runtime.set_telemetry_sink(Box::new(shared.clone()));

        let view = counter_view();
        let sheet = counter_stylesheet();
        let sheets = std::slice::from_ref(&sheet);
        for _ in 0..3 {
            let _ = runtime.render_now(&view, sheets);
        }

        let sink = shared.0.borrow();
        // Three full frames, each opening a "frame" span.
        let frame_spans = sink.spans.iter().filter(|s| s.name == "frame").count();
        assert_eq!(frame_spans, 3);
        assert_eq!(sink.spans.len(), 3 * 5, "5 spans per full frame");
    }

    #[test]
    fn an_exceeded_budget_records_a_violation_that_becomes_a_warning_diagnostic() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let shared = SharedSink::default();
        runtime.set_telemetry_sink(Box::new(shared.clone()));
        // Bound the reconcile phase to zero nodes: any populated tree exceeds it.
        runtime.set_frame_budget(FrameBudget::unbounded().with_reconcile_nodes(0));
        assert_eq!(
            runtime.frame_budget(),
            FrameBudget::unbounded().with_reconcile_nodes(0)
        );

        let view = counter_view();
        let sheet = counter_stylesheet();
        let _ = runtime.render_now(&view, std::slice::from_ref(&sheet));

        let sink = shared.0.borrow();
        assert_eq!(sink.violations.len(), 1, "one reconcile budget violation");
        let violation = &sink.violations[0];
        assert_eq!(violation.subsystem(), DiagnosticSubsystem::Tree);
        // The violation flows into the diagnostics channel as a warning, not a
        // crash (spec §7.9).
        let diagnostic = violation.to_diagnostic();
        assert_eq!(diagnostic.severity, DiagnosticSeverity::Warning);
        assert_eq!(diagnostic.subsystem, DiagnosticSubsystem::Tree);
    }

    #[test]
    fn a_generous_budget_records_no_violation() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let shared = SharedSink::default();
        runtime.set_telemetry_sink(Box::new(shared.clone()));
        runtime.set_frame_budget(
            FrameBudget::unbounded()
                .with_reconcile_nodes(1_000)
                .with_style_nodes(1_000)
                .with_layout_boxes(1_000)
                .with_compose_commands(1_000),
        );

        let view = counter_view();
        let sheet = counter_stylesheet();
        let _ = runtime.render_now(&view, std::slice::from_ref(&sheet));

        assert!(shared.0.borrow().violations.is_empty());
    }

    #[test]
    fn the_default_null_sink_records_nothing_yet_the_frame_still_runs() {
        // No sink installed => the default NullTelemetrySink. The frame must still
        // produce correct output; telemetry is simply discarded (spec §7.9:
        // lightweight counters are opt-in).
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let view = counter_view();
        let sheet = counter_stylesheet();
        let result = runtime.render_now(&view, std::slice::from_ref(&sheet));
        assert!(result.reconciled());
        assert!(result.is_balanced());
        assert!(!result.snapshot().display_list().is_empty());

        // Swapping in a collecting sink only after the frame proves the default
        // path recorded nowhere observable: the fresh sink is empty until the
        // next frame runs.
        let shared = SharedSink::default();
        runtime.set_telemetry_sink(Box::new(shared.clone()));
        assert!(shared.0.borrow().spans.is_empty());
        assert!(shared.0.borrow().counters.is_empty());
    }

    #[test]
    fn an_idle_frame_records_a_zero_phase_frame() {
        // A diagnostics-capture (idle) frame runs no phases but still marks the
        // frame with a phases_run == 0 counter.
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let view = counter_view();
        let sheet = counter_stylesheet();
        let sheets = std::slice::from_ref(&sheet);
        // First a full frame so the tree exists (an idle frame requires a
        // populated tree; the first frame is always full).
        let _ = runtime.render_now(&view, sheets);

        let shared = SharedSink::default();
        runtime.set_telemetry_sink(Box::new(shared.clone()));
        runtime.request_frame(FrameReason::DiagnosticsCapture);
        let idle = runtime
            .run_due_frame(&view, sheets)
            .expect("a requested capture is due");
        assert!(!idle.reconciled() && !idle.restyled() && !idle.relaid_out());

        let sink = shared.0.borrow();
        let span_names: Vec<&str> = sink.spans.iter().map(|s| s.name).collect();
        assert_eq!(span_names, vec!["frame"], "idle frame opens only the span");
        assert_eq!(sink.counters.len(), 1);
        assert_eq!(sink.counters[0].label(), "frame.phases_run");
        assert_eq!(sink.counters[0].value(), 0);
    }

    // ---- Phase 3 step 2: input dispatch (hit-test → hover/focus → pseudo) ----

    use obscur_geometry::Point;
    use obscur_interaction::command::{
        KeyEvent, LogicalKey, Modifier, PhysicalKey, Shortcut,
    };
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

    fn key_a() -> KeyEvent {
        KeyEvent::press(PhysicalKey::KeyA, Some(LogicalKey::character('a')))
    }

    #[test]
    fn handle_key_routes_to_focused_node() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&panel_view(), sheets);
        let (_panel, a, _b) = panel_nodes(&runtime);

        // With nothing focused, a key routes nowhere and schedules no frame.
        let unfocused = runtime.handle_key(key_a());
        assert_eq!(unfocused.target(), None);
        assert!(!unfocused.is_handled());
        assert!(!runtime.has_pending_frame());

        // Focus a via a press, drain its frame, then a key routes to a.
        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
        let _ = runtime.run_due_frame(&panel_view(), sheets);
        let dispatch = runtime.handle_key(key_a());
        assert_eq!(dispatch.target(), Some(a));
        assert!(dispatch.is_handled());
        // No shortcut bound, so nothing resolves to a command.
        assert_eq!(dispatch.command(), None);
        assert!(runtime.has_pending_frame());
    }

    #[test]
    fn handle_key_resolves_a_bound_command_at_the_focused_node() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&panel_view(), sheets);
        let (_panel, a, _b) = panel_nodes(&runtime);

        // Ctrl+S → "file.save".
        runtime.bind_command(
            Shortcut::new(LogicalKey::character('s')).with(Modifier::Ctrl),
            "file.save",
        );

        // Focus a, then hold Ctrl and press S.
        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
        let _ = runtime.run_due_frame(&panel_view(), sheets);

        // A physical Ctrl press folds into the held modifier set (no logical key).
        let ctrl = runtime.handle_key(KeyEvent::press(PhysicalKey::ControlLeft, None));
        assert_eq!(ctrl.target(), Some(a));
        assert_eq!(ctrl.command(), None, "a bare modifier resolves no command");

        // Now Ctrl is held: S resolves to the bound command at the focused node.
        let save = runtime.handle_key(KeyEvent::press(
            PhysicalKey::KeyS,
            Some(LogicalKey::character('s')),
        ));
        assert_eq!(save.target(), Some(a));
        assert_eq!(save.command(), Some("file.save"));
        assert!(save.resolved_command());

        // Releasing Ctrl clears the modifier: S alone no longer resolves.
        let _ = runtime.handle_key(KeyEvent::release(PhysicalKey::ControlLeft, None));
        let bare_s = runtime.handle_key(KeyEvent::press(
            PhysicalKey::KeyS,
            Some(LogicalKey::character('s')),
        ));
        assert_eq!(bare_s.command(), None);
    }

    #[test]
    fn handle_key_without_focus_is_a_noop() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&panel_view(), sheets);
        runtime.bind_command(Shortcut::new(LogicalKey::character('a')), "noop");

        // Nothing focused: even a bound key resolves to nothing and schedules
        // no frame — the runtime state is unchanged.
        let dispatch = runtime.handle_key(key_a());
        assert_eq!(dispatch, KeyDispatch::unhandled());
        assert!(!dispatch.is_handled());
        assert_eq!(dispatch.command(), None);
        assert!(!runtime.has_pending_frame());
    }

    #[test]
    fn handle_key_is_deterministic() {
        let build = || {
            let mut runtime = UiRuntime::new(runtime_id(), viewport());
            let sheet = interaction_sheet();
            let sheets = std::slice::from_ref(&sheet);
            let _ = runtime.render_now(&panel_view(), sheets);
            runtime.bind_command(
                Shortcut::new(LogicalKey::character('s')).with(Modifier::Ctrl),
                "file.save",
            );
            runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
            let _ = runtime.run_due_frame(&panel_view(), sheets);
            runtime.handle_key(KeyEvent::press(PhysicalKey::ControlLeft, None));
            runtime.handle_key(KeyEvent::press(
                PhysicalKey::KeyS,
                Some(LogicalKey::character('s')),
            ))
        };
        assert_eq!(build(), build());
    }

    // ---- Phase 4 final step: motion orchestration (style → motion → paint) ----

    use obscur_motion::track::Keyframe;
    use obscur_motion::value::MotionValue;
    use obscur_style::computed::ComputedStyle;

    /// A linear keyframe track from `from` to `to` over the full `[0, 1]` range.
    fn scalar_track(from: f32, to: f32) -> KeyframeTrack {
        KeyframeTrack::from_keyframes(vec![
            Keyframe::new(0.0, MotionValue::scalar(from)),
            Keyframe::new(1.0, MotionValue::scalar(to)),
        ])
    }

    /// `.counter { height: 50px; background-color: red }` — height and paint but
    /// **no width**, so the node's width is driven purely by the animation.
    fn heightonly_stylesheet() -> Stylesheet {
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
                        PropertyId::Height,
                        PropertyValue::Length(Length::px(50.0)),
                    ),
                ],
            )],
        )
    }

    fn box_width(runtime: &UiRuntime, node: NodeId) -> f32 {
        runtime
            .layout_boxes()
            .get(&node)
            .expect("the animated node has a box")
            .size()
            .width
    }

    #[test]
    fn animated_width_flows_through_style_motion_layout_paint() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let sheet = heightonly_stylesheet();
        let sheets = std::slice::from_ref(&sheet);
        let view = counter_view();

        // First frame populates the tree so the node id exists.
        let _ = runtime.render_now(&view, sheets);
        let node = counter_child(&runtime);

        // Animate width 0px -> 100px over one logical second.
        runtime.animate(
            node,
            PropertyId::Width,
            scalar_track(0.0, 100.0),
            1.0,
            AnimatedKind::LengthPx,
            TransitionInterruption::Blend,
        );
        assert_eq!(runtime.active_animations(), 1, "one running animation");

        // t = 0: width starts near 0.
        let _ = runtime.render_now(&view, sheets);
        assert!(box_width(&runtime, node) < 0.5, "width starts at ~0");

        // t = 0.5: width is ~50 — the animated value reached the layout box.
        runtime.advance_motion(0.5);
        let _ = runtime.render_now(&view, sheets);
        assert!(
            (box_width(&runtime, node) - 50.0).abs() < 0.5,
            "width is ~50 halfway through the animation"
        );

        // t = 1.0: width is ~100 and the timeline finishes/deactivates.
        runtime.advance_motion(0.5);
        let _ = runtime.render_now(&view, sheets);
        assert!(
            (box_width(&runtime, node) - 100.0).abs() < 0.5,
            "width is ~100 at the end of the animation"
        );
        assert_eq!(
            runtime.active_animations(),
            0,
            "the finished timeline deactivated"
        );

        // A further advance holds 100 (the binding keeps applying the endpoint).
        runtime.advance_motion(0.5);
        let _ = runtime.render_now(&view, sheets);
        assert!(
            (box_width(&runtime, node) - 100.0).abs() < 0.5,
            "the settled animation holds its final value"
        );
        assert_eq!(runtime.active_animations(), 0);
    }

    #[test]
    fn animate_requests_an_animation_frame() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let sheet = heightonly_stylesheet();
        let _ = runtime.render_now(&counter_view(), std::slice::from_ref(&sheet));
        let node = counter_child(&runtime);
        assert!(!runtime.has_pending_frame());

        runtime.animate(
            node,
            PropertyId::Width,
            scalar_track(0.0, 100.0),
            1.0,
            AnimatedKind::LengthPx,
            TransitionInterruption::Blend,
        );
        assert!(
            runtime.has_pending_frame(),
            "starting an animation schedules a frame"
        );
    }

    #[test]
    fn opacity_number_overlay_progresses_the_computed_value() {
        // The `Number` kind writes a bare number onto the computed style. Drive
        // the private overlay directly and read the resulting computed opacity.
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let node = NodeId::from_raw(1);
        runtime.animate(
            node,
            PropertyId::Opacity,
            scalar_track(0.0, 1.0),
            1.0,
            AnimatedKind::Number,
            TransitionInterruption::Blend,
        );

        let mut computed = ComputedStyles::new();
        computed.insert(node, ComputedStyle::new());

        // t = 0 -> opacity 0.
        runtime.apply_motion_overlay(&mut computed);
        assert_eq!(computed.get(&node).and_then(ComputedStyle::opacity), Some(0.0));

        // t = 0.5 -> opacity ~0.5.
        runtime.advance_motion(0.5);
        runtime.apply_motion_overlay(&mut computed);
        let mid = computed
            .get(&node)
            .and_then(ComputedStyle::opacity)
            .expect("opacity is set");
        assert!((mid - 0.5).abs() < 1e-6, "opacity progressed to ~0.5");

        // t = 1 -> opacity 1 and the animation deactivates.
        runtime.advance_motion(0.5);
        runtime.apply_motion_overlay(&mut computed);
        assert_eq!(computed.get(&node).and_then(ComputedStyle::opacity), Some(1.0));
        assert_eq!(runtime.active_animations(), 0);
    }

    #[test]
    fn overlay_skips_nodes_without_a_computed_entry() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let node = NodeId::from_raw(7);
        runtime.animate(
            node,
            PropertyId::Width,
            scalar_track(0.0, 100.0),
            1.0,
            AnimatedKind::LengthPx,
            TransitionInterruption::Blend,
        );
        // No entry for `node`: the overlay must not panic or insert one.
        let mut computed = ComputedStyles::new();
        runtime.apply_motion_overlay(&mut computed);
        assert!(computed.is_empty());
    }

    #[test]
    fn advance_motion_with_no_animations_is_a_noop() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        runtime.advance_motion(1.0);
        assert_eq!(runtime.active_animations(), 0);
        assert!(!runtime.has_pending_frame(), "no motion requested no frame");
    }

    #[test]
    fn finished_animation_is_removed_from_the_scheduler() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let node = NodeId::from_raw(3);
        runtime.animate(
            node,
            PropertyId::Width,
            scalar_track(0.0, 100.0),
            1.0,
            AnimatedKind::LengthPx,
            TransitionInterruption::Blend,
        );
        assert_eq!(runtime.active_animations(), 1);
        // One advance past the duration finishes and prunes the timeline.
        runtime.advance_motion(2.0);
        assert_eq!(
            runtime.active_animations(),
            0,
            "a finished timeline is pruned from the scheduler"
        );
    }

    #[test]
    fn re_animating_a_target_supersedes_the_previous_timeline() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let node = NodeId::from_raw(9);
        let first = runtime.animate(
            node,
            PropertyId::Width,
            scalar_track(0.0, 100.0),
            1.0,
            AnimatedKind::LengthPx,
            TransitionInterruption::Blend,
        );
        let second = runtime.animate(
            node,
            PropertyId::Width,
            scalar_track(0.0, 200.0),
            1.0,
            AnimatedKind::LengthPx,
            TransitionInterruption::Blend,
        );
        assert_ne!(first, second, "each animation mints a fresh timeline id");
        // The old timeline was removed; only the new one is active.
        assert_eq!(runtime.active_animations(), 1);
    }

    // ---- Audit V3: transition interruption policy on re-animate (§4.17) ----

    /// Reads the overlaid opacity for `node` after applying the motion overlay to
    /// a fresh computed-style map holding just that node.
    fn overlaid_opacity(runtime: &mut UiRuntime, node: NodeId) -> f32 {
        let mut computed = ComputedStyles::new();
        computed.insert(node, ComputedStyle::new());
        runtime.apply_motion_overlay(&mut computed);
        computed
            .get(&node)
            .and_then(ComputedStyle::opacity)
            .expect("opacity overlaid")
    }

    #[test]
    fn blend_reanimate_starts_from_the_current_sampled_value() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let node = NodeId::from_raw(11);
        // Animate opacity 0 -> 1, then interrupt halfway (current ~= 0.5).
        runtime.animate(
            node,
            PropertyId::Opacity,
            scalar_track(0.0, 1.0),
            1.0,
            AnimatedKind::Number,
            TransitionInterruption::Blend,
        );
        runtime.advance_motion(0.5);
        assert!((overlaid_opacity(&mut runtime, node) - 0.5).abs() < 1e-6);

        // Re-animate toward a *new* target 0 -> 1 with the default Blend policy.
        runtime.animate(
            node,
            PropertyId::Opacity,
            scalar_track(0.0, 1.0),
            1.0,
            AnimatedKind::Number,
            TransitionInterruption::Blend,
        );
        // At t = 0 of the new timeline the overlay must read the *current* value
        // (~0.5) — a seamless blend — NOT the track's original start (0.0) and
        // NOT a snap to the target.
        let start = overlaid_opacity(&mut runtime, node);
        assert!(
            (start - 0.5).abs() < 1e-6,
            "Blend re-animate starts from the current sampled value, got {start}"
        );
        assert_eq!(runtime.active_animations(), 1, "one running animation");
    }

    #[test]
    fn restart_reanimate_starts_from_the_new_track_start() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let node = NodeId::from_raw(12);
        // Start at 0.2 -> 1.0; halfway the current value is ~0.6.
        runtime.animate(
            node,
            PropertyId::Opacity,
            scalar_track(0.2, 1.0),
            1.0,
            AnimatedKind::Number,
            TransitionInterruption::Blend,
        );
        runtime.advance_motion(0.5);
        assert!((overlaid_opacity(&mut runtime, node) - 0.6).abs() < 1e-6);

        // Restart from a fresh 0 -> 1 track: t = 0 must read the track start (0),
        // deliberately snapping away from the current 0.6.
        runtime.animate(
            node,
            PropertyId::Opacity,
            scalar_track(0.0, 1.0),
            1.0,
            AnimatedKind::Number,
            TransitionInterruption::Restart,
        );
        let start = overlaid_opacity(&mut runtime, node);
        assert!(
            start.abs() < 1e-6,
            "Restart re-animate starts from the new track's start, got {start}"
        );
    }

    #[test]
    fn ignore_reanimate_keeps_the_existing_timeline() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let node = NodeId::from_raw(13);
        let first = runtime.animate(
            node,
            PropertyId::Width,
            scalar_track(0.0, 100.0),
            1.0,
            AnimatedKind::LengthPx,
            TransitionInterruption::Blend,
        );
        // An Ignore re-animate drops the incoming request and keeps the running
        // timeline: the same id comes back and nothing new is registered.
        let again = runtime.animate(
            node,
            PropertyId::Width,
            scalar_track(0.0, 200.0),
            1.0,
            AnimatedKind::LengthPx,
            TransitionInterruption::Ignore,
        );
        assert_eq!(first, again, "Ignore keeps the existing timeline id");
        assert_eq!(runtime.active_animations(), 1);
    }

    #[test]
    fn reverse_reanimate_plays_back_toward_the_origin() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let node = NodeId::from_raw(14);
        // Opacity 0 -> 1, interrupted at ~0.5.
        runtime.animate(
            node,
            PropertyId::Opacity,
            scalar_track(0.0, 1.0),
            1.0,
            AnimatedKind::Number,
            TransitionInterruption::Blend,
        );
        runtime.advance_motion(0.5);

        // Reverse: the new track runs from the current value (~0.5) back to the
        // prior origin (0). At t = 0 it holds the current value (no jump)...
        runtime.animate(
            node,
            PropertyId::Opacity,
            scalar_track(0.0, 1.0),
            1.0,
            AnimatedKind::Number,
            TransitionInterruption::Reverse,
        );
        assert!((overlaid_opacity(&mut runtime, node) - 0.5).abs() < 1e-6);
        // ...and after finishing it settles back at the origin (0).
        runtime.advance_motion(1.0);
        assert!(overlaid_opacity(&mut runtime, node).abs() < 1e-6);
    }

    // ---- Audit S1: invalidation classification gates frame phases (§3.4 p5) ----

    #[test]
    fn first_frame_runs_all_phases_even_when_the_reason_is_input() {
        // Classification: the first frame over an empty tree is full regardless
        // of reason, so the tree, styles, boxes and scene all exist.
        assert_eq!(FramePlan::for_frame(FrameReason::Input, true), FramePlan::full());

        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let view = counter_view();
        let sheet = counter_stylesheet();
        let sheets = std::slice::from_ref(&sheet);

        // The very first frame is requested as `Input` yet must reconcile, style,
        // lay out and compose — otherwise the tree would never be built.
        runtime.request_frame(FrameReason::Input);
        let first = runtime
            .run_due_frame(&view, sheets)
            .expect("the requested first frame is due");
        assert!(first.reconciled(), "the first frame populates the tree");
        assert!(first.restyled(), "the first frame resolves styles");
        assert!(first.relaid_out(), "the first frame arranges boxes");
        assert!(
            !first.snapshot().display_list().is_empty(),
            "the first frame composes the matched background"
        );
        assert!(!runtime.layout_boxes().is_empty(), "boxes are retained");
    }

    #[test]
    fn input_frame_skips_reconcile_yet_restyles_and_composes() {
        // Classification: an Input frame over a built tree does NOT reconcile,
        // but does restyle/relayout/recompose (pseudo-state changed).
        let plan = FramePlan::for_frame(FrameReason::Input, false);
        assert!(!plan.reconcile(), "hover/focus does not re-reconcile");
        assert!(plan.style() && plan.layout() && plan.compose());

        // End-to-end: a hover-only Input frame paints the `:hover` rule (proving
        // style + compose ran) while its FrameResult reports no reconciliation.
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);

        let cold = runtime.render_now(&panel_view(), sheets); // full first frame
        let cold_fills = cold.snapshot().display_list().len();

        // Point at b → schedules an Input frame; run it.
        runtime.handle_pointer(mouse(PointerEventKind::Move, 50.0, 75.0));
        let hot = runtime
            .run_due_frame(&panel_view(), sheets)
            .expect("the input-requested frame is due");

        assert!(
            !hot.reconciled(),
            "the input frame skipped reconciliation (no structural change)"
        );
        assert!(hot.restyled(), "the input frame restyled for the new hover");
        assert!(
            hot.snapshot().display_list().len() > cold_fills,
            ":hover reached paint, so style + compose ran on the input frame"
        );
    }

    #[test]
    fn diagnostics_capture_frame_is_idle_and_reuses_the_retained_scene() {
        // Classification: a diagnostics capture recalculates nothing.
        assert!(FramePlan::for_frame(FrameReason::DiagnosticsCapture, false).is_idle());

        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let view = counter_view();
        let sheet = counter_stylesheet();
        let sheets = std::slice::from_ref(&sheet);

        // Build a real scene first.
        let base = runtime.render_now(&view, sheets);
        let base_commands: Vec<_> = base.snapshot().display_list().commands().to_vec();
        assert!(!base_commands.is_empty());

        // A diagnostics-capture frame does no phase work: it re-presents the
        // retained scene verbatim and reports every phase flag false.
        runtime.request_frame(FrameReason::DiagnosticsCapture);
        let capture = runtime
            .run_due_frame(&view, sheets)
            .expect("the capture frame is due");
        assert!(!capture.reconciled());
        assert!(!capture.restyled());
        assert!(!capture.relaid_out());
        let capture_commands: Vec<_> = capture.snapshot().display_list().commands().to_vec();
        assert_eq!(
            capture_commands, base_commands,
            "an idle capture re-presents the retained scene unchanged"
        );
    }

    /// A stub [`TextShaper`] emitting a fixed glyph id per source char, so an
    /// installed shaper is observable in the composed `DrawText` run.
    struct FixedIdShaper(u32);

    impl obscur_text::shaping::TextShaper for FixedIdShaper {
        fn shape(
            &self,
            request: &obscur_text::shaping::ShapingRequest,
        ) -> obscur_text::shaping::GlyphRun {
            let glyphs = request
                .text()
                .char_indices()
                .map(|(byte, _)| obscur_text::shaping::ShapedGlyph::new(self.0, byte, 8.0))
                .collect();
            obscur_text::shaping::GlyphRun::new(
                glyphs,
                request.direction(),
                request.font_size(),
                obscur_text::span::TextRange::new(0, request.text().len()),
            )
        }

        fn can_shape(&self, _face: &obscur_text::font::FontFace) -> bool {
            true
        }
    }

    /// Collects the glyph ids of every `DrawText` run in a frame's display list.
    fn drawtext_glyph_ids(result: &FrameResult) -> Vec<u32> {
        result
            .snapshot()
            .display_list()
            .commands()
            .iter()
            .filter_map(|c| match c {
                obscur_scene::command::PaintCommand::DrawText { run, .. } => Some(run),
                _ => None,
            })
            .flat_map(|run| run.glyphs().iter().map(|g| g.glyph_id()))
            .collect()
    }

    #[test]
    fn set_text_shaper_changes_the_emitted_glyph_run() {
        // A view with a single text node "AB": the pipeline lays it out and the
        // scene coordinator emits a DrawText run for it.
        let view = View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(10))
                .with_children(ViewFragment::from_nodes(vec![ViewNode::text("AB")])),
        ]));

        // Default (mock) shaper: glyph ids are the Unicode scalars of "AB".
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let mock = runtime.render_now(&view, &[]);
        let mock_ids = drawtext_glyph_ids(&mock);
        assert_eq!(mock_ids, vec![u32::from('A'), u32::from('B')]);

        // Install a stub shaper through the public runtime seam and re-render: the
        // composed run now reflects it, proving the shaper injection plumbing.
        runtime.set_text_shaper(Box::new(FixedIdShaper(7)));
        let stub = runtime.render_now(&view, &[]);
        let stub_ids = drawtext_glyph_ids(&stub);
        assert_eq!(stub_ids, vec![7, 7]);
        assert_ne!(mock_ids, stub_ids);
    }

    /// A shaper that tallies every `shape` call through a shared cell, so a test
    /// can read the count after the shaper is moved behind the runtime seam.
    struct TallyShaper(std::rc::Rc<std::cell::Cell<usize>>);

    impl obscur_text::shaping::TextShaper for TallyShaper {
        fn shape(
            &self,
            request: &obscur_text::shaping::ShapingRequest,
        ) -> obscur_text::shaping::GlyphRun {
            self.0.set(self.0.get() + 1);
            obscur_text::shaping::MockTextShaper::new().shape(request)
        }

        fn can_shape(&self, _face: &obscur_text::font::FontFace) -> bool {
            true
        }
    }

    #[test]
    fn shaping_cache_dedups_measure_arrange_and_compose_within_a_frame() {
        // A single text node needs shaping in the layout measure pass, the layout
        // arrange pass, AND the scene compose pass. The runtime wraps the installed
        // shaper in a CachingShaper, so the underlying engine must be invoked
        // exactly ONCE for that one unique run despite all three needing it.
        let view = View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(11))
                .with_children(ViewFragment::from_nodes(vec![ViewNode::text("cached")])),
        ]));

        let calls = std::rc::Rc::new(std::cell::Cell::new(0));
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        runtime.set_text_shaper(Box::new(TallyShaper(std::rc::Rc::clone(&calls))));

        let frame = runtime.render_now(&view, &[]);
        // The text still shaped correctly (mock: glyph ids are the scalars).
        assert_eq!(
            drawtext_glyph_ids(&frame),
            "cached".chars().map(u32::from).collect::<Vec<_>>()
        );
        // ...but the inner engine ran once, not once per measure/arrange/compose.
        assert_eq!(
            calls.get(),
            1,
            "measure + arrange + compose must share one cached shaping call"
        );
    }

    /// Collects the `(font, glyph_ids)` of every `DrawText` run in a frame, in
    /// paint order.
    fn drawtext_font_runs(result: &FrameResult) -> Vec<(u32, Vec<u32>)> {
        result
            .snapshot()
            .display_list()
            .commands()
            .iter()
            .filter_map(|c| match c {
                obscur_scene::command::PaintCommand::DrawText { run, .. } => Some((
                    run.font(),
                    run.glyphs().iter().map(|g| g.glyph_id()).collect(),
                )),
                _ => None,
            })
            .collect()
    }

    /// A stub itemizer cutting the fixed two-script string "abВГ" into a Latin run
    /// `[0..2)` and a Cyrillic run `[2..6)` (each Cyrillic char is 2 bytes).
    struct TwoScriptItemizer;

    impl obscur_text::shaping::TextItemizer for TwoScriptItemizer {
        fn itemize(
            &self,
            _text: &str,
            _base: obscur_text::shaping::TextDirection,
        ) -> Vec<obscur_text::shaping::TextItem> {
            use obscur_text::shaping::{ScriptTag, TextDirection, TextItem};
            use obscur_text::span::TextRange;
            vec![
                TextItem::new(TextRange::new(0, 2), ScriptTag::LATIN, TextDirection::LeftToRight, 0),
                TextItem::new(
                    TextRange::new(2, 6),
                    ScriptTag::CYRILLIC,
                    TextDirection::LeftToRight,
                    0,
                ),
            ]
        }
    }

    #[test]
    fn multi_font_text_renders_font_tagged_segments_end_to_end() {
        use obscur_text::font::{FontId, MockFontResolver};
        use obscur_text::shaping::ScriptTag;

        // A single two-script text node "abВГ". Installing a script itemizer plus a
        // covering resolver through the public runtime seams makes the whole
        // measure → arrange → compose path route through font fallback, so the
        // frame paints two font-tagged DrawText runs on one line.
        let view = View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(12))
                .with_children(ViewFragment::from_nodes(vec![ViewNode::text("abВГ")])),
        ]));

        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        runtime.set_text_itemizer(Box::new(TwoScriptItemizer));
        let mut resolver = MockFontResolver::new();
        let face = obscur_text::font::FontFace::new(
            obscur_text::font::FontFamily::generic(obscur_text::font::GenericFamily::Monospace),
            obscur_text::font::FontAttributes::default(),
            obscur_text::font::FontSource::system("mock"),
            obscur_text::font::FontMetrics::new(1000, 800.0, -200.0, 0.0),
        );
        resolver.register(ScriptTag::LATIN, FontId::new(0), face.clone());
        resolver.register(ScriptTag::CYRILLIC, FontId::new(1), face);
        runtime.set_font_resolver(Box::new(resolver));

        let frame = runtime.render_now(&view, &[]);
        let runs = drawtext_font_runs(&frame);
        // Two font-tagged runs: Latin (font 0, "ab") then Cyrillic (font 1, "ВГ").
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].0, 0);
        assert_eq!(runs[0].1, vec![u32::from('a'), u32::from('b')]);
        assert_eq!(runs[1].0, 1);
        assert_eq!(runs[1].1, vec![u32::from('В'), u32::from('Г')]);
    }

    /// An itemizer that counts every `itemize` call into a shared cell (readable
    /// after it is boxed into the runtime) and delegates to a single-run
    /// itemizer. Proves how often a frame reaches the underlying pipeline.
    struct CountingItemizer(std::rc::Rc<std::cell::Cell<usize>>);

    impl obscur_text::shaping::TextItemizer for CountingItemizer {
        fn itemize(
            &self,
            text: &str,
            base: obscur_text::shaping::TextDirection,
        ) -> Vec<obscur_text::shaping::TextItem> {
            self.0.set(self.0.get() + 1);
            obscur_text::shaping::SingleRunItemizer::new().itemize(text, base)
        }
    }

    #[test]
    fn one_frame_itemizes_a_text_node_once_across_measure_arrange_compose() {
        // Requirement (d): a text node is shaped in the layout pass (measure +
        // arrange) AND again in the scene pass (compose), every frame. The
        // paragraph cache — shared because both passes borrow the same
        // `ParagraphShaper` — must collapse all of that to a single itemization.
        let view = View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(14))
                .with_children(ViewFragment::from_nodes(vec![ViewNode::text("hello")])),
        ]));

        let calls = std::rc::Rc::new(std::cell::Cell::new(0));
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        runtime.set_text_itemizer(Box::new(CountingItemizer(calls.clone())));

        let _ = runtime.render_now(&view, &[]);
        assert_eq!(
            calls.get(),
            1,
            "measure + arrange + compose must share one cached itemization"
        );
    }

    #[test]
    fn default_single_font_text_stays_one_primary_run() {
        // Requirement (a): with no ports installed the default single-run path
        // renders a text node exactly as before — one DrawText run on the primary
        // face (font 0) whose glyph ids are the Unicode scalars.
        let view = View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(13))
                .with_children(ViewFragment::from_nodes(vec![ViewNode::text("hi")])),
        ]));
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let runs = drawtext_font_runs(&runtime.render_now(&view, &[]));
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].0, 0);
        assert_eq!(runs[0].1, vec![u32::from('h'), u32::from('i')]);
    }
}
