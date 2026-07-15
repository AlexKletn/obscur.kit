//! The embeddable UI runtime and application-core integrator: [`UiRuntime`].

use std::collections::{HashMap, HashSet};
use std::fmt;

use obscur_accessibility::port::AccessibilityBridge;
use obscur_accessibility::tree::{SemanticSnapshot, SemanticTree};
use obscur_diagnostics::{
    BudgetViolation, DiagnosticSubsystem, NullTelemetrySink, PerformanceCounter, PerformanceSpan,
    TelemetrySink,
};
use obscur_geometry::{AffineTransform, Point, Size};
use obscur_identity::{InternedSymbol, NodeId, RuntimeId};
use obscur_interaction::command::{
    CommandRouter, KeyEvent, LogicalKey, Modifier, Modifiers, NamedKey, Shortcut,
};
use obscur_interaction::hover::{HoverChain, HoverTransition};
use obscur_interaction::pointer::{PointerEvent, PointerEventKind};
use obscur_interaction::spatial::{HitTest, SpatialIndex};
use obscur_motion::scheduler::MotionScheduler;
use obscur_motion::timeline::{Timeline, TimelineId};
use obscur_motion::track::{Keyframe, KeyframeTrack};
use obscur_motion::transition::TransitionInterruption;
use obscur_motion::value::MotionValue;
use obscur_resources::handle::{ResourceHandle, ResourceKind};
use obscur_resources::hub::{ResourceHub, ResourceRequest};
use obscur_resources::port::{ResourceDecoder, ResourceLoader};
use obscur_resources::source::ResourceSource;
use obscur_resources::state::{ResourceError, ResourceMetadata};
use obscur_scene::color::AlphaMode;
use obscur_scene::command::{DisplayList, PaintCommand};
use obscur_scene::paint::SceneImage;
use obscur_scene::scene::{Scene, SceneSnapshot};
use obscur_layout::invalidation::LayoutInvalidation;
use obscur_style::computed::{ComputedStyle, ComputedValue};
use obscur_style::invalidation::StyleInvalidation;
use obscur_style::matching::{MediaEnvironment, Stylesheet};
use obscur_style::property::{Length, PropertyId, PropertyValue};
use obscur_style::selector::{PseudoState, SelectorMatcher};
use obscur_text::editing::{DeletionUnit, EditCommand, EditDirection, NavigationUnit};
use obscur_text::font::FontResolver;
use obscur_text::ime::CompositionEvent;
use obscur_text::layout::{Hyphenator, TextAlignment};
use obscur_text::selection::TextSelection;
use obscur_text::shaping::{TextDirection, TextItemizer, TextShaper};
use obscur_tree::node::NodeKind;
use obscur_tree::reconciliation::Reconciler;
use obscur_view::component::{ComponentRegistration, RegistryError};
use obscur_tree::tree::RetainedTree;
use obscur_view::view::{View, EVENT_CHANGE, EVENT_CLICK, EVENT_INPUT};

use super::{
    FrameBudget, FramePlan, KeyDispatch, PointerDispatch, SemanticTreeBuilder, TextEditSession,
};
use crate::frame::{FrameContext, FrameId, FrameReason, FrameResult};
use crate::invalidation::{Invalidation, InvalidationReason, InvalidationRouter};
use crate::orchestration::{
    overflow_scrolls, root_font_size, scroll_containers, AnimatedKind, AnimationBinding, ComputedStyles,
    overlay_commands, CssMotionRules, EditingView, InspectInfo, LayoutBoxes, LayoutCoordinator,
    MotionCoordinator, NodeInspection, PreeditView, PseudoStates, SceneCoordinator, ScrollContainer,
    ScrollPaint, StyleCoordinator, StyleInputs, StyleMatchingCoordinator, TreeElement, TreeInspection,
};
use crate::orchestration::{alignment_for, direction_for, white_space_for};
use crate::scheduler::{BackpressureDecision, FrameScheduler};
use crate::component::ComponentStore;

/// How many times [`UiRuntime::render_components`] re-renders newly appeared
/// components within one frame before giving up.
///
/// A component's output can contain further component elements, so rendering is a
/// fixpoint. This bound is not a design limit on nesting depth — real trees nest
/// far shallower — but a backstop: a component that renders itself, directly or
/// mutually, would otherwise spin the frame forever. Exceeding it truncates that
/// frame's tree instead of hanging, and the `component.rendered` counter shows it.
const MAX_COMPONENT_RENDER_PASSES: usize = 32;
use crate::transaction::ReconcileEnactor;

/// The maximum gap, in microseconds, between two presses for them to count as a
/// multi-click (the platform double-click window; 500 ms is the common default).
const MULTI_CLICK_WINDOW_MICROS: u64 = 500_000;

/// The maximum distance, in logical pixels, between two presses for them to count
/// as a multi-click — a press that jumps farther starts a fresh count.
const MULTI_CLICK_RADIUS: f32 = 4.0;

/// The last [`PointerEventKind::Down`] press, retained so a rapid run of presses
/// on the same target near the same point is recognised as a double/triple
/// *click* (spec §5.11 editing engine — word/line selection via the pointer).
#[derive(Debug, Clone, Copy)]
struct PointerClick {
    /// The interactive target the press landed on (`None` for empty space).
    target: Option<NodeId>,
    /// The press point in content space.
    point: Point,
    /// The press's logical timestamp, in microseconds.
    time_micros: u64,
    /// The running click count of this press (`1`, `2`, or `3`, then cycling).
    count: u32,
}

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
    /// The non-style inputs the retained boxes were last arranged under (see
    /// [`LayoutInputs`]). `None` until the first layout runs.
    last_layout_inputs: Option<LayoutInputs>,
    /// Whether the in-window inspect overlay is active (spec §7.6). When set, a
    /// composed frame appends a highlight over the hovered node's border box so a
    /// developer can debug the running UI directly in the window.
    inspect_mode: bool,
    /// The node that currently holds focus, if any.
    ///
    /// Tracked directly rather than via
    /// [`FocusTree`](obscur_interaction::focus::FocusTree): the focus tree
    /// requires nodes to be pre-registered as [`FocusNode`]s, which this runtime
    /// does not yet do, so a plain `Option<NodeId>` is the simpler correct model
    /// for pointer-press focus (documented fallback).
    focused: Option<NodeId>,
    /// The node the most recent [`PointerEventKind::Down`] press landed on, if
    /// any — the anchor for click detection. A [`PointerEventKind::Up`] whose
    /// hit-test target equals this node completes a *click* on it; the field is
    /// then cleared. `None` between clicks (no press outstanding).
    press_target: Option<NodeId>,
    /// The in-flight pointer drag-selection over the focused editable, if any:
    /// `(field node, anchor byte offset)`. Armed by a
    /// [`PointerEventKind::Down`] whose editable ancestor has an active editing
    /// session — the anchor is the click's byte offset, the fixed end of the
    /// selection a subsequent drag extends from. A [`PointerEventKind::Move`]
    /// while this is set re-selects `[anchor, current]` over the same field; a
    /// [`PointerEventKind::Up`] clears it. `None` between presses (spec §5.11).
    text_drag: Option<(NodeId, usize)>,
    /// The most recent [`PointerEventKind::Down`] press, used to detect a
    /// double/triple *click* (a rapid run of presses on the same target at
    /// nearly the same point) so the pointer path can select a word or line
    /// under the cursor (spec §5.11 editing engine — word/line navigation).
    /// `None` until the first press; see [`next_click_count`](Self::next_click_count).
    last_click: Option<PointerClick>,
    /// The click count of the current [`PointerEventKind::Down`]: `1` for a
    /// single click (caret placement), `2` for a double click (word select),
    /// `3` for a triple click (line select), cycling back to `1`. Resolved once
    /// per press in [`handle_pointer`](Self::handle_pointer) and consumed by
    /// [`begin_text_pointer`](Self::begin_text_pointer).
    click_count: u32,
    /// The command-handler registry: application logic to run when a dispatched
    /// command fires, keyed by the command's [`InternedSymbol`] (spec §2.3,
    /// §3.1 — events flow up to a command, the command runs a handler).
    ///
    /// The app owns its command symbols: it binds `on-click → command` in the
    /// View (via [`ViewEventBinding`](obscur_view::view::ViewEventBinding)) and
    /// registers the *same* symbol here via
    /// [`on_command`](Self::on_command); the runtime only *compares* symbols, so
    /// no interner coherence is needed. A handler is a `FnMut()` that typically
    /// captures app state through an `Rc<RefCell<_>>`/`Rc<Cell<_>>`. Exactly one
    /// handler per command (a later registration replaces an earlier one), so
    /// dispatch is a deterministic single lookup — no iteration order observed.
    handlers: HashMap<InternedSymbol, Box<dyn FnMut()>>,
    /// The text-input handler registry: application logic to run when a focused
    /// editable receives a per-keystroke [`TextEdit`] (spec §6.7). Keyed, like
    /// [`handlers`](Self::handlers), by the command symbol the app binds
    /// `on-input → command` to in the View (via
    /// [`EVENT_INPUT`](obscur_view::view::EVENT_INPUT)) and registers the *same*
    /// symbol for via [`on_text_input`](Self::on_text_input).
    ///
    /// Distinct from `handlers` because a text-input handler takes the field's
    /// **resulting text** (`FnMut(&str)`), whereas a command handler takes
    /// nothing. The runtime now *owns* the focused field's editing state (see
    /// [`editing`](Self::editing)); after each mutating edit it hands the app the
    /// whole resulting session text so the app's mirror (typically an
    /// `Rc<RefCell<String>>` the reactive view builder reads back) stays in sync
    /// with the runtime-owned buffer. Exactly one handler per command (a later
    /// registration replaces an earlier one).
    text_input_handlers: HashMap<InternedSymbol, Box<dyn FnMut(&str)>>,
    /// The **change** handler registry: application logic to run when a focused
    /// editable *commits* its value (WHATWG HTML §4.10.5.5 — the `change` event
    /// fires when the user commits an edit, i.e. on blur or, for a single-line
    /// `input`, on `Enter`; *not* per keystroke and *not* on a focus click).
    /// Keyed, like [`text_input_handlers`](Self::text_input_handlers), by the
    /// command symbol the app binds `on-change → command` to in the View (via
    /// [`EVENT_CHANGE`](obscur_view::view::EVENT_CHANGE) — an
    /// [`Input`](../../obscur_widgets/input/struct.Input.html) binds it) and
    /// registers for via [`on_change`](Self::on_change).
    ///
    /// A **separate** registry from [`handlers`](Self::handlers) and
    /// [`text_input_handlers`](Self::text_input_handlers) precisely because an
    /// editable binds `EVENT_INPUT` *and* `EVENT_CHANGE` to the *same* command
    /// symbol: routing per-keystroke `input` and commit `change` through
    /// distinct registries lets one app receive both without conflating them (an
    /// `on_text_input` fires per char; an `on_change` fires once on commit with
    /// the committed value). A change handler takes the field's **committed
    /// text** (`FnMut(&str)`). Exactly one handler per command (a later
    /// registration replaces an earlier one).
    change_handlers: HashMap<InternedSymbol, Box<dyn FnMut(&str)>>,
    /// The runtime-owned editing session for the focused editable, if any (spec
    /// §5.11, §6.7). Seeded from the field's current text when focus lands on an
    /// editable (a node whose ancestor chain binds
    /// [`EVENT_INPUT`](obscur_view::view::EVENT_INPUT)) and cleared when focus
    /// leaves one. The runtime is the source of truth for the focused field's
    /// text/caret/selection; keys resolve to
    /// [`EditCommand`](obscur_text::editing::EditCommand)s applied to its buffer.
    editing: Option<TextEditSession>,
    /// The shortcut→command-symbol binding table keyboard input resolves against
    /// (spec §6.7). Populated by [`bind_command`](Self::bind_command); consulted
    /// by [`handle_key`](Self::handle_key) on each key press, which dispatches
    /// the resolved command through the same handler registry a pointer click
    /// uses.
    commands: CommandRouter,
    /// The modifier keys currently held, folded from physical modifier-key
    /// press/release transitions so shortcut matching sees the live chord.
    modifiers: Modifiers,
    /// The id assigned to the next produced frame.
    next_frame: FrameId,
    /// The viewport frames are laid out under.
    viewport: Size,
    /// The viewport scroll offset `(x, y)` in logical pixels — the document-
    /// coordinate point currently at the top-left of the viewport (spec §4.1).
    ///
    /// The single scroll container is the viewport; there is no live scroll-gesture
    /// system yet, so this is driven by tests / hosts through
    /// [`set_viewport_scroll`](Self::set_viewport_scroll) and defaults to
    /// `(0.0, 0.0)`. It is consumed only by `position: sticky` layout: at `(0, 0)`
    /// every sticky node lays out at its static flow position.
    viewport_scroll: (f32, f32),
    /// The host device-pixel scale (HiDPI factor × user zoom) the composed scene
    /// is rendered at (spec §5.2 device independence). Default `1.0`.
    ///
    /// Layout runs in **logical** CSS pixels (the host points [`viewport`] at
    /// `physical_extent / device_scale`); the produced display list is then
    /// bracketed by a single root `PushTransform(scale(s, s))` … `PopTransform`
    /// so the whole scene rasterizes at physical resolution — crisp on any
    /// backend, since the root transform is an ordinary balanced transform pair
    /// both renderers already honor. At `1.0` no bracket is emitted, so the
    /// display list (and every existing test / headless caller) is byte-identical.
    ///
    /// Pointer coordinates arrive in the host's physical space, so
    /// [`handle_pointer`](Self::handle_pointer) divides them by this scale before
    /// hit-testing against the logical layout. Clamped to a sane minimum by
    /// [`set_device_scale`](Self::set_device_scale).
    device_scale: f32,
    /// The content scroll offset `(x, y)` in **logical** CSS pixels — the point of
    /// the content currently pinned to the viewport's top-left. Default `(0.0,
    /// 0.0)`.
    ///
    /// Composed with [`device_scale`](Self::device_scale) as the scene's single
    /// root transform `scale(s) · translate(-ox, -oy)`: scrolling moves content
    /// up/left, then the whole scene scales to physical resolution. At offset
    /// `(0, 0)` **and** scale `1.0` no root bracket is emitted, so the display
    /// list is byte-identical to the pre-scroll path (the regression guard).
    ///
    /// Pointer coordinates arrive in the host's physical space, so
    /// [`handle_pointer`](Self::handle_pointer) maps them to content space as
    /// `physical / device_scale + offset` (the offset added back after the
    /// device-scale divide) so hit-testing stays correct while scrolled. Set by a
    /// host via [`set_content_offset`](Self::set_content_offset).
    content_offset: (f32, f32),
    /// The per-element vertical scroll position (`scroll_top`, logical CSS px)
    /// of each `overflow: scroll`/`auto` scroll container the host or a wheel
    /// gesture has scrolled (CSS Overflow Module Level 3 §scroll-container). Keyed
    /// by the container's [`NodeId`]; a node **absent** from the map is at
    /// `scroll_top == 0` (the initial, unscrolled position), so an untouched tree
    /// keeps an empty map and composes byte-identically.
    ///
    /// Stored values are re-clamped to the container's live
    /// `0 ..= content_height − scrollport_height` range each time they are read or
    /// written (see [`scroll_paint`](Self::scroll_paint)), so a layout change that
    /// shrinks the content never leaves a stale over-scroll.
    scroll_top: HashMap<NodeId, f32>,
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
    /// The host-installed view **builder** — the keystone of the reactive
    /// re-render loop (spec §3.1). When set (via
    /// [`set_root_view`](Self::set_root_view)), the reactive frame path
    /// [`run_due_frame_reactive`](Self::run_due_frame_reactive) calls it each due
    /// frame to *rebuild* the desired [`View`] from whatever application state a
    /// command handler just mutated, closing the loop
    /// click → handler mutates state → `request_frame` → rebuild → reconcile →
    /// paint. It is a `FnMut() -> View` that typically captures the same
    /// `Rc<RefCell<_>>`/`Rc<Cell<_>>` app state the command handlers do, so the
    /// view it produces reflects the latest state. `None` until installed, in
    /// which case the reactive path is a no-op — the explicit
    /// [`run_frame`](Self::run_frame)/[`run_due_frame`](Self::run_due_frame)
    /// callers that pass a `&View` directly are entirely unaffected.
    root_view: Option<Box<dyn FnMut() -> View>>,
    /// The registered component kinds and one live instance per component-backed
    /// node (spec §2.1, §2.14).
    ///
    /// Empty and inert until a host calls
    /// [`register_component`](Self::register_component): with no registrations
    /// nothing mounts, so a host that drives the tree purely from its root view
    /// builder pays nothing for this.
    components: ComponentStore,
    /// The stylesheets the reactive frame path matches against the rebuilt view
    /// (installed via [`set_stylesheets`](Self::set_stylesheets)), parallel to the
    /// held [`root_view`](Self::root_view). Empty by default; the explicit
    /// `&[Stylesheet]`-taking render paths ignore this field.
    stylesheets: Vec<Stylesheet>,
    /// The built-in **user-agent (UA) default** stylesheet, installed via
    /// [`set_default_stylesheet`](Self::set_default_stylesheet). When present it is
    /// matched *first* — below every author sheet — on **all** render paths
    /// (`run_frame`/`run_due_frame`/`render_now`/the reactive path), so its rules
    /// form the toolkit's overridable base look. It carries the
    /// [`CascadeOrigin::SystemDefault`](obscur_style::cascade::CascadeOrigin::SystemDefault)
    /// origin — the lowest in the cascade — so any author/theme rule of the same or
    /// higher specificity wins over it (§4.14). [`None`] by default, in which case
    /// matching is byte-identical to passing the author sheets alone.
    default_stylesheet: Option<Stylesheet>,
    /// The injected loader that resolves a `background-image: url(...)` source's
    /// raw encoded bytes (spec §5.15), or [`None`] when no loader was installed —
    /// in which case `url()` backgrounds silently resolve to nothing (the
    /// honest-capability contract). The runtime holds only the domain
    /// [`ResourceLoader`] port; the facade injects a concrete adapter (e.g. an
    /// `FsResourceLoader`) via [`set_resource_providers`](Self::set_resource_providers).
    resource_loader: Option<Box<dyn ResourceLoader>>,
    /// The injected decoder that turns a loaded image's encoded bytes into RGBA8
    /// pixels + intrinsic size (spec §5.16), or [`None`] when none was installed.
    /// Both a loader and a decoder are required to resolve an image; either absent
    /// means `url()` backgrounds paint nothing.
    resource_decoder: Option<Box<dyn ResourceDecoder>>,
    /// The pure lifecycle/versioning registry for requested image resources (spec
    /// §5.15). It stores only lifecycle + metadata, never the decoded pixels — the
    /// pixels live in [`image_cache`](Self::image_cache) — so the two are kept in
    /// step by URL.
    resource_hub: ResourceHub,
    /// The decoded images resolved from `background-image: url(...)`, keyed by the
    /// raw authored URL/path string, handed to the scene coordinator each compose.
    /// A URL is inserted once its bytes load and decode; a failed load caches
    /// nothing (the image simply is not painted).
    image_cache: HashMap<Box<str>, SceneImage>,
    /// The [`ResourceHandle`] minted for each requested image URL, retained so the
    /// hub lifecycle (begin_load → complete/fail) is driven per resource and a URL
    /// is requested at most once.
    image_handles: HashMap<Box<str>, ResourceHandle>,
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
            last_layout_inputs: None,
            inspect_mode: false,
            focused: None,
            press_target: None,
            text_drag: None,
            last_click: None,
            click_count: 1,
            handlers: HashMap::new(),
            text_input_handlers: HashMap::new(),
            change_handlers: HashMap::new(),
            editing: None,
            commands: CommandRouter::new(),
            modifiers: Modifiers::new(),
            next_frame: FrameId::FIRST,
            viewport,
            viewport_scroll: (0.0, 0.0),
            device_scale: 1.0,
            content_offset: (0.0, 0.0),
            scroll_top: HashMap::new(),
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
            root_view: None,
            components: ComponentStore::new(),
            stylesheets: Vec::new(),
            default_stylesheet: None,
            resource_loader: None,
            resource_decoder: None,
            resource_hub: ResourceHub::new(),
            image_cache: HashMap::new(),
            image_handles: HashMap::new(),
        }
    }

    /// Installs the resource `loader` + `decoder` the runtime uses to resolve
    /// `background-image: url(...)` images (spec §5.15, §5.16), replacing any
    /// previously installed pair and clearing the resolved-image cache so the new
    /// providers re-resolve every URL from scratch.
    ///
    /// Resource loading/decoding are hexagonal ports: the runtime (application
    /// layer) depends only on the domain [`ResourceLoader`] / [`ResourceDecoder`]
    /// traits, never on a concrete adapter. The **facade** builds real adapters
    /// (e.g. a sandbox-rooted `FsResourceLoader` + the `image`-crate `ImageDecoder`)
    /// and injects them here. Leaving them unset is the honest-capability default:
    /// a `url()` background then resolves to nothing and paints nothing — never a
    /// crash.
    pub fn set_resource_providers(
        &mut self,
        loader: Box<dyn ResourceLoader>,
        decoder: Box<dyn ResourceDecoder>,
    ) {
        self.resource_loader = Some(loader);
        self.resource_decoder = Some(decoder);
        self.image_cache.clear();
        self.image_handles.clear();
    }

    /// Resolves every not-yet-cached `background-image: url(...)` referenced by the
    /// freshly `computed` styles into a decoded [`SceneImage`] in
    /// [`image_cache`](Self::image_cache) (spec §5.15 load → §5.16 decode).
    ///
    /// For each new URL this drives the full hub lifecycle: `request` a handle for
    /// a [`ResourceKind::Image`] source built from the URL, `begin_load`, then
    /// `loader.load` → `decoder.decode` → build a `SceneImage` from the decoded
    /// RGBA8 + intrinsic size, and `complete` the handle. A loader/decoder error
    /// (or a malformed decoded buffer) `fail`s the handle and caches nothing, so
    /// the image is simply not painted — no panic. With no loader/decoder installed
    /// this is a no-op (the cache stays empty). Synchronous with the filesystem
    /// loader (ready the same frame); the design is async-friendly — an unready URL
    /// is just absent from the cache and paints on a later frame once decoded.
    fn resolve_background_images(&mut self, computed: &ComputedStyles) {
        // Both a loader and a decoder are required; without either, `url()`
        // backgrounds honestly resolve to nothing.
        let (Some(loader), Some(decoder)) =
            (self.resource_loader.as_deref(), self.resource_decoder.as_deref())
        else {
            return;
        };
        // Disjoint field borrows: `loader`/`decoder` borrow their fields while the
        // hub and caches are mutated through their own distinct fields.
        for style in computed.values() {
            let Some(url) = style
                .get(PropertyId::BackgroundImage)
                .and_then(ComputedValue::as_background_image_url)
            else {
                continue;
            };
            if self.image_cache.contains_key(url) || self.image_handles.contains_key(url) {
                // Already resolved, or already attempted (and failed): do not retry.
                continue;
            }
            let key: Box<str> = url.into();
            // A `url()` is resolved by the loader port from a plain path/source.
            let source = ResourceSource::file(key.clone());
            let handle = self
                .resource_hub
                .request(ResourceRequest::new(source.clone(), ResourceKind::Image));
            self.image_handles.insert(key.clone(), handle);
            let Some(version) = self.resource_hub.begin_load(handle) else {
                continue;
            };
            // Load → decode → build the SceneImage; any failure fails the handle
            // and caches nothing (the image is not painted).
            match load_and_decode_image(loader, decoder, &source) {
                Ok(image) => {
                    let metadata = ResourceMetadata::new().with_intrinsic_size(Size::new(
                        image.width() as f32,
                        image.height() as f32,
                    ));
                    self.resource_hub.complete(handle, version, metadata);
                    self.image_cache.insert(key, image);
                }
                Err(error) => {
                    self.resource_hub.fail(handle, version, error);
                }
            }
        }
    }

    /// Resolves every not-yet-cached element image `src` (an `<img>` /
    /// [`Image`](obscur_primitives::Image) primitive's source) referenced by the
    /// retained tree into a decoded [`SceneImage`] in
    /// [`image_cache`](Self::image_cache) (spec §5.15 load → §5.16 decode).
    ///
    /// Element images share the **same** cache, hub lifecycle, and load/decode path
    /// as `background-image: url(...)` (see
    /// [`resolve_background_images`](Self::resolve_background_images)), keyed by the
    /// raw `src` string. A URL already resolved (or already attempted and failed) is
    /// not retried; a loader/decoder error `fail`s the handle and caches nothing, so
    /// the image is simply not painted — no panic. With no loader/decoder installed
    /// this is a no-op (the honest-capability default).
    fn resolve_element_images(&mut self) {
        // Both a loader and a decoder are required; without either, element images
        // honestly resolve to nothing.
        let (Some(loader), Some(decoder)) =
            (self.resource_loader.as_deref(), self.resource_decoder.as_deref())
        else {
            return;
        };
        // Gather each element node's own image source up front, so the retained-tree
        // borrow is released before the hub / caches are mutated below.
        let sources: Vec<Box<str>> = self
            .tree
            .iter()
            .filter_map(|(_, node)| node.attributes().image_source().map(Box::<str>::from))
            .collect();
        for key in sources {
            if self.image_cache.contains_key(&key) || self.image_handles.contains_key(&key) {
                // Already resolved, or already attempted (and failed): do not retry.
                continue;
            }
            let source = ResourceSource::file(key.clone());
            let handle = self
                .resource_hub
                .request(ResourceRequest::new(source.clone(), ResourceKind::Image));
            self.image_handles.insert(key.clone(), handle);
            let Some(version) = self.resource_hub.begin_load(handle) else {
                continue;
            };
            match load_and_decode_image(loader, decoder, &source) {
                Ok(image) => {
                    let metadata = ResourceMetadata::new().with_intrinsic_size(Size::new(
                        image.width() as f32,
                        image.height() as f32,
                    ));
                    self.resource_hub.complete(handle, version, metadata);
                    self.image_cache.insert(key, image);
                }
                Err(error) => {
                    self.resource_hub.fail(handle, version, error);
                }
            }
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

    /// Sets the viewport scroll offset `(x, y)` in logical pixels — the
    /// document-coordinate point at the top-left of the viewport (spec §4.1).
    ///
    /// The single scroll container is the viewport. There is no live scroll-gesture
    /// system yet, so this is the test/host-driven input that engages
    /// `position: sticky` clamping on the next frame: at `(0.0, 0.0)` (the default)
    /// every sticky node lays out at its static flow position; a non-zero offset
    /// scrolls the document under the viewport, so a sticky node sticks to the
    /// scrollport edge once scrolled past its inset threshold. Wiring a real scroll
    /// gesture to this setter is a separate, later step.
    pub fn set_viewport_scroll(&mut self, x: f32, y: f32) {
        self.viewport_scroll = (x, y);
    }

    /// Returns the current viewport scroll offset `(x, y)` in logical pixels.
    #[must_use]
    pub fn viewport_scroll(&self) -> (f32, f32) {
        self.viewport_scroll
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

    /// The [`FrameReason`] of the most-urgent pending frame request, if any (test
    /// introspection — lets a test assert which frame a keystroke scheduled).
    #[cfg(test)]
    fn peek_frame_reason(&self) -> Option<FrameReason> {
        self.scheduler.peek().map(|request| request.reason())
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

    /// Renders every component whose output is stale and reconciles it into that
    /// node's children, returning whether any of those sub-reconciles changed the
    /// tree.
    ///
    /// # Only the stale ones
    ///
    /// A component is stale when it has just mounted (nothing has rendered it),
    /// when its node's authored props changed (its typed props may decode
    /// differently), or when it asked to be re-rendered via
    /// [`ComponentContext::request_update`]. Everything else keeps the output it
    /// already has — this is what makes a component's state change cost its own
    /// subtree rather than the document (spec §3.1: "dependency tracking
    /// invalidates only consumers of changed values").
    ///
    /// Each render is enacted as a [`UiOperation::Rerender`] through the same
    /// transaction boundary a root reconcile uses, so a component's output is
    /// committed and versioned exactly like the root's.
    ///
    /// A component's props are decoded from the node's **retained** properties —
    /// the desired tree is already consumed by the time this runs, which is why
    /// the reconciler retains them.
    ///
    /// # Why a fixpoint loop
    ///
    /// A component's output may itself contain component-backed elements, which
    /// only exist (and so only mount) once that output is reconciled. Each pass
    /// therefore renders the components that appeared in the previous one, until a
    /// pass produces no new stale instances. Each component renders **at most
    /// once** per frame: a request raised during its own render is honoured by the
    /// next frame, so a component that always asks for an update cannot spin this
    /// loop. [`MAX_COMPONENT_RENDER_PASSES`] additionally bounds mutual nesting,
    /// so a self-rendering pair truncates the frame rather than hanging it.
    ///
    /// # A component whose props do not decode
    ///
    /// It is not rendered and its existing children are left alone — an authored
    /// mistake must not silently delete a subtree. The failure is counted as
    /// `component.props_errors` rather than swallowed.
    fn render_components(&mut self) -> bool {
        // The common case by far: no host registrations, nothing mounted.
        if self.components.is_empty() {
            return false;
        }
        let mut pending: Vec<NodeId> = self.components.stale_nodes();
        let mut done: HashSet<NodeId> = HashSet::new();
        let mut changed = false;
        let mut errors: u64 = 0;
        let mut rendered: u64 = 0;

        for _ in 0..MAX_COMPONENT_RENDER_PASSES {
            if pending.is_empty() {
                break;
            }
            let mut next = Vec::new();
            for node in std::mem::take(&mut pending) {
                // At most once per frame, however many times it is re-requested.
                if !done.insert(node) {
                    continue;
                }
                // The node may have been removed by an earlier pass in this loop.
                let Some(props) = self.tree.get(node).map(|n| n.props().to_vec()) else {
                    continue;
                };
                let Some(mounted) = self.components.get_mut(node) else {
                    continue;
                };
                let Ok(view) = mounted.render(&props) else {
                    errors += 1;
                    continue;
                };
                rendered += 1;

                let (_commit, plan) = self
                    .enactor
                    .commit_subtree(&self.reconciler, &mut self.tree, node, view.root())
                    .expect("a component rerender is a single, structurally-valid Rerender");
                changed |= !plan.is_empty();
                self.components.sync(&mut self.tree, &plan);
                // Only newly inserted nodes can have minted new instances.
                let fresh: Vec<NodeId> = plan
                    .iter()
                    .filter(|op| op.is_insert())
                    .map(|op| op.id())
                    .filter(|id| self.components.get(*id).is_some())
                    .collect();
                next.extend(fresh);
            }
            pending = next;
        }

        let mut counter = PerformanceCounter::new(DiagnosticSubsystem::Tree, "component.rendered");
        counter.add(rendered);
        self.telemetry.record_counter(&counter);
        if errors > 0 {
            let mut counter =
                PerformanceCounter::new(DiagnosticSubsystem::Tree, "component.props_errors");
            counter.add(errors);
            self.telemetry.record_counter(&counter);
        }
        changed
    }

    /// Registers a component kind so nodes naming its
    /// [`ComponentId`](obscur_identity::ComponentId) instantiate it (spec §2.14).
    ///
    /// Static registration is the default. Once registered, every reconciled node
    /// whose element names this kind mounts an instance — which is rendered with
    /// its typed props decoded from the node's authored properties, and whose
    /// local state lives for as long as the node does.
    ///
    /// Registering nothing keeps the whole component path inert, so a host that
    /// drives its tree from a root view builder alone is unaffected.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError`](obscur_view::component::RegistryError) when a
    /// kind is already registered under the same id.
    pub fn register_component(
        &mut self,
        registration: ComponentRegistration,
    ) -> Result<(), RegistryError> {
        self.components.register(registration)
    }

    /// The number of component instances currently mounted.
    #[must_use]
    pub fn mounted_component_count(&self) -> usize {
        self.components.len()
    }

    /// Installs the runtime-held **view builder** that closes the reactive
    /// re-render loop (spec §3.1), replacing any previously installed builder.
    ///
    /// Once installed, [`run_due_frame_reactive`](Self::run_due_frame_reactive)
    /// calls `builder` each due frame to *rebuild* the desired [`View`] from
    /// current application state — so a command handler that mutates app state (via
    /// a captured `Rc<RefCell<_>>`/`Rc<Cell<_>>`) and requests a frame produces a
    /// freshly-built, reconciled view on the next due frame with **no** view passed
    /// in by the caller. `builder` typically captures the *same* state the
    /// [`on_command`](Self::on_command) handlers mutate.
    ///
    /// This is additive: the explicit
    /// [`run_frame`](Self::run_frame)/[`run_due_frame`](Self::run_due_frame)/[`render_now`](Self::render_now)
    /// paths that take a `&View` are unaffected, and with no builder installed the
    /// reactive path is a no-op.
    pub fn set_root_view(&mut self, builder: impl FnMut() -> View + 'static) {
        self.root_view = Some(Box::new(builder));
    }

    /// Returns whether a reactive view builder has been installed via
    /// [`set_root_view`](Self::set_root_view).
    #[must_use]
    pub fn has_root_view(&self) -> bool {
        self.root_view.is_some()
    }

    /// Installs the stylesheets the reactive frame path
    /// ([`run_due_frame_reactive`](Self::run_due_frame_reactive)) matches against
    /// the rebuilt view, replacing any previously installed set.
    ///
    /// Parallel to [`set_root_view`](Self::set_root_view); the explicit
    /// `&[Stylesheet]`-taking render paths ignore these held sheets.
    pub fn set_stylesheets(&mut self, sheets: Vec<Stylesheet>) {
        self.stylesheets = sheets;
    }

    /// Installs the built-in **user-agent (UA) default** stylesheet — the
    /// toolkit's overridable base component styles.
    ///
    /// Unlike [`set_stylesheets`](Self::set_stylesheets), the default sheet is a
    /// separate slot that is matched on **every** render path (`run_frame`,
    /// `run_due_frame`, `render_now`, and the reactive path), *always first* —
    /// prepended below whatever author sheets that frame matches. Because it
    /// carries the lowest cascade origin
    /// ([`CascadeOrigin::SystemDefault`](obscur_style::cascade::CascadeOrigin::SystemDefault)),
    /// any author/theme declaration of the same or higher specificity overrides it,
    /// exactly like a browser's UA stylesheet (§4.14).
    ///
    /// Passing a sheet at any other origin still works — only the matched *order*
    /// is fixed (default first); precedence is governed entirely by the origins
    /// carried on the rules. Not calling this leaves the runtime with **no** UA
    /// sheet, in which case matching is byte-identical to before (the author sheets
    /// alone).
    pub fn set_default_stylesheet(&mut self, sheet: Stylesheet) {
        self.default_stylesheet = Some(sheet);
    }

    /// Returns whether a UA default stylesheet has been installed via
    /// [`set_default_stylesheet`](Self::set_default_stylesheet).
    #[must_use]
    pub fn has_default_stylesheet(&self) -> bool {
        self.default_stylesheet.is_some()
    }

    /// Runs **one** reactive frame if the scheduler has a request pending: it calls
    /// the runtime-held [`root_view`](Self::set_root_view) builder to REBUILD the
    /// desired [`View`] from current application state, then runs the full
    /// deterministic core (reconcile → style → layout → compose) over it with the
    /// held [`stylesheets`](Self::set_stylesheets). Returns the [`FrameResult`], or
    /// [`None`] when no builder is installed or nothing is due.
    ///
    /// This is the keystone that closes the event loop: a pointer click runs a
    /// registered command handler, the handler mutates app state and requests a
    /// frame, and *this* method — on the next due frame — rebuilds the view from
    /// that mutated state and reconciles it into the retained tree, so the paint
    /// reflects the new state. Unlike [`run_due_frame`](Self::run_due_frame), the
    /// caller passes no view: the builder is the single source of truth.
    ///
    /// A reactive frame is always classified as a **structural**
    /// [`FrameReason::UiCommit`] regardless of which coalesced request the
    /// scheduler drained, because the builder may have produced a *different* tree
    /// from the mutated state and that change must be reconciled — an `Input`
    /// (hover/focus) or `Animation` reason would skip the reconcile phase and never
    /// pick up the rebuilt view. Reconcile is an idempotent no-op when the rebuilt
    /// view is structurally unchanged, so a hover-only reactive frame still costs
    /// only the (cheap) diff.
    ///
    /// Borrow handling: the builder is a `&mut` field and the render is `&mut
    /// self`, so the builder and the held stylesheets are *taken out* for the
    /// duration of the render and restored afterward, keeping the `FnMut` call and
    /// the `run_frame` borrow disjoint.
    pub fn run_due_frame_reactive(&mut self) -> Option<FrameResult> {
        // No builder installed: the reactive path is a no-op and leaves any
        // pending request untouched (a `&View`-taking caller can still drain it).
        self.root_view.as_ref()?;
        // Starvation prevention (§3.6), identical to `run_due_frame`.
        if self.scheduler.background_starved() {
            self.service_background_unit();
        }
        // Nothing due: no side effects, no rebuild.
        self.scheduler.next_frame()?;

        // Take the builder and the held stylesheets out so the `FnMut` call and
        // the `&mut self` render below do not overlap the `&mut`/`&` field borrows;
        // both are restored before returning. `take()` yields `Some` here (the
        // builder was just observed present, and nothing above removes it).
        let mut builder = self.root_view.take()?;
        let view = builder();
        let sheets = std::mem::take(&mut self.stylesheets);

        // A reactive frame rebuilt the view from current state, so it is a UI
        // transaction commit: classify it structurally so the rebuilt view is
        // reconciled into the tree (see the doc comment).
        let ctx = FrameContext::new(self.next_frame, FrameReason::UiCommit, self.viewport);
        let result = self.run_frame(ctx, &view, &sheets);

        self.stylesheets = sheets;
        self.root_view = Some(builder);
        Some(result)
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
            let (commit, reconcile_plan) = self
                .enactor
                .commit_root(&self.reconciler, &mut self.tree, desired)
                .expect("a root reconcile is a single, structurally-valid ReplaceRoot");
            let changed = !commit.is_empty();
            // 1b. Mount an instance for every inserted component-backed node and
            //     unmount every removed one, from the plan's structural deltas.
            //     A host that registered no component kinds pays nothing here.
            self.components.sync(&mut self.tree, &reconcile_plan);
            // 1c. Render every mounted component and reconcile its output into
            //     its own children, so a component-backed node's subtree exists
            //     before style/layout read the tree.
            let rendered = self.render_components();
            let changed = changed || rendered;
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

        // 2-3. Match + style: rebuild cascade inputs and resolve computed styles.
        //    When the style phase is skipped, reuse the retained computed styles
        //    so a downstream layout/compose still has values to work from.
        let (computed, restyled) = if plan.style() {
            // Match authored sheets against the tree, honouring live interaction
            // state so `:hover`/`:focus` participate and gating `@media`/
            // `@container` rules by this frame's viewport (§4.13).
            let environment =
                MediaEnvironment::viewport(ctx.viewport().width, ctx.viewport().height);
            // Prepend the UA default sheet (if installed) so it is matched FIRST —
            // below every author sheet. It carries the lowest cascade origin
            // (`SystemDefault`), so any author declaration overrides it (§4.14).
            // When absent, the author slice is passed through untouched, keeping
            // matching byte-identical to before.
            let matched_sheets: Vec<Stylesheet>;
            let sheets: &[Stylesheet] = match &self.default_stylesheet {
                Some(default) => {
                    matched_sheets = std::iter::once(default.clone())
                        .chain(sheets.iter().cloned())
                        .collect();
                    &matched_sheets
                }
                None => sheets,
            };
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
        //
        //    The plan's flag is a conservative upper bound derived from the frame
        //    REASON; now that the reconcile result and the real computed styles
        //    exist, refine it with what ACTUALLY changed (spec §3.4 phase 5, §4.3)
        //    — a hover that only shifts a fill reuses every retained box, even on
        //    the reactive path where every frame is classified structural.
        let layout_needed = self.layout_is_needed(plan, &ctx, first_frame, reconciled, &computed);
        let relaid_out = if layout_needed {
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
            // Thread the runtime-owned editing session into the measure pass so the
            // focused editable's text is SIZED from the live session buffer (the
            // SAME string, RAW, that compose paints), not the retained tree value
            // that lags by a keystroke — so the arranged box height (line count)
            // equals the painted line count on a `white-space: normal`/`pre-wrap`
            // field (CSS Text 3 §4.1.1). `None` when no session is active ⇒
            // byte-identical to the retained-tree measure. `editing`, `layout`,
            // `tree` and `scene_composer` are disjoint fields, so this is a legal
            // split borrow.
            let editing = self.editing_view();
            let boxes = self.layout.layout_editing(
                &self.tree,
                &layout_styles,
                &computed,
                &grid_templates,
                ctx.viewport(),
                // The viewport scroll offset (test/host-driven; `(0, 0)` by
                // default) engages `position: sticky` clamping this frame.
                self.viewport_scroll,
                self.scene_composer.paragraph(),
                self.scene_composer.hyphenator(),
                editing,
            );
            let relaid_out = !boxes.is_empty();
            self.boxes = boxes;
            // Remember the non-style inputs these boxes were arranged under, so a
            // later frame can only reuse them while the viewport and the sticky
            // scroll offset are unchanged.
            self.last_layout_inputs = Some(self.current_layout_inputs(&ctx));
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
            // Resolve any `background-image: url(...)` referenced by this frame's
            // computed styles into decoded images (load → decode → cache) BEFORE
            // composing, so the coordinator can paint the ready ones. A no-op when
            // no loader/decoder is installed or no URL is authored.
            self.resolve_background_images(&computed);
            // Resolve each element's own image `src` (an `<img>` primitive's
            // source) into the SAME cache via the SAME load/decode path, so the
            // coordinator can paint the ready ones over their box as element
            // content. A no-op when no loader/decoder is installed or no `src` is
            // authored.
            self.resolve_element_images();
            // Thread the runtime-owned editing session into compose so the focused
            // editable paints its live session text plus a caret + selection
            // highlight (spec §5.11) — the SAME [`EditingView`] the measure pass
            // sized the box with, so paint and box agree (CSS Text 3 §4.1.1).
            // `scene_composer`, `tree`, `boxes` and `editing` are disjoint fields, so
            // this shared borrow is legal; with no session the view is `None` and
            // compose is byte-identical.
            let editing = self.editing_view();
            // Resolve each scroll container's live scroll offset + content height
            // (CSS Overflow 3 §scroll-container) from the retained boxes, so the
            // coordinator paints the scroll offset + scrollbar. Empty when no
            // container overflows → the compose is byte-identical to before scroll
            // containers existed.
            let scroll = self.scroll_paint(&computed);
            let display_list = self.scene_composer.compose_with_images_scrolled(
                &self.tree,
                &computed,
                &self.boxes,
                editing,
                &self.image_cache,
                &scroll,
            );
            // Inspect overlay (spec §7.6): when inspect mode is on, append a
            // highlight over the hovered node's border box on top of the scene.
            // A no-op (byte-identical) when inspect mode is off or nothing is
            // hovered. Appended before the device-scale root so the highlight is
            // in the same logical space as the layout boxes it traces.
            let display_list = self.apply_inspect_overlay(display_list);
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
            // Bracket the whole scene in a root device-scale transform so layout
            // (in logical px) rasterizes at physical resolution. At scale `1.0`
            // this is a no-op and the list is byte-identical (see `scaled_root`).
            self.scene.set_root(self.scaled_root(display_list));
        }
        let snapshot = self.scene.snapshot();

        // A component that called `request_update` during its own render is still
        // stale: the request is deliberately not serviced inside the frame that
        // raised it (that would let a component spin the frame forever), so ask
        // the scheduler for the next one. Coalescing is the scheduler's job, so
        // many requesting components still cost one frame.
        if self.components.has_stale() {
            self.scheduler.request_reason(FrameReason::UiCommit);
        }

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

    /// Returns the [`ComputedStyle`] resolved for `node` on the last produced
    /// frame, or [`None`] when the node was never styled (absent from the tree, or
    /// a frame with a skipped style phase never resolved it).
    ///
    /// This exposes the post-cascade computed value a host (or test) can inspect —
    /// e.g. to observe that an author sheet's declaration overrode the UA default
    /// sheet's for the same property (§4.14).
    #[must_use]
    pub fn computed_style(&self, node: NodeId) -> Option<&ComputedStyle> {
        self.computed.get(&node)
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
    pub fn handle_pointer(&mut self, mut event: PointerEvent) -> PointerDispatch {
        // 0. Device-scale + content scroll: incoming coordinates are in the host's
        //    physical (device) space, but layout runs in logical CSS px scrolled by
        //    `content_offset`. Map the pointer to content space once, here, as
        //    `physical / device_scale + offset` — the inverse of the root render
        //    transform `scale(s) · translate(-ox, -oy)` — so every downstream
        //    consumer (the hit-test below, the caret seat in `begin_text_pointer`,
        //    and the drag re-select in `update_text_drag`) sees content-space
        //    coordinates. At scale `1.0` and offset `(0, 0)` this is the identity,
        //    so the mapping is byte-identical for existing hosts.
        let s = self.device_scale;
        let (ox, oy) = self.content_offset;
        if (s - 1.0).abs() > f32::EPSILON || ox != 0.0 || oy != 0.0 {
            let physical = event.position();
            event.position = Point::new(physical.x / s + ox, physical.y / s + oy);
        }

        // 0b. Per-element scroll: the spatial index is keyed by each node's
        //     UNSCROLLED layout box, but a scroll container's descendants render
        //     translated by `−scroll_top`. Add back each crossed container's
        //     `scroll_top` so a click while scrolled hit-tests the scrolled content
        //     (CSS Overflow 3 §scroll-container). With no scroll container this is
        //     the identity, so the hit-test is byte-identical for existing hosts.
        let containers = scroll_containers(&self.tree, &self.computed, &self.boxes);
        if !containers.is_empty() {
            let (mapped, _) = self.apply_scroll_to_point(event.position(), &containers);
            event.position = mapped;
        }

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

        // 3. Focus: only a press moves focus. A press also anchors click
        //    detection on the pressed node.
        let mut focus_changed = false;
        let mut press_changed = false;
        if event.kind() == PointerEventKind::Down {
            // A disabled control is not a focus / press / editing target: a press
            // that lands inside one is inert (it seats no focus, no caret, and no
            // press anchor), so typing can't reach it and its release activates
            // nothing. Focus/editing simply stay wherever they were.
            let interactive_target = target.filter(|&t| !self.is_within_disabled(t));
            if self.focused != interactive_target {
                self.focused = interactive_target;
                focus_changed = true;
                // Focus moved: begin/clear the runtime-owned editing session so
                // the focused editable's text/caret/selection is live for keys
                // (and step-2 rendering).
                self.sync_editing_to_focus();
            }
            if self.press_target != interactive_target {
                press_changed = true;
            }
            self.press_target = interactive_target;
            // Resolve the click count (single / double / triple) from this press's
            // relationship to the previous one, so a double click can select the
            // word and a triple click the line under the pointer (spec §5.11).
            self.click_count =
                self.next_click_count(interactive_target, event.position(), event.timestamp());
            // Click → caret / word / line: after `sync_editing_to_focus` has
            // (re)seated the session for the clicked editable, a single click places
            // the caret and arms a potential drag-selection; a double click selects
            // the word and a triple click the line under the point. A press on a
            // non-editable (or disabled) target clears any armed drag and leaves
            // the session untouched.
            self.begin_text_pointer(interactive_target, event.position());
        }

        // 3a-drag. While a drag is armed, a pointer move re-selects the range from
        //    the press anchor to the current pointer offset over the same field —
        //    live drag-selection (spec §5.11). Not a press/release, so it never
        //    perturbs focus or click detection.
        if event.kind() == PointerEventKind::Move {
            self.update_text_drag(event.position());
        }

        // 3b. Click: a release on the *same* node the press landed on is a click
        //    on that node — it **activates** that control. Activation dispatches
        //    the node's `on-click` command if one is bound (bubbling up its
        //    logical-element ancestors, first match wins), else its `on-change`
        //    command — so a checkbox/input bound to `on-change` fires on click
        //    just as a button bound to `on-click` does. The resolved command is
        //    reported back; the press anchor is cleared on every release.
        let mut command = None;
        if event.kind() == PointerEventKind::Up {
            if let Some(pressed) = self.press_target
                && Some(pressed) == target
            {
                command = self.activation_command_for(pressed);
                self.activate(pressed);
            }
            if self.press_target.is_some() {
                press_changed = true;
            }
            self.press_target = None;
            // A release ends any in-flight drag-selection: the selection stays
            // wherever the last move left it.
            self.text_drag = None;
        }

        // 4. Recompute the interaction pseudo-states from the new hover/focus.
        if hover_changed || focus_changed || press_changed {
            self.recompute_pseudo_states();
            // 5. Schedule a restyle for the next frame.
            self.request_frame(FrameReason::Input);
        }

        PointerDispatch::new(target, hover_changed, focus_changed, command)
    }

    /// Resolves `event`'s command for a `target` by walking its logical-element
    /// ancestor chain (target first, then toward the root) and returning the
    /// first binding for that `event` found — so an event on a child of a bound
    /// element still fires that element's command (event bubbling, spec §2.3).
    /// Returns `None` when no node in the chain binds `event`.
    ///
    /// Reuses [`hover_chain_for`](Self::hover_chain_for) for the ancestor walk,
    /// which is the same target-plus-ancestors sequence hover already computes.
    fn command_for_event_bubbling(&self, target: NodeId, event: InternedSymbol) -> Option<InternedSymbol> {
        self.hover_chain_for(Some(target))
            .into_iter()
            .find_map(|node| self.tree.get(node).and_then(|n| n.command_for_event(event)))
    }

    /// Whether `node` lies within a **disabled** control — the node itself or any
    /// of its element ancestors carries the reserved `:disabled` structural state.
    ///
    /// A disabled control and its whole subtree are inert to interaction: a press
    /// seats no focus and no editing caret, typing never reaches it, and a click /
    /// Enter / Space fires no activation command — matching a native disabled form
    /// control. `:hover` is deliberately unaffected (CSS still hovers a disabled
    /// element), so only the *behavioural* paths gate on this, not styling.
    fn is_within_disabled(&self, node: NodeId) -> bool {
        self.hover_chain_for(Some(node))
            .into_iter()
            .any(|n| self.tree.get(n).is_some_and(|el| el.attributes().is_disabled()))
    }

    /// Whether `node` lies within a **readonly** control — the node itself or any
    /// of its element ancestors carries the reserved `readonly` structural state
    /// (WHATWG HTML §attr-input-readonly).
    ///
    /// **Distinct from [`is_within_disabled`](Self::is_within_disabled).** A
    /// readonly control is still *focusable* and its text still *selectable /
    /// copyable* — a press seats focus, a caret, and drag-selection exactly as on a
    /// normal field. Only its **value is immutable**: the text-mutating edits
    /// (insert / delete / backspace, IME commit, paste, a multiline `Enter`
    /// newline) are dropped. So — unlike the disabled gate, which blocks
    /// focus/caret/editing wholesale — this gates **only** the mutation path
    /// ([`apply_edit`](Self::apply_edit) mutating commands and
    /// [`handle_ime`](Self::handle_ime)), never focus / caret / selection.
    fn is_within_readonly(&self, node: NodeId) -> bool {
        self.hover_chain_for(Some(node))
            .into_iter()
            .any(|n| self.tree.get(n).is_some_and(|el| el.attributes().is_readonly()))
    }

    /// Resolves the **activation** command for `target`: a control is activated
    /// by a click or an Enter/Space keypress, and activation fires its
    /// [`EVENT_CLICK`] command if one is bound (bubbling up its ancestors), else —
    /// for a **non-editable** control — its [`EVENT_CHANGE`] command (spec §2.3,
    /// §3.1).
    ///
    /// This is the single mapping both pointer and keyboard activation share: a
    /// button (bound to `on-click`) fires its click command; a
    /// checkbox/radio/select (bound to `on-change`, and *not* editable) fires its
    /// change command *on the click* — for those controls the click **is** the
    /// change (WHATWG HTML §4.10.5.5: a checkbox/radio `change` fires on the
    /// toggling click; a `select`'s on the pick). Click binds are checked first,
    /// so a node carrying both prefers its click command.
    ///
    /// A **text field is different.** For an *editable* control — one whose
    /// ancestor chain binds [`EVENT_INPUT`] — `EVENT_CHANGE` is **not** a
    /// pointer-click activation: per the HTML model a text field's `change` fires
    /// once on **commit** (blur, or `Enter` for a single-line `input`) carrying the
    /// committed value, and must *not* fire on a focus/click. So the change
    /// fallback is deliberately skipped for an editable target here — a click into
    /// a text field resolves no activation command — and the commit-change is fired
    /// separately (see [`commit_editable_change`](Self::commit_editable_change),
    /// driven by [`sync_editing_to_focus`](Self::sync_editing_to_focus) on blur and
    /// the single-line `Enter` path). Per-keystroke [`EVENT_INPUT`] entry is
    /// likewise not activation. Returns `None` when neither event applies.
    ///
    /// [`EVENT_INPUT`]: obscur_view::view::EVENT_INPUT
    fn activation_command_for(&self, target: NodeId) -> Option<InternedSymbol> {
        if let Some(click) = self.command_for_event_bubbling(target, EVENT_CLICK) {
            return Some(click);
        }
        // `EVENT_CHANGE`-on-click is a toggle/select activation only; for an
        // editable field `change` is a commit event, so never resolve it here.
        if self.editable_ancestor(target).is_some() {
            return None;
        }
        self.command_for_event_bubbling(target, EVENT_CHANGE)
    }

    /// Activates the control at `node`: dispatches its
    /// [`activation_command_for`](Self::activation_command_for) command
    /// (`EVENT_CLICK` else `EVENT_CHANGE`), running the registered handler.
    /// Returns whether a handler actually ran.
    ///
    /// This is the shared entry point for both a pointer click (in
    /// [`handle_pointer`](Self::handle_pointer)) and an Enter/Space keypress on
    /// the focused control (in [`handle_key`](Self::handle_key)). With no
    /// activation binding in the node's ancestor chain — or no handler
    /// registered for the resolved command — it is a harmless no-op that returns
    /// `false` and never panics.
    fn activate(&mut self, node: NodeId) -> bool {
        // A disabled control is inert: it fires no activation command, whether the
        // activation came from a pointer click or an Enter/Space keypress.
        if self.is_within_disabled(node) {
            return false;
        }
        match self.activation_command_for(node) {
            Some(command) => self.dispatch_command(command),
            None => false,
        }
    }

    /// Registers `handler` as the application logic to run when `command` is
    /// dispatched, replacing any handler previously registered for that command
    /// (spec §2.3, §3.1 — a resolved command runs a handler).
    ///
    /// The `command` symbol is app-owned and must be the **same** symbol the
    /// application binds its `on-click` event to in the View (via
    /// [`ViewEventBinding`](obscur_view::view::ViewEventBinding)); the runtime
    /// only compares symbols, so no interner is shared. The handler is a
    /// `FnMut()` that typically captures mutable app state through an
    /// `Rc<RefCell<_>>`/`Rc<Cell<_>>`. Registering does not itself schedule a
    /// frame.
    pub fn on_command(&mut self, command: InternedSymbol, handler: impl FnMut() + 'static) {
        self.handlers.insert(command, Box::new(handler));
    }

    /// Returns whether a handler is registered for `command`.
    #[must_use]
    pub fn has_command_handler(&self, command: InternedSymbol) -> bool {
        self.handlers.contains_key(&command)
    }

    /// Runs the handler registered for `command` (if any) and, when one ran,
    /// requests a [`FrameReason::Input`] frame — a handler likely mutated app
    /// state, so the next `run_due_frame`/`render_now` rebuilds from it. Returns
    /// whether a handler actually ran.
    ///
    /// With no handler registered this is a no-op that returns `false` and
    /// schedules no frame, so an unbound command (or one whose handler the app
    /// never registered) can never panic or perturb runtime state.
    fn dispatch_command(&mut self, command: InternedSymbol) -> bool {
        let ran = if let Some(handler) = self.handlers.get_mut(&command) {
            handler();
            true
        } else {
            false
        };
        if ran {
            self.request_frame(FrameReason::Input);
        }
        ran
    }

    /// Registers `handler` as the application logic to run when the focused
    /// editable bound to `command` produces a text edit (spec §6.7), replacing
    /// any handler previously registered for that command.
    ///
    /// The `command` symbol is app-owned and must be the **same** symbol the
    /// application binds its `on-input` event to in the View (via
    /// [`EVENT_INPUT`](obscur_view::view::EVENT_INPUT) — an
    /// [`Input`](../../obscur_widgets/input/struct.Input.html) does this). The
    /// runtime now *owns* the focused field's editing session (caret, selection
    /// and undo; spec §5.11): each mutating keystroke is applied to that session's
    /// buffer and `handler` is called with the whole **resulting text**
    /// (`FnMut(&str)`), which the app mirrors into its *own* state (typically an
    /// `Rc<RefCell<String>>` the reactive view builder reads back). Caret-only
    /// motions (arrows/Home/End/select) do not call the handler. Registering
    /// does not itself schedule a frame.
    pub fn on_text_input(&mut self, command: InternedSymbol, handler: impl FnMut(&str) + 'static) {
        self.text_input_handlers.insert(command, Box::new(handler));
    }

    /// Returns whether a text-input handler is registered for `command`.
    #[must_use]
    pub fn has_text_input_handler(&self, command: InternedSymbol) -> bool {
        self.text_input_handlers.contains_key(&command)
    }

    /// Runs the text-input handler registered for `command` (if any) with the
    /// field's resulting `text` and, when one ran, requests a
    /// [`FrameReason::Input`] frame so the app's mirrored text reaches paint on
    /// the next reactive frame. Returns whether a handler actually ran.
    ///
    /// With no handler registered this is a no-op that returns `false` and
    /// schedules no frame — a bound `on-input` whose handler the app never
    /// registered can never panic or perturb runtime state.
    fn dispatch_text_input(&mut self, command: InternedSymbol, text: &str) -> bool {
        let ran = if let Some(handler) = self.text_input_handlers.get_mut(&command) {
            handler(text);
            true
        } else {
            false
        };
        if ran {
            self.request_frame(FrameReason::Input);
        }
        ran
    }

    /// Registers `handler` as the application logic to run when the focused
    /// editable bound to `command` **commits** its value — the toolkit's `change`
    /// event (WHATWG HTML §4.10.5.5 *Common input element APIs*: `change` fires
    /// when the user commits an edit, i.e. on blur or, for a single-line `input`,
    /// on `Enter`), replacing any handler previously registered for that command.
    ///
    /// The `command` symbol is app-owned and must be the **same** symbol the
    /// application binds its `on-change` event to in the View (via
    /// [`EVENT_CHANGE`](obscur_view::view::EVENT_CHANGE) — an
    /// [`Input`](../../obscur_widgets/input/struct.Input.html) binds it). Unlike a
    /// per-keystroke [`on_text_input`](Self::on_text_input) handler, a change
    /// handler runs **once per edit session** — only when the value actually
    /// changed since focus — and receives the field's **committed text**
    /// (`FnMut(&str)`). Because it is a distinct registry from `on_text_input`, an
    /// app can register both on the same field command and receive per-keystroke
    /// `input` *and* a single commit `change` without conflation. Registering does
    /// not itself schedule a frame.
    ///
    /// (A non-editable checkbox/radio/select fires its `EVENT_CHANGE` command
    /// through the *activation* path on click — see
    /// [`activation_command_for`](Self::activation_command_for) — so those keep
    /// using [`on_command`](Self::on_command); this registry is for editable
    /// text-field commits.)
    pub fn on_change(&mut self, command: InternedSymbol, handler: impl FnMut(&str) + 'static) {
        self.change_handlers.insert(command, Box::new(handler));
    }

    /// Returns whether a change (commit) handler is registered for `command`.
    #[must_use]
    pub fn has_change_handler(&self, command: InternedSymbol) -> bool {
        self.change_handlers.contains_key(&command)
    }

    /// Runs the change handler registered for `command` (if any) with the field's
    /// committed `text` and, when one ran, requests a [`FrameReason::Input`] frame.
    /// Returns whether a handler actually ran. With no handler registered this is a
    /// no-op that returns `false` and schedules no frame.
    fn dispatch_change(&mut self, command: InternedSymbol, text: &str) -> bool {
        let ran = if let Some(handler) = self.change_handlers.get_mut(&command) {
            handler(text);
            true
        } else {
            false
        };
        if ran {
            self.request_frame(FrameReason::Input);
        }
        ran
    }

    /// Commits the active editing session's `change` event when its value is
    /// dirty (differs from the last committed baseline / the value at focus) — the
    /// shared entry point for the two HTML commit triggers on an editable text
    /// field: **blur** (focus leaving the field, from
    /// [`sync_editing_to_focus`](Self::sync_editing_to_focus)) and a single-line
    /// **`Enter`** (from [`try_text_edit`](Self::try_text_edit)).
    ///
    /// Fires the editable node's [`EVENT_CHANGE`](obscur_view::view::EVENT_CHANGE)
    /// command through the [`change_handlers`](Self::change_handlers) registry with
    /// the committed text (WHATWG HTML §4.10.5.5: `change` fires on commit, once,
    /// and only when the value actually changed). When `rebaseline` is set (an
    /// `Enter` commit that keeps the field focused) the session's baseline is
    /// advanced to the committed value first, so a later blur without further edits
    /// does not re-fire. A no session, a clean value, or no `EVENT_CHANGE` binding
    /// makes this a harmless no-op. Returns whether a change handler ran.
    fn commit_editable_change(&mut self, rebaseline: bool) -> bool {
        let Some(session) = self.editing.as_ref() else {
            return false;
        };
        if !session.is_dirty() {
            return false;
        }
        let node = session.node();
        // A **readonly** field's value can never change (WHATWG HTML
        // §attr-input-readonly), so its `change` never fires. The session can never
        // become dirty on a readonly field (every mutation is dropped upstream), so
        // this is normally unreachable — but guard explicitly so a commit is a
        // hard no-op regardless of how the session got here.
        if self.is_within_readonly(node) {
            return false;
        }
        let text = session.text().to_owned();
        if rebaseline {
            if let Some(session) = self.editing.as_mut() {
                session.mark_committed();
            }
        }
        match self.tree.get(node).and_then(|n| n.command_for_event(EVENT_CHANGE)) {
            Some(command) => self.dispatch_change(command, &text),
            None => false,
        }
    }

    /// Begins a fresh editing session on `node` seeded with `initial_text`
    /// (caret at the end), replacing any existing session — the explicit,
    /// app-driven way to seat the focused field's editable state (spec §5.11).
    ///
    /// The runtime otherwise seeds a session automatically when focus lands on an
    /// editable (see [`sync_editing_to_focus`](Self::sync_editing_to_focus)); an
    /// app whose field carries its text as a *value property* rather than a text
    /// node (e.g. [`Input`](../../obscur_widgets/input/struct.Input.html)) calls
    /// this to seed the buffer with the real value.
    pub fn begin_text_editing(&mut self, node: NodeId, initial_text: impl Into<String>) {
        // A disabled control accepts no editing session — a defensive guard mirroring
        // the pointer path, so even an explicit app call can't make a disabled field
        // editable.
        if self.is_within_disabled(node) {
            return;
        }
        self.editing = Some(TextEditSession::seeded(node, initial_text));
    }

    /// Re-seeds the active editing session's text (caret at the end, history
    /// cleared). A no-op when no session is active.
    pub fn set_editing_text(&mut self, text: impl Into<String>) {
        if let Some(session) = self.editing.as_mut() {
            session.reseed(text);
        }
    }

    /// Returns the focused field's current editing text, if a session is active.
    #[must_use]
    pub fn editing_text(&self) -> Option<&str> {
        self.editing.as_ref().map(TextEditSession::text)
    }

    /// Returns the focused field's current caret/selection, if a session is
    /// active — the input to step-2 caret + selection rendering.
    #[must_use]
    pub fn editing_selection(&self) -> Option<TextSelection> {
        self.editing.as_ref().map(TextEditSession::selection)
    }

    /// Applies an IME composition `event` against the focused editable's editing
    /// session (spec §5.12 IME), returning whether it was applied.
    ///
    /// A composition lets the user compose text (a *preedit*, shown inline but
    /// **not** inserted into the buffer) then commit it — the standard path for
    /// CJK and other complex input. The event maps 1:1 onto the session's
    /// composition lifecycle:
    ///
    /// * [`Start`](CompositionEvent::Start) anchors a fresh composition at the
    ///   current caret/selection; the preedit begins empty.
    /// * [`Update`](CompositionEvent::Update) stores a new preedit on the
    ///   composition. The preedit is shown (spliced for display, underlined) but
    ///   is **never** written into the buffer, so [`editing_text`](Self::editing_text)
    ///   is unchanged while composing (spec §5.12 "preedit is separate from
    ///   committed text").
    /// * [`Commit`](CompositionEvent::Commit) replaces the composition's anchor
    ///   span in the buffer with the committed text as a **single** edit (one
    ///   composition = one undo unit) and fires the field's
    ///   [`EVENT_INPUT`](obscur_view::view::EVENT_INPUT) handler with the resulting
    ///   text. Because the preedit was never inserted, an empty anchor makes this
    ///   an insert at the caret.
    /// * [`Cancel`](CompositionEvent::Cancel) drops the preedit with no buffer
    ///   change.
    ///
    /// Returns `false` (a no-op) when no editing session is active, or on an
    /// out-of-order event (an `Update`/`Cancel`/`Commit` with no live composition).
    /// Every applied event requests a [`FrameReason::Input`] frame so the preedit
    /// (or the commit) reaches paint.
    pub fn handle_ime(&mut self, event: CompositionEvent) -> bool {
        // The follow-up the runtime owes after mutating the session — resolved
        // while the `session` borrow is live, then acted on below so the
        // `&mut self` calls (`request_frame`/`apply_edit`) do not overlap it.
        enum ImeFollowUp {
            /// Applied a preedit/state change: just request a paint frame.
            Frame,
            /// Committed: replace the anchor span with the text as one edit.
            Commit(obscur_text::span::TextRange, Box<str>),
            /// Out-of-order or no-op: nothing applied.
            None,
        }

        // No focused editable → nothing to compose against.
        let Some(node) = self.editing.as_ref().map(TextEditSession::node) else {
            return false;
        };
        // A **readonly** field cannot be edited (WHATWG HTML §attr-input-readonly):
        // IME composition — whose whole purpose is to compose text for insertion —
        // is inert on it. Drop the event before any preedit is anchored so nothing
        // is ever committed into the buffer. (Focus / caret / selection are
        // unaffected; only this mutation entry gates on readonly.)
        if self.is_within_readonly(node) {
            return false;
        }
        // Re-borrow mutably to drive the composition (the session still exists — we
        // hold `&mut self` throughout — so this never fails; no panic path).
        let Some(session) = self.editing.as_mut() else {
            return false;
        };
        let follow_up = match event {
            CompositionEvent::Start => {
                session.start_composition();
                ImeFollowUp::Frame
            }
            CompositionEvent::Update(preedit) => {
                if session.is_composing() {
                    session.update_composition(preedit);
                    ImeFollowUp::Frame
                } else {
                    ImeFollowUp::None
                }
            }
            CompositionEvent::Commit(text) => match session.commit_composition() {
                Some(anchor) => ImeFollowUp::Commit(anchor, text),
                None => ImeFollowUp::None,
            },
            CompositionEvent::Cancel => {
                if session.is_composing() {
                    session.cancel_composition();
                    ImeFollowUp::Frame
                } else {
                    ImeFollowUp::None
                }
            }
        };
        match follow_up {
            ImeFollowUp::Frame => {
                self.request_frame(FrameReason::Input);
                true
            }
            ImeFollowUp::Commit(anchor, text) => {
                // `apply_edit` runs `after_edit`: it fires the field's `on-input`
                // with the resulting text and requests an input frame. The preedit
                // was never inserted, so an empty anchor makes this an insert.
                self.apply_edit(EditCommand::Replace { range: anchor, text });
                true
            }
            ImeFollowUp::None => false,
        }
    }

    /// Returns the active IME preedit text, if a composition is live (spec §5.12).
    ///
    /// This is *uncommitted* composition text shown inline (spliced for display,
    /// underlined) but deliberately kept OUT of the buffer — so it is distinct
    /// from [`editing_text`](Self::editing_text), which returns only committed
    /// content. `None` when no composition is active.
    #[must_use]
    pub fn composition_preedit(&self) -> Option<&str> {
        self.editing
            .as_ref()
            .and_then(TextEditSession::composition)
            .filter(|composition| composition.is_active())
            .map(|composition| composition.preedit().text())
    }

    /// Resolves the nearest editable ancestor of `target` (target first, toward
    /// the root): the first node whose own bindings carry
    /// [`EVENT_INPUT`](obscur_view::view::EVENT_INPUT). Returns `None` when no
    /// node in the chain is editable.
    fn editable_ancestor(&self, target: NodeId) -> Option<NodeId> {
        self.hover_chain_for(Some(target)).into_iter().find(|&node| {
            self.tree
                .get(node)
                .and_then(|n| n.command_for_event(EVENT_INPUT))
                .is_some()
        })
    }

    /// Whether the editable `node` is a **multi-line** field (a `<textarea>`): it
    /// carries the reserved CSS type `textarea` (emitted by a multi-line
    /// [`Input`](obscur_widgets::input::Input)). A multi-line editable inserts a
    /// newline on `Enter` and moves the caret vertically on ArrowUp/ArrowDown; a
    /// single-line `input` does neither (its Enter stays activation/submit).
    fn is_multiline(&self, node: NodeId) -> bool {
        self.tree
            .get(node)
            .and_then(|n| n.attributes().local_name())
            == Some("textarea")
    }

    /// Concatenates the text content of the editable subtree rooted at `node`
    /// (the node plus its descendants, in tree order) — the seed for a fresh
    /// editing session. A single-line text field has one `Text` child, so this
    /// is that child's content.
    fn collect_editable_text(&self, node: NodeId) -> String {
        let mut out = String::new();
        self.collect_text_into(node, &mut out);
        out
    }

    /// Appends the text content of `node` and its descendants into `out`
    /// (depth-first, in child order), bounded by the tree so a malformed subtree
    /// terminates.
    ///
    /// The `.placeholder` overlay is **skipped** in its entirety: it is a
    /// non-editable rendering layer (the CSS `::placeholder` pseudo-element, shown
    /// while `:placeholder-shown` holds), *not* the field's value, so it must never
    /// be seeded into the editing buffer. An empty placeholdered field therefore
    /// collects no text — it focuses to an empty buffer, caret at 0.
    fn collect_text_into(&self, node: NodeId, out: &mut String) {
        let Some(n) = self.tree.get(node) else {
            return;
        };
        if n.attributes().has_class("placeholder") {
            return;
        }
        if let Some(text) = n.kind().text() {
            out.push_str(text);
        }
        for &child in n.children() {
            self.collect_text_into(child, out);
        }
    }

    /// Reconciles the editing session with the current focus: begins a fresh
    /// session (seeded from the field's text, caret at the end) when focus is on
    /// an editable the session does not already cover, and clears the session
    /// when focus is not on any editable. Called whenever focus changes.
    ///
    /// **Blur commits `change`.** When focus *leaves* the field the current
    /// session covers (focus moving to a different editable, a non-editable, or
    /// nothing), that is a **commit** in the HTML sense (WHATWG HTML §4.10.5.5: a
    /// text field's `change` fires when it loses focus after its value changed).
    /// So before re-seating/clearing the session this dispatches the outgoing
    /// field's [`EVENT_CHANGE`](obscur_view::view::EVENT_CHANGE) command with the
    /// committed text — once, and only if the value actually changed since focus
    /// (see [`commit_editable_change`](Self::commit_editable_change)). Re-focusing
    /// the *same* field is not a blur, so the session (and its baseline) persists
    /// untouched.
    fn sync_editing_to_focus(&mut self) {
        let new_editable = self.focused.and_then(|f| self.editable_ancestor(f));
        let current = self.editing.as_ref().map(TextEditSession::node);
        // Same field (or both absent): no focus transition across an editable —
        // leave the live session (caret/selection/baseline) exactly as it was.
        if current == new_editable {
            return;
        }
        // Focus is leaving `current` (an editable): blur → commit its `change`
        // with the committed value if it changed since focus.
        if current.is_some() {
            self.commit_editable_change(false);
        }
        match new_editable {
            Some(node) => {
                let text = self.collect_editable_text(node);
                self.editing = Some(TextEditSession::seeded(node, text));
            }
            None => self.editing = None,
        }
    }

    /// Reconciles the focused editing session with an **external** rewrite of the
    /// field's retained value while it stays focused — the controlled-input reseed
    /// (WHATWG HTML §4.10.5.5: a text control reflects its `value`, so when the app
    /// rewrites that value the control displays the new value). Called after
    /// reconcile on a structural frame, when the retained tree already carries the
    /// app's value.
    ///
    /// # The reseed discriminator (external rewrite vs the session's own edit)
    ///
    /// The session is normally the **sole source of truth** for the focused field
    /// (an *uncontrolled* input), and the compose/measure passes paint it — so the
    /// retained tree must NOT clobber the buffer on every keystroke. Two conditions
    /// gate a reseed, using the session's [`mirror`](TextEditSession::mirror)
    /// watermark (the last retained value it reconciled against):
    ///
    /// 1. the retained editable value **changed** since the session last saw it
    ///    (`retained != session.mirror()`). A field whose retained value never
    ///    changes — a static, uncontrolled, one-shot-seeded field — never reseeds,
    ///    so the user's in-flight typing is never fought (this is what keeps the
    ///    measure↔compose "session leads the lagging tree" invariant intact); and
    /// 2. the new retained value **differs from the session's own live text**
    ///    (`retained != session.text()`) — i.e. it is NOT just the session's own
    ///    edit mirrored back through the app (a passthrough controlled input). A
    ///    normalizing `on-input` (trim / uppercase / max-length) whose write-back
    ///    differs from the buffer trips this and DOES reseed, so the displayed text
    ///    tracks the model; a plain passthrough does not (it only advances the
    ///    watermark).
    ///
    /// On a reseed the caret is preserved (clamped into the new value); otherwise
    /// only the watermark is advanced.
    fn reseed_editing_from_retained(&mut self) {
        let Some(field) = self.editing.as_ref().map(TextEditSession::node) else {
            return;
        };
        // The field's value as the freshly-reconciled tree now carries it (the
        // placeholder overlay is skipped, exactly as at seed time).
        let retained = self.collect_editable_text(field);
        let Some(session) = self.editing.as_ref() else {
            return;
        };
        // (1) Unchanged retained value: a static/uncontrolled field, or a frame
        //     that rewrote something else. Never reseed — that would fight typing.
        if retained == session.mirror() {
            return;
        }
        // (2) The session's OWN edit reached the tree (passthrough controlled
        //     input): advance the watermark but keep the live buffer + caret.
        if retained == session.text() {
            if let Some(session) = self.editing.as_mut() {
                session.set_mirror(retained);
            }
            return;
        }
        // External rewrite (Clear / normalize): reseed so the field shows the model
        // value, caret preserved (clamped). Re-derive the caret scroll afterwards.
        if let Some(session) = self.editing.as_mut() {
            session.reseed_controlled(retained);
        }
        self.update_caret_scroll();
    }

    /// On a pointer press: if it landed on the active editable, places the caret
    /// at the click point and arms drag-selection anchored there (spec §5.11).
    ///
    /// Resolves `target`'s editable ancestor and acts only when the active editing
    /// session already covers it (it was just (re)seeded by
    /// [`sync_editing_to_focus`](Self::sync_editing_to_focus)). The click's byte
    /// offset — [`caret_offset_at`](Self::caret_offset_at), the inverse of the
    /// caret-render advance walk — collapses the selection to a caret there and
    /// becomes the drag anchor. A press on a non-editable (or one whose text box is
    /// not laid out) clears any armed drag and leaves the session byte-identical.
    fn begin_text_pointer(&mut self, target: Option<NodeId>, point: Point) {
        // A new press always ends any prior drag; it is re-armed only below.
        self.text_drag = None;
        let Some(target) = target else {
            return;
        };
        let Some(field) = self.editable_ancestor(target) else {
            return;
        };
        // The session must already cover this field (sync runs on focus change; on
        // a repeat click into the already-focused field the session persists).
        if self.editing.as_ref().map(TextEditSession::node) != Some(field) {
            return;
        }
        let Some(offset) = self.caret_offset_at(field, point) else {
            return;
        };
        match self.click_count {
            // Double click: select the WORD under the pointer (word boundaries via
            // `NavigationUnit::Word`, spec §5.11). No drag is armed — the selection
            // is the whole word.
            2 => self.select_word_at(field, offset),
            // Triple click (or beyond): select the whole LINE under the pointer
            // (logical line boundaries via `NavigationUnit::LineBoundary`; a
            // single-line field's whole span, spec §5.11).
            n if n >= 3 => self.select_line_at(field, offset),
            // Single click: a collapsed caret at the click; `SetSelection` is
            // non-mutating, so `after_edit` (inside `apply_edit`) fires no
            // `on-input`, only a frame. Arms a potential drag-selection.
            _ => {
                self.apply_edit(EditCommand::SetSelection(TextSelection::caret(offset)));
                self.text_drag = Some((field, offset));
            }
        }
    }

    /// Resolves the click count of a [`PointerEventKind::Down`] press at `point`
    /// (content space) on `target` with logical timestamp `time_micros`: `1` for a
    /// fresh click, `2` for a double click, `3` for a triple click, then cycling
    /// back to `1` (spec §5.11 editing engine — pointer word/line selection).
    ///
    /// A press counts as a *repeat* of the previous one when it lands on the same
    /// [`target`](PointerClick::target), within [`MULTI_CLICK_WINDOW_MICROS`] of
    /// the previous press, and within [`MULTI_CLICK_RADIUS`] of its point — the
    /// standard platform double-click test. Any press that fails one of these
    /// conditions restarts the count at `1`. The resolved press is recorded as the
    /// new [`last_click`](Self::last_click) so the next press can chain off it.
    fn next_click_count(&mut self, target: Option<NodeId>, point: Point, time_micros: u64) -> u32 {
        let count = self
            .last_click
            .filter(|prev| {
                prev.target == target
                    && time_micros.saturating_sub(prev.time_micros) <= MULTI_CLICK_WINDOW_MICROS
                    && (point.x - prev.point.x).hypot(point.y - prev.point.y) <= MULTI_CLICK_RADIUS
            })
            // A run cycles 1 → 2 → 3 → 1 so a fourth rapid click collapses back to a
            // single caret (matching platform multi-click behaviour) and the count
            // stays bounded.
            .map_or(1, |prev| (prev.count % 3) + 1);
        self.last_click = Some(PointerClick {
            target,
            point,
            time_micros,
            count,
        });
        count
    }

    /// Selects the word containing byte `offset` in the focused editable `field`
    /// (double-click, spec §5.11) using the buffer's `NavigationUnit::Word` engine.
    ///
    /// The word is bracketed by a forward word move from the click (to the word's
    /// end) then a backward word extend (to its start), so the click may land
    /// anywhere within — or at the leading edge of — the word. The buffer's word
    /// motion is a simplified UAX-29 approximation (whitespace-delimited runs); the
    /// resulting `[start, end)` is set through the standard non-mutating
    /// [`apply_edit`](Self::apply_edit) choke point (a frame, no `on-input`). A
    /// no-op when no session covers `field`.
    fn select_word_at(&mut self, field: NodeId, offset: usize) {
        let Some(session) = self.editing.as_mut().filter(|s| s.node() == field) else {
            return;
        };
        let buffer = session.buffer_mut();
        buffer.apply(EditCommand::SetSelection(TextSelection::caret(offset)));
        buffer.apply(EditCommand::Move {
            unit: NavigationUnit::Word,
            direction: EditDirection::Forward,
            extend: false,
        });
        buffer.apply(EditCommand::Move {
            unit: NavigationUnit::Word,
            direction: EditDirection::Backward,
            extend: true,
        });
        let range = buffer.selection().range();
        // Normalise to a forward selection (anchor at the word start) and flush the
        // caret/selection through the standard non-mutating edit path.
        self.apply_edit(EditCommand::SetSelection(TextSelection::new(
            range.start,
            range.end,
        )));
    }

    /// Selects the whole logical line containing byte `offset` in the focused
    /// editable `field` (triple-click, spec §5.11) using the buffer's
    /// `NavigationUnit::LineBoundary` engine — the byte span between the enclosing
    /// `'\n'` boundaries (or the whole text in a single-line field). The `[start,
    /// end)` is set through the standard non-mutating
    /// [`apply_edit`](Self::apply_edit) choke point. A no-op when no session covers
    /// `field`.
    fn select_line_at(&mut self, field: NodeId, offset: usize) {
        let Some(session) = self.editing.as_mut().filter(|s| s.node() == field) else {
            return;
        };
        let buffer = session.buffer_mut();
        buffer.apply(EditCommand::SetSelection(TextSelection::caret(offset)));
        buffer.apply(EditCommand::Move {
            unit: NavigationUnit::LineBoundary,
            direction: EditDirection::Backward,
            extend: false,
        });
        buffer.apply(EditCommand::Move {
            unit: NavigationUnit::LineBoundary,
            direction: EditDirection::Forward,
            extend: true,
        });
        let range = buffer.selection().range();
        self.apply_edit(EditCommand::SetSelection(TextSelection::new(
            range.start,
            range.end,
        )));
    }

    /// While a drag is armed, extends the selection from the press anchor to the
    /// pointer's current byte offset over the **same** field — live drag-selection
    /// (spec §5.11).
    ///
    /// The offset is resolved at `point` against the anchored field (clamped to
    /// that field's text regardless of where the pointer currently is, mirroring
    /// how [`caret_offset_at`](Self::caret_offset_at) clamps before/after the run),
    /// then the selection is set to the range `[anchor, current]` — anchor fixed,
    /// focus tracking the pointer, so dragging left of the anchor selects backward.
    /// A no-op when no drag is armed or the field's text box has gone.
    fn update_text_drag(&mut self, point: Point) {
        let Some((field, anchor)) = self.text_drag else {
            return;
        };
        let Some(current) = self.caret_offset_at(field, point) else {
            return;
        };
        self.apply_edit(EditCommand::SetSelection(TextSelection::new(anchor, current)));
    }

    /// Resolves the byte offset in the focused editable `field`'s text at pointer
    /// `point` (spec §5.11): finds `field`'s text leaf, maps `point.x` into that
    /// run's local x (`point.x − text-origin.x`), and asks the scene coordinator
    /// for the nearest cluster boundary — the inverse of the caret-render advance
    /// walk, so a click at the rendered caret x of offset `i` returns `i`.
    ///
    /// The live **session** text is the source of truth for the field's glyphs
    /// (matching what compose paints), and the retained box origin + computed style
    /// (from the last produced frame) supply the run's position and shaping inputs.
    /// Returns `None` when no active session covers `field`, the field has no text
    /// descendant, or no box was retained for it.
    fn caret_offset_at(&self, field: NodeId, point: Point) -> Option<usize> {
        let session = self.editing.as_ref().filter(|s| s.node() == field)?;
        let text_node = self.first_text_descendant(field)?;
        let text_box = self.boxes.get(&text_node)?;
        let origin = text_box.origin();
        // Undo the paint's `−scroll_left` translation: the run is drawn at
        // `origin.x + glyph_x − scroll_left`, so a pointer at screen `point.x`
        // sits at run-local `point.x − origin.x + scroll_left`. Adding the field's
        // scroll offset back lands a click on a scrolled field at the right byte
        // (the inverse of the caret paint, CSS Overflow 3 §3.1). `scroll_left`
        // is `0` on an unscrolled field ⇒ byte-identical to the pre-scroll inverse.
        // `local_y` picks the wrapped LINE (2D hit-test); a single-line field has
        // one line, so `local_y` is immaterial and the resolved byte is identical.
        let local_x = point.x - origin.x + session.scroll_left();
        let local_y = point.y - origin.y;
        Some(self.scene_composer.caret_offset_at_point(
            &self.tree,
            &self.computed,
            text_node,
            session.text(),
            text_box.size().width,
            local_x,
            local_y,
        ))
    }

    /// Returns `field`'s first text leaf (depth-first, in child order) — the node
    /// the session text is painted on, whose box origin the click-caret x is
    /// measured from. Mirrors the compose walk's editing-text selection: the field
    /// itself if it directly carries text, else its first `Text` descendant. `None`
    /// when the subtree has no text node.
    fn first_text_descendant(&self, node: NodeId) -> Option<NodeId> {
        let n = self.tree.get(node)?;
        if n.kind().text().is_some() {
            return Some(node);
        }
        for &child in n.children() {
            if let Some(found) = self.first_text_descendant(child) {
                return Some(found);
            }
        }
        None
    }

    /// Attempts to consume a key press as an **edit** into the runtime-owned
    /// editing session for the focused editable `target` (spec §5.11, §6.7).
    /// Returns whether the key was consumed (so [`handle_key`](Self::handle_key)
    /// does *not* also treat it as a shortcut or activation).
    ///
    /// Requires an active session whose node is `target`'s editable ancestor.
    /// The keymap:
    /// * a printable [`LogicalKey::Character`] with no command modifier →
    ///   [`EditCommand::Insert`] (replacing any selection);
    /// * `Backspace`/`Delete` → grapheme [`EditCommand::Delete`] backward/forward;
    /// * `ArrowLeft`/`ArrowRight` → grapheme [`EditCommand::Move`], extending the
    ///   selection when `Shift` is held;
    /// * `Home`/`End` → line-boundary `Move` (logical line — the buffer collapses
    ///   Line motion to `'\n'` boundaries; a single-line field's whole span),
    ///   shift-extendable;
    /// * `Ctrl`/`Meta` + `A` → [`EditCommand::SelectAll`]; `+Z` → undo,
    ///   `+Shift+Z`/`+Y` → redo (the buffer exposes both).
    ///
    /// Any other key — including a `Ctrl`/`Alt`/`Meta` chord that is not one of
    /// the editing chords above, and `Enter`/`Escape`/`Tab` — is *not* consumed,
    /// leaving the shortcut/activation path byte-identical. After a mutating edit
    /// the field's `on-input` handler is called with the resulting text; every
    /// consumed edit requests an input frame.
    fn try_text_edit(&mut self, target: NodeId, event: &KeyEvent) -> bool {
        // An edit needs an active session bound to this focus's editable ancestor.
        let Some(node) = self.editing.as_ref().map(TextEditSession::node) else {
            return false;
        };
        if self.editable_ancestor(target) != Some(node) {
            return false;
        }

        let shift = self.modifiers.contains(Modifier::Shift);
        let alt = self.modifiers.contains(Modifier::Alt);
        let cmd = self.modifiers.contains(Modifier::Ctrl) || self.modifiers.contains(Modifier::Meta);

        match event.logical() {
            // Editing chords: Ctrl/Meta (+Shift) letter — select-all / undo / redo.
            // Alt is excluded so a distinct Alt/AltGr chord is not swallowed.
            Some(LogicalKey::Character(c)) if cmd && !alt => {
                match c.to_ascii_lowercase() {
                    'a' => {
                        self.editing
                            .as_mut()
                            .expect("session present")
                            .buffer_mut()
                            .apply(EditCommand::SelectAll);
                        self.after_edit(false);
                        true
                    }
                    'z' if !shift => {
                        let changed = self
                            .editing
                            .as_mut()
                            .is_some_and(|s| s.buffer_mut().undo());
                        self.after_edit(changed);
                        true
                    }
                    'z' | 'y' => {
                        let changed = self
                            .editing
                            .as_mut()
                            .is_some_and(|s| s.buffer_mut().redo());
                        self.after_edit(changed);
                        true
                    }
                    // Any other command chord is a potential app shortcut.
                    _ => false,
                }
            }
            // A command modifier that is not an editing chord: not text.
            _ if cmd || alt => false,
            // Plain printable character: insert (replaces the selection).
            Some(LogicalKey::Character(c)) => {
                self.apply_edit(EditCommand::insert(c.to_string()));
                true
            }
            Some(LogicalKey::Named(named)) => {
                let command = match named {
                    NamedKey::Backspace => EditCommand::Delete {
                        unit: DeletionUnit::Grapheme,
                        direction: EditDirection::Backward,
                    },
                    NamedKey::Delete => EditCommand::Delete {
                        unit: DeletionUnit::Grapheme,
                        direction: EditDirection::Forward,
                    },
                    NamedKey::ArrowLeft => EditCommand::Move {
                        unit: NavigationUnit::Grapheme,
                        direction: EditDirection::Backward,
                        extend: shift,
                    },
                    NamedKey::ArrowRight => EditCommand::Move {
                        unit: NavigationUnit::Grapheme,
                        direction: EditDirection::Forward,
                        extend: shift,
                    },
                    NamedKey::Home => EditCommand::Move {
                        unit: NavigationUnit::LineBoundary,
                        direction: EditDirection::Backward,
                        extend: shift,
                    },
                    NamedKey::End => EditCommand::Move {
                        unit: NavigationUnit::LineBoundary,
                        direction: EditDirection::Forward,
                        extend: shift,
                    },
                    // Space produces a character in a focused text field. The
                    // platform delivers it as a *named* key (`NamedKey::Space`, so a
                    // control that is NOT being edited can use it for activation), so
                    // the editing path — reached only with an active session — must
                    // itself turn it into an inserted `' '` (replacing any selection);
                    // otherwise the field would silently swallow every space.
                    NamedKey::Space => EditCommand::insert(" "),
                    // ArrowUp/ArrowDown move the caret between wrapped lines in a
                    // MULTILINE field (spec §5.11). In a single-line `input` there is
                    // no line to move to, so they are NOT consumed here (byte-identical
                    // — they fall through to the shortcut/activation path as before).
                    NamedKey::ArrowUp | NamedKey::ArrowDown => {
                        return self.try_vertical_move(
                            node,
                            matches!(named, NamedKey::ArrowDown),
                            shift,
                        );
                    }
                    // Enter inserts a newline in a MULTILINE field (CSS Text 3
                    // §4.1.1 segment break); a single-line `input`'s Enter is NOT
                    // consumed — it remains an activation/submit key (byte-identical).
                    NamedKey::Enter if self.is_multiline(node) => EditCommand::insert("\n"),
                    // Enter in a SINGLE-LINE `input` is a **commit** (WHATWG HTML
                    // §4.10.5.5 / §4.10.21.3 implicit form submission): it fires
                    // `change` with the committed value (once, only if it changed
                    // since focus), rebaselining so a later blur doesn't re-fire,
                    // but inserts NOTHING and is NOT consumed as an edit — it falls
                    // through (`return false`) to the activation/shortcut path so
                    // Enter's routing is otherwise byte-identical.
                    NamedKey::Enter => {
                        self.commit_editable_change(true);
                        return false;
                    }
                    // Escape / Tab / etc. remain activation or shortcut keys.
                    _ => return false,
                };
                self.apply_edit(command);
                true
            }
            None => false,
        }
    }

    /// Moves the caret **vertically** between the wrapped lines of a multiline
    /// field on ArrowUp (`down = false`) / ArrowDown (`down = true`), extending the
    /// selection when `Shift` is held — standard caret vertical motion (spec §5.11).
    ///
    /// Returns whether the key was consumed. A **single-line** field is never
    /// consumed (`false`), so its ArrowUp/ArrowDown fall through to the
    /// shortcut/activation path byte-identically. In a multiline field the target
    /// byte is resolved against the SAME wrapped layout the caret paints from
    /// ([`SceneCoordinator::caret_line_move`]) at roughly the caret's current x;
    /// when there is no line in that direction the caret clamps to the text start
    /// (Up on the first line) or end (Down on the last line).
    fn try_vertical_move(&mut self, node: NodeId, down: bool, shift: bool) -> bool {
        if !self.is_multiline(node) {
            return false;
        }
        let Some(session) = self.editing.as_ref().filter(|s| s.node() == node) else {
            return false;
        };
        let Some(text_node) = self.first_text_descendant(node) else {
            return false;
        };
        let Some(text_box) = self.boxes.get(&text_node) else {
            return false;
        };
        let content = session.text();
        let selection = session.selection();
        let focus = selection.focus();
        let new_focus = self
            .scene_composer
            .caret_line_move(
                &self.tree,
                &self.computed,
                text_node,
                content,
                text_box.size().width,
                focus,
                down,
            )
            // No line in that direction: clamp to the text end (Down on the last
            // line) or start (Up on the first line).
            .unwrap_or(if down { content.len() } else { 0 });
        let new_selection = if shift {
            TextSelection::new(selection.anchor(), new_focus)
        } else {
            TextSelection::caret(new_focus)
        };
        self.apply_edit(EditCommand::SetSelection(new_selection));
        true
    }

    /// Applies `command` to the active editing session, then notifies the app
    /// (on a mutating edit) and schedules an input frame via
    /// [`after_edit`](Self::after_edit).
    fn apply_edit(&mut self, command: EditCommand) {
        let mutating = command.is_mutating();
        // A **readonly** field (WHATWG HTML §attr-input-readonly) is focusable and
        // selectable but not editable: drop any text-mutating edit (insert / delete
        // / replace — this is the one choke point every typed insert, backspace/
        // delete, and IME commit flows through) while letting the non-mutating
        // caret/selection edits (Move / SetSelection / SelectAll) apply unchanged.
        if mutating
            && let Some(node) = self.editing.as_ref().map(TextEditSession::node)
            && self.is_within_readonly(node)
        {
            return;
        }
        if let Some(session) = self.editing.as_mut() {
            session.buffer_mut().apply(command);
        }
        self.after_edit(mutating);
    }

    /// Post-edit bookkeeping: on a text-changing edit, dispatch the field's
    /// `on-input` command with the session's resulting text; keep the caret in
    /// view; then schedule the frame that flushes the edit to paint.
    ///
    /// # Frame scheduling: compose-only vs full
    /// When an `on-input` handler actually **ran** (a *controlled* field: the app
    /// mirrored the value into its state), [`dispatch_text_input`] already
    /// requested a full [`FrameReason::Input`] frame — the mirrored value must be
    /// reconciled (and, for a normalizing handler, that reconcile is what drives
    /// the Defect-A controlled-value reseed). So this does **not** schedule a
    /// second, redundant frame.
    ///
    /// When **no** handler ran (a caret move, or an *uncontrolled* field whose
    /// session is the sole source of truth), no reconcile is needed, so a
    /// box-stable edit is flushed with a COMPOSE-ONLY [`FrameReason::TextEdit`]
    /// frame — the per-keystroke fast path (Defect B) that re-shapes + repaints the
    /// field while skipping style + layout. An edit that can resize the box
    /// (wrapping / multiline growth / auto width) still takes a full `Input` frame.
    fn after_edit(&mut self, mutating: bool) {
        let mut dispatched = false;
        if mutating
            && let Some((node, text)) =
                self.editing.as_ref().map(|s| (s.node(), s.text().to_owned()))
            && let Some(command) =
                self.tree.get(node).and_then(|n| n.command_for_event(EVENT_INPUT))
        {
            dispatched = self.dispatch_text_input(command, &text);
        }
        // Keep the caret in view: after any edit or caret move, re-derive the
        // single-line field's horizontal scroll offset so the caret stays inside
        // its content box (CSS Overflow 3 §3.1 scroll-into-view). A no-op for a
        // fitting or wrapping field.
        self.update_caret_scroll();
        // The vertical analogue for a MULTILINE field (`<textarea>`): re-derive the
        // overflow scroll container's `scroll_top` so the caret's LINE stays inside
        // the field's content box (CSS Overflow 3 §3.1 scroll-into-view). A no-op
        // for a single-line field or a textarea whose content fits.
        self.update_multiline_caret_scroll();
        // A dispatched `on-input` already requested a full frame (a reconcile is
        // needed for the mirror / reseed); only schedule our own when it did not.
        if !dispatched {
            let reason = if self.edit_is_compose_only(mutating) {
                FrameReason::TextEdit
            } else {
                FrameReason::Input
            };
            self.request_frame(reason);
        }
    }

    /// Whether the just-applied edit can be flushed with a **compose-only** frame
    /// ([`FrameReason::TextEdit`]) because it provably cannot change any retained
    /// box — a perf optimization that stays pixel-identical to the full-frame path.
    ///
    /// A **non-mutating** edit (a caret move / selection change) never changes the
    /// text, so no box can change: always compose-only.
    ///
    /// A **mutating** edit only stays box-stable when the focused field:
    /// * is single-line (not a `<textarea>`) and does **not** wrap — so the line
    ///   count (box height) is pinned at one; and
    /// * hugs the leading-left edge — its `text-align`/`direction` give a paragraph
    ///   alignment offset that is independent of the box width (see
    ///   [`alignment_ignores_width`]) — so the stale (content-sized) text-node box
    ///   width cannot shift the painted run/caret; and
    /// * has a **definite** (non-auto) width — so neither the field's own box nor
    ///   any ancestor resizes with the typed content (a compose-only frame reuses
    ///   every retained box, so nothing may depend on the grown content width).
    ///
    /// Any other field (wrapping, multiline, centre/right/RTL-aligned, or
    /// auto/content-sized width) returns `false` and takes a full `Input` frame so
    /// its box relayouts — the measure↔compose box-sync invariant is preserved.
    fn edit_is_compose_only(&self, mutating: bool) -> bool {
        // A caret/selection change leaves the text — and therefore every box —
        // untouched: compose-only regardless of the field's shape.
        if !mutating {
            return true;
        }
        let Some(field) = self.editing.as_ref().map(TextEditSession::node) else {
            return false;
        };
        // A multiline field grows a line box when a line is added: not box-stable.
        if self.is_multiline(field) {
            return false;
        }
        let Some(text_node) = self.first_text_descendant(field) else {
            return false;
        };
        let text_style = self.computed.get(&text_node);
        // A wrapping field reflows (its height tracks the wrapped line count).
        if white_space_for(text_style).wraps() {
            return false;
        }
        // A width-dependent alignment would shift the run when the stale text-node
        // box width differs from the (grown) content width.
        if !alignment_ignores_width(alignment_for(text_style), direction_for(text_style)) {
            return false;
        }
        // The field (and thus its ancestors) must be definite-sized so the typed
        // content cannot propagate a size change out to any reused box.
        has_definite_px_width(self.computed.get(&field))
    }

    /// Recomputes the focused single-line field's horizontal scroll offset so its
    /// caret stays within the field's visible content width `W` — the standard
    /// single-line text-control scroll model (CSS Overflow 3 §3.1
    /// scroll-into-view; the text-control behaviour a `<input type=text>` scrolls
    /// with). Called from [`after_edit`](Self::after_edit), so every path that
    /// moves the caret (typing, delete, arrows, Home/End, click, drag-select,
    /// select-all, undo/redo, IME commit) keeps the caret visible.
    ///
    /// Only a **single-line** (non-wrapping) field scrolls horizontally: a
    /// wrapping field (`white-space: normal`/`pre-wrap`/`pre-line`) is pinned to
    /// `scroll_left = 0` (its overflow is handled by wrapping / a later
    /// vertical-scroll item). `W` is the field's border-box width minus its
    /// left/right padding + border (its content-box width, the run's scrollport).
    ///
    /// The clamp keeps the caret's pen x within `[scroll_left, scroll_left + W]`:
    /// when the caret hits the left edge (`caret_x − scroll_left < 0`)
    /// `scroll_left` snaps to `caret_x`; when it hits the right edge
    /// (`caret_x − scroll_left > W`) it snaps to `caret_x − W`. `scroll_left` is
    /// then clamped to `0 ..= max(0, content_width − W)` so the field never scrolls
    /// past the text. `caret_x` and `content_width` are measured with the SAME
    /// advance model the compose caret paints with
    /// ([`SceneCoordinator::caret_advance_x`]), so paint and the scroll agree.
    ///
    /// A no-op (leaving `scroll_left` unchanged) when the field has no text leaf or
    /// no retained box yet; a field whose value fits (`content_width ≤ W`) resolves
    /// to `scroll_left = 0` (byte-identical, unscrolled paint).
    fn update_caret_scroll(&mut self) {
        let Some(session) = self.editing.as_ref() else {
            return;
        };
        let field = session.node();
        let Some(text_node) = self.first_text_descendant(field) else {
            return;
        };
        // Wrapping (multi-line) fields never scroll horizontally: pin to 0.
        if white_space_for(self.computed.get(&text_node)).wraps() {
            if let Some(session) = self.editing.as_mut() {
                session.set_scroll_left(0.0);
            }
            return;
        }
        // The field's visible content width `W` (its scrollport): the border-box
        // width minus the left/right padding + border edges. No retained box yet ⇒
        // nothing to scroll into.
        let Some(field_box) = self.boxes.get(&field) else {
            return;
        };
        let w = field_content_width(self.computed.get(&field), field_box.size().width);
        // The caret's pen x and the whole run advance (content width), in the SAME
        // advance model the compose caret uses, over the live session text.
        let focus = session.selection().focus();
        let text = session.text();
        let style = self.computed.get(&text_node);
        let caret_x = self.scene_composer.caret_advance_x(text, style, focus);
        let content_width = self.scene_composer.caret_advance_x(text, style, text.len());
        let mut scroll_left = session.scroll_left();
        // Scroll-into-view: keep the caret within `[scroll_left, scroll_left + W]`.
        if caret_x - scroll_left < 0.0 {
            scroll_left = caret_x;
        }
        if caret_x - scroll_left > w {
            scroll_left = caret_x - w;
        }
        // Never scroll past the text (and never negative).
        scroll_left = scroll_left.clamp(0.0, (content_width - w).max(0.0));
        if let Some(session) = self.editing.as_mut() {
            session.set_scroll_left(scroll_left);
        }
    }

    /// Recomputes the focused **multiline** field's (`<textarea>`) vertical scroll
    /// offset so its caret's LINE stays within the field's visible content height
    /// `H` — the VERTICAL analogue of [`update_caret_scroll`](Self::update_caret_scroll)'s
    /// horizontal single-line scroll (CSS Overflow 3 §3.1 scroll-into-view). Called
    /// from [`after_edit`](Self::after_edit), so every path that moves the caret
    /// (typing, `Enter`, delete, arrows incl. ArrowUp/Down, Home/End, click,
    /// drag-select, select-all, undo/redo, IME commit) keeps the caret's line
    /// visible.
    ///
    /// This drives the **same** per-node `scroll_top` the overflow scroll container
    /// uses (the wheel + scrollbar move the same value), so the widget's `scroll_top`
    /// map is the single source of truth: typing scrolls the container, and the
    /// caret-follow, wheel, and scrollbar all move `scroll_top` (CSS Overflow 3
    /// §scroll-container). Paint + hit-test already offset by it (the container's
    /// `translate(0, −scroll_top)` brackets the field's subtree, so the editing run/
    /// caret/selection move with it, and the pointer maps back through it), so this
    /// method only has to place `scroll_top`.
    ///
    /// Only a multiline field whose computed `overflow` scrolls (`auto`/`scroll`)
    /// **and** whose height is a definite `px` length participates — that is exactly
    /// a fixed-height `<textarea>` whose content overflows rather than growing the
    /// box. A single-line field, a `hidden`-overflow field, or an auto-height field
    /// that grows to fit is left at `scroll_top == 0` (byte-identical, unscrolled).
    ///
    /// The clamp keeps the caret's line box within `[scroll_top, scroll_top + H]`,
    /// `H` the field's content-box height: when the line top rises above the top
    /// edge (`caret_top − scroll_top < 0`) `scroll_top` snaps to `caret_top`; when
    /// the line bottom drops past the bottom edge (`caret_bottom − scroll_top > H`)
    /// it snaps to `caret_bottom − H`. `scroll_top` is then clamped to
    /// `0 ..= max(0, content_height − H)` so the field never scrolls past its
    /// content (a field whose content fits resolves to `0`). Line geometry +
    /// content height are measured from the live session text with the SAME wrapped
    /// layout the caret paints from ([`SceneCoordinator::caret_line_scroll_geometry`]),
    /// so paint and the scroll agree; the field/text boxes supply the definite
    /// scrollport dimensions.
    fn update_multiline_caret_scroll(&mut self) {
        let Some(session) = self.editing.as_ref() else {
            return;
        };
        let field = session.node();
        // Only a multiline field with a scrolling, definite-height box scrolls
        // vertically with the caret — the fixed-height `<textarea>` case.
        if !self.is_multiline(field)
            || !overflow_scrolls(&self.computed, field)
            || !has_definite_px_height(self.computed.get(&field))
        {
            return;
        }
        let Some(text_node) = self.first_text_descendant(field) else {
            return;
        };
        let (Some(text_box), Some(field_box)) =
            (self.boxes.get(&text_node), self.boxes.get(&field))
        else {
            return;
        };
        // The field's visible content height `H` (border box minus vertical padding
        // + border) — the scrollport the caret's line must stay within. A definite
        // height keeps `field_box` stable across typing, so `H` is exact even before
        // the grown frame lays out.
        let h = field_content_height(self.computed.get(&field), field_box.size().height);
        // The caret's line box top/bottom and the wrapped content height, measured
        // from the text box origin over the live session text — the frame the
        // container's `translate(0, −scroll_top)` shifts the painted line by.
        let focus = session.selection().focus();
        let (line_top, line_height, content_height) =
            self.scene_composer.caret_line_scroll_geometry(
                &self.tree,
                &self.computed,
                text_node,
                session.text(),
                text_box.size().width,
                focus,
            );
        let caret_top = line_top;
        let caret_bottom = line_top + line_height;
        let mut scroll_top = self.scroll_top.get(&field).copied().unwrap_or(0.0);
        // Scroll-into-view: keep the caret's line within `[scroll_top, scroll_top + H]`.
        if caret_top - scroll_top < 0.0 {
            scroll_top = caret_top;
        }
        if caret_bottom - scroll_top > h {
            scroll_top = caret_bottom - h;
        }
        // Never scroll past the content (and never negative); a fitting textarea
        // (`content_height ≤ H`) resolves to `0` — byte-identical, unscrolled.
        scroll_top = scroll_top.clamp(0.0, (content_height - h).max(0.0));
        self.scroll_top.insert(field, scroll_top);
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
        // `:active` — while a press is outstanding (a `Down` with no matching
        // `Up` yet), the pressed node and its element ancestors are active; CSS
        // applies `:active` up the ancestor chain, mirroring the hover chain.
        for node in self.hover_chain_for(self.press_target) {
            let merged = self.pseudo.get(node).union(PseudoState::ACTIVE);
            self.pseudo.set(node, merged);
        }
    }

    /// Binds `shortcut` to the app-owned command symbol `command` in the
    /// runtime's command router, so a matching key press at
    /// [`handle_key`](Self::handle_key) resolves to — and dispatches — that
    /// command (spec §6.7).
    ///
    /// `command` is the **same** [`InternedSymbol`] the application registers a
    /// handler for via [`on_command`](Self::on_command) (and may also bind an
    /// `on-click`/`on-change` event to in the View): keyboard and pointer input
    /// share one command identity, so a bound shortcut runs the same handler a
    /// click would. Registration order is priority: an earlier binding shadows a
    /// later one with the same shortcut (see [`CommandRouter::bind`]). Binding
    /// does not itself schedule a frame.
    pub fn bind_command(&mut self, shortcut: Shortcut, command: InternedSymbol) {
        self.commands.bind(shortcut, command);
    }

    /// Returns the shortcut→command-symbol binding table keyboard input resolves
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
    /// 3. **Resolve + dispatch** — on a **press**, the event's layout-dependent
    ///    [`LogicalKey`](obscur_interaction::command::LogicalKey) plus the held
    ///    modifiers are looked up in the [`CommandRouter`]; a match yields the
    ///    bound command [`InternedSymbol`], whose registered handler is
    ///    **run** via the same [`dispatch_command`](Self::dispatch_command) path
    ///    a pointer click uses. A release, an event with no logical key, or an
    ///    unbound chord resolves to (and dispatches) no command.
    /// 3b. **Activate** — *in addition*, a press whose logical key is Enter or
    ///    Space [`activate`](Self::activate)s the focused control, so a plain
    ///    Enter/Space with no bound shortcut still fires the focused node's
    ///    `on-click` (button) or `on-change` (checkbox) command. Both steps can
    ///    apply on one keypress.
    /// 4. **Schedule** — because the key reached a focused node, a
    ///    [`FrameReason::Input`] frame is requested so any focus/command-driven
    ///    visual reaches paint on the next `run_due_frame`/`render_now`.
    ///
    /// Returns a [`KeyDispatch`] describing the routed target and any resolved
    /// (and dispatched) shortcut command.
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

        // 3. On a press, resolve a bound command from the produced logical key +
        //    held modifiers and DISPATCH it, so a keyboard shortcut runs the same
        //    handler a pointer click would (a release never triggers a shortcut).
        // Whether the key was consumed as a text EDIT. When it was, the editing
        // path already scheduled the appropriate frame via `after_edit` (a
        // compose-only `TextEdit` for a same-size edit, or a full `Input`), so the
        // generic restyle-frame request below must NOT run — overriding it with a
        // full `Input` frame would defeat the per-keystroke compose-only fast path.
        let mut consumed_edit = false;
        let command = if event.pressed() {
            // 3a. Editing takes precedence for a focused editable: a key that the
            //     runtime-owned editing session consumes (a printable char,
            //     Backspace/Delete, an arrow/Home/End motion, or a
            //     select-all/undo/redo chord) is an EDIT, not a shortcut. When
            //     consumed, it is neither resolved as a shortcut nor treated as
            //     activation — a bare 'a' in a field edits the field.
            if self.try_text_edit(target, &event) {
                consumed_edit = true;
                None
            } else {
                // Not text: resolve a bound command from the produced logical key
                // + held modifiers and DISPATCH it, so a keyboard shortcut runs
                // the same handler a pointer click would.
                let resolved = event
                    .logical()
                    .and_then(|key| self.commands.resolve(key, self.modifiers));
                if let Some(command) = resolved {
                    self.dispatch_command(command);
                }
                // 3b. Enter/Space activates the focused control regardless of any
                //     bound shortcut: a button fires its click command, a checkbox
                //     its change command. Both the shortcut above and this
                //     activation can fire on the same keypress.
                if is_activation_key(event.logical()) {
                    self.activate(target);
                }
                resolved
            }
        } else {
            None
        };

        // 4. The key reached a focused node: schedule a restyle for the next
        //    frame, mirroring the pointer path's request on a change — UNLESS the
        //    key was consumed as a text edit, which already scheduled its own
        //    (possibly compose-only) frame via `after_edit`.
        if !consumed_edit {
            self.request_frame(FrameReason::Input);
        }
        KeyDispatch::new(Some(target), command)
    }

    /// Returns the node currently under the pointer (the hover target), if any.
    #[must_use]
    pub fn hovered(&self) -> Option<NodeId> {
        self.hover.topmost()
    }

    /// Enables or disables the in-window inspect overlay (spec §7.6).
    ///
    /// When on, a composed frame highlights the hovered node's border box so the
    /// running UI can be debugged directly in the window. Toggling requests an
    /// [`FrameReason::Input`] frame so the highlight appears/disappears without
    /// waiting for the next pointer move; a no-op when the state is unchanged.
    pub fn set_inspect_mode(&mut self, on: bool) {
        if self.inspect_mode != on {
            self.inspect_mode = on;
            self.request_frame(FrameReason::Input);
        }
    }

    /// Returns whether the in-window inspect overlay is active.
    #[must_use]
    pub const fn inspect_mode(&self) -> bool {
        self.inspect_mode
    }

    /// Returns a readout of the node currently under the pointer — its identity
    /// (CSS `type#id.class`), component id, and absolute box — for a host to
    /// print alongside the in-window highlight (spec §7.6). `None` when nothing
    /// is hovered or the hovered node has no retained box yet.
    #[must_use]
    pub fn hovered_inspect(&self) -> Option<InspectInfo> {
        let node = self.hovered()?;
        let bounds = self.boxes.get(&node)?.bounds;
        let (component, local_name, id, classes) = match self.tree.get(node) {
            Some(record) => {
                let component = match record.kind() {
                    NodeKind::Element { component } => Some(*component),
                    _ => None,
                };
                let attributes = record.attributes();
                (
                    component,
                    attributes.local_name().map(str::to_owned),
                    attributes.id().map(str::to_owned),
                    attributes.classes().iter().map(|class| class.to_string()).collect(),
                )
            }
            None => (None, None, None, Vec::new()),
        };
        Some(InspectInfo { node, component, local_name, id, classes, bounds })
    }

    /// Snapshots the non-style inputs the layout phase reads (see
    /// [`LayoutInputs`]), so a later frame can tell whether the retained boxes
    /// are still valid.
    fn current_layout_inputs(&self, ctx: &FrameContext) -> LayoutInputs {
        LayoutInputs {
            viewport: ctx.viewport(),
            scroll: self.viewport_scroll,
            editing: self.editing_view().map(|view| {
                (
                    view.node,
                    view.text.to_owned(),
                    view.preedit.map(|preedit| preedit.text.to_owned()),
                )
            }),
        }
    }

    /// Refines the plan's conservative layout flag with the **actual** per-node
    /// changes (spec §3.4 phase 5 "classify invalidation", §4.3).
    ///
    /// [`FramePlan`] derives its layout flag from the frame's [`FrameReason`]
    /// alone, so *any* hover asks for a full relayout even when the rule only
    /// shifts a fill. Layout's real inputs are the tree structure, the viewport +
    /// sticky scroll offset, and the **layout-affecting** computed properties
    /// ([`PropertyId::affects_layout`]). For the two hot non-structural reasons
    /// this routes the real per-node style deltas through the
    /// [`InvalidationRouter`] — feeding the domain accumulators the invalidation
    /// model was built for — and answers `false` when no node's box can have
    /// moved, so the whole layout phase is skipped and the retained boxes (and
    /// hit-test index) carry over.
    ///
    /// Conservative by construction — it only ever *removes* provably redundant
    /// work:
    /// * a plan that already skips layout is left alone;
    /// * a first frame, or a reconcile that ACTUALLY changed the tree, lays out;
    /// * any reason outside the refined set below keeps the plan's answer;
    /// * a moved viewport / scroll offset, or a changed node count, forces layout;
    /// * an unknown property is layout-affecting by default.
    ///
    /// # Why `reconciled` and not `plan.reconcile()`
    ///
    /// The reactive path ([`run_due_frame_reactive`](Self::run_due_frame_reactive))
    /// classifies EVERY frame as a structural [`FrameReason::UiCommit`], because the
    /// builder may have produced a different tree and that must be reconciled. So a
    /// windowed host — which drives every hover through the reactive path — would
    /// hit `plan.reconcile() == true` on every frame and never reach this
    /// refinement at all. But reconcile itself reports whether it changed anything:
    /// it diffs text content, attributes and event bindings *by value*, so an empty
    /// commit proves the tree is byte-identical. Gating on that actual result rather
    /// than the plan's conservative upper bound is what makes a reactive hover
    /// layout-free.
    ///
    /// # Why the reason allowlist stays
    ///
    /// The checks below cover layout's inputs for the listed reasons only. Notably
    /// [`ResourceCompletion`](FrameReason::ResourceCompletion) is NOT refined: a
    /// font or image that finished loading changes text metrics / intrinsic sizes,
    /// and neither is a computed style nor a tracked layout input, so its deltas are
    /// invisible here and it must keep laying out.
    fn layout_is_needed(
        &self,
        plan: FramePlan,
        ctx: &FrameContext,
        first_frame: bool,
        reconciled: bool,
        computed: &ComputedStyles,
    ) -> bool {
        if !plan.layout() {
            return false;
        }
        // The tree actually changed, or there is nothing retained to reuse.
        if reconciled || first_frame {
            return true;
        }
        // Only the reasons whose layout inputs are fully covered below are refined.
        if !matches!(
            ctx.reason(),
            FrameReason::Input | FrameReason::Animation | FrameReason::UiCommit
        ) {
            return true;
        }
        // Layout also reads the viewport, the sticky scroll offset and the live
        // editing text — none of which are computed styles.
        if self.last_layout_inputs.as_ref() != Some(&self.current_layout_inputs(ctx)) {
            return true;
        }
        // A node cannot appear/disappear without a reconcile, but if the styled
        // set differs at all the retained boxes cannot be proven valid.
        if computed.len() != self.computed.len() {
            return true;
        }

        // Route the REAL per-node deltas: only a node whose layout-affecting
        // properties changed dirties the layout phase.
        let mut router = InvalidationRouter::new();
        let mut style_invalidation = StyleInvalidation::new();
        let mut layout_invalidation = LayoutInvalidation::new();
        for (node, style) in computed {
            if layout_affecting_changed(self.computed.get(node), style) {
                router.route(
                    Invalidation::new(Some(*node), InvalidationReason::StyleChanged),
                    &mut style_invalidation,
                    &mut layout_invalidation,
                );
            }
        }
        router.needs_layout()
    }

    /// Returns a full structured snapshot of the live tree — one
    /// [`NodeInspection`] per node in pre-order — for an agent to observe and
    /// assert on in a headless run (spec §7.6 inspectable tree/styles/layout).
    ///
    /// Each node carries its identity (kind, component, `type#id.class`), its
    /// literal text (for text nodes), its absolute border box (when laid out this
    /// frame), and a few key computed styles. This is the read half of the
    /// agent debugging loop; the write half is
    /// [`handle_pointer`](Self::handle_pointer)/[`handle_key`](Self::handle_key).
    #[must_use]
    pub fn inspect_snapshot(&self) -> TreeInspection {
        let mut nodes = Vec::new();
        if let Some(root) = self.tree.root() {
            self.collect_inspection(root, None, 0, &mut nodes);
        }
        TreeInspection { nodes }
    }

    /// Appends `node` (and its subtree, pre-order) to `out` as [`NodeInspection`]s.
    fn collect_inspection(
        &self,
        node: NodeId,
        parent: Option<NodeId>,
        depth: u32,
        out: &mut Vec<NodeInspection>,
    ) {
        let Some(record) = self.tree.get(node) else {
            return;
        };
        let (kind, component, text) = match record.kind() {
            NodeKind::Element { component } => ("element", Some(*component), None),
            NodeKind::Text { content } => ("text", None, Some(content.to_string())),
            NodeKind::Root => ("group", None, None),
            NodeKind::Synthetic { .. } => ("synthetic", None, None),
        };
        let attributes = record.attributes();
        out.push(NodeInspection {
            node,
            parent,
            depth,
            kind,
            component,
            local_name: attributes.local_name().map(str::to_owned),
            id: attributes.id().map(str::to_owned),
            classes: attributes.classes().iter().map(|class| class.to_string()).collect(),
            text,
            bounds: self.boxes.get(&node).map(|layout_box| layout_box.bounds),
            styles: self.inspect_styles(node),
        });
        // Clone the child ids so the immutable borrow of `record` ends before the
        // recursive calls re-borrow `self`.
        let children = record.children().to_vec();
        for child in children {
            self.collect_inspection(child, Some(node), depth + 1, out);
        }
    }

    /// Reads a small, fixed set of the most diagnostic computed styles for `node`
    /// as `(name, value)` pairs, omitting any the node did not resolve.
    fn inspect_styles(&self, node: NodeId) -> Vec<(String, String)> {
        const KEYS: &[PropertyId] = &[
            PropertyId::Display,
            PropertyId::Position,
            PropertyId::Width,
            PropertyId::Height,
            PropertyId::Color,
            PropertyId::BackgroundColor,
        ];
        let Some(style) = self.computed_style(node) else {
            return Vec::new();
        };
        KEYS.iter()
            .filter_map(|property| {
                style.get(*property).map(|value| (format!("{property:?}"), format!("{value:?}")))
            })
            .collect()
    }

    /// Appends the inspect overlay for the hovered node to `display_list`, or
    /// returns it unchanged when inspect mode is off, nothing is hovered, or the
    /// hovered node has no retained box.
    fn apply_inspect_overlay(&self, display_list: DisplayList) -> DisplayList {
        if !self.inspect_mode {
            return display_list;
        }
        let Some(bounds) = self.hovered().and_then(|node| self.boxes.get(&node)) else {
            return display_list;
        };
        let mut commands = display_list.commands().to_vec();
        commands.extend(overlay_commands(bounds.bounds));
        DisplayList::from_commands(commands)
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

    /// Configures the runtime for a window surface in one call: sets the device
    /// [`scale`](Self::set_device_scale) and derives the layout
    /// [`viewport`](Self::set_viewport) from the surface's **physical** extent.
    ///
    /// This is the standard HiDPI mapping every windowed host needs (spec §5.2),
    /// so a host does not hand-roll (and drift on) it: `scale` is the display
    /// density (a windowing layer's `scale_factor`, optionally times a user zoom)
    /// and the viewport is that physical extent in **logical** CSS pixels
    /// (`physical / scale`). The UI therefore renders at its authored size for the
    /// display density and reflows to the real window size on resize, rather than
    /// being scaled to fit the window. The viewport divides by the *clamped*
    /// device scale, so the two stay consistent.
    ///
    /// Does not itself schedule a frame; pair with
    /// [`request_frame`](Self::request_frame)`(`[`FrameReason::WindowChange`]`)`.
    pub fn set_surface(&mut self, physical: Size, scale: f32) {
        self.set_device_scale(scale);
        let effective = self.device_scale;
        self.set_viewport(Size::new(physical.width / effective, physical.height / effective));
    }

    /// Returns the current device-pixel scale (HiDPI factor × user zoom) the
    /// composed scene renders at. Default `1.0`.
    #[must_use]
    pub fn device_scale(&self) -> f32 {
        self.device_scale
    }

    /// Sets the device-pixel scale (HiDPI factor × user zoom) the composed scene
    /// renders at, clamped to a sane minimum of `0.1` (a non-positive or
    /// non-finite request is ignored in favour of the clamp) (spec §5.2).
    ///
    /// The scale wraps the composed display list in a single root
    /// `PushTransform(scale(s, s))` … `PopTransform`, so a host that lays out in
    /// **logical** CSS pixels (viewport = physical extent / `s`) still rasterizes
    /// crisply at physical resolution. Pointer coordinates are divided by `s` in
    /// [`handle_pointer`](Self::handle_pointer) so hit-testing stays in logical
    /// space. Setting `1.0` restores the byte-identical (untransformed) path.
    ///
    /// Does not itself schedule a frame; a host typically pairs this with
    /// [`set_viewport`](Self::set_viewport) and
    /// [`request_frame`](Self::request_frame)`(`[`FrameReason::WindowChange`]`)`.
    pub fn set_device_scale(&mut self, scale: f32) {
        self.device_scale = if scale.is_finite() {
            scale.max(0.1)
        } else {
            0.1
        };
    }

    /// Returns the current content scroll offset `(x, y)` in logical CSS pixels.
    /// Default `(0.0, 0.0)`.
    #[must_use]
    pub fn content_offset(&self) -> (f32, f32) {
        self.content_offset
    }

    /// Sets the content scroll `offset` `(x, y)` in **logical** CSS pixels — the
    /// content point pinned to the viewport top-left. A positive offset scrolls
    /// content up/left (spec §4.1).
    ///
    /// The offset is composed with [`device_scale`](Self::device_scale) into the
    /// scene's single root transform `scale(s) · translate(-ox, -oy)`, and added
    /// back to pointer coordinates in [`handle_pointer`](Self::handle_pointer)
    /// (`physical / s + offset`) so hit-testing stays correct while scrolled.
    /// Non-finite components are ignored (the previous value is kept for that
    /// axis). Setting `(0, 0)` at scale `1.0` restores the byte-identical
    /// (untransformed) path.
    ///
    /// Does not itself schedule a frame; a host typically pairs this with
    /// [`request_frame`](Self::request_frame)`(`[`FrameReason::WindowChange`]`)`.
    pub fn set_content_offset(&mut self, x: f32, y: f32) {
        let ox = if x.is_finite() { x } else { self.content_offset.0 };
        let oy = if y.is_finite() { y } else { self.content_offset.1 };
        self.content_offset = (ox, oy);
    }

    // ---- Per-element scroll containers (CSS Overflow Module Level 3) ----------

    /// The current vertical scroll position (`scroll_top`, logical CSS px) of the
    /// scroll container `node`, clamped to its live scrollable range. Returns `0`
    /// for a node that is not a scroll container, does not overflow, or has never
    /// been scrolled (CSS Overflow 3 §scroll-container).
    #[must_use]
    pub fn scroll_top(&self, node: NodeId) -> f32 {
        let containers = scroll_containers(&self.tree, &self.computed, &self.boxes);
        containers
            .iter()
            .find(|c| c.node == node)
            .map_or(0.0, |c| self.clamped_scroll_top(*c))
    }

    /// Sets the scroll container `node`'s vertical scroll position to `px` (logical
    /// CSS px), clamped to `0 ..= content_height − scrollport_height` (CSS Overflow
    /// 3 §scrolling — the position is clamped to the scrollable overflow rect).
    ///
    /// A `node` that is not an overflowing scroll container in the current layout
    /// is ignored (no scroll state is recorded). Does not itself schedule a frame;
    /// pair it with [`request_frame`](Self::request_frame)`(`[`FrameReason::WindowChange`]`)`.
    pub fn set_scroll_top(&mut self, node: NodeId, px: f32) {
        let containers = scroll_containers(&self.tree, &self.computed, &self.boxes);
        let Some(container) = containers.iter().find(|c| c.node == node).copied() else {
            return;
        };
        let clamped = if px.is_finite() {
            px.clamp(0.0, container.max_scroll_top())
        } else {
            return;
        };
        self.scroll_top.insert(node, clamped);
    }

    /// Scrolls the scroll container `node` by `dy` logical px (positive = toward the
    /// content's end), clamped to the scrollable range, and returns the resulting
    /// `scroll_top`. A `node` that is not an overflowing scroll container is
    /// ignored and `0` is returned (CSS Overflow 3 §scrolling).
    pub fn scroll_by(&mut self, node: NodeId, dy: f32) -> f32 {
        let containers = scroll_containers(&self.tree, &self.computed, &self.boxes);
        let Some(container) = containers.iter().find(|c| c.node == node).copied() else {
            return 0.0;
        };
        let current = self.clamped_scroll_top(container);
        let next = (current + dy).clamp(0.0, container.max_scroll_top());
        self.scroll_top.insert(node, next);
        next
    }

    /// Routes a wheel/scroll gesture of `delta_y` logical px at host `position` to
    /// the **innermost** scroll container under the pointer, scrolling it (clamped)
    /// and requesting an [`Input`](FrameReason::Input) frame. Returns the scrolled
    /// container's [`NodeId`], or [`None`] when no scroll container lies under the
    /// pointer (CSS Overflow 3 §scroll-container).
    ///
    /// `position` is in the host's physical (device) space; it is mapped to content
    /// space through the same `physical / device_scale + content_offset` inverse
    /// [`handle_pointer`](Self::handle_pointer) uses, then through each crossed
    /// container's own `scroll_top`, so the hit-tested container is the one the
    /// pointer visually sits over while scrolled.
    pub fn handle_scroll(&mut self, delta_y: f32, position: Point) -> Option<NodeId> {
        let containers = scroll_containers(&self.tree, &self.computed, &self.boxes);
        if containers.is_empty() {
            return None;
        }
        let content = self.to_content_point(position);
        let (_, target) = self.apply_scroll_to_point(content, &containers);
        let target = target?;
        self.scroll_by(target, delta_y);
        self.request_frame(FrameReason::Input);
        Some(target)
    }

    /// The scroll container `node`'s stored `scroll_top`, re-clamped to `c`'s live
    /// range — so a layout change that shrinks the content never surfaces a stale
    /// over-scroll.
    fn clamped_scroll_top(&self, c: ScrollContainer) -> f32 {
        self.scroll_top
            .get(&c.node)
            .copied()
            .unwrap_or(0.0)
            .clamp(0.0, c.max_scroll_top())
    }

    /// Maps a host `position` (physical space) to content space via the inverse of
    /// the root render transform `scale(s) · translate(−ox, −oy)` — the same
    /// mapping [`handle_pointer`](Self::handle_pointer) applies before hit-testing.
    fn to_content_point(&self, position: Point) -> Point {
        let s = self.device_scale;
        let (ox, oy) = self.content_offset;
        if (s - 1.0).abs() > f32::EPSILON || ox != 0.0 || oy != 0.0 {
            Point::new(position.x / s + ox, position.y / s + oy)
        } else {
            position
        }
    }

    /// Adjusts a content-space `point` by each scroll container it lies within,
    /// walking `containers` **outermost-first** (they arrive depth-sorted): for a
    /// container whose scrollport contains the running point, the container's
    /// `scroll_top` is added to the point's `y` (undoing that container's
    /// `translate(0, −scroll_top)`), and the walk continues with the adjusted point
    /// so a nested container is tested in its own (outer-adjusted) coordinate space.
    ///
    /// Returns the fully adjusted point (the layout-space coordinate the unscrolled
    /// spatial index is keyed by) and the **innermost** container hit — exact for
    /// the common single-container case; deeply nested scrollers rely on each
    /// scrollport being tested after its ancestors' adjustment, a documented
    /// best-effort.
    fn apply_scroll_to_point(
        &self,
        mut point: Point,
        containers: &[ScrollContainer],
    ) -> (Point, Option<NodeId>) {
        let mut innermost = None;
        for &c in containers {
            if c.scrollport.contains(point) {
                point = Point::new(point.x, point.y + self.clamped_scroll_top(c));
                innermost = Some(c.node);
            }
        }
        (point, innermost)
    }

    /// Resolves the live [`ScrollPaint`] (clamped `scroll_top` + `content_height`)
    /// of every overflowing scroll container, keyed by [`NodeId`], for the scene
    /// coordinator to paint the scroll offset + scrollbar. Empty when nothing
    /// overflows → a byte-identical compose.
    /// Borrows the runtime-owned editing session into the per-frame
    /// [`EditingView`] that BOTH the layout (measure/arrange) and scene (compose)
    /// passes consume, so the focused editable is sized and painted from the SAME
    /// live session state (CSS Text 3 §4.1.1): the measure pass shapes the session
    /// buffer to size the box, and compose paints the identical string plus the
    /// caret/selection. The view carries the field node, the session `text`,
    /// `selection`, `scroll_left`, and — while composing — the live IME preedit
    /// (spec §5.12). [`None`] when no session is active, which leaves both passes
    /// byte-identical to the non-editing path.
    fn editing_view(&self) -> Option<EditingView<'_>> {
        self.editing.as_ref().map(|session| {
            // The field's live horizontal scroll offset, so compose translates the
            // run/caret/selection/preedit by `−scroll_left` and the field's
            // `overflow: hidden` clip hides the scrolled-out text (single-line
            // text-control scroll model, CSS Overflow 3 §3.1). `0.0` for a fitting
            // or wrapping field ⇒ byte-identical unscrolled paint. The measure pass
            // ignores `scroll_left` (it does not change the shaped string).
            let view = EditingView::new(session.node(), session.text(), session.selection())
                .with_scroll_left(session.scroll_left());
            // While composing, attach the live preedit so both passes splice it into
            // the field for DISPLAY — the buffer text is untouched (spec §5.12).
            match session.composition().filter(|c| c.is_active()) {
                Some(comp) => view.with_preedit(PreeditView::new(
                    comp.preedit().text(),
                    comp.anchor().start,
                    comp.anchor().end,
                    comp.preedit().caret(),
                )),
                None => view,
            }
        })
    }

    fn scroll_paint(&self, computed: &ComputedStyles) -> HashMap<NodeId, ScrollPaint> {
        scroll_containers(&self.tree, computed, &self.boxes)
            .into_iter()
            .map(|c| {
                (
                    c.node,
                    ScrollPaint {
                        scroll_top: self.clamped_scroll_top(c),
                        content_height: c.content_height,
                    },
                )
            })
            .collect()
    }

    /// Wraps a freshly composed `display_list` in the scene's single root
    /// transform `scale(s) · translate(-ox, -oy)` — the composition of
    /// [`device_scale`](Self::device_scale) `s` and the content scroll
    /// [`content_offset`](Self::content_offset) `(ox, oy)` — so the whole scene
    /// rasterizes at physical resolution and scrolled to the current offset while
    /// layout stays in logical px.
    ///
    /// At scale exactly `1.0` **and** offset `(0, 0)` the list is returned
    /// unchanged — no `PushTransform` / `PopTransform` is emitted, so the output is
    /// byte-identical to the pre-scroll/pre-device-scale path (the regression
    /// guard). Otherwise the list is re-wrapped as `PushTransform(root)` + original
    /// commands + `PopTransform`, which keeps it balanced (one push, one matching
    /// pop) since the composed list is itself already balanced. When the offset is
    /// `(0, 0)` the root matrix reduces exactly to `scale(s, s)`.
    fn scaled_root(&self, display_list: DisplayList) -> DisplayList {
        let s = self.device_scale;
        let (ox, oy) = self.content_offset;
        let scale_identity = (s - 1.0).abs() <= f32::EPSILON;
        if scale_identity && ox == 0.0 && oy == 0.0 {
            return display_list;
        }
        // Scroll first (translate content up/left by the offset), then scale to
        // device resolution: `scale(s) · translate(-ox, -oy)`.
        let root = AffineTransform::scale(s, s).multiply(AffineTransform::translate(-ox, -oy));
        let inner = display_list.commands();
        let mut commands = Vec::with_capacity(inner.len() + 2);
        commands.push(PaintCommand::PushTransform(root));
        commands.extend_from_slice(inner);
        commands.push(PaintCommand::PopTransform);
        DisplayList::from_commands(commands)
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

/// Returns whether `key` is an **activation** key — Enter or Space — the two
/// keystrokes that activate a focused control (spec §3.1, §6.7).
///
/// Space is accepted both as the [`NamedKey::Space`] key and as a produced `' '`
/// [`LogicalKey::Character`], since a layout may surface it either way. (A space
/// press over a scroll container would conventionally scroll rather than
/// activate; disambiguating activation from scrolling is a later follow-up.)
fn is_activation_key(key: Option<LogicalKey>) -> bool {
    matches!(
        key,
        Some(LogicalKey::Named(NamedKey::Enter | NamedKey::Space) | LogicalKey::Character(' '))
    )
}

/// Loads and decodes a `background-image: url(...)` `source` into a drawable
/// [`SceneImage`] (spec §5.15 load → §5.16 decode).
///
/// `loader.load` fetches the raw encoded bytes; `decoder.decode` turns them into
/// RGBA8 pixels plus the intrinsic pixel size in the returned
/// [`ResourceMetadata`]; a [`SceneImage`] is built from that size and buffer. The
/// decoder yields non-premultiplied RGBA8, so the image is
/// [`AlphaMode::Straight`]. A free function (not a method) so the caller can hold
/// disjoint borrows of its loader/decoder fields while mutating the hub/cache.
///
/// # Errors
///
/// Propagates any [`ResourceError`] from the loader or decoder, and returns
/// [`ResourceError::DecodeFailed`] when the decoded buffer's length does not match
/// its reported intrinsic size (so [`SceneImage::new`]'s byte-length guard fails).
fn load_and_decode_image(
    loader: &dyn ResourceLoader,
    decoder: &dyn ResourceDecoder,
    source: &ResourceSource,
) -> Result<SceneImage, ResourceError> {
    let loaded = loader.load(source)?;
    let decoded = decoder.decode(ResourceKind::Image, loaded.bytes())?;
    // The decoder records the intrinsic pixel size in its metadata; the buffer is
    // width*height*4 RGBA8 bytes.
    let size = decoded.metadata().intrinsic_size().ok_or_else(|| {
        ResourceError::DecodeFailed("decoded image carries no intrinsic size".into())
    })?;
    // The decoder records the intrinsic size as whole-pixel counts stored in an
    // `f32` (exact for any real image dimension); truncating back to `u32` is
    // lossless, and clamping at 0 rejects a nonsensical negative. A wrong count is
    // caught by `SceneImage::new`'s byte-length guard below, not silently accepted.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let width = size.width.max(0.0) as u32;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let height = size.height.max(0.0) as u32;
    SceneImage::new(width, height, decoded.data().clone(), AlphaMode::Straight).ok_or_else(|| {
        ResourceError::DecodeFailed("decoded image bytes do not match its intrinsic size".into())
    })
}

/// The editing field's visible content width `W` (logical px): its border-box
/// `border_box_width` minus the left/right padding and border edges — the
/// scrollport the single-line run scrolls within (CSS Overflow 3 §3.1). Only
/// absolute `px` padding/border widths are honored (mirroring the scene
/// coordinator's `border_width_of`); the result is clamped non-negative so a box
/// smaller than its insets yields `0` rather than a negative width.
fn field_content_width(style: Option<&ComputedStyle>, border_box_width: f32) -> f32 {
    let pad_left = px_length_of(style, PropertyId::PaddingLeft);
    let pad_right = px_length_of(style, PropertyId::PaddingRight);
    let border_left = px_length_of(style, PropertyId::BorderLeftWidth);
    let border_right = px_length_of(style, PropertyId::BorderRightWidth);
    (border_box_width - pad_left - pad_right - border_left - border_right).max(0.0)
}

/// The field's visible content **height** (its vertical scrollport) — the border
/// box height minus the top/bottom padding + border edges. The VERTICAL analogue
/// of [`field_content_width`]: the height a multiline `<textarea>`'s caret line
/// must stay within as it scrolls vertically (CSS Overflow 3 §3.1). Never
/// negative.
fn field_content_height(style: Option<&ComputedStyle>, border_box_height: f32) -> f32 {
    let pad_top = px_length_of(style, PropertyId::PaddingTop);
    let pad_bottom = px_length_of(style, PropertyId::PaddingBottom);
    let border_top = px_length_of(style, PropertyId::BorderTopWidth);
    let border_bottom = px_length_of(style, PropertyId::BorderBottomWidth);
    (border_box_height - pad_top - pad_bottom - border_top - border_bottom).max(0.0)
}

/// Whether a paragraph alignment offset is **independent of the available (box)
/// width** — i.e. the line hugs the leading-left edge, so the offset is `0`
/// regardless of the box width.
///
/// This is exactly the case where a same-size text edit can reuse a stale,
/// content-sized text-node box width without shifting the painted run/caret
/// (`TextAlignment::x_offset` returns `0` for any available width): `Left` and
/// `Justify` always, `Start` under left-to-right, and `End` under right-to-left.
/// Every other combination distributes the remaining space and therefore depends
/// on the box width.
fn alignment_ignores_width(alignment: TextAlignment, direction: TextDirection) -> bool {
    match alignment {
        TextAlignment::Left | TextAlignment::Justify => true,
        TextAlignment::Start => !direction.is_rtl(),
        TextAlignment::End => direction.is_rtl(),
        TextAlignment::Right | TextAlignment::Center => false,
    }
}

/// Whether `style` carries a **definite** absolute-`px` `width` — a concrete
/// length, not `auto`/content-based sizing.
///
/// A definite-width field cannot grow or shrink with its typed content, so its
/// own box (and every ancestor's) is invariant across a text edit — the
/// precondition for a compose-only frame to reuse the retained boxes unchanged. A
/// percentage or `auto` width returns `false` (conservatively taking the full
/// frame), mirroring the scene coordinator's pixel-only length reads.
fn has_definite_px_width(style: Option<&ComputedStyle>) -> bool {
    style
        .and_then(|s| s.get(PropertyId::Width))
        .and_then(ComputedValue::as_length)
        .and_then(Length::as_px)
        .is_some()
}

/// Whether `style` carries a **definite** absolute-`px` `height` — a concrete
/// length, not `auto`/content-based sizing (the vertical analogue of
/// [`has_definite_px_width`]).
///
/// A definite-height field has a fixed box its content overflows (rather than
/// growing to fit), so it is the field that scrolls VERTICALLY with the caret. An
/// auto/percentage height returns `false`, so an auto-growing multiline field is
/// never treated as a vertical scroll container.
fn has_definite_px_height(style: Option<&ComputedStyle>) -> bool {
    style
        .and_then(|s| s.get(PropertyId::Height))
        .and_then(ComputedValue::as_length)
        .and_then(Length::as_px)
        .is_some()
}

/// Reads one computed length property in absolute `px`, defaulting to `0.0` when
/// the property is absent or is a non-pixel length (mirroring the scene
/// coordinator's pixel-only inset reads).
fn px_length_of(style: Option<&ComputedStyle>, id: PropertyId) -> f32 {
    style
        .and_then(|s| s.get(id))
        .and_then(ComputedValue::as_length)
        .and_then(Length::as_px)
        .map_or(0.0, |px| px.0)
}

/// Records one frame phase's telemetry into `sink` (spec §7.9): a
/// [`PerformanceSpan`] marker for the phase, a [`PerformanceCounter`] carrying
/// the phase's deterministic cost `metric`, and — when the phase's optional
/// budget was exceeded — the resulting [`BudgetViolation`].
///
/// Taking `&mut dyn TelemetrySink` (rather than `&mut self`) keeps the borrow
/// disjoint from the caller's other `self` fields, and makes every call a no-op
/// under the default [`NullTelemetrySink`].
/// The **non-style** inputs the retained boxes were arranged under.
///
/// Layout does not read the computed styles alone: it also reads the viewport,
/// the sticky scroll offset, and the focused editable's live text — a keystroke
/// grows a textarea's box without changing any computed style, which is exactly
/// why a multiline edit schedules a full [`FrameReason::Input`] frame. The
/// incremental classification in [`UiRuntime::layout_is_needed`] may only reuse
/// the retained boxes while **all** of these are unchanged.
#[derive(Debug, Clone, PartialEq)]
struct LayoutInputs {
    /// The viewport the boxes were arranged under.
    viewport: Size,
    /// The viewport scroll offset (drives `position: sticky`).
    scroll: (f32, f32),
    /// The focused editable's `(node, text, preedit)` — the string the text leaf
    /// was measured from; `None` when no editing session is active.
    editing: Option<(NodeId, String, Option<String>)>,
}

/// Whether any **layout-affecting** computed property differs between the
/// retained style `old` and the freshly resolved `new` (spec §4.3).
///
/// A missing `old` (a node with no retained style) counts as changed. Both
/// directions are compared so a property that was *dropped* this frame — not just
/// one that changed value — still dirties layout. Paint-only properties
/// ([`PropertyId::is_paint_only`]) are ignored: they provably cannot move a box.
fn layout_affecting_changed(old: Option<&ComputedStyle>, new: &ComputedStyle) -> bool {
    let Some(old) = old else {
        return true;
    };
    new.iter()
        .any(|(id, value)| id.affects_layout() && old.get(*id) != Some(value))
        || old
            .iter()
            .any(|(id, value)| id.affects_layout() && new.get(*id) != Some(value))
}

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
    use obscur_view::view::{
        CLASS, EVENT_CHANGE, EVENT_CLICK, SRC, ViewEventBinding, ViewFragment, ViewNode,
        ViewProperty, ViewValue,
    };
    use std::cell::Cell;

    /// App-owned command symbols the click tests bind and register against.
    const CMD_PANEL: InternedSymbol = InternedSymbol::from_raw(0xC0DE);
    const CMD_B: InternedSymbol = InternedSymbol::from_raw(0xB0B0);
    /// A keyboard-shortcut command symbol the key tests bind and dispatch.
    const CMD_SAVE: InternedSymbol = InternedSymbol::from_raw(0x5A7E);
    /// A checkbox `on-change` command symbol the change-activation tests use.
    const CMD_CHECK: InternedSymbol = InternedSymbol::from_raw(0xC4EC);

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

    // ---- runtime-driven components (§2.1, §2.14, §3.1) ----------------------

    mod components {
        use super::{runtime_id, viewport};
        use crate::frame::FrameReason;
        use crate::runtime::UiRuntime;
        use obscur_identity::{ComponentId, InternedSymbol, NodeId};
        use obscur_view::component::{
            Component, ComponentContext, ComponentContract, ComponentFactory, ComponentInstance,
            ComponentRegistration, ComponentVersion, ErasedComponent, FromProps, InstanceId,
            PropsError, decode_required,
        };
        use obscur_view::state::LocalState;
        use obscur_view::view::{View, ViewFragment, ViewNode, ViewProperty, ViewValue};

        const COUNTER: ComponentId = ComponentId::from_raw(0xC0FFEE);

        /// A non-reserved prop symbol, resolved the way the view tree resolves it.
        fn step() -> InternedSymbol {
            InternedSymbol::from_raw(0x8000_0021)
        }

        struct CounterProps {
            step: u32,
        }

        impl FromProps for CounterProps {
            fn from_props(props: &[ViewProperty]) -> Result<Self, PropsError> {
                Ok(Self {
                    step: decode_required(props, step())?,
                })
            }
        }

        /// Adds `step` to its own local state on every render and emits the total
        /// as a text child — so the text reveals both the decoded prop and whether
        /// the state survived.
        struct Counter {
            total: LocalState<u32>,
            /// Ask for exactly one more render, then settle — modelling a
            /// component that advances its own state once.
            self_updates: u32,
        }

        impl Component for Counter {
            type Props = CounterProps;
            fn render(&mut self, ctx: &mut ComponentContext<'_>, props: &Self::Props) -> View {
                self.total.set(*self.total.get() + props.step);
                if self.self_updates > 0 {
                    self.self_updates -= 1;
                    ctx.request_update();
                }
                View::from_fragment(ViewFragment::from_nodes(vec![ViewNode::text(
                    self.total.get().to_string(),
                )]))
            }
        }

        #[derive(Debug)]
        struct CounterFactory {
            self_updates: u32,
        }

        impl ComponentFactory for CounterFactory {
            fn component_id(&self) -> ComponentId {
                COUNTER
            }
            fn contract(&self) -> ComponentContract {
                ComponentContract::new(COUNTER)
            }
            fn create_instance(&self, instance: InstanceId) -> ComponentInstance {
                ComponentInstance::new(instance, COUNTER)
            }
            fn create_component(&self) -> Box<dyn ErasedComponent> {
                Box::new(Counter {
                    total: LocalState::new(0),
                    self_updates: self.self_updates,
                })
            }
        }

        fn runtime_with(self_updates: u32) -> UiRuntime {
            let mut runtime = UiRuntime::new(runtime_id(), viewport());
            runtime
                .register_component(ComponentRegistration::new(
                    Box::new(CounterFactory { self_updates }),
                    ComponentVersion::INITIAL,
                ))
                .expect("first registration");
            runtime
        }

        fn runtime_with_counter() -> UiRuntime {
            runtime_with(0)
        }

        /// A view with one `<Counter step=N/>` element.
        fn counter_view(step_by: i64) -> View {
            View::from_fragment(ViewFragment::from_nodes(vec![
                ViewNode::element(COUNTER)
                    .with_prop(ViewProperty::new(step(), ViewValue::Int(step_by))),
            ]))
        }

        /// The text the component rendered underneath its own node.
        fn rendered_text(runtime: &UiRuntime) -> String {
            let root = runtime.tree().root().expect("a root");
            let node = runtime.tree().get(root).expect("live").children()[0];
            let child = runtime.tree().get(node).expect("live").children()[0];
            match runtime.tree().get(child).expect("live").kind() {
                obscur_tree::node::NodeKind::Text { content } => content.to_string(),
                other => panic!("expected the component's text output, got {other:?}"),
            }
        }

        /// The node the component instance is mounted on.
        fn component_node(runtime: &UiRuntime) -> NodeId {
            let root = runtime.tree().root().expect("a root");
            runtime.tree().get(root).expect("live").children()[0]
        }

        #[test]
        fn the_runtime_mounts_and_renders_a_registered_component() {
            // THE vertical slice: a node naming a registered kind mounts an
            // instance, its typed props are decoded from the node's authored
            // properties, and its rendered output becomes that node's children.
            let mut runtime = runtime_with_counter();
            runtime.render_now(&counter_view(2), &[]);

            assert_eq!(runtime.mounted_component_count(), 1);
            assert_eq!(rendered_text(&runtime), "2", "the decoded step reached render");
        }

        #[test]
        fn local_state_survives_across_frames_on_a_stable_node() {
            // The node is reused across frames (same kind, same position), so the
            // instance — and the LocalState it owns — is never re-created.
            let mut runtime = runtime_with_counter();
            runtime.render_now(&counter_view(2), &[]);
            let node = component_node(&runtime);
            assert_eq!(rendered_text(&runtime), "2");

            runtime.render_now(&counter_view(3), &[]);
            assert_eq!(component_node(&runtime), node, "the node was reused");
            assert_eq!(runtime.mounted_component_count(), 1, "no remount");
            assert_eq!(rendered_text(&runtime), "5", "2 + 3: the state persisted");
        }

        #[test]
        fn request_update_schedules_a_frame_and_is_serviced_by_the_next_one() {
            // The loop `ComponentContext::request_update` was written for, and
            // which had no reader at all until now: the component asks from inside
            // its own render, the frame does NOT service it (that would spin), the
            // scheduler is asked for another, and that frame re-renders it.
            let mut runtime = runtime_with(1);
            runtime.render_now(&counter_view(2), &[]);
            assert_eq!(rendered_text(&runtime), "2");
            assert_eq!(
                runtime.peek_frame_reason(),
                Some(FrameReason::UiCommit),
                "the pending request scheduled a frame"
            );

            // The scheduled frame services it — rebuilding the SAME view, so the
            // re-render is driven purely by the component's own request.
            runtime
                .run_due_frame(&counter_view(2), &[])
                .expect("the requested frame is due");
            assert_eq!(rendered_text(&runtime), "4", "2 + 2: it re-rendered once");

            // It asked only once, so it now settles: no further frame is pending.
            assert_eq!(
                runtime.peek_frame_reason(),
                None,
                "a settled component stops requesting frames"
            );
        }

        #[test]
        fn only_the_component_whose_props_changed_re_renders() {
            // THE incremental-invalidation property for components (§3.1:
            // "dependency tracking invalidates only consumers of changed values").
            // Two sibling counters; only the first one's props change. Before this,
            // every structural frame re-rendered EVERY mounted component, so both
            // totals would advance.
            let view = |a: i64, b: i64| {
                View::from_fragment(ViewFragment::from_nodes(vec![
                    ViewNode::element(COUNTER)
                        .with_prop(ViewProperty::new(step(), ViewValue::Int(a))),
                    ViewNode::element(COUNTER)
                        .with_prop(ViewProperty::new(step(), ViewValue::Int(b))),
                ]))
            };
            let text_at = |runtime: &UiRuntime, index: usize| {
                let root = runtime.tree().root().expect("a root");
                let host = runtime.tree().get(root).expect("live").children()[index];
                let child = runtime.tree().get(host).expect("live").children()[0];
                match runtime.tree().get(child).expect("live").kind() {
                    obscur_tree::node::NodeKind::Text { content } => content.to_string(),
                    other => panic!("expected text, got {other:?}"),
                }
            };

            let mut runtime = runtime_with_counter();
            runtime.render_now(&view(2, 10), &[]);
            assert_eq!(text_at(&runtime, 0), "2");
            assert_eq!(text_at(&runtime, 1), "10");

            // Only the first counter's `step` changes.
            runtime.render_now(&view(3, 10), &[]);
            assert_eq!(text_at(&runtime, 0), "5", "2 + 3: it re-rendered");
            assert_eq!(
                text_at(&runtime, 1),
                "10",
                "the untouched sibling must NOT have re-rendered"
            );
        }

        #[test]
        fn an_unchanged_frame_re_renders_nothing() {
            // Re-rendering an identical view touches no component at all: every
            // instance's output is already current.
            let mut runtime = runtime_with_counter();
            runtime.render_now(&counter_view(2), &[]);
            assert_eq!(rendered_text(&runtime), "2");

            runtime.render_now(&counter_view(2), &[]);
            assert_eq!(
                rendered_text(&runtime),
                "2",
                "an identical frame must not re-run render (it would read 4)"
            );
        }

        #[test]
        fn the_root_reconcile_does_not_destroy_the_components_output() {
            // The desired node `<Counter step=N/>` has NO children — the component
            // supplies them. Without COMPONENT_HOST, the root reconcile would
            // treat the desired (empty) children as authoritative and delete the
            // component's output every frame, re-creating it afterwards with fresh
            // NodeIds — which would also unmount and remount any nested component,
            // losing its local state. Identity of the output node proves it stands.
            let mut runtime = runtime_with_counter();
            runtime.render_now(&counter_view(2), &[]);

            let host = component_node(&runtime);
            assert!(
                runtime
                    .tree()
                    .get(host)
                    .expect("live")
                    .flags()
                    .contains(obscur_tree::node::NodeFlags::COMPONENT_HOST),
                "mounting flags the node as a component host"
            );
            let output = runtime.tree().get(host).expect("live").children()[0];

            runtime.render_now(&counter_view(3), &[]);
            assert_eq!(
                runtime.tree().get(host).expect("live").children()[0],
                output,
                "the component's output node survived the root reconcile"
            );
        }

        #[test]
        fn unmounting_clears_the_component_host_flag() {
            // Otherwise a node whose component was unmounted would stay opaque to
            // reconciliation forever.
            let mut runtime = runtime_with_counter();
            runtime.render_now(&counter_view(2), &[]);
            let host = component_node(&runtime);
            assert!(runtime.tree().get(host).is_some());

            runtime.render_now(&View::empty(), &[]);
            // The node is gone with its instance; nothing is left flagged.
            assert_eq!(runtime.mounted_component_count(), 0);
            assert!(runtime.tree().get(host).is_none(), "the host node is gone");
        }

        #[test]
        fn removing_the_node_unmounts_the_instance() {
            let mut runtime = runtime_with_counter();
            runtime.render_now(&counter_view(2), &[]);
            assert_eq!(runtime.mounted_component_count(), 1);

            runtime.render_now(&View::empty(), &[]);
            assert_eq!(runtime.mounted_component_count(), 0);
        }

        #[test]
        fn a_host_that_registers_nothing_is_unaffected() {
            // The same view, with no registration: the element is a plain node,
            // nothing mounts, and it renders no children of its own.
            let mut runtime = UiRuntime::new(runtime_id(), viewport());
            runtime.render_now(&counter_view(2), &[]);
            assert_eq!(runtime.mounted_component_count(), 0);
            let node = component_node(&runtime);
            assert!(
                runtime.tree().get(node).expect("live").children().is_empty(),
                "an unregistered element gets no component output"
            );
        }

        #[test]
        fn undecodable_props_leave_the_subtree_alone_rather_than_deleting_it() {
            // First a good frame, so the component has output.
            let mut runtime = runtime_with_counter();
            runtime.render_now(&counter_view(2), &[]);
            assert_eq!(rendered_text(&runtime), "2");

            // Now the author writes `step="2"` — text where a number is wanted.
            let bad = View::from_fragment(ViewFragment::from_nodes(vec![
                ViewNode::element(COUNTER)
                    .with_prop(ViewProperty::new(step(), ViewValue::Text("2".into()))),
            ]));
            runtime.render_now(&bad, &[]);
            // The instance is still mounted and its previous output still stands:
            // an authored mistake must not silently delete a subtree.
            assert_eq!(runtime.mounted_component_count(), 1);
            assert_eq!(rendered_text(&runtime), "2");
        }
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

    /// A mock [`ResourceLoader`] that returns fixed encoded bytes for any File
    /// source, or fails every source (for the error path).
    #[derive(Debug, Clone)]
    struct MockLoader {
        fails: bool,
    }

    impl ResourceLoader for MockLoader {
        fn can_load(&self, source: &ResourceSource) -> bool {
            source.is_file()
        }

        fn load(
            &self,
            _source: &ResourceSource,
        ) -> Result<obscur_resources::port::LoadedBytes, ResourceError> {
            if self.fails {
                return Err(ResourceError::NotFound);
            }
            let bytes: std::sync::Arc<[u8]> = std::sync::Arc::from(vec![1u8, 2, 3, 4]);
            Ok(obscur_resources::port::LoadedBytes::new(
                bytes,
                ResourceMetadata::new().with_byte_size(4),
            ))
        }
    }

    /// A mock [`ResourceDecoder`] that decodes any Image into a canned 2×2 opaque
    /// RGBA buffer with a 2×2 intrinsic size.
    #[derive(Debug, Clone)]
    struct MockDecoder;

    impl ResourceDecoder for MockDecoder {
        fn supports(&self, kind: ResourceKind) -> bool {
            kind == ResourceKind::Image
        }

        fn decode(
            &self,
            _kind: ResourceKind,
            _bytes: &[u8],
        ) -> Result<obscur_resources::port::DecodedResource, ResourceError> {
            let rgba: std::sync::Arc<[u8]> = std::sync::Arc::from(vec![255u8; 2 * 2 * 4]);
            Ok(obscur_resources::port::DecodedResource::new(
                rgba,
                ResourceMetadata::new().with_intrinsic_size(Size::new(2.0, 2.0)),
            ))
        }
    }

    /// A view whose single element carries the `img` class.
    fn img_view() -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(11))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("img".into()))),
        ]))
    }

    /// `.img { background-image: url("x"); width: 40px; height: 30px }`.
    fn img_stylesheet() -> Stylesheet {
        let selectors = SelectorList::new(vec![ComplexSelector::new(
            CompoundSelector::new(vec![SimpleSelector::Class("img".into())]),
            Vec::new(),
        )]);
        Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![StyleRule::new(
                selectors,
                vec![
                    BlockDeclaration::normal(
                        PropertyId::BackgroundImage,
                        PropertyValue::BackgroundImageUrl("x".into()),
                    ),
                    BlockDeclaration::normal(
                        PropertyId::Width,
                        PropertyValue::Length(Length::px(40.0)),
                    ),
                    BlockDeclaration::normal(
                        PropertyId::Height,
                        PropertyValue::Length(Length::px(30.0)),
                    ),
                ],
            )],
        )
    }

    /// Counts the `DrawImage` commands in a display list.
    fn draw_image_count(list: &DisplayList) -> usize {
        list.commands()
            .iter()
            .filter(|c| matches!(c, obscur_scene::command::PaintCommand::DrawImage { .. }))
            .count()
    }

    #[test]
    fn url_background_image_resolves_and_paints_a_draw_image() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        runtime.set_resource_providers(Box::new(MockLoader { fails: false }), Box::new(MockDecoder));

        let sheet = img_stylesheet();
        let result = runtime.render_now(&img_view(), std::slice::from_ref(&sheet));
        let list = result.snapshot().display_list();
        assert_eq!(
            draw_image_count(list),
            1,
            "the resolved url() image paints one DrawImage"
        );
        assert!(result.is_balanced());
    }

    /// A view whose single element carries the `counter` class (so
    /// `counter_stylesheet` sizes it — with a solid background but NO
    /// background-image) plus an image `src`, an `<img>` primitive's content. Using
    /// `counter` rather than `img` keeps the only DrawImage the element's own image.
    fn img_src_view(src: &str) -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(11))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("counter".into())))
                .with_prop(ViewProperty::new(SRC, ViewValue::Text(src.into()))),
        ]))
    }

    #[test]
    fn element_image_src_resolves_and_paints_a_draw_image() {
        // A `<img>` element's own `src` loads + decodes into the cache (same path as
        // background images) and paints one DrawImage over its 100×50 box.
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        runtime.set_resource_providers(Box::new(MockLoader { fails: false }), Box::new(MockDecoder));
        let sheet = counter_stylesheet();
        let result = runtime.render_now(&img_src_view("logo"), std::slice::from_ref(&sheet));
        assert_eq!(
            draw_image_count(result.snapshot().display_list()),
            1,
            "the element image resolves and paints one DrawImage"
        );
        assert!(result.is_balanced());
    }

    #[test]
    fn element_image_loader_error_paints_no_image_and_does_not_panic() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        runtime.set_resource_providers(Box::new(MockLoader { fails: true }), Box::new(MockDecoder));
        let sheet = counter_stylesheet();
        let result = runtime.render_now(&img_src_view("logo"), std::slice::from_ref(&sheet));
        assert_eq!(
            draw_image_count(result.snapshot().display_list()),
            0,
            "a failed load caches nothing → no element DrawImage"
        );
        assert!(result.is_balanced());
    }

    #[test]
    fn url_background_image_with_no_providers_paints_no_image() {
        // Honest-capability default: no loader/decoder installed → no image.
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let sheet = img_stylesheet();
        let result = runtime.render_now(&img_view(), std::slice::from_ref(&sheet));
        assert_eq!(draw_image_count(result.snapshot().display_list()), 0);
    }

    #[test]
    fn url_background_image_loader_error_paints_no_image_and_does_not_panic() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        runtime.set_resource_providers(Box::new(MockLoader { fails: true }), Box::new(MockDecoder));

        let sheet = img_stylesheet();
        let result = runtime.render_now(&img_view(), std::slice::from_ref(&sheet));
        assert_eq!(
            draw_image_count(result.snapshot().display_list()),
            0,
            "a failed load caches nothing → no DrawImage"
        );
        assert!(result.is_balanced());
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

    // ---- UA default stylesheet: lowest-origin, author-overridable base look ----

    /// A single `button` (CSS type) element carrying a text child (so it lays out
    /// with an intrinsic box). Mirrors what `Button::view` now emits.
    fn button_view() -> View {
        use obscur_view::view::TYPE;
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(20))
                .with_prop(ViewProperty::new(TYPE, ViewValue::Text("button".into())))
                .with_children(ViewFragment::from_nodes(vec![ViewNode::text("Save")])),
        ]))
    }

    /// A `button { … }` type rule at the given origin carrying `decls`.
    fn button_type_rule(decls: Vec<BlockDeclaration>) -> StyleRule {
        StyleRule::new(
            SelectorList::new(vec![ComplexSelector::new(
                CompoundSelector::new(vec![SimpleSelector::Type("button".into())]),
                Vec::new(),
            )]),
            decls,
        )
    }

    /// Returns the `button` node id after a frame has populated the tree.
    fn button_node(runtime: &UiRuntime) -> NodeId {
        let root = runtime.tree().root().expect("root after a frame");
        runtime.tree().get(root).unwrap().children()[0]
    }

    /// The UA background fill the default sheet applies (a neutral `#f4f2ee`).
    fn ua_fill() -> StyleColor {
        StyleColor::rgba(
            f32::from(0xf4u8) / 255.0,
            f32::from(0xf2u8) / 255.0,
            f32::from(0xeeu8) / 255.0,
            1.0,
        )
    }

    #[test]
    fn ua_default_sheet_supplies_the_base_background_when_no_author_rule() {
        // A UA sheet at the LOWEST origin (`SystemDefault`) installed via the
        // default slot; NO author sheets. The button's computed background is the
        // UA fill — the base look shows through.
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        runtime.set_default_stylesheet(Stylesheet::new(
            CascadeOrigin::SystemDefault,
            vec![button_type_rule(background(
                ua_fill().r,
                ua_fill().g,
                ua_fill().b,
            ))],
        ));
        let view = button_view();
        let _ = runtime.render_now(&view, &[]);

        let button = button_node(&runtime);
        let bg = runtime
            .computed_style(button)
            .and_then(|s| s.get(PropertyId::BackgroundColor))
            .and_then(ComputedValue::as_color);
        assert_eq!(bg, Some(ua_fill()), "UA default background must show through");
    }

    #[test]
    fn author_sheet_overrides_the_ua_default_background() {
        // The crux: with the UA sheet at `SystemDefault` AND an author sheet at
        // `GlobalApp` both styling `button { background }`, the AUTHOR value wins by
        // origin precedence (§4.14) — the UA sheet is a true, overridable base.
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        runtime.set_default_stylesheet(Stylesheet::new(
            CascadeOrigin::SystemDefault,
            vec![button_type_rule(background(
                ua_fill().r,
                ua_fill().g,
                ua_fill().b,
            ))],
        ));
        // Author override: `button { background: #cc0000 }`.
        let author = Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![button_type_rule(background(0.8, 0.0, 0.0))],
        );
        let view = button_view();
        let _ = runtime.render_now(&view, std::slice::from_ref(&author));

        let button = button_node(&runtime);
        let bg = runtime
            .computed_style(button)
            .and_then(|s| s.get(PropertyId::BackgroundColor))
            .and_then(ComputedValue::as_color);
        assert_eq!(
            bg,
            Some(StyleColor::rgba(0.8, 0.0, 0.0, 1.0)),
            "author sheet (GlobalApp) must beat the UA default (SystemDefault)"
        );
        assert_ne!(bg, Some(ua_fill()), "the UA value must NOT survive the override");
    }

    #[test]
    fn no_default_sheet_leaves_the_element_unstyled() {
        // Absent a default sheet, a bare `button` has NO UA-derived background: it
        // resolves to the initial transparent — byte-identical to before the slot
        // existed.
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        assert!(!runtime.has_default_stylesheet());
        let view = button_view();
        let _ = runtime.render_now(&view, &[]);

        let button = button_node(&runtime);
        let bg = runtime
            .computed_style(button)
            .and_then(|s| s.get(PropertyId::BackgroundColor))
            .and_then(ComputedValue::as_color);
        // The initial background is transparent, never the UA fill.
        assert_eq!(bg, Some(StyleColor::TRANSPARENT));
        assert_ne!(bg, Some(ua_fill()));
    }

    /// A `button` element that carries the reserved `:disabled` structural-state
    /// attribute when `disabled` — mirroring what `Button::view` now emits.
    fn button_state_view(disabled: bool) -> View {
        use obscur_view::view::{DISABLED, TYPE};
        let mut node = ViewNode::element(ComponentId::from_raw(20))
            .with_prop(ViewProperty::new(TYPE, ViewValue::Text("button".into())))
            .with_children(ViewFragment::from_nodes(vec![ViewNode::text("Save")]));
        if disabled {
            node = node.with_prop(ViewProperty::new(DISABLED, ViewValue::Bool(true)));
        }
        View::from_fragment(ViewFragment::from_nodes(vec![node]))
    }

    /// A `button:disabled { … }` rule carrying `decls`.
    fn button_disabled_rule(decls: Vec<BlockDeclaration>) -> StyleRule {
        use obscur_style::selector::PseudoClassSelector;
        StyleRule::new(
            SelectorList::new(vec![ComplexSelector::new(
                CompoundSelector::new(vec![
                    SimpleSelector::Type("button".into()),
                    SimpleSelector::PseudoClass(PseudoClassSelector::Disabled),
                ]),
                Vec::new(),
            )]),
            decls,
        )
    }

    #[test]
    fn disabled_pseudo_class_bites_only_a_disabled_button_and_re_resolves_on_toggle() {
        // Author sheet: base `button { background: ua_fill }` plus a
        // `button:disabled { background: #808080 }` overlay. The disabled overlay
        // must apply ONLY when the button reports the reserved DISABLED state.
        let disabled_fill = StyleColor::rgba(0.5, 0.5, 0.5, 1.0);
        let sheet = Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![
                button_type_rule(background(ua_fill().r, ua_fill().g, ua_fill().b)),
                button_disabled_rule(background(disabled_fill.r, disabled_fill.g, disabled_fill.b)),
            ],
        );
        let sheets = std::slice::from_ref(&sheet);

        let mut runtime = UiRuntime::new(runtime_id(), viewport());

        // Enabled button: the base rule wins, the :disabled overlay is inert.
        let _ = runtime.render_now(&button_state_view(false), sheets);
        let bg = |runtime: &UiRuntime| {
            let node = button_node(runtime);
            runtime
                .computed_style(node)
                .and_then(|s| s.get(PropertyId::BackgroundColor))
                .and_then(ComputedValue::as_color)
        };
        assert_eq!(
            bg(&runtime),
            Some(ua_fill()),
            "an enabled button must NOT pick up the :disabled overlay"
        );

        // Toggle to disabled: the reconcile updates the reserved attribute, the
        // node restyles, and the :disabled overlay now wins.
        let _ = runtime.render_now(&button_state_view(true), sheets);
        assert_eq!(
            bg(&runtime),
            Some(disabled_fill),
            ":disabled must bite once the button reports the disabled state"
        );

        // Toggle back to enabled: the overlay drops and the base fill re-resolves.
        let _ = runtime.render_now(&button_state_view(false), sheets);
        assert_eq!(
            bg(&runtime),
            Some(ua_fill()),
            "clearing disabled must re-resolve the computed style back to the base"
        );
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

    /// `.box { width: 100px; height: 50px } .box:hover { background-color: red }`
    /// — a hover that shifts only a fill (the wireframe kit's own hover model).
    fn box_paint_only_hover_sheet() -> Stylesheet {
        Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![
                class_rule("box", size(100.0, 50.0)),
                class_hover_rule(
                    "box",
                    vec![BlockDeclaration::normal(
                        PropertyId::BackgroundColor,
                        PropertyValue::Color(StyleColor::rgba(1.0, 0.0, 0.0, 1.0)),
                    )],
                ),
            ],
        )
    }

    #[test]
    fn a_paint_only_hover_reuses_the_retained_boxes() {
        // THE incremental-invalidation win (§3.4 p5, §4.3): the plan says an
        // `Input` frame may relayout (it is classified from the REASON alone),
        // but the hover rule only changes `background-color` — a paint-only
        // property — so no box can have moved and the whole layout phase is
        // skipped, reusing the retained boxes.
        let view = box_view();
        let sheet = box_paint_only_hover_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        runtime.render_now(&view, sheets);
        let node = counter_child(&runtime);
        let cold_box = runtime.layout_boxes().get(&node).copied().expect("a laid-out box");

        runtime.set_pseudo_state(node, PseudoState::HOVER);
        runtime.request_frame(FrameReason::Input);
        let hot = runtime
            .run_due_frame(&view, sheets)
            .expect("the input frame is due");

        assert!(
            !hot.relaid_out(),
            "a paint-only hover must not relayout — the retained boxes are provably valid"
        );
        assert!(hot.restyled(), "it still restyles so the fill reaches paint");
        assert!(hot.composed(), "and still re-composes");
        assert_eq!(
            runtime.layout_boxes().get(&node).copied(),
            Some(cold_box),
            "the retained box carried over unchanged"
        );
    }

    #[test]
    fn a_layout_affecting_hover_still_relayouts() {
        // The other side of the classification: `.box:hover { width: 200px }`
        // changes a layout-affecting property, so the layout phase must run.
        let view = box_view();
        let sheet = box_hover_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        runtime.render_now(&view, sheets);
        let node = counter_child(&runtime);

        runtime.set_pseudo_state(node, PseudoState::HOVER);
        runtime.request_frame(FrameReason::Input);
        let hot = runtime
            .run_due_frame(&view, sheets)
            .expect("the input frame is due");

        assert!(hot.relaid_out(), "a width change must relayout");
        assert_eq!(
            runtime.layout_boxes().get(&node).map(|b| b.bounds.size.width),
            Some(200.0),
            "the hovered box grew to its hover width"
        );
    }

    #[test]
    fn a_paint_only_hover_is_layout_free_on_the_reactive_path_too() {
        // The refinement has to bite where hosts actually live. A windowed host
        // drives every frame through `run_due_frame_reactive`, which classifies
        // ALL of them as a structural `UiCommit` — so gating on the plan's
        // `reconcile()` flag would relayout on every hover and the whole
        // optimization would be dead code in real windows. The gate is the
        // reconcile's ACTUAL result: an unchanged rebuild commits empty.
        let sheet = box_paint_only_hover_sheet();
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        runtime.set_root_view(box_view);
        runtime.set_stylesheets(vec![sheet]);
        runtime.request_frame(FrameReason::UiCommit);
        let first = runtime.run_due_frame_reactive().expect("the first frame is due");
        assert!(first.relaid_out(), "the first frame arranges the boxes");

        let node = counter_child(&runtime);
        let cold_box = runtime.layout_boxes().get(&node).copied().expect("a laid-out box");

        runtime.set_pseudo_state(node, PseudoState::HOVER);
        runtime.request_frame(FrameReason::UiCommit);
        let hot = runtime.run_due_frame_reactive().expect("the hover frame is due");

        assert!(!hot.reconciled(), "the rebuilt view is identical: reconcile commits empty");
        assert!(
            !hot.relaid_out(),
            "a paint-only hover must be layout-free on the reactive path as well"
        );
        assert!(hot.restyled(), "it still restyles so the fill reaches paint");
        assert_eq!(
            runtime.layout_boxes().get(&node).copied(),
            Some(cold_box),
            "the retained box carried over unchanged"
        );
    }

    #[test]
    fn a_reactive_rebuild_that_changes_text_still_relayouts() {
        // The safety side of the same gate: reconcile diffs text content BY VALUE,
        // so a builder whose state produced new text commits a real change and the
        // frame must lay out — text is not a computed style, and skipping here
        // would strand the old text box (the wireframe's status line does exactly
        // this on every click).
        let label = Rc::new(RefCell::new("one".to_owned()));
        let sheet = box_paint_only_hover_sheet();
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        {
            let label = label.clone();
            runtime.set_root_view(move || {
                View::from_fragment(ViewFragment::from_nodes(vec![
                    ViewNode::element(ComponentId::from_raw(10))
                        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("box".into())))
                        .with_children(ViewFragment::from_nodes(vec![ViewNode::text(
                            label.borrow().as_str(),
                        )])),
                ]))
            });
        }
        runtime.set_stylesheets(vec![sheet]);
        runtime.request_frame(FrameReason::UiCommit);
        runtime.run_due_frame_reactive().expect("the first frame is due");

        // An identical rebuild: provably layout-free (the control for the assert
        // below — it proves the next frame's relayout is caused by the TEXT).
        runtime.request_frame(FrameReason::UiCommit);
        let same = runtime.run_due_frame_reactive().expect("the repeat frame is due");
        assert!(!same.relaid_out(), "an identical rebuild is layout-free");

        // Now the state changes the text the builder emits.
        *label.borrow_mut() = "two".to_owned();
        runtime.request_frame(FrameReason::UiCommit);
        let changed = runtime.run_due_frame_reactive().expect("the commit frame is due");
        assert!(changed.reconciled(), "the new text is a real tree change");
        assert!(changed.relaid_out(), "changed text must relayout");
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
        // Three frames, each opening a "frame" span.
        let frame_spans = sink.spans.iter().filter(|s| s.name == "frame").count();
        assert_eq!(frame_spans, 3);
        // Only the FIRST frame lays out. Frames 2-3 re-render an identical view:
        // reconcile commits empty and no layout-affecting style changed, so
        // `layout_is_needed` skips the phase and the retained boxes carry over.
        let layout_spans = sink.spans.iter().filter(|s| s.name == "layout").count();
        assert_eq!(layout_spans, 1, "an unchanged re-render must not relayout");
        // 5 spans for the full first frame, 4 for each layout-free repeat.
        assert_eq!(sink.spans.len(), 5 + 2 * 4);
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
    fn press_sets_active_pseudo_and_release_clears_it() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&panel_view(), sheets);
        let (panel, _a, b) = panel_nodes(&runtime);

        // A press marks the pressed node `b` and its ancestor `panel` `:active`
        // (CSS applies `:active` up the ancestor chain, like hover).
        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 75.0));
        assert!(runtime.pseudo_state(b).contains(PseudoState::ACTIVE));
        assert!(runtime.pseudo_state(panel).contains(PseudoState::ACTIVE));

        // The matching release clears `:active` everywhere (hover/focus remain).
        runtime.handle_pointer(mouse(PointerEventKind::Up, 50.0, 75.0));
        assert!(!runtime.pseudo_state(b).contains(PseudoState::ACTIVE));
        assert!(!runtime.pseudo_state(panel).contains(PseudoState::ACTIVE));
    }

    #[test]
    fn disabled_control_takes_no_focus_and_fires_no_command() {
        use std::cell::Cell;
        use std::rc::Rc;

        use obscur_view::view::{DISABLED, EVENT_CLICK, TYPE, ViewEventBinding};

        // A single 100x50 `button` carrying the reserved DISABLED attribute and an
        // `on-click` binding. A native disabled control is inert.
        let cmd = InternedSymbol::from_raw(0x00BE_EF01);
        let view = View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(200))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("a".into())))
                .with_prop(ViewProperty::new(TYPE, ViewValue::from("button")))
                .with_prop(ViewProperty::new(DISABLED, ViewValue::Bool(true)))
                .with_event(ViewEventBinding::new(EVENT_CLICK, cmd)),
        ]));
        let sheet = Stylesheet::new(CascadeOrigin::GlobalApp, vec![class_rule("a", size(100.0, 50.0))]);

        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let ran = Rc::new(Cell::new(false));
        {
            let r = ran.clone();
            runtime.on_command(cmd, move || r.set(true));
        }
        let _ = runtime.render_now(&view, std::slice::from_ref(&sheet));

        // Press inside the disabled control: it takes NO focus.
        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
        assert_eq!(runtime.focused(), None, "a disabled control must not take focus");
        assert!(runtime.editing_text().is_none(), "a disabled control seats no editing session");

        // Release on it activates NOTHING — no command reported, no handler ran.
        let dispatch = runtime.handle_pointer(mouse(PointerEventKind::Up, 50.0, 25.0));
        assert_eq!(dispatch.command(), None, "a disabled control reports no activation command");
        assert!(!ran.get(), "a disabled control fires no activation command");

        // `:hover` is unaffected — CSS still hovers a disabled element.
        let node = panel_single_child(&runtime);
        assert!(runtime.pseudo_state(node).contains(PseudoState::HOVER));
    }

    /// The lone child node of the root after a frame (for single-element views).
    fn panel_single_child(runtime: &UiRuntime) -> NodeId {
        let root = runtime.tree().root().expect("root after a frame");
        runtime.tree().get(root).unwrap().children()[0]
    }

    // ---- Per-element vertical scroll containers (CSS Overflow 3) ----------------

    /// An `overflow: auto` decl carrying the reserved keyword handle.
    fn overflow_auto() -> BlockDeclaration {
        BlockDeclaration::normal(
            PropertyId::Overflow,
            PropertyValue::Keyword(obscur_style::property::layout_keyword::OVERFLOW_AUTO),
        )
    }

    /// A 200×120 `.scroller { overflow: auto }` holding six 200×40 `.row`s (240 px
    /// of content), plus the sheet that sizes them. Returns `(view, sheets)`.
    fn scroller_view() -> View {
        let rows: Vec<ViewNode> = (0..6)
            .map(|i| {
                ViewNode::element(ComponentId::from_raw(200 + i))
                    .with_prop(ViewProperty::new(CLASS, ViewValue::Text("row".into())))
            })
            .collect();
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(100))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("scroller".into())))
                .with_children(ViewFragment::from_nodes(rows)),
        ]))
    }

    fn scroller_sheet() -> Stylesheet {
        let mut scroller = size(200.0, 120.0);
        scroller.push(overflow_auto());
        Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![class_rule("scroller", scroller), class_rule("row", size(200.0, 40.0))],
        )
    }

    /// Returns `(scroller, [rows])` after a frame.
    fn scroller_nodes(runtime: &UiRuntime) -> (NodeId, Vec<NodeId>) {
        let root = runtime.tree().root().expect("root after a frame");
        let scroller = runtime.tree().get(root).unwrap().children()[0];
        let rows = runtime.tree().get(scroller).unwrap().children().to_vec();
        (scroller, rows)
    }

    #[test]
    fn overflowing_content_makes_a_scroll_container_with_a_clamped_scroll_top() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let sheet = scroller_sheet();
        let _ = runtime.render_now(&scroller_view(), std::slice::from_ref(&sheet));
        let (scroller, _rows) = scroller_nodes(&runtime);

        // Content (240) overflows the 120-px box → it is a scroll container at 0.
        assert_eq!(runtime.scroll_top(scroller), 0.0);
        // scroll_by advances, clamped to content_height − scrollport = 240 − 120.
        assert_eq!(runtime.scroll_by(scroller, 80.0), 80.0);
        assert_eq!(runtime.scroll_top(scroller), 80.0);
        assert_eq!(runtime.scroll_by(scroller, 1000.0), 120.0, "clamps to the scrollable range");
        assert_eq!(runtime.scroll_by(scroller, -1000.0), 0.0, "clamps at the top");
        // set_scroll_top clamps identically; a non-container node is ignored.
        runtime.set_scroll_top(scroller, 60.0);
        assert_eq!(runtime.scroll_top(scroller), 60.0);
    }

    #[test]
    fn scroll_offset_translates_content_and_reduces_to_identity_at_zero() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let sheet = scroller_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&scroller_view(), sheets);
        let (scroller, _rows) = scroller_nodes(&runtime);

        // At scroll_top 0 there is NO scroll PushTransform (identity) — the only
        // change from a plain overflow:hidden box is the scrollbar fills.
        let idle = runtime.snapshot();
        let idle_translates = idle
            .display_list()
            .commands()
            .iter()
            .filter(|c| matches!(c, PaintCommand::PushTransform(t) if t.coeffs[5] != 0.0))
            .count();
        assert_eq!(idle_translates, 0, "scroll transform is elided at scroll_top == 0");

        // Scrolled: a PushTransform(translate(0,-80)) wraps the content.
        runtime.scroll_by(scroller, 80.0);
        let _ = runtime.render_now(&scroller_view(), sheets);
        let scrolled = runtime.snapshot();
        let has = scrolled
            .display_list()
            .commands()
            .iter()
            .any(|c| matches!(c, PaintCommand::PushTransform(t) if (t.coeffs[5] + 80.0).abs() < 0.01));
        assert!(has, "scrolled content is wrapped in translate(0,-80)");
    }

    #[test]
    fn pointer_hit_test_composes_the_container_scroll_offset() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let sheet = scroller_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&scroller_view(), sheets);
        let (scroller, rows) = scroller_nodes(&runtime);

        // Idle: a click at the top of the scrollport lands on row 0.
        let hit = runtime.handle_pointer(mouse(PointerEventKind::Move, 100.0, 20.0)).target();
        assert_eq!(hit, Some(rows[0]), "idle: (100,20) hits row 0");

        // Scroll up two rows: the SAME point now maps to row 2 (content y 20+80).
        runtime.scroll_by(scroller, 80.0);
        let hit = runtime.handle_pointer(mouse(PointerEventKind::Move, 100.0, 20.0)).target();
        assert_eq!(hit, Some(rows[2]), "scrolled: (100,20) hits row 2");
    }

    #[test]
    fn absent_overflow_records_no_scroll_state_and_composes_identically() {
        // Byte-identical guard: a view with NO scroll container composes exactly as
        // before per-element scroll existed (no scroll transform, no scrollbar).
        let mut a = UiRuntime::new(runtime_id(), viewport());
        let mut b = UiRuntime::new(runtime_id(), viewport());
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let list_a = a.render_now(&panel_view(), sheets).snapshot().display_list().commands().to_vec();
        // Populate b, then drive the scroll API against a non-container node (inert).
        let _ = b.render_now(&panel_view(), sheets);
        let (panel, _a, _b) = panel_nodes(&b);
        b.set_scroll_top(panel, 40.0);
        assert_eq!(b.scroll_top(panel), 0.0, "a non-overflowing node owns no scroll position");
        let list_b = b.render_now(&panel_view(), sheets).snapshot().display_list().commands().to_vec();
        assert_eq!(list_a, list_b, "no scroll container ⇒ byte-identical display list");
    }

    // ---- Device scale (HiDPI): root scale transform + pointer coord mapping ----

    #[test]
    fn device_scale_wraps_the_display_list_in_a_root_scale_transform() {
        // A sheet where `panel` actually paints (a background), so the wrapped
        // list has real content between the brackets — not just the brackets.
        let panel_decls = {
            let mut d = size(100.0, 100.0);
            d.extend(background(0.1, 0.2, 0.3));
            d
        };
        let sheet = Stylesheet::new(CascadeOrigin::GlobalApp, vec![class_rule("panel", panel_decls)]);
        let sheets = std::slice::from_ref(&sheet);

        // Scale 2.0: the composed list is bracketed by a single root
        // PushTransform(scale(2, 2)) … PopTransform, and stays balanced.
        let mut scaled = UiRuntime::new(runtime_id(), viewport());
        scaled.set_device_scale(2.0);
        assert_eq!(scaled.device_scale(), 2.0);
        let frame = scaled.render_now(&panel_view(), sheets);
        let list = frame.snapshot().display_list();
        assert!(list.is_balanced(), "the wrapped list stays balanced");
        assert!(list.len() >= 3, "at least the two brackets plus content");
        assert_eq!(
            list.commands().first(),
            Some(&PaintCommand::PushTransform(AffineTransform::scale(2.0, 2.0))),
            "the list opens with the root device-scale transform",
        );
        assert_eq!(
            list.commands().last(),
            Some(&PaintCommand::PopTransform),
            "and closes with its matching pop",
        );

        // Scale 1.0 (default): no root transform is emitted — byte-identical to
        // the pre-device-scale path.
        let mut plain = UiRuntime::new(runtime_id(), viewport());
        let plain_frame = plain.render_now(&panel_view(), sheets);
        let plain_list = plain_frame.snapshot().display_list();
        assert!(
            !plain_list
                .commands()
                .iter()
                .any(|c| matches!(c, PaintCommand::PushTransform(_))),
            "no root transform at scale 1.0",
        );

        // The scaled list is exactly the plain list wrapped in the two brackets.
        let mut expected = vec![PaintCommand::PushTransform(AffineTransform::scale(2.0, 2.0))];
        expected.extend_from_slice(plain_list.commands());
        expected.push(PaintCommand::PopTransform);
        assert_eq!(list.commands(), expected.as_slice());
    }

    #[test]
    fn device_scale_one_is_byte_identical_to_before() {
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);

        // Default (implicit 1.0) vs an explicit set to 1.0: identical display
        // lists, and an identical pointer hit-test.
        let mut default_rt = UiRuntime::new(runtime_id(), viewport());
        let default_list = default_rt
            .render_now(&panel_view(), sheets)
            .snapshot()
            .display_list()
            .commands()
            .to_vec();
        let d_target = default_rt
            .handle_pointer(mouse(PointerEventKind::Move, 50.0, 75.0))
            .target();

        let mut explicit_rt = UiRuntime::new(runtime_id(), viewport());
        explicit_rt.set_device_scale(1.0);
        let explicit_list = explicit_rt
            .render_now(&panel_view(), sheets)
            .snapshot()
            .display_list()
            .commands()
            .to_vec();
        let e_target = explicit_rt
            .handle_pointer(mouse(PointerEventKind::Move, 50.0, 75.0))
            .target();

        assert_eq!(default_list, explicit_list, "scale 1.0 changes no commands");
        assert_eq!(d_target, e_target, "scale 1.0 changes no hit-test");
    }

    #[test]
    fn device_scale_divides_incoming_pointer_coordinates() {
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);

        // Baseline at scale 1: physical (50, 75) lands in `b` (y ∈ [50, 100)).
        let mut base = UiRuntime::new(runtime_id(), viewport());
        let _ = base.render_now(&panel_view(), sheets);
        let (_panel, a, b) = panel_nodes(&base);
        assert_eq!(
            base.handle_pointer(mouse(PointerEventKind::Move, 50.0, 75.0)).target(),
            Some(b),
        );

        // At scale 2, physical (100, 150) maps to logical (50, 75) → same `b`.
        let mut scaled = UiRuntime::new(runtime_id(), viewport());
        scaled.set_device_scale(2.0);
        let _ = scaled.render_now(&panel_view(), sheets);
        assert_eq!(
            scaled.handle_pointer(mouse(PointerEventKind::Move, 100.0, 150.0)).target(),
            Some(b),
            "physical (100,150) / 2 = logical (50,75) hits the same node as scale-1 (50,75)",
        );

        // And physical (50, 75) at scale 2 maps to logical (25, 37.5) → `a`
        // (y ∈ [0, 50)), proving the coordinates were actually divided.
        assert_eq!(
            scaled.handle_pointer(mouse(PointerEventKind::Move, 50.0, 75.0)).target(),
            Some(a),
        );
    }

    #[test]
    fn set_device_scale_clamps_to_a_sane_minimum() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        runtime.set_device_scale(0.0);
        assert_eq!(runtime.device_scale(), 0.1, "non-positive clamps to 0.1");
        runtime.set_device_scale(-4.0);
        assert_eq!(runtime.device_scale(), 0.1, "negative clamps to 0.1");
        runtime.set_device_scale(f32::NAN);
        assert_eq!(runtime.device_scale(), 0.1, "non-finite clamps to 0.1");
        runtime.set_device_scale(2.5);
        assert_eq!(runtime.device_scale(), 2.5, "a sane value passes through");
    }

    #[test]
    fn set_surface_maps_physical_extent_to_a_logical_viewport() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        // A 1000×600 physical surface at 2× density → a 500×300 logical viewport.
        runtime.set_surface(Size::new(1000.0, 600.0), 2.0);
        assert_eq!(runtime.device_scale(), 2.0);
        assert_eq!(runtime.viewport(), Size::new(500.0, 300.0));
        // At 1× the viewport equals the physical extent (no scaling).
        runtime.set_surface(Size::new(890.0, 1140.0), 1.0);
        assert_eq!(runtime.viewport(), Size::new(890.0, 1140.0));
        // The viewport divides by the *clamped* scale, so the two never disagree.
        runtime.set_surface(Size::new(100.0, 100.0), 0.0);
        assert_eq!(runtime.device_scale(), 0.1);
        assert_eq!(runtime.viewport(), Size::new(1000.0, 1000.0));
    }

    // ---- Content scroll offset: root translate + pointer coord mapping ----

    #[test]
    fn content_offset_translates_the_display_list_root() {
        let panel_decls = {
            let mut d = size(100.0, 100.0);
            d.extend(background(0.1, 0.2, 0.3));
            d
        };
        let sheet = Stylesheet::new(CascadeOrigin::GlobalApp, vec![class_rule("panel", panel_decls)]);
        let sheets = std::slice::from_ref(&sheet);

        // Offset (0, 100) at scale 1.0: the list is bracketed by a single root
        // PushTransform of the pure translate(0, -100) … PopTransform.
        let mut scrolled = UiRuntime::new(runtime_id(), viewport());
        scrolled.set_content_offset(0.0, 100.0);
        assert_eq!(scrolled.content_offset(), (0.0, 100.0));
        let frame = scrolled.render_now(&panel_view(), sheets);
        let list = frame.snapshot().display_list();
        assert!(list.is_balanced(), "the wrapped list stays balanced");
        assert_eq!(
            list.commands().first(),
            Some(&PaintCommand::PushTransform(AffineTransform::translate(0.0, -100.0))),
            "offset 100 translates content up by 100 (scale·translate reduces to translate at s=1)",
        );
        assert_eq!(list.commands().last(), Some(&PaintCommand::PopTransform));

        // Offset (0, 0) at scale 1.0: no root transform emitted (byte-identical).
        let mut plain = UiRuntime::new(runtime_id(), viewport());
        let plain_frame = plain.render_now(&panel_view(), sheets);
        let plain_list = plain_frame.snapshot().display_list();
        assert!(
            !plain_list
                .commands()
                .iter()
                .any(|c| matches!(c, PaintCommand::PushTransform(_))),
            "no root transform at offset (0,0) + scale 1.0",
        );
        let mut expected = vec![PaintCommand::PushTransform(AffineTransform::translate(0.0, -100.0))];
        expected.extend_from_slice(plain_list.commands());
        expected.push(PaintCommand::PopTransform);
        assert_eq!(list.commands(), expected.as_slice());
    }

    #[test]
    fn content_offset_composes_with_device_scale() {
        let panel_decls = {
            let mut d = size(100.0, 100.0);
            d.extend(background(0.1, 0.2, 0.3));
            d
        };
        let sheet = Stylesheet::new(CascadeOrigin::GlobalApp, vec![class_rule("panel", panel_decls)]);
        let sheets = std::slice::from_ref(&sheet);

        // scale 2 + offset (0, 100) → root = scale(2)·translate(0, -100).
        let mut rt = UiRuntime::new(runtime_id(), viewport());
        rt.set_device_scale(2.0);
        rt.set_content_offset(0.0, 100.0);
        let frame = rt.render_now(&panel_view(), sheets);
        let list = frame.snapshot().display_list();
        let expected_root =
            AffineTransform::scale(2.0, 2.0).multiply(AffineTransform::translate(0.0, -100.0));
        assert_eq!(
            list.commands().first(),
            Some(&PaintCommand::PushTransform(expected_root)),
        );
        assert!(list.is_balanced());
    }

    #[test]
    fn content_offset_maps_incoming_pointer_coordinates() {
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);

        // Baseline at offset 0: physical (50, 25) lands in `a` (y ∈ [0, 50)).
        let mut base = UiRuntime::new(runtime_id(), viewport());
        let _ = base.render_now(&panel_view(), sheets);
        let (_panel, a, b) = panel_nodes(&base);
        assert_eq!(
            base.handle_pointer(mouse(PointerEventKind::Move, 50.0, 25.0)).target(),
            Some(a),
        );

        // With offset (0, 100): physical (50, 25) maps to content y = 25/1 + 100
        // = 125 → out of the 100-tall panel → no target; and physical (50, -75)
        // maps to content y = -75 + 100 = 25 → `a`. Prove the offset is added.
        let mut scrolled = UiRuntime::new(runtime_id(), viewport());
        scrolled.set_content_offset(0.0, 100.0);
        let _ = scrolled.render_now(&panel_view(), sheets);
        assert_eq!(
            scrolled.handle_pointer(mouse(PointerEventKind::Move, 50.0, -75.0)).target(),
            Some(a),
            "physical y -75 + offset 100 = content y 25 hits `a`",
        );

        // physical (50, -25) → content y = 75 → `b` (y ∈ [50, 100)).
        assert_eq!(
            scrolled.handle_pointer(mouse(PointerEventKind::Move, 50.0, -25.0)).target(),
            Some(b),
        );
    }

    #[test]
    fn content_offset_maps_pointer_with_device_scale() {
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);

        // scale 2 + offset (0, 100): content y = physical / 2 + 100.
        let mut rt = UiRuntime::new(runtime_id(), viewport());
        rt.set_device_scale(2.0);
        rt.set_content_offset(0.0, 100.0);
        let _ = rt.render_now(&panel_view(), sheets);
        let (_panel, _a, b) = panel_nodes(&rt);
        // physical y -50 → -25 + 100 = content y 75 → `b`.
        assert_eq!(
            rt.handle_pointer(mouse(PointerEventKind::Move, 100.0, -50.0)).target(),
            Some(b),
            "physical (100,-50): x=100/2=50, y=-50/2+100=75 → `b`",
        );
    }

    #[test]
    fn content_offset_zero_is_byte_identical_and_pointer_unchanged() {
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);

        let mut default_rt = UiRuntime::new(runtime_id(), viewport());
        let default_list = default_rt
            .render_now(&panel_view(), sheets)
            .snapshot()
            .display_list()
            .commands()
            .to_vec();
        let d_target = default_rt
            .handle_pointer(mouse(PointerEventKind::Move, 50.0, 75.0))
            .target();

        let mut explicit_rt = UiRuntime::new(runtime_id(), viewport());
        explicit_rt.set_content_offset(0.0, 0.0);
        assert_eq!(explicit_rt.content_offset(), (0.0, 0.0));
        let explicit_list = explicit_rt
            .render_now(&panel_view(), sheets)
            .snapshot()
            .display_list()
            .commands()
            .to_vec();
        let e_target = explicit_rt
            .handle_pointer(mouse(PointerEventKind::Move, 50.0, 75.0))
            .target();

        assert_eq!(default_list, explicit_list, "offset (0,0) changes no commands");
        assert_eq!(d_target, e_target, "offset (0,0) changes no hit-test");
    }

    #[test]
    fn set_content_offset_ignores_non_finite_components() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        runtime.set_content_offset(10.0, 20.0);
        runtime.set_content_offset(f32::NAN, 40.0);
        assert_eq!(runtime.content_offset(), (10.0, 40.0), "NaN x keeps prior x");
        runtime.set_content_offset(50.0, f32::INFINITY);
        assert_eq!(runtime.content_offset(), (50.0, 40.0), "inf y keeps prior y");
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

    /// The `panel` view (same geometry as [`panel_view`]) but with click
    /// bindings: the panel binds `on-click → CMD_PANEL`, its child `a` is
    /// unbound (so a click on it bubbles to the panel), and its child `b` binds
    /// `on-click → CMD_B` (so a click on it fires directly).
    fn bound_view() -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(200))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into())))
                .with_event(ViewEventBinding::new(EVENT_CLICK, CMD_PANEL))
                .with_children(ViewFragment::from_nodes(vec![
                    ViewNode::element(ComponentId::from_raw(201))
                        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("a".into()))),
                    ViewNode::element(ComponentId::from_raw(202))
                        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("b".into())))
                        .with_event(ViewEventBinding::new(EVENT_CLICK, CMD_B)),
                ])),
        ]))
    }

    #[test]
    fn click_on_bound_node_runs_its_registered_handler() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&bound_view(), sheets);
        let (_panel, _a, b) = panel_nodes(&runtime);

        // Register the handler for `b`'s command with the SAME symbol the View
        // bound `on-click` to — the runtime only compares symbols.
        let counter = Rc::new(Cell::new(0));
        {
            let c = counter.clone();
            runtime.on_command(CMD_B, move || c.set(c.get() + 1));
        }
        assert!(runtime.has_command_handler(CMD_B));

        // Down then Up on `b` (50, 75) is a click on `b`.
        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 75.0));
        let up = runtime.handle_pointer(mouse(PointerEventKind::Up, 50.0, 75.0));

        assert_eq!(up.target(), Some(b));
        assert_eq!(up.command(), Some(CMD_B), "the click resolved b's command");
        assert_eq!(counter.get(), 1, "the registered handler RAN exactly once");
    }

    #[test]
    fn press_and_release_on_different_nodes_do_not_dispatch() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&bound_view(), sheets);

        let counter = Rc::new(Cell::new(0));
        {
            let c = counter.clone();
            runtime.on_command(CMD_B, move || c.set(c.get() + 1));
        }
        {
            let c = counter.clone();
            runtime.on_command(CMD_PANEL, move || c.set(c.get() + 1));
        }

        // Press on `a` (50, 25), release on `b` (50, 75): not a click on either.
        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
        let up = runtime.handle_pointer(mouse(PointerEventKind::Up, 50.0, 75.0));

        assert_eq!(up.command(), None, "a drag across nodes is not a click");
        assert_eq!(counter.get(), 0, "no handler ran");
    }

    #[test]
    fn click_on_unbound_child_bubbles_to_the_bound_ancestor() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&bound_view(), sheets);
        let (_panel, a, _b) = panel_nodes(&runtime);

        let counter = Rc::new(Cell::new(0));
        {
            let c = counter.clone();
            runtime.on_command(CMD_PANEL, move || c.set(c.get() + 1));
        }

        // `a` (50, 25) has no click binding; the click bubbles up to the panel.
        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
        let up = runtime.handle_pointer(mouse(PointerEventKind::Up, 50.0, 25.0));

        assert_eq!(up.target(), Some(a));
        assert_eq!(
            up.command(),
            Some(CMD_PANEL),
            "the click bubbled to the bound ancestor"
        );
        assert_eq!(counter.get(), 1, "the ancestor's handler ran");
    }

    #[test]
    fn click_with_no_registered_handler_is_a_harmless_noop() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&bound_view(), sheets);

        // `b` binds a command in the View, but NO handler is registered for it.
        assert!(!runtime.has_command_handler(CMD_B));
        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 75.0));
        let up = runtime.handle_pointer(mouse(PointerEventKind::Up, 50.0, 75.0));

        // The command still resolves (it is bound on the node), but nothing ran
        // and nothing panicked.
        assert_eq!(up.command(), Some(CMD_B));
    }

    #[test]
    fn dispatching_a_click_requests_a_frame() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&bound_view(), sheets);

        let counter = Rc::new(Cell::new(0));
        {
            let c = counter.clone();
            runtime.on_command(CMD_B, move || c.set(c.get() + 1));
        }

        // The press moves focus → requests a frame; drain it so the pending
        // state is clean before the release.
        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 75.0));
        let _ = runtime.run_due_frame(&bound_view(), sheets);
        assert!(!runtime.has_pending_frame(), "the press frame drained");

        // The release completes the click: the dispatch alone requests a frame
        // (hover and focus are unchanged from the press on the same node).
        runtime.handle_pointer(mouse(PointerEventKind::Up, 50.0, 75.0));
        assert!(
            runtime.has_pending_frame(),
            "dispatching the click requested a re-render frame"
        );
        assert_eq!(counter.get(), 1);
    }

    // ---- Phase 2 step 3: the reactive re-render loop (held view builder) -----

    /// App-owned increment command the reactive counter binds and registers.
    const CMD_INC: InternedSymbol = InternedSymbol::from_raw(0x1C0);

    /// A reactive counter view: a `panel` (100x100) holding a single `b`
    /// (100x50, at y∈[0,50)) that binds `on-click → CMD_INC` and contains a text
    /// node showing `count`. A click anywhere in `b` (e.g. `(50, 25)`) fires the
    /// increment command; the text node is what the assertions read back.
    fn counter_reactive_view(count: i32) -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(300))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into())))
                .with_children(ViewFragment::from_nodes(vec![
                    ViewNode::element(ComponentId::from_raw(301))
                        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("b".into())))
                        .with_event(ViewEventBinding::new(EVENT_CLICK, CMD_INC))
                        .with_children(ViewFragment::from_nodes(vec![ViewNode::text(
                            count.to_string(),
                        )])),
                ])),
        ]))
    }

    /// Returns the content of the first text node in the retained tree — the
    /// rendered counter value the reactive loop rebuilt.
    fn tree_text(runtime: &UiRuntime) -> String {
        runtime
            .tree()
            .iter()
            .find_map(|(_, node)| node.kind().text().map(str::to_owned))
            .expect("the retained tree has a text node after a frame")
    }

    #[test]
    fn reactive_loop_rebuilds_view_on_click_end_to_end() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());

        // The counter state shared between the click handler (which mutates it)
        // and the view builder (which reads it) — an `Rc<Cell<i32>>`.
        let count = Rc::new(Cell::new(0));
        {
            let c = count.clone();
            runtime.on_command(CMD_INC, move || c.set(c.get() + 1));
        }
        {
            let c = count.clone();
            runtime.set_root_view(move || counter_reactive_view(c.get()));
        }
        runtime.set_stylesheets(vec![interaction_sheet()]);

        // First reactive frame: schedule one, run it, the tree shows "0".
        runtime.request_frame(FrameReason::UiCommit);
        let first = runtime
            .run_due_frame_reactive()
            .expect("the first reactive frame is due");
        assert!(first.reconciled(), "the first reactive frame reconciles");
        assert_eq!(tree_text(&runtime), "0", "the counter starts at 0");

        // Click on `b` (50, 25): Down then Up on the same node dispatches the
        // increment command, which bumps the state and requests a frame.
        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
        let up = runtime.handle_pointer(mouse(PointerEventKind::Up, 50.0, 25.0));
        assert_eq!(up.command(), Some(CMD_INC), "the click fired the increment");
        assert_eq!(count.get(), 1, "the handler mutated the shared state");
        assert!(runtime.has_pending_frame(), "the click requested a frame");

        // Second reactive frame: the builder REBUILDS from the mutated state, so
        // the reconciled tree now shows "1" — proving click→state→re-render.
        runtime
            .run_due_frame_reactive()
            .expect("the click-requested reactive frame is due");
        assert_eq!(tree_text(&runtime), "1", "the rebuilt view shows 1");

        // A second click → the loop runs again → the tree shows "2".
        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
        runtime.handle_pointer(mouse(PointerEventKind::Up, 50.0, 25.0));
        assert_eq!(count.get(), 2);
        runtime
            .run_due_frame_reactive()
            .expect("the second click's reactive frame is due");
        assert_eq!(tree_text(&runtime), "2", "the loop is repeatable");
    }

    #[test]
    fn reactive_frame_re_renders_a_changed_builder_output() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        // Drive the builder's output directly (no click) to prove the reactive
        // frame reflects whatever the builder returns each time it is called.
        let value = Rc::new(Cell::new(7));
        {
            let v = value.clone();
            runtime.set_root_view(move || counter_reactive_view(v.get()));
        }
        runtime.set_stylesheets(vec![interaction_sheet()]);

        runtime.request_frame(FrameReason::UiCommit);
        runtime.run_due_frame_reactive().expect("first frame due");
        assert_eq!(tree_text(&runtime), "7");

        // Mutate the state the builder reads, request another frame, run it: the
        // rebuilt tree reflects the new value.
        value.set(42);
        runtime.request_frame(FrameReason::UiCommit);
        runtime.run_due_frame_reactive().expect("second frame due");
        assert_eq!(tree_text(&runtime), "42", "the builder's new output rendered");
    }

    #[test]
    fn reactive_frame_with_no_builder_returns_none() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        assert!(!runtime.has_root_view());
        // A frame is pending, but with no builder the reactive path is a no-op
        // that returns None and leaves the pending request untouched.
        runtime.request_frame(FrameReason::UiCommit);
        assert!(runtime.run_due_frame_reactive().is_none());
        assert!(
            runtime.has_pending_frame(),
            "the no-op reactive path did not drain the pending frame"
        );
    }

    #[test]
    fn reactive_frame_returns_none_when_nothing_is_due() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        runtime.set_root_view(|| counter_reactive_view(0));
        runtime.set_stylesheets(vec![interaction_sheet()]);
        // No frame requested → nothing due → None, no rebuild, no side effects.
        assert!(!runtime.has_pending_frame());
        assert!(runtime.run_due_frame_reactive().is_none());
        assert!(runtime.tree().root().is_none(), "no frame ran");
    }

    #[test]
    fn explicit_view_path_is_unaffected_by_an_installed_builder() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        // Install a builder that would render "99", then drive the EXPLICIT
        // `&View` path with a different view: the explicit path must ignore the
        // held builder entirely and render what it was handed.
        runtime.set_root_view(|| counter_reactive_view(99));
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);

        let result = runtime.render_now(&counter_reactive_view(3), sheets);
        assert!(result.reconciled());
        assert_eq!(
            tree_text(&runtime),
            "3",
            "render_now rendered the passed view, not the builder's"
        );

        // The scheduler-gated `&View` path is likewise unaffected.
        runtime.request_frame(FrameReason::UiCommit);
        runtime
            .run_due_frame(&counter_reactive_view(5), sheets)
            .expect("frame due");
        assert_eq!(tree_text(&runtime), "5", "run_due_frame used the passed view");
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

        // Ctrl+S → CMD_SAVE.
        runtime.bind_command(
            Shortcut::new(LogicalKey::character('s')).with(Modifier::Ctrl),
            CMD_SAVE,
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
        assert_eq!(save.command(), Some(CMD_SAVE));
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
        runtime.bind_command(Shortcut::new(LogicalKey::character('a')), CMD_SAVE);

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
                CMD_SAVE,
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

    // ---- Keyboard command + activation dispatch (this step) ----------------

    /// A `panel` (100x100) holding a single child `b` (100x50, at y∈[0,50)) that
    /// binds `on-change → CMD_CHECK` — a checkbox-like control that fires its
    /// *change* command (not a click command) when activated.
    fn checkbox_view() -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(400))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into())))
                .with_children(ViewFragment::from_nodes(vec![
                    ViewNode::element(ComponentId::from_raw(401))
                        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("b".into())))
                        .with_event(ViewEventBinding::new(EVENT_CHANGE, CMD_CHECK)),
                ])),
        ]))
    }

    #[test]
    fn bound_shortcut_key_press_runs_its_registered_handler() {
        // (b) A keyboard shortcut resolves to a command symbol AND runs the
        // handler registered for that same symbol — keyboard now dispatches.
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&panel_view(), sheets);

        // Bind Ctrl+S to CMD_SAVE and register the handler for the SAME symbol.
        runtime.bind_command(
            Shortcut::new(LogicalKey::character('s')).with(Modifier::Ctrl),
            CMD_SAVE,
        );
        let counter = Rc::new(Cell::new(0));
        {
            let c = counter.clone();
            runtime.on_command(CMD_SAVE, move || c.set(c.get() + 1));
        }

        // Focus a node so the key routes, then hold Ctrl and press S.
        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
        let _ = runtime.run_due_frame(&panel_view(), sheets);
        runtime.handle_key(KeyEvent::press(PhysicalKey::ControlLeft, None));
        let save = runtime.handle_key(KeyEvent::press(
            PhysicalKey::KeyS,
            Some(LogicalKey::character('s')),
        ));

        assert_eq!(save.command(), Some(CMD_SAVE), "the shortcut resolved");
        assert_eq!(counter.get(), 1, "the bound shortcut RAN its handler once");
    }

    #[test]
    fn clicking_a_checkbox_fires_its_change_command_via_activate() {
        // (c) A checkbox-like node bound only to EVENT_CHANGE fires that change
        // command when clicked (Down+Up) — activation resolves click ELSE change.
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&checkbox_view(), sheets);

        let counter = Rc::new(Cell::new(0));
        {
            let c = counter.clone();
            runtime.on_command(CMD_CHECK, move || c.set(c.get() + 1));
        }

        // The single child `b` occupies y∈[0,50); click at (50, 25).
        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
        let up = runtime.handle_pointer(mouse(PointerEventKind::Up, 50.0, 25.0));

        assert_eq!(
            up.command(),
            Some(CMD_CHECK),
            "the click resolved the change command"
        );
        assert_eq!(counter.get(), 1, "the change handler RAN once on click");
    }

    #[test]
    fn enter_on_focused_button_fires_its_click_command_via_activate() {
        // (d) Enter on a focused button (bound to EVENT_CLICK) runs the click
        // handler even with no keyboard shortcut bound.
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&bound_view(), sheets);
        let (_panel, _a, b) = panel_nodes(&runtime);

        let counter = Rc::new(Cell::new(0));
        {
            let c = counter.clone();
            runtime.on_command(CMD_B, move || c.set(c.get() + 1));
        }

        // Focus `b` (bound to EVENT_CLICK → CMD_B) via a press at (50, 75), drain
        // the focus frame, then press Enter: activation fires b's click command.
        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 75.0));
        let _ = runtime.run_due_frame(&bound_view(), sheets);
        assert_eq!(runtime.focused(), Some(b));

        let enter = runtime.handle_key(KeyEvent::press(
            PhysicalKey::Enter,
            Some(LogicalKey::named(NamedKey::Enter)),
        ));
        assert_eq!(enter.target(), Some(b));
        // No shortcut is bound, so nothing resolved through the router...
        assert_eq!(enter.command(), None);
        // ...but Enter activated the focused button's click command.
        assert_eq!(counter.get(), 1, "Enter activated the button's click handler");
    }

    #[test]
    fn space_on_focused_checkbox_fires_change_and_missing_handler_is_a_noop() {
        // (e-adjacent) Space activates too, firing the checkbox's change command;
        // and an activation whose command has no registered handler never panics.
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&checkbox_view(), sheets);

        // Focus the checkbox with NO handler registered: Space must not panic.
        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
        let _ = runtime.run_due_frame(&checkbox_view(), sheets);
        let space_noop = runtime.handle_key(KeyEvent::press(
            PhysicalKey::Space,
            Some(LogicalKey::named(NamedKey::Space)),
        ));
        assert!(space_noop.is_handled());

        // Now register the handler; a Space keypress fires the change command.
        let counter = Rc::new(Cell::new(0));
        {
            let c = counter.clone();
            runtime.on_command(CMD_CHECK, move || c.set(c.get() + 1));
        }
        runtime.handle_key(KeyEvent::press(
            PhysicalKey::Space,
            Some(LogicalKey::named(NamedKey::Space)),
        ));
        assert_eq!(counter.get(), 1, "Space activated the change handler");
    }

    #[test]
    fn button_click_still_fires_event_click_after_activation_change() {
        // (e) Regression: a button click still dispatches EVENT_CLICK (activation
        // prefers the click binding over any change binding).
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&bound_view(), sheets);
        let (_panel, _a, b) = panel_nodes(&runtime);

        let counter = Rc::new(Cell::new(0));
        {
            let c = counter.clone();
            runtime.on_command(CMD_B, move || c.set(c.get() + 1));
        }

        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 75.0));
        let up = runtime.handle_pointer(mouse(PointerEventKind::Up, 50.0, 75.0));

        assert_eq!(up.target(), Some(b));
        assert_eq!(up.command(), Some(CMD_B), "click still resolves EVENT_CLICK");
        assert_eq!(counter.get(), 1);
    }

    // ---- EVENT_INPUT: per-keystroke text input dispatch (this step) ----------

    /// App-owned command an editable binds `on-input` to and registers a
    /// text-input handler for.
    const CMD_INPUT: InternedSymbol = InternedSymbol::from_raw(0x1247);

    /// A reactive editable view: a `panel` (100x100) holding a single `b`
    /// (100x50, at y∈[0,50)) that binds `on-input → CMD_INPUT` (per-keystroke)
    /// and contains a text node showing the current `text`. Pressing inside `b`
    /// focuses the editable; the text node is what the reactive assertions read
    /// back (via [`tree_text`]).
    fn input_reactive_view(text: &str) -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(500))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into())))
                .with_children(ViewFragment::from_nodes(vec![
                    ViewNode::element(ComponentId::from_raw(501))
                        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("b".into())))
                        .with_event(ViewEventBinding::new(EVENT_INPUT, CMD_INPUT))
                        .with_children(ViewFragment::from_nodes(vec![ViewNode::text(
                            text.to_string(),
                        )])),
                ])),
        ]))
    }

    /// A plain (no-modifier) character key press. The physical key is irrelevant
    /// to text entry (only modifier folding reads it), so any non-modifier
    /// physical stands in.
    fn char_press(c: char) -> KeyEvent {
        KeyEvent::press(PhysicalKey::KeyA, Some(LogicalKey::character(c)))
    }

    /// An input-like view whose EMPTY editable shows its placeholder inside a
    /// marked `.placeholder` overlay child (mirroring the `Input` widget), rather
    /// than as an editable text value — so the runtime must SKIP the placeholder
    /// when seeding the editing buffer (empty focus → empty buffer).
    fn placeholder_input_view() -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(500))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into())))
                .with_children(ViewFragment::from_nodes(vec![
                    ViewNode::element(ComponentId::from_raw(501))
                        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("b".into())))
                        .with_event(ViewEventBinding::new(EVENT_INPUT, CMD_INPUT))
                        .with_children(ViewFragment::from_nodes(vec![
                            ViewNode::element(ComponentId::from_raw(502))
                                .with_prop(ViewProperty::new(
                                    CLASS,
                                    ViewValue::Text("placeholder".into()),
                                ))
                                .with_children(ViewFragment::from_nodes(vec![ViewNode::text(
                                    "Type here…".to_string(),
                                )])),
                        ])),
                ])),
        ]))
    }

    #[test]
    fn typing_into_a_focused_editable_edits_app_state_and_re_renders() {
        // (b) End-to-end: typing 'h' then 'i' into a focused `on-input` editable
        // drives the runtime-owned editing session; after each mutating edit the
        // app mirrors the RESULTING TEXT into its own Rc<RefCell<String>>, which
        // the reactive builder reads back so the re-render shows "hi", and
        // Backspace deletes back to "h".
        let mut runtime = UiRuntime::new(runtime_id(), viewport());

        let text = Rc::new(RefCell::new(String::new()));
        {
            let s = text.clone();
            runtime.on_text_input(CMD_INPUT, move |resulting| {
                *s.borrow_mut() = resulting.to_owned();
            });
        }
        {
            let s = text.clone();
            runtime.set_root_view(move || {
                let current = s.borrow().clone();
                input_reactive_view(&current)
            });
        }
        runtime.set_stylesheets(vec![interaction_sheet()]);

        // First reactive frame populates the tree/boxes so the press can focus.
        runtime.request_frame(FrameReason::UiCommit);
        runtime.run_due_frame_reactive().expect("first frame due");

        // Focus the editable by pressing inside `b`, drain the focus frame.
        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
        runtime.run_due_frame_reactive().expect("focus frame due");
        assert!(runtime.focused().is_some(), "the press focused a node");

        // Type 'h' then 'i': each keystroke edits the app's own string.
        let h = runtime.handle_key(char_press('h'));
        assert_eq!(h.command(), None, "a plain char resolves no shortcut command");
        runtime.handle_key(char_press('i'));
        assert_eq!(&*text.borrow(), "hi", "typing edited the app's string");

        // The reactive re-render reflects the edited string.
        runtime.run_due_frame_reactive().expect("re-render frame due");
        assert_eq!(tree_text(&runtime), "hi", "the rebuilt view shows the text");

        // Backspace deletes the last character; the re-render follows.
        runtime.handle_key(KeyEvent::press(
            PhysicalKey::Backspace,
            Some(LogicalKey::named(NamedKey::Backspace)),
        ));
        assert_eq!(&*text.borrow(), "h", "Backspace deleted the last char");
        runtime.run_due_frame_reactive().expect("re-render frame due");
        assert_eq!(tree_text(&runtime), "h", "the rebuilt view shows the deletion");
    }

    #[test]
    fn focusing_an_empty_placeholdered_field_seeds_an_empty_buffer_then_typing_yields_just_the_typed_text()
     {
        // Placeholder is a non-editable overlay (CSS `::placeholder` /
        // `:placeholder-shown`), NOT the field value. Focusing an empty
        // placeholdered field must seed an EMPTY editing buffer (caret at 0) — the
        // `.placeholder` hint text must never leak in — and typing 'x' must yield
        // exactly "x", never "Type here…x".
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let mirror = Rc::new(RefCell::new(String::new()));
        {
            let s = mirror.clone();
            runtime.on_text_input(CMD_INPUT, move |resulting| {
                *s.borrow_mut() = resulting.to_owned();
            });
        }
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&placeholder_input_view(), sheets);

        // Focus the editable by pressing inside it; the runtime auto-seeds.
        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
        assert!(runtime.focused().is_some(), "the press focused the editable");
        assert_eq!(
            runtime.editing_text(),
            Some(""),
            "an empty placeholdered field seeds an EMPTY buffer, not the placeholder text"
        );
        let sel = runtime.editing_selection().expect("session active");
        assert!(sel.is_collapsed());
        assert_eq!(sel.focus(), 0, "caret seeded at offset 0 in the empty field");

        runtime.handle_key(char_press('x'));
        assert_eq!(runtime.editing_text(), Some("x"), "typing yields exactly the typed text");
        assert_eq!(&*mirror.borrow(), "x", "on-input delivered exactly \"x\", not the placeholder");
    }

    #[test]
    fn ctrl_char_on_a_focused_editable_is_not_inserted_as_text() {
        // (c) A command modifier makes a char an editing chord (Ctrl+A =
        // SelectAll) or a shortcut, never inserted text: Ctrl+'a' on a focused
        // editable changes no text, so the app's mirror stays empty.
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let text = Rc::new(RefCell::new(String::new()));
        {
            let s = text.clone();
            runtime.on_text_input(CMD_INPUT, move |resulting| {
                *s.borrow_mut() = resulting.to_owned();
            });
        }
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&input_reactive_view(""), sheets);

        // Focus the editable.
        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
        let _ = runtime.run_due_frame(&input_reactive_view(""), sheets);
        assert!(runtime.focused().is_some());

        // Hold Ctrl, then press 'a': it is treated as a (possible) shortcut.
        runtime.handle_key(KeyEvent::press(PhysicalKey::ControlLeft, None));
        runtime.handle_key(KeyEvent::press(
            PhysicalKey::KeyA,
            Some(LogicalKey::character('a')),
        ));
        assert_eq!(&*text.borrow(), "", "Ctrl+a was not inserted as text");
    }

    #[test]
    fn char_press_with_no_focused_editable_does_not_dispatch_text() {
        // (d) A char over a focused NON-editable node (no `on-input` in its
        // ancestor chain) is not text — the shortcut/activation path is
        // byte-identical and no TextEdit is dispatched.
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&panel_view(), sheets);
        let (_panel, a, _b) = panel_nodes(&runtime);

        let text = Rc::new(RefCell::new(String::new()));
        {
            let s = text.clone();
            runtime.on_text_input(CMD_INPUT, move |resulting| {
                *s.borrow_mut() = resulting.to_owned();
            });
        }

        // Focus `a`, which binds no `on-input`.
        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
        let _ = runtime.run_due_frame(&panel_view(), sheets);
        assert_eq!(runtime.focused(), Some(a));

        let dispatch = runtime.handle_key(char_press('a'));
        assert_eq!(dispatch.target(), Some(a), "routed to the focused node");
        assert_eq!(dispatch.command(), None);
        assert_eq!(&*text.borrow(), "", "no editable → no text dispatched");
    }

    #[test]
    fn text_input_with_no_registered_handler_is_a_harmless_noop() {
        // (e) A focused editable bound to `on-input` but with NO handler
        // registered: typing consumes the key as text (so it is not re-read as a
        // shortcut) yet dispatches nothing and never panics.
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&input_reactive_view(""), sheets);

        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
        let _ = runtime.run_due_frame(&input_reactive_view(""), sheets);
        assert!(runtime.focused().is_some());
        assert!(!runtime.has_text_input_handler(CMD_INPUT));

        let dispatch = runtime.handle_key(char_press('h'));
        assert!(dispatch.is_handled(), "the key still routed to the focused node");
        assert_eq!(dispatch.command(), None, "no shortcut resolved");
    }

    // ---- Runtime-owned editing session: caret + selection (this step) --------

    /// A key press carrying a named (non-character) logical key. The physical
    /// key is irrelevant to editing (only modifier folding reads it), so a
    /// non-modifier physical stands in.
    fn named_press(named: NamedKey) -> KeyEvent {
        KeyEvent::press(PhysicalKey::KeyA, Some(LogicalKey::named(named)))
    }

    /// Sets up a focused editable seeded (from its text child) with `seed`, an
    /// `on-input` handler mirroring the resulting text into the returned string,
    /// and the caret at the end of the seed. Uses the explicit render path.
    fn focused_editable(seed: &str) -> (UiRuntime, Rc<RefCell<String>>) {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let mirror = Rc::new(RefCell::new(String::new()));
        {
            let s = mirror.clone();
            runtime.on_text_input(CMD_INPUT, move |resulting| {
                *s.borrow_mut() = resulting.to_owned();
            });
        }
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&input_reactive_view(seed), sheets);
        // Press inside `b` to focus the editable; the runtime seeds the session.
        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
        assert!(runtime.focused().is_some(), "the press focused the editable");
        (runtime, mirror)
    }

    #[test]
    fn seed_places_caret_at_end_then_type_and_backspace() {
        // (a) A focused editable seeded with "hi" starts with the caret at the
        // end (index 2); typing 'x' -> "hix" caret 3; Backspace -> "hi".
        let (mut runtime, mirror) = focused_editable("hi");
        assert_eq!(runtime.editing_text(), Some("hi"));
        let sel = runtime.editing_selection().expect("session active");
        assert!(sel.is_collapsed());
        assert_eq!(sel.focus(), 2, "caret seeded at the end");

        runtime.handle_key(char_press('x'));
        assert_eq!(runtime.editing_text(), Some("hix"));
        assert_eq!(runtime.editing_selection().unwrap().focus(), 3);
        assert_eq!(&*mirror.borrow(), "hix", "on-input got the resulting text");

        runtime.handle_key(named_press(NamedKey::Backspace));
        assert_eq!(runtime.editing_text(), Some("hi"));
        assert_eq!(runtime.editing_selection().unwrap().focus(), 2);
        assert_eq!(&*mirror.borrow(), "hi");
    }

    #[test]
    fn arrow_and_shift_extend_then_typing_replaces_selection() {
        // (b) ArrowLeft moves the caret to 1; Shift+ArrowLeft extends the
        // selection to [0,1); typing 'z' replaces that selection -> "zi".
        let (mut runtime, _mirror) = focused_editable("hi");

        runtime.handle_key(named_press(NamedKey::ArrowLeft));
        let sel = runtime.editing_selection().unwrap();
        assert!(sel.is_collapsed());
        assert_eq!(sel.focus(), 1);

        runtime.handle_key(KeyEvent::press(PhysicalKey::ShiftLeft, None));
        runtime.handle_key(named_press(NamedKey::ArrowLeft));
        let sel = runtime.editing_selection().unwrap();
        assert!(!sel.is_collapsed(), "shift extended the selection");
        assert_eq!((sel.start(), sel.end()), (0, 1));
        runtime.handle_key(KeyEvent::release(PhysicalKey::ShiftLeft, None));

        runtime.handle_key(char_press('z'));
        assert_eq!(runtime.editing_text(), Some("zi"), "insert replaced [0,1)");
    }

    #[test]
    fn home_end_and_ctrl_a_select_all_then_replace() {
        // (c) Home -> caret 0, End -> caret at len; Ctrl+A selects all and typing
        // replaces the whole field.
        let (mut runtime, _mirror) = focused_editable("hi");

        runtime.handle_key(named_press(NamedKey::Home));
        assert_eq!(runtime.editing_selection().unwrap().focus(), 0);
        runtime.handle_key(named_press(NamedKey::End));
        assert_eq!(runtime.editing_selection().unwrap().focus(), 2);

        runtime.handle_key(KeyEvent::press(PhysicalKey::ControlLeft, None));
        runtime.handle_key(char_press('a')); // Ctrl+A -> SelectAll
        let sel = runtime.editing_selection().unwrap();
        assert_eq!((sel.start(), sel.end()), (0, 2), "select-all covers the field");
        runtime.handle_key(KeyEvent::release(PhysicalKey::ControlLeft, None));

        runtime.handle_key(char_press('Z'));
        assert_eq!(runtime.editing_text(), Some("Z"), "typing replaced everything");
    }

    #[test]
    fn mutating_edit_fires_on_input_but_caret_move_does_not() {
        // (d) A mutating edit fires EVENT_INPUT with the resulting text; a
        // caret-only move updates the session without firing on-input or
        // corrupting the text.
        let (mut runtime, mirror) = focused_editable("hi");

        runtime.handle_key(char_press('!'));
        assert_eq!(&*mirror.borrow(), "hi!", "mutating edit delivered the text");

        // A sentinel proves the caret move does not re-fire the handler.
        *mirror.borrow_mut() = String::from("SENTINEL");
        runtime.handle_key(named_press(NamedKey::ArrowLeft));
        assert_eq!(runtime.editing_text(), Some("hi!"), "text intact after a move");
        assert_eq!(&*mirror.borrow(), "SENTINEL", "a caret move does not fire on-input");
    }

    // ---- Controlled-value reseed while focused (Defect A) --------------------
    // The HTML controlled-input model (WHATWG HTML §4.10.5.5): a text control
    // reflects its `value`, so an app rewrite of the bound value MID-FOCUS must
    // reach the display. The session (the sole source of truth for the focused
    // field, an uncontrolled input) reseeds ONLY on an EXTERNAL rewrite — never on
    // the user's own keystroke mirrored back.

    /// Drives a reactive, controlled editable: `transform` maps each keystroke's
    /// resulting text to the value the app stores (identity = passthrough,
    /// uppercase = a normalizing handler), which the reactive builder reflects back
    /// into the field. Returns the runtime plus the shared app value. Focus lands
    /// on the field (empty), ready for keystrokes.
    fn controlled_field(
        transform: impl Fn(&str) -> String + 'static,
    ) -> (UiRuntime, Rc<RefCell<String>>) {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let value = Rc::new(RefCell::new(String::new()));
        {
            let s = value.clone();
            runtime.on_text_input(CMD_INPUT, move |resulting| {
                *s.borrow_mut() = transform(resulting);
            });
        }
        {
            let s = value.clone();
            runtime.set_root_view(move || input_reactive_view(&s.borrow()));
        }
        runtime.set_stylesheets(vec![interaction_sheet()]);
        // First reactive frame builds the tree/boxes; then focus the field.
        runtime.request_frame(FrameReason::UiCommit);
        runtime.run_due_frame_reactive().expect("first frame");
        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
        runtime.run_due_frame_reactive().expect("focus frame");
        assert!(runtime.focused().is_some(), "the press focused the field");
        (runtime, value)
    }

    #[test]
    fn external_value_rewrite_mid_focus_reseeds_the_session_and_shows_it() {
        // (A1) Type into a focused field, then an app `set` of a DIFFERENT value
        // mid-focus: the session reseeds and compose shows the NEW value, not the
        // stale buffer that used to persist until blur.
        let (mut runtime, value) = controlled_field(str::to_owned); // passthrough
        for c in "abc".chars() {
            runtime.handle_key(char_press(c));
        }
        runtime.run_due_frame_reactive().expect("rebuild frame");
        assert_eq!(runtime.editing_text(), Some("abc"), "typing filled the session");
        assert_eq!(tree_text(&runtime), "abc", "the passthrough value reached the tree");

        // The app rewrites the bound value to something else while still focused
        // (a "Clear"-style set, out of band from the session's own edits).
        *value.borrow_mut() = "reset".to_string();
        runtime.request_frame(FrameReason::UiCommit);
        runtime.run_due_frame_reactive().expect("external-rewrite frame");

        assert_eq!(
            runtime.editing_text(),
            Some("reset"),
            "the session reseeded from the external value (not the stale buffer)"
        );
        assert_eq!(tree_text(&runtime), "reset");
        let sel = runtime.editing_selection().expect("session active");
        assert!(
            sel.is_collapsed() && sel.focus() <= "reset".len(),
            "the caret is clamped into the new value (sane)"
        );
    }

    #[test]
    fn a_normalizing_on_input_uppercases_and_the_display_tracks_the_model() {
        // (A2) An `on-input` that uppercases writes back a value that DIFFERS from
        // the session buffer, so the session reseeds to the uppercased model value
        // — the displayed text is uppercased and the caret stays sane, keystroke
        // after keystroke (continuous controlled normalization).
        let (mut runtime, _value) = controlled_field(str::to_uppercase);

        runtime.handle_key(char_press('a'));
        runtime.run_due_frame_reactive().expect("rebuild after 'a'");
        assert_eq!(runtime.editing_text(), Some("A"), "the model uppercased 'a' → 'A'");
        assert_eq!(runtime.editing_selection().unwrap().focus(), 1, "caret at the end");

        runtime.handle_key(char_press('b'));
        runtime.run_due_frame_reactive().expect("rebuild after 'b'");
        assert_eq!(runtime.editing_text(), Some("AB"), "typing keeps uppercasing");
        assert_eq!(tree_text(&runtime), "AB");
        assert_eq!(runtime.editing_selection().unwrap().focus(), 2, "caret stays sane at the end");
    }

    #[test]
    fn a_plain_keystroke_does_not_reseed_and_preserves_the_caret() {
        // (A3) A plain passthrough keystroke does NOT reseed — the retained value
        // catches up to the session's OWN edit, so the discriminator sees no
        // external change and leaves the buffer + caret untouched (no fight with
        // the user). Proven by editing in the MIDDLE: a wrongful reseed would fling
        // the caret to the end.
        let (mut runtime, _value) = controlled_field(str::to_owned); // passthrough
        for c in "hello".chars() {
            runtime.handle_key(char_press(c));
        }
        runtime.run_due_frame_reactive().expect("rebuild frame");
        // Move the caret into the middle (byte 2), a non-mutating edit.
        runtime.handle_key(named_press(NamedKey::ArrowLeft));
        runtime.handle_key(named_press(NamedKey::ArrowLeft));
        runtime.handle_key(named_press(NamedKey::ArrowLeft));
        assert_eq!(runtime.editing_selection().unwrap().focus(), 2, "caret in the middle");

        // A reactive frame with UNCHANGED state must not reseed (retained == the
        // session's own value), so the caret is preserved where the user left it.
        runtime.request_frame(FrameReason::UiCommit);
        runtime.run_due_frame_reactive().expect("no-op rebuild frame");
        assert_eq!(runtime.editing_text(), Some("hello"), "text untouched (no reseed)");
        assert_eq!(
            runtime.editing_selection().unwrap().focus(),
            2,
            "the caret stayed put — a plain keystroke never reseeds/refights"
        );
    }

    // ---- readonly: focusable + selectable but NOT editable -------------------
    // (WHATWG HTML §attr-input-readonly). Distinct from `disabled`
    // (`disabled_control_takes_no_focus_and_fires_no_command` covers the disabled
    // gate, which is deliberately unchanged).

    /// App-owned command a readonly editable binds `on-change` to; used to prove a
    /// readonly field never fires `change` (its value can't change).
    const CMD_RO_CHANGE: InternedSymbol = InternedSymbol::from_raw(0x1249);

    /// A **readonly** editable view: same shape as [`input_reactive_view`] (a
    /// focusable `b` binding `on-input` with a seeded text child), plus the
    /// reserved `READONLY` structural-state attribute (as the `Input` widget emits)
    /// and an `on-change → CMD_RO_CHANGE` binding — so the runtime gates only its
    /// mutation path while focus/caret/selection stay live.
    fn readonly_input_view(text: &str) -> View {
        use obscur_view::view::READONLY;
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(500))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into())))
                .with_children(ViewFragment::from_nodes(vec![
                    ViewNode::element(ComponentId::from_raw(501))
                        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("b".into())))
                        .with_prop(ViewProperty::new(READONLY, ViewValue::Bool(true)))
                        .with_event(ViewEventBinding::new(EVENT_INPUT, CMD_INPUT))
                        .with_event(ViewEventBinding::new(EVENT_CHANGE, CMD_RO_CHANGE))
                        .with_children(ViewFragment::from_nodes(vec![ViewNode::text(
                            text.to_string(),
                        )])),
                ])),
        ]))
    }

    /// Focuses the readonly editable seeded with `seed` (via a press inside `b`)
    /// and returns the runtime plus a mirror of any `on-input` text and a flag set
    /// if `on-change` ever fires (it must not on a readonly field).
    fn focused_readonly(seed: &str) -> (UiRuntime, Rc<RefCell<String>>, Rc<std::cell::Cell<bool>>) {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let mirror = Rc::new(RefCell::new(String::new()));
        let changed = Rc::new(std::cell::Cell::new(false));
        {
            let s = mirror.clone();
            runtime.on_text_input(CMD_INPUT, move |resulting| {
                *s.borrow_mut() = resulting.to_owned();
            });
            let c = changed.clone();
            runtime.on_change(CMD_RO_CHANGE, move |_| c.set(true));
        }
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&readonly_input_view(seed), sheets);
        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
        (runtime, mirror, changed)
    }

    #[test]
    fn readonly_field_focuses_and_places_a_caret_and_selects_all() {
        // (a) A readonly field IS focusable and selectable (WHATWG HTML
        // §attr-input-readonly): a press seats focus + a caret, and Ctrl+A selects
        // the whole value — none of which is gated on readonly.
        let (mut runtime, _mirror, _changed) = focused_readonly("hi");
        assert!(runtime.focused().is_some(), "a readonly field takes focus");
        assert_eq!(runtime.editing_text(), Some("hi"), "session seeded from its value");
        let sel = runtime.editing_selection().expect("session active");
        assert!(sel.is_collapsed(), "caret placed (collapsed) on focus");

        // Select-all works: the selection becomes the whole value (non-empty).
        runtime.handle_key(KeyEvent::press(PhysicalKey::ControlLeft, None));
        runtime.handle_key(char_press('a'));
        runtime.handle_key(KeyEvent::release(PhysicalKey::ControlLeft, None));
        let sel = runtime.editing_selection().expect("session active");
        assert!(!sel.is_collapsed(), "select-all selected the text (selectable)");
        assert_eq!((sel.start(), sel.end()), (0, 2), "the whole value is selected");
        assert_eq!(runtime.editing_text(), Some("hi"), "selecting did not change the value");
    }

    #[test]
    fn readonly_field_typing_and_backspace_change_nothing_and_fire_no_change() {
        // (b) A readonly field CANNOT be edited: typing, Backspace, Delete and a
        // (single-line) Enter are all no-ops on its value, no `on-input` fires, and
        // `on-change` never fires (its value can't change).
        let (mut runtime, mirror, changed) = focused_readonly("hi");

        runtime.handle_key(char_press('x'));
        assert_eq!(runtime.editing_text(), Some("hi"), "typing inserts nothing");
        runtime.handle_key(named_press(NamedKey::Backspace));
        assert_eq!(runtime.editing_text(), Some("hi"), "Backspace deletes nothing");
        runtime.handle_key(named_press(NamedKey::Delete));
        assert_eq!(runtime.editing_text(), Some("hi"), "Delete deletes nothing");
        assert_eq!(&*mirror.borrow(), "", "no mutating edit fired on-input");

        // Enter (single-line) commits — but a readonly value never changed, so no
        // `change` fires; then blur (focus a non-editable) also fires nothing.
        runtime.handle_key(named_press(NamedKey::Enter));
        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 75.0));
        assert!(!changed.get(), "a readonly field never fires on-change");
    }

    #[test]
    fn readonly_field_ime_commit_inserts_nothing() {
        // IME is a mutation entry too: a composition commit on a readonly field is
        // inert — the buffer is unchanged and no `on-input` fires.
        let (mut runtime, mirror, _changed) = focused_readonly("ab");
        assert!(!runtime.handle_ime(CompositionEvent::Start), "readonly rejects IME start");
        assert!(!runtime.handle_ime(CompositionEvent::Commit("に".into())), "readonly rejects IME commit");
        assert_eq!(runtime.editing_text(), Some("ab"), "IME committed nothing");
        assert_eq!(&*mirror.borrow(), "", "no on-input fired");
    }

    #[test]
    fn no_editable_focused_makes_a_char_press_a_noop() {
        // (e) With a focused NON-editable node there is no session, so a char
        // press mutates nothing and routes exactly as before.
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&panel_view(), sheets);
        let (_panel, a, _b) = panel_nodes(&runtime);

        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
        assert_eq!(runtime.focused(), Some(a));
        assert!(runtime.editing_text().is_none(), "non-editable focus has no session");

        let dispatch = runtime.handle_key(char_press('a'));
        assert_eq!(dispatch.target(), Some(a), "routed to the focused node");
        assert_eq!(dispatch.command(), None);
        assert!(runtime.editing_text().is_none(), "still no session");
    }

    #[test]
    fn typing_renders_session_text_and_a_caret_in_the_frame() {
        // End-to-end (step 2): focus an editable "hi", type 'x' so the session is
        // "hix" with the caret at index 3, render a frame, and confirm the display
        // list paints the SESSION text "hix" plus a caret rect at 3·advance past
        // the text origin. The retained view still carries "hi" — the session
        // override is what makes the typed 'x' visible.
        let (mut runtime, _mirror) = focused_editable("hi");
        runtime.handle_key(char_press('x'));
        assert_eq!(runtime.editing_text(), Some("hix"));
        assert_eq!(runtime.editing_selection().unwrap().focus(), 3);

        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let result = runtime.render_now(&input_reactive_view("hi"), sheets);
        let list = result.snapshot().display_list();

        // The painted text is the live session text "hix" (concatenated across any
        // wrapped runs), not the retained "hi".
        let drawn: String = list
            .commands()
            .iter()
            .filter_map(|c| match c {
                obscur_scene::command::PaintCommand::DrawText { run, .. } => Some(run.text()),
                _ => None,
            })
            .collect();
        assert_eq!(drawn, "hix", "the frame paints the live session text");

        // The first text run's origin x is the editing text box's left edge; the
        // caret (the thin width-1 fill) sits 3 advances (3·9.6) past it.
        let advance = 16.0_f32 * obscur_text::shaping::MockTextShaper::ADVANCE_RATIO;
        let text_origin_x = list
            .commands()
            .iter()
            .find_map(|c| match c {
                obscur_scene::command::PaintCommand::DrawText { origin, .. } => Some(origin.x),
                _ => None,
            })
            .expect("a text run");
        let caret = list
            .commands()
            .iter()
            .find_map(|c| match c {
                obscur_scene::command::PaintCommand::DrawPath { path, .. } => path
                    .bounds()
                    .filter(|r| (r.size.width - 1.0).abs() < 1e-6),
                _ => None,
            })
            .expect("a thin caret fill");
        assert!(
            (caret.origin.x - (text_origin_x + advance * 3.0)).abs() < 1e-3,
            "caret at {} != text_origin {text_origin_x} + 3·advance {}",
            caret.origin.x,
            advance * 3.0
        );
        assert!(list.is_balanced(), "the display list stays balanced");
    }

    // ---- Single-line horizontal caret scroll (CSS Overflow 3 §3.1) ------------

    /// A single-line editable field: a `panel` (200×100) holding one `field`
    /// (class `field`) that binds `on-input` and carries `text`. The `field`
    /// rule makes it 100px wide, `white-space: nowrap` (so it scrolls rather than
    /// wraps) and `overflow: hidden` (so the scrolled-out glyphs clip).
    fn scroll_field_view(text: &str) -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(600))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into())))
                .with_children(ViewFragment::from_nodes(vec![
                    ViewNode::element(ComponentId::from_raw(601))
                        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("field".into())))
                        .with_event(ViewEventBinding::new(EVENT_INPUT, CMD_INPUT))
                        .with_children(ViewFragment::from_nodes(vec![ViewNode::text(
                            text.to_string(),
                        )])),
                ])),
        ]))
    }

    /// Sheet for [`scroll_field_view`]: a 200×100 `panel` and a 100×20 single-line
    /// (`nowrap`, `overflow: hidden`) `field`. `white-space` inherits, so the
    /// field's text child shapes on one line.
    fn scroll_field_sheet() -> Stylesheet {
        use obscur_style::property::layout_keyword;
        let field = vec![
            BlockDeclaration::normal(PropertyId::Width, PropertyValue::Length(Length::px(100.0))),
            BlockDeclaration::normal(PropertyId::Height, PropertyValue::Length(Length::px(20.0))),
            BlockDeclaration::normal(
                PropertyId::WhiteSpace,
                PropertyValue::Keyword(layout_keyword::WHITE_SPACE_NOWRAP),
            ),
            BlockDeclaration::normal(
                PropertyId::Overflow,
                PropertyValue::Keyword(layout_keyword::OVERFLOW_HIDDEN),
            ),
        ];
        Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![class_rule("panel", size(200.0, 100.0)), class_rule("field", field)],
        )
    }

    /// Focuses the single-line `field` (seeded empty) and returns the runtime. The
    /// `on-input` handler is registered but the resulting text is not mirrored.
    fn focused_scroll_field() -> UiRuntime {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        runtime.on_text_input(CMD_INPUT, move |_| {});
        let sheet = scroll_field_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&scroll_field_view(""), sheets);
        // Press inside the field to focus it; the runtime seeds an empty session.
        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 10.0));
        assert!(runtime.focused().is_some(), "the press focused the field");
        runtime
    }

    /// The `field` node id (root → panel → field) after a frame populated the tree.
    fn scroll_field_node(runtime: &UiRuntime) -> NodeId {
        let root = runtime.tree().root().expect("root after a frame");
        let panel = runtime.tree().get(root).unwrap().children()[0];
        runtime.tree().get(panel).unwrap().children()[0]
    }

    /// The field's current horizontal scroll offset (reads the private session).
    fn scroll_left_of(runtime: &UiRuntime) -> f32 {
        runtime.editing.as_ref().expect("active session").scroll_left()
    }

    /// Re-renders the seed view and returns the frame's display list — compose
    /// paints the live session text + caret at the current scroll offset.
    fn render_scroll_frame(runtime: &mut UiRuntime) -> DisplayList {
        let sheet = scroll_field_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let result = runtime.render_now(&scroll_field_view(""), sheets);
        result.snapshot().display_list().clone()
    }

    /// The painted caret's x (the thin, width-1 fill), or `None` if absent.
    fn painted_caret_x(list: &DisplayList) -> Option<f32> {
        list.commands().iter().find_map(|c| match c {
            PaintCommand::DrawPath { path, .. } => path
                .bounds()
                .filter(|r| (r.size.width - 1.0).abs() < 1e-6)
                .map(|r| r.origin.x),
            _ => None,
        })
    }

    /// The first `DrawText` run's origin x (the run's left edge, `origin −
    /// scroll_left`).
    fn first_run_origin_x(list: &DisplayList) -> Option<f32> {
        list.commands().iter().find_map(|c| match c {
            PaintCommand::DrawText { origin, .. } => Some(origin.x),
            _ => None,
        })
    }

    #[test]
    fn typing_past_the_box_scrolls_the_caret_into_view() {
        // (a) Typing a value far wider than the 100px box scrolls the field:
        // scroll_left > 0 and the painted caret stays within the box's right edge.
        let mut runtime = focused_scroll_field();
        for _ in 0..20 {
            runtime.handle_key(char_press('x'));
        }
        assert_eq!(runtime.editing_text().map(str::len), Some(20));
        assert!(
            scroll_left_of(&runtime) > 0.0,
            "a value wider than the box scrolls left: {}",
            scroll_left_of(&runtime),
        );

        let field = scroll_field_node(&runtime);
        let field_box = runtime.layout_boxes().get(&field).expect("a field box").bounds;
        let box_right = field_box.origin.x + field_box.size.width;

        let list = render_scroll_frame(&mut runtime);
        let caret_x = painted_caret_x(&list).expect("a caret is painted");
        assert!(
            caret_x <= box_right + 1e-3,
            "caret x {caret_x} must stay within the box right {box_right}",
        );
        // The run's left edge is scrolled OUT past the box's left (head clipped).
        let run_left = first_run_origin_x(&list).expect("a run");
        assert!(
            run_left < field_box.origin.x - 1e-3,
            "the head is scrolled left of the box: run_left {run_left} vs box left {}",
            field_box.origin.x,
        );
    }

    #[test]
    fn home_resets_scroll_and_shows_the_head() {
        // (b) Home moves the caret to 0, which resets scroll_left to 0 so the
        // field's first character is visible again (run left back at the box left).
        let mut runtime = focused_scroll_field();
        for _ in 0..20 {
            runtime.handle_key(char_press('x'));
        }
        assert!(scroll_left_of(&runtime) > 0.0, "scrolled while typing");

        runtime.handle_key(named_press(NamedKey::Home));
        assert_eq!(runtime.editing_selection().unwrap().focus(), 0, "caret at start");
        assert_eq!(scroll_left_of(&runtime), 0.0, "Home resets scroll_left to 0");

        let field = scroll_field_node(&runtime);
        let field_left = runtime.layout_boxes().get(&field).unwrap().bounds.origin.x;
        let list = render_scroll_frame(&mut runtime);
        let run_left = first_run_origin_x(&list).expect("a run");
        assert!(
            (run_left - field_left).abs() < 1e-3,
            "the head is visible at the box left: run_left {run_left} vs {field_left}",
        );
    }

    #[test]
    fn click_on_a_scrolled_field_resolves_the_scrolled_byte() {
        // (c) With the field scrolled, a click adds scroll_left back before
        // resolving the byte, so a click near the visible left edge lands on the
        // character actually painted there — NOT the head character.
        let mut runtime = focused_scroll_field();
        for _ in 0..20 {
            runtime.handle_key(char_press('x'));
        }
        let scroll_left = scroll_left_of(&runtime);
        assert!(scroll_left > 0.0, "scrolled while typing");

        let field = scroll_field_node(&runtime);
        let field_left = runtime.layout_boxes().get(&field).unwrap().bounds.origin.x;
        // Click 10px inside the visible box. Without the +scroll_left inverse this
        // would resolve to byte ~1; with it, it lands well into the string.
        runtime.handle_pointer(mouse(PointerEventKind::Down, field_left + 10.0, 10.0));
        let focus = runtime.editing_selection().unwrap().focus();
        // The mock shaper advances each char by 9.6px; local_x = 10 + scroll_left.
        let advance = 16.0_f32 * obscur_text::shaping::MockTextShaper::ADVANCE_RATIO;
        let expected = ((10.0 + scroll_left) / advance).round() as usize;
        assert_eq!(
            focus, expected,
            "click on a scrolled field resolves the scrolled byte (advance {advance}, scroll {scroll_left})",
        );
        assert!(focus > 1, "the resolved byte is not the head character: {focus}");
    }

    #[test]
    fn a_fitting_value_never_scrolls() {
        // (d) A value that fits the box keeps scroll_left == 0, so the paint and
        // hit-test are byte-identical to the pre-scroll path.
        let mut runtime = focused_scroll_field();
        for c in "abc".chars() {
            runtime.handle_key(char_press(c));
        }
        assert_eq!(runtime.editing_text(), Some("abc"));
        assert_eq!(
            scroll_left_of(&runtime),
            0.0,
            "a value narrower than the box never scrolls",
        );
        // The caret paints at the un-offset pen x (origin + 3·advance), unchanged.
        let list = render_scroll_frame(&mut runtime);
        let caret_x = painted_caret_x(&list).expect("a caret");
        let run_left = first_run_origin_x(&list).expect("a run");
        let advance = 16.0_f32 * obscur_text::shaping::MockTextShaper::ADVANCE_RATIO;
        assert!(
            (caret_x - (run_left + advance * 3.0)).abs() < 1e-3,
            "caret at the un-scrolled pen x: {caret_x} vs {}",
            run_left + advance * 3.0,
        );
    }

    // ---- MEASURE↔COMPOSE editing consistency (CSS Text 3 §4.1.1) --------------

    /// A `field` (bound to `on-input`) with a wrapping (`white-space: normal`)
    /// text child, mirroring [`scroll_field_view`] but narrow so a space-separated
    /// value wraps to several lines.
    fn wrap_field_view(text: &str) -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(700))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into())))
                .with_children(ViewFragment::from_nodes(vec![
                    ViewNode::element(ComponentId::from_raw(701))
                        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("wrapfield".into())))
                        .with_event(ViewEventBinding::new(EVENT_INPUT, CMD_INPUT))
                        .with_children(ViewFragment::from_nodes(vec![ViewNode::text(
                            text.to_string(),
                        )])),
                ])),
        ]))
    }

    /// Sheet for [`wrap_field_view`]: a 40px-wide `wrapfield` with an exact 20px
    /// `line-height` and `white-space: normal`, so its text wraps at spaces and
    /// every wrapped line is exactly 20px tall.
    fn wrap_field_sheet() -> Stylesheet {
        use obscur_style::property::layout_keyword;
        let field = vec![
            BlockDeclaration::normal(PropertyId::Width, PropertyValue::Length(Length::px(40.0))),
            BlockDeclaration::normal(PropertyId::LineHeight, PropertyValue::Length(Length::px(20.0))),
            BlockDeclaration::normal(
                PropertyId::WhiteSpace,
                PropertyValue::Keyword(layout_keyword::WHITE_SPACE_NORMAL),
            ),
        ];
        Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![class_rule("panel", size(200.0, 100.0)), class_rule("wrapfield", field)],
        )
    }

    /// Focuses the empty `wrapfield` and returns the runtime with an (empty) session.
    fn focused_wrap_field() -> UiRuntime {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        runtime.on_text_input(CMD_INPUT, move |_| {});
        let sheet = wrap_field_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&wrap_field_view(""), sheets);
        runtime.handle_pointer(mouse(PointerEventKind::Down, 20.0, 10.0));
        assert!(runtime.focused().is_some(), "the press focused the wrap field");
        runtime
    }

    /// The `wrapfield`'s text-leaf node id (root → panel → field → text).
    fn wrap_text_node(runtime: &UiRuntime) -> NodeId {
        let root = runtime.tree().root().expect("root after a frame");
        let panel = runtime.tree().get(root).unwrap().children()[0];
        let field = runtime.tree().get(panel).unwrap().children()[0];
        runtime.tree().get(field).unwrap().children()[0]
    }

    /// Re-renders the seed view (retained text `""`) and returns the display list;
    /// the SESSION still holds the typed value, so this is the lag frame.
    fn render_wrap_frame(runtime: &mut UiRuntime) -> DisplayList {
        let sheet = wrap_field_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let result = runtime.render_now(&wrap_field_view(""), sheets);
        result.snapshot().display_list().clone()
    }

    #[test]
    fn an_edited_wrapping_field_is_measured_from_its_session_so_box_equals_painted_lines() {
        // The MEASURE↔COMPOSE invariant (CSS Text 3 §4.1.1): while a field is
        // focused, its box is SIZED and PAINTED from the SAME session buffer with
        // the SAME processing (raw, the field's real `white-space`). The retained
        // tree lags the session between a keystroke and the app's reactive rebuild —
        // here it stays `""` — so a pass reading the retained value would size a
        // 1-line box while compose painted several lines. With the session threaded
        // into measure, the arranged text box height equals the painted line count ×
        // line-height, and differs from the stale retained box.
        let mut runtime = focused_wrap_field();

        // Baseline: the (empty) retained/session text box is a single 20px line.
        let text_node = wrap_text_node(&runtime);
        let stale_height = runtime
            .layout_boxes()
            .get(&text_node)
            .expect("a text box")
            .bounds
            .size
            .height;

        // Type a space-separated value that wraps to several lines in the 40px box
        // (this fixture is a single-line-typed `input`, so wrapping is driven by
        // spaces; the textarea `\n` path is covered by the multiline tests below).
        for c in "aa aa aa aa".chars() {
            runtime.handle_key(char_press(c));
        }
        assert_eq!(runtime.editing_text(), Some("aa aa aa aa"), "session holds the value");

        // The lag frame: the retained text is still "", the session drives paint.
        let list = render_wrap_frame(&mut runtime);
        let text_node = wrap_text_node(&runtime);
        let measured_height = runtime
            .layout_boxes()
            .get(&text_node)
            .expect("a text box")
            .bounds
            .size
            .height;
        // One `DrawText` run is emitted per wrapped line, so the run count IS the
        // painted line count.
        let painted_lines = list
            .commands()
            .iter()
            .filter(|c| matches!(c, PaintCommand::DrawText { .. }))
            .count();

        let line_height = 20.0_f32;
        assert!(
            painted_lines >= 2,
            "the session buffer wraps to multiple lines (got {painted_lines})",
        );
        assert!(
            (measured_height - painted_lines as f32 * line_height).abs() < 1e-3,
            "measured box height {measured_height} == painted lines {painted_lines} × {line_height}",
        );
        assert!(
            measured_height > stale_height + 1e-3,
            "the box grew past the stale retained (1-line) height {stale_height} → measure reads \
             the SESSION, not the lagging retained tree (measured {measured_height})",
        );
    }

    // ---- Compose-only text edits (Defect B) ----------------------------------
    // A same-size text edit (a single-line, non-wrapping, definite-width,
    // left-aligned field) is a COMPOSE-ONLY frame: it re-shapes + repaints the
    // field WITHOUT a restyle or relayout, and must stay pixel-identical to the
    // full frame. An edit that can resize the box (multiline growth, wrapping,
    // auto width) falls back to a full frame so the box relayouts.

    #[test]
    fn a_same_size_single_line_edit_schedules_a_compose_only_frame() {
        // (B1) Typing into a nowrap, definite-width, left-aligned single-line field
        // cannot change any box, so the edit schedules a compose-only `TextEdit`
        // frame (no restyle / relayout) — and that frame paints the SAME pixels the
        // full frame would.
        // An UNCONTROLLED field (no `on-input` handler): its session is the sole
        // source of truth, so a keystroke needs no reconcile and can be flushed
        // compose-only. (A controlled field mirrors via `on-input`, which forces a
        // full frame so the reconcile / reseed can run — covered by the A tests.)
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let sheet = scroll_field_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&scroll_field_view(""), sheets);
        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 10.0));
        assert!(runtime.focused().is_some(), "the press focused the field");
        // Drain every frame the focus/caret-placement queued, so the only thing
        // pending after the keystroke below is that keystroke's own frame.
        while runtime.has_pending_frame() {
            runtime.run_due_frame(&scroll_field_view(""), sheets);
        }

        // Type a char: a same-size edit → the runtime requests a compose-only frame.
        runtime.handle_key(char_press('x'));
        assert!(
            runtime.edit_is_compose_only(true),
            "a nowrap definite-width single-line edit is box-stable"
        );
        assert_eq!(
            runtime.peek_frame_reason(),
            Some(FrameReason::TextEdit),
            "the keystroke scheduled a compose-only TextEdit frame (not a full Input)"
        );

        // Run it: the plan skips reconcile/style/layout and only composes.
        let compose_only = runtime
            .run_due_frame(&scroll_field_view(""), sheets)
            .expect("the TextEdit frame is due");
        assert!(!compose_only.reconciled(), "compose-only: no reconcile");
        assert!(!compose_only.restyled(), "compose-only: no restyle");
        assert!(!compose_only.relaid_out(), "compose-only: no relayout");
        let compose_only_list = compose_only.snapshot().display_list().clone();

        // A full frame over the SAME session must paint byte-identical pixels — the
        // compose-only path is a perf optimization, not a visual change.
        let full = runtime.render_now(&scroll_field_view(""), sheets);
        assert!(full.relaid_out(), "the full frame relaid out (control)");
        assert_eq!(
            compose_only_list.commands(),
            full.snapshot().display_list().commands(),
            "the compose-only frame is pixel-identical to the full frame"
        );
    }

    #[test]
    fn a_multiline_edit_that_adds_a_line_still_relayouts() {
        // (B2) A textarea grows a line box when Enter inserts a newline, so a
        // multiline edit is NOT box-stable: it falls back to a full `Input` frame
        // that relayouts (the measure↔compose box-sync invariant needs the box to
        // track the painted line count).
        let mut runtime = focused_textarea();
        let sheet = textarea_sheet();
        let sheets = std::slice::from_ref(&sheet);
        while runtime.has_pending_frame() {
            runtime.run_due_frame(&textarea_view(""), sheets); // drain focus frames
        }

        type_multiline(&mut runtime, "a\nb"); // inserts a newline → a second line
        assert!(
            !runtime.edit_is_compose_only(true),
            "a multiline field is never compose-only (a line can be added)"
        );
        assert_eq!(
            runtime.peek_frame_reason(),
            Some(FrameReason::Input),
            "a multiline edit schedules a full Input frame"
        );
        let relaid = runtime
            .run_due_frame(&textarea_view(""), sheets)
            .expect("the Input frame is due");
        assert!(relaid.relaid_out(), "the multiline edit relaid out the grown box");
    }

    // ---- EVENT_CHANGE as a COMMIT (blur / single-line Enter) event -----------
    // The HTML `change`/`input` event model (WHATWG HTML §4.10.5.5 *Common input
    // element APIs*): for a text field `input` fires per keystroke, while `change`
    // fires ONCE on COMMIT (blur, or Enter for a single-line input) carrying the
    // committed value — and NOT on a focus/click. (For a checkbox/radio/select the
    // toggling click IS the change; that click→change path is covered by
    // `clicking_a_checkbox_fires_its_change_command_via_activate`.)

    /// Field `a`'s command (bound to BOTH `on-input` and `on-change`, as the
    /// `Input` widget binds both symbols to one command).
    const CMD_FIELD_A: InternedSymbol = InternedSymbol::from_raw(0x1301);
    /// Field `b`'s command — a second editable to move focus to (blur `a`).
    const CMD_FIELD_B: InternedSymbol = InternedSymbol::from_raw(0x1302);

    /// A panel (100x100) with two editable single-line fields: `a` (y∈[0,50)) and
    /// `b` (y∈[50,100)), each binding BOTH `on-input` and `on-change` to its own
    /// command — mirroring the `Input` widget, which binds `EVENT_INPUT` and
    /// `EVENT_CHANGE` to the same symbol. Neither carries a `type`, so both are
    /// single-line (`Enter` = commit, not newline).
    fn two_editable_fields_view() -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(600))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into())))
                .with_children(ViewFragment::from_nodes(vec![
                    ViewNode::element(ComponentId::from_raw(601))
                        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("a".into())))
                        .with_event(ViewEventBinding::new(EVENT_INPUT, CMD_FIELD_A))
                        .with_event(ViewEventBinding::new(EVENT_CHANGE, CMD_FIELD_A)),
                    ViewNode::element(ComponentId::from_raw(602))
                        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("b".into())))
                        .with_event(ViewEventBinding::new(EVENT_INPUT, CMD_FIELD_B))
                        .with_event(ViewEventBinding::new(EVENT_CHANGE, CMD_FIELD_B)),
                ])),
        ]))
    }

    #[test]
    fn clicking_into_a_text_field_fires_no_change() {
        // (a) A click into an editable text field is a FOCUS, not a commit: it must
        // resolve NO activation command and fire NO `change` (WHATWG HTML §4.10.5.5
        // — a text field's `change` fires on commit, never on focus/click). This is
        // the fix for the old bug where an editable's `EVENT_CHANGE` was resolved as
        // a pointer-click activation and fired on every focus-click.
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&two_editable_fields_view(), sheets);

        let changes = Rc::new(Cell::new(0));
        {
            let c = changes.clone();
            runtime.on_change(CMD_FIELD_A, move |_| c.set(c.get() + 1));
        }

        // Click into field `a` (Down+Up at (50, 25)).
        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
        let up = runtime.handle_pointer(mouse(PointerEventKind::Up, 50.0, 25.0));

        assert!(runtime.focused().is_some(), "the click focused the field");
        assert_eq!(
            up.command(),
            None,
            "an editable click resolves NO activation command (change is not a click)"
        );
        assert_eq!(changes.get(), 0, "clicking into a text field fires NO change");
    }

    #[test]
    fn input_fires_per_keystroke_and_change_fires_once_on_blur_with_the_committed_text() {
        // (b) The full HTML input/change call sequence on a text field:
        //   type "hi"  → `input` fires per char (×2), `change` fires 0×;
        //   click into the (already-focused) field → `change` fires 0×;
        //   blur (focus another field) → `change` fires ONCE with the committed "hi".
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&two_editable_fields_view(), sheets);

        // Separate registries, both keyed by the SAME field command: per-keystroke
        // `on_text_input` and commit `on_change` — proving they never conflate.
        let inputs = Rc::new(RefCell::new(Vec::<String>::new()));
        let committed = Rc::new(RefCell::new(Vec::<String>::new()));
        {
            let i = inputs.clone();
            runtime.on_text_input(CMD_FIELD_A, move |t| i.borrow_mut().push(t.to_owned()));
        }
        {
            let c = committed.clone();
            runtime.on_change(CMD_FIELD_A, move |t| c.borrow_mut().push(t.to_owned()));
        }

        // Focus field `a`.
        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));

        // Type "hi": each keystroke fires `input`, none fires `change`.
        runtime.handle_key(char_press('h'));
        runtime.handle_key(char_press('i'));
        assert_eq!(&*inputs.borrow(), &["h".to_owned(), "hi".to_owned()], "input per keystroke");
        assert!(committed.borrow().is_empty(), "typing fires NO change");

        // Click again into the SAME field: still no commit, no change.
        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
        runtime.handle_pointer(mouse(PointerEventKind::Up, 50.0, 25.0));
        assert!(committed.borrow().is_empty(), "a focus-click fires NO change");

        // Blur: move focus to field `b` (press at (50, 75)) — this COMMITS `a`.
        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 75.0));
        assert_eq!(
            &*committed.borrow(),
            &["hi".to_owned()],
            "blur fires `change` ONCE with the committed text"
        );
        // `input` was not fired again by the blur.
        assert_eq!(inputs.borrow().len(), 2, "blur fires no extra `input`");
    }

    #[test]
    fn blur_of_an_unchanged_field_fires_no_change() {
        // The HTML change model fires `change` only when the value ACTUALLY changed
        // since focus: focusing a field and blurring it without editing fires
        // nothing (WHATWG HTML §4.10.5.5).
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&two_editable_fields_view(), sheets);

        let changes = Rc::new(Cell::new(0));
        {
            let c = changes.clone();
            runtime.on_change(CMD_FIELD_A, move |_| c.set(c.get() + 1));
        }

        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0)); // focus a
        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 75.0)); // blur to b
        assert_eq!(changes.get(), 0, "blurring an UNCHANGED field fires no change");
    }

    #[test]
    fn enter_on_a_single_line_input_commits_change_without_inserting_and_does_not_refire() {
        // (c) `Enter` in a single-line `input` is a COMMIT (WHATWG HTML §4.10.5.5 /
        // implicit form submission §4.10.21.3): it fires `change` with the committed
        // value but inserts NO newline (multiline insertion is a separate item), and
        // rebaselines so a second `Enter` with no further edits does NOT re-fire.
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&two_editable_fields_view(), sheets);

        let committed = Rc::new(RefCell::new(Vec::<String>::new()));
        {
            let c = committed.clone();
            runtime.on_change(CMD_FIELD_A, move |t| c.borrow_mut().push(t.to_owned()));
        }

        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0)); // focus a
        runtime.handle_key(char_press('h'));
        runtime.handle_key(char_press('i'));

        // Enter: commit "hi", but insert nothing (the session text stays "hi").
        runtime.handle_key(named_press(NamedKey::Enter));
        assert_eq!(runtime.editing_text(), Some("hi"), "single-line Enter inserts NO newline");
        assert_eq!(&*committed.borrow(), &["hi".to_owned()], "Enter committed `change` once");

        // A second Enter with no edits does not re-fire (baseline was advanced).
        runtime.handle_key(named_press(NamedKey::Enter));
        assert_eq!(committed.borrow().len(), 1, "Enter with no new edit does not re-fire change");

        // A subsequent blur (still unchanged since the Enter commit) also stays quiet.
        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 75.0));
        assert_eq!(committed.borrow().len(), 1, "blur after an Enter-commit does not re-fire");
    }

    #[test]
    fn on_change_and_on_text_input_are_distinct_registries() {
        // Guard the separation directly: registering only `on_text_input` for a
        // field's command leaves `has_change_handler` false (and vice-versa), so an
        // app can bind per-keystroke `input` and commit `change` independently even
        // though the widget binds both events to one symbol.
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        runtime.on_text_input(CMD_FIELD_A, |_| {});
        assert!(runtime.has_text_input_handler(CMD_FIELD_A));
        assert!(!runtime.has_change_handler(CMD_FIELD_A), "input registry is not the change registry");
        runtime.on_change(CMD_FIELD_A, |_| {});
        assert!(runtime.has_change_handler(CMD_FIELD_A));
    }

    // ---- Multi-line editing: textarea, Enter → newline, line-aware caret -------

    /// A **textarea** view: an editable node carrying the reserved CSS type
    /// `textarea` (so the runtime treats it MULTILINE — Enter inserts `\n`,
    /// ArrowUp/ArrowDown move between lines) bound to `on-input`, with a single
    /// text child. Mirrors a multi-line [`Input`](obscur_widgets::input::Input).
    fn textarea_view(text: &str) -> View {
        use obscur_view::view::TYPE;
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(710))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into())))
                .with_children(ViewFragment::from_nodes(vec![
                    ViewNode::element(ComponentId::from_raw(711))
                        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("area".into())))
                        .with_prop(ViewProperty::new(TYPE, ViewValue::from("textarea")))
                        .with_event(ViewEventBinding::new(EVENT_INPUT, CMD_INPUT))
                        .with_children(ViewFragment::from_nodes(vec![ViewNode::text(
                            text.to_string(),
                        )])),
                ])),
        ]))
    }

    /// Sheet for [`textarea_view`]: a WIDE `area` (300px, so newline-split lines do
    /// not further wrap) with an exact 20px `line-height` and `white-space:
    /// pre-wrap`, so authored `\n`s are preserved as separate lines each 20px tall.
    fn textarea_sheet() -> Stylesheet {
        use obscur_style::property::layout_keyword;
        let field = vec![
            BlockDeclaration::normal(PropertyId::Width, PropertyValue::Length(Length::px(300.0))),
            BlockDeclaration::normal(PropertyId::LineHeight, PropertyValue::Length(Length::px(20.0))),
            BlockDeclaration::normal(
                PropertyId::WhiteSpace,
                PropertyValue::Keyword(layout_keyword::WHITE_SPACE_PRE_WRAP),
            ),
        ];
        Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![class_rule("panel", size(400.0, 200.0)), class_rule("area", field)],
        )
    }

    /// Focuses the empty `textarea` and returns the runtime with an (empty) session.
    fn focused_textarea() -> UiRuntime {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        runtime.on_text_input(CMD_INPUT, move |_| {});
        let sheet = textarea_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&textarea_view(""), sheets);
        runtime.handle_pointer(mouse(PointerEventKind::Down, 20.0, 10.0));
        assert!(runtime.focused().is_some(), "the press focused the textarea");
        runtime
    }

    /// The textarea's editable node id (root → panel → area) and its text-leaf id.
    fn textarea_nodes(runtime: &UiRuntime) -> (NodeId, NodeId) {
        let root = runtime.tree().root().expect("root after a frame");
        let panel = runtime.tree().get(root).unwrap().children()[0];
        let field = runtime.tree().get(panel).unwrap().children()[0];
        let text = runtime.tree().get(field).unwrap().children()[0];
        (field, text)
    }

    /// Re-renders the seed view (retained text `""`); the SESSION holds the typed
    /// value, so compose paints the live multi-line text + line-aware caret.
    fn render_textarea_frame(runtime: &mut UiRuntime) -> DisplayList {
        let sheet = textarea_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let result = runtime.render_now(&textarea_view(""), sheets);
        result.snapshot().display_list().clone()
    }

    /// The painted caret rect (the thin, width-1 fill), or `None` if absent.
    fn painted_caret_rect(list: &DisplayList) -> Option<obscur_geometry::Rect> {
        list.commands().iter().find_map(|c| match c {
            PaintCommand::DrawPath { path, .. } => {
                path.bounds().filter(|r| (r.size.width - 1.0).abs() < 1e-6)
            }
            _ => None,
        })
    }

    /// Types `text` into the focused editable one printable char at a time, mapping
    /// a `\n` to a real `Enter` key press (through the runtime's editing keymap).
    fn type_multiline(runtime: &mut UiRuntime, text: &str) {
        for ch in text.chars() {
            if ch == '\n' {
                runtime.handle_key(named_press(NamedKey::Enter));
            } else {
                runtime.handle_key(char_press(ch));
            }
        }
    }

    #[test]
    fn enter_in_a_textarea_inserts_a_newline_and_moves_the_caret_to_the_next_line() {
        // (a) Enter in a MULTILINE field inserts `\n` (CSS Text 3 §4.1.1 segment
        // break) and the caret advances past it — to line 2, column 0. Typing "ab"
        // then Enter yields the buffer "ab\n" with the caret at byte 3, and the
        // painted caret sits on the SECOND line (y ≈ line-height) at x ≈ 0.
        let mut runtime = focused_textarea();
        type_multiline(&mut runtime, "ab\n");
        assert_eq!(runtime.editing_text(), Some("ab\n"), "Enter inserted a newline");
        assert_eq!(
            runtime.editing_selection().unwrap().focus(),
            3,
            "caret is past the newline (line 2, col 0)",
        );

        let (_, text_node) = textarea_nodes(&runtime);
        let text_origin = runtime.layout_boxes().get(&text_node).unwrap().bounds.origin;
        let list = render_textarea_frame(&mut runtime);
        let caret = painted_caret_rect(&list).expect("a caret is painted");
        assert!(
            (caret.origin.y - text_origin.y - 20.0).abs() < 1e-3,
            "the caret is on the 2nd line (y ≈ line-height 20): rel y {}",
            caret.origin.y - text_origin.y,
        );
        assert!(
            (caret.origin.x - text_origin.x).abs() < 1e-3,
            "the caret is at column 0 on line 2: rel x {}",
            caret.origin.x - text_origin.x,
        );
    }

    #[test]
    fn arrow_up_and_down_move_the_caret_between_wrapped_lines() {
        // (b) In a textarea holding "ab\ncd" (caret at end, line 2 col 2), ArrowUp
        // moves to line 1 at roughly the same x (byte 2, end of "ab"); ArrowDown
        // moves back to line 2 at that x (byte 5, end of "cd").
        let mut runtime = focused_textarea();
        type_multiline(&mut runtime, "ab\ncd");
        assert_eq!(runtime.editing_text(), Some("ab\ncd"));
        assert_eq!(runtime.editing_selection().unwrap().focus(), 5, "caret at end");

        runtime.handle_key(named_press(NamedKey::ArrowUp));
        assert_eq!(
            runtime.editing_selection().unwrap().focus(),
            2,
            "ArrowUp lands on line 1 at ~the same x (byte 2)",
        );

        runtime.handle_key(named_press(NamedKey::ArrowDown));
        assert_eq!(
            runtime.editing_selection().unwrap().focus(),
            5,
            "ArrowDown returns to line 2 at ~the same x (byte 5)",
        );
    }

    #[test]
    fn caret_on_the_second_line_paints_within_that_line_not_the_summed_width() {
        // (c) With "ab\ncd" (caret at end), the painted caret sits on line 2 at
        // y ≈ line-height, and its x is measured WITHIN line 2 ("cd", ~2 advances) —
        // NOT the summed width of both lines ("abcd", ~4 advances), which is the
        // single-line bug this replaces.
        let mut runtime = focused_textarea();
        type_multiline(&mut runtime, "ab\ncd");

        let (_, text_node) = textarea_nodes(&runtime);
        let text_origin = runtime.layout_boxes().get(&text_node).unwrap().bounds.origin;
        let list = render_textarea_frame(&mut runtime);
        let caret = painted_caret_rect(&list).expect("a caret");
        let advance = 16.0_f32 * obscur_text::shaping::MockTextShaper::ADVANCE_RATIO;
        assert!(
            (caret.origin.y - text_origin.y - 20.0).abs() < 1e-3,
            "caret y is on line 2 (≈ line-height 20): rel y {}",
            caret.origin.y - text_origin.y,
        );
        let rel_x = caret.origin.x - text_origin.x;
        assert!(
            (rel_x - advance * 2.0).abs() < 1e-2,
            "caret x is line-2-local (~2 advances = {}), got {rel_x}",
            advance * 2.0,
        );
        assert!(
            rel_x < advance * 3.0,
            "caret x is NOT the summed width of both lines (~4 advances): {rel_x}",
        );
    }

    #[test]
    fn a_click_on_the_second_line_resolves_to_a_second_line_byte() {
        // (d) A pointer press within the SECOND line's y-band resolves to a byte on
        // line 2 (>= 3, the start of "cd" after "ab\n") — the 2D hit-test picks the
        // line by y then the byte by x, so a click never lands on line 1.
        let mut runtime = focused_textarea();
        type_multiline(&mut runtime, "ab\ncd");
        // Lay out the multi-line text box (measured from the session) so the second
        // line has a real y-band to click into.
        let _ = render_textarea_frame(&mut runtime);

        let (_, text_node) = textarea_nodes(&runtime);
        let origin = runtime.layout_boxes().get(&text_node).unwrap().bounds.origin;
        // Click into line 2 (y ∈ [20, 40)), a little in from the left.
        runtime.handle_pointer(mouse(PointerEventKind::Down, origin.x + 12.0, origin.y + 25.0));
        let focus = runtime.editing_selection().unwrap().focus();
        assert!(
            (3..=5).contains(&focus),
            "a click on line 2 resolves to a line-2 byte (3..=5), got {focus}",
        );
    }

    #[test]
    fn enter_in_a_single_line_input_does_not_insert_a_newline() {
        // (e) A single-line `input`'s Enter is NOT an edit — it is left for the
        // activation/submit path — so the buffer is unchanged (no `\n`), proving the
        // multiline Enter behaviour is gated on `type=textarea`.
        let mut runtime = focused_scroll_field();
        for c in "ab".chars() {
            runtime.handle_key(char_press(c));
        }
        assert_eq!(runtime.editing_text(), Some("ab"));
        runtime.handle_key(named_press(NamedKey::Enter));
        assert_eq!(
            runtime.editing_text(),
            Some("ab"),
            "single-line Enter inserts no newline (activation, not an edit)",
        );
    }

    // ---- Multi-line VERTICAL caret scroll (textarea, CSS Overflow 3 §3.1) ------

    /// Sheet for a **scrolling** `<textarea>`: like [`textarea_sheet`] but with a
    /// fixed 60-px `height` and `overflow: auto`, so it is a VERTICAL scroll
    /// container — content taller than 60 px (3 lines at the 20-px line-height)
    /// overflows and scrolls, and the caret follows. No padding/border, so the
    /// content box height is the full 60 px.
    fn scrolling_textarea_sheet() -> Stylesheet {
        use obscur_style::property::layout_keyword;
        let mut field = vec![
            BlockDeclaration::normal(PropertyId::Width, PropertyValue::Length(Length::px(300.0))),
            BlockDeclaration::normal(PropertyId::Height, PropertyValue::Length(Length::px(60.0))),
            BlockDeclaration::normal(PropertyId::LineHeight, PropertyValue::Length(Length::px(20.0))),
            BlockDeclaration::normal(
                PropertyId::WhiteSpace,
                PropertyValue::Keyword(layout_keyword::WHITE_SPACE_PRE_WRAP),
            ),
        ];
        field.push(overflow_auto());
        Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![class_rule("panel", size(400.0, 200.0)), class_rule("area", field)],
        )
    }

    /// Focuses the empty scrolling `<textarea>` and returns the runtime with an
    /// (empty) session, ready for typing.
    fn focused_scrolling_textarea() -> UiRuntime {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        runtime.on_text_input(CMD_INPUT, move |_| {});
        let sheet = scrolling_textarea_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&textarea_view(""), sheets);
        runtime.handle_pointer(mouse(PointerEventKind::Down, 20.0, 10.0));
        assert!(runtime.focused().is_some(), "the press focused the textarea");
        runtime
    }

    /// Re-renders the scrolling-textarea seed view (retained text `""`; the SESSION
    /// holds the typed value) so the field's grown box + scroll offset lay out.
    fn render_scrolling_textarea_frame(runtime: &mut UiRuntime) -> DisplayList {
        let sheet = scrolling_textarea_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let result = runtime.render_now(&textarea_view(""), sheets);
        result.snapshot().display_list().clone()
    }

    #[test]
    fn typing_past_the_box_scrolls_the_textarea_caret_line_into_view() {
        // (a) Typing enough lines to exceed the 60-px (3-line) box scrolls the
        // overflow container's `scroll_top` past 0 and keeps the caret's LINE inside
        // the visible box (CSS Overflow 3 §3.1 scroll-into-view) — the vertical
        // analogue of the single-line horizontal caret scroll.
        let mut runtime = focused_scrolling_textarea();
        // Six lines (120 px of content) in a 3-line (60 px) box; the caret ends on
        // line 6.
        type_multiline(&mut runtime, "l1\nl2\nl3\nl4\nl5\nl6");
        let (field, _text) = textarea_nodes(&runtime);
        // Lay out the grown box + scroll offset.
        let list = render_scrolling_textarea_frame(&mut runtime);

        // The overflow container scrolled: the caret at the bottom pushed `scroll_top`
        // to its max (120 content − 60 box = 60).
        let scroll_top = runtime.scroll_top(field);
        assert!(scroll_top > 0.0, "typing past the box scrolls it (scroll_top {scroll_top})");
        assert!((scroll_top - 60.0).abs() < 1e-3, "scrolled to the max (60): {scroll_top}");

        // The painted caret line, offset by the container's `translate(0, −scroll_top)`,
        // sits within the visible box's y-band — the last line is visible, not clipped.
        let field_box = runtime.layout_boxes().get(&field).unwrap().bounds;
        let caret = painted_caret_rect(&list).expect("a caret is painted");
        let caret_top = caret.origin.y - scroll_top;
        let caret_bottom = caret.bottom() - scroll_top;
        assert!(
            caret_top >= field_box.origin.y - 1e-3 && caret_bottom <= field_box.bottom() + 1e-3,
            "caret line y [{caret_top}, {caret_bottom}] within the box [{}, {}]",
            field_box.origin.y,
            field_box.bottom(),
        );
        // The unscrolled caret sits below the box (the last line, before the scroll
        // transform), proving the top lines are scrolled out + clipped.
        assert!(
            caret.origin.y > field_box.bottom(),
            "the caret's unscrolled line is below the box (top lines scrolled out): {} vs {}",
            caret.origin.y,
            field_box.bottom(),
        );

        // A scroll transform brackets the content: `translate(0, −scroll_top)`.
        let has_scroll_transform = list.commands().iter().any(|c| {
            matches!(c, PaintCommand::PushTransform(t) if (t.coeffs[5] + scroll_top).abs() < 0.01)
        });
        assert!(has_scroll_transform, "the content is bracketed in a scroll transform");
    }

    #[test]
    fn a_textarea_whose_content_fits_stays_unscrolled() {
        // (b) A scrolling textarea whose content fits its 3-line box keeps
        // `scroll_top == 0` — byte-identical to the unscrolled paint (the caret
        // follow is a no-op when nothing overflows).
        let mut runtime = focused_scrolling_textarea();
        type_multiline(&mut runtime, "l1\nl2"); // 40 px of content in a 60-px box
        let (field, _text) = textarea_nodes(&runtime);
        let _ = render_scrolling_textarea_frame(&mut runtime);
        assert_eq!(runtime.scroll_top(field), 0.0, "a fitting textarea does not scroll");
    }

    #[test]
    fn the_wheel_still_scrolls_the_textarea_sharing_the_caret_scroll_top() {
        // (c) The wheel/overflow path is intact and shares the SAME `scroll_top` the
        // caret follow drives: after typing scrolls it to the max, a wheel-up gesture
        // moves that same value back (CSS Overflow 3 §scroll-container).
        let mut runtime = focused_scrolling_textarea();
        type_multiline(&mut runtime, "l1\nl2\nl3\nl4\nl5\nl6");
        let (field, _text) = textarea_nodes(&runtime);
        let _ = render_scrolling_textarea_frame(&mut runtime);
        let before = runtime.scroll_top(field);
        assert!((before - 60.0).abs() < 1e-3, "caret follow put scroll_top at the max: {before}");

        // Wheel up by 25 px over the field: the SAME container scrolls back.
        let target = runtime.handle_scroll(-25.0, Point::new(20.0, 10.0));
        assert_eq!(target, Some(field), "the wheel targets the textarea container");
        let after = runtime.scroll_top(field);
        assert!(
            after < before && (after - 35.0).abs() < 1e-3,
            "the wheel moved the shared scroll_top back (before {before}, after {after})",
        );
    }

    #[test]
    fn a_single_line_input_has_no_vertical_scroll() {
        // (d) A single-line `input` (nowrap, `overflow: hidden`, not a textarea) never
        // scrolls vertically: typing past its box leaves `scroll_top == 0` (only the
        // horizontal scroll_left moves). Byte-identical to the pre-change behaviour.
        let mut runtime = focused_scroll_field();
        for c in "abcdefghijklmnopqrstuvwxyz".chars() {
            runtime.handle_key(char_press(c));
        }
        let field = scroll_field_node(&runtime);
        assert!(scroll_left_of(&runtime) > 0.0, "the single-line field scrolls horizontally");
        assert_eq!(runtime.scroll_top(field), 0.0, "a single-line input has no vertical scroll");
    }

    // ---- IME composition: preedit (shown, not committed) + commit (§5.12) -----

    /// Collects the composed text (concatenated across every `DrawText` run) of a
    /// freshly rendered frame — the DISPLAYED text, including any spliced preedit.
    fn composed_text(runtime: &mut UiRuntime, seed: &str) -> String {
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let result = runtime.render_now(&input_reactive_view(seed), sheets);
        result
            .snapshot()
            .display_list()
            .commands()
            .iter()
            .filter_map(|c| match c {
                obscur_scene::command::PaintCommand::DrawText { run, .. } => Some(run.text()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn ime_preedit_is_shown_but_not_committed_then_commit_folds_it_in() {
        // (a) Focus "ab" (caret at end). Start a composition, then update it with
        // the preedit "ni": the BUFFER stays "ab" (preedit is not committed text,
        // §5.12), `composition_preedit` is "ni", and the DISPLAYED text splices it
        // to "abni". Committing "に" replaces the (empty) composition span, so the
        // buffer becomes "abに", the composition is cleared, and the field's
        // `on-input` fires with the resulting text.
        let (mut runtime, mirror) = focused_editable("ab");
        assert_eq!(runtime.editing_selection().unwrap().focus(), 2, "caret at end");

        assert!(runtime.handle_ime(CompositionEvent::Start), "start applied");
        assert!(
            runtime.handle_ime(CompositionEvent::Update(obscur_text::ime::Preedit::new("ni"))),
            "update applied"
        );

        assert_eq!(runtime.editing_text(), Some("ab"), "buffer unchanged by preedit");
        assert_eq!(runtime.composition_preedit(), Some("ni"), "preedit is shown");
        assert_eq!(composed_text(&mut runtime, "ab"), "abni", "preedit spliced for display");

        assert!(
            runtime.handle_ime(CompositionEvent::Commit(Box::from("に"))),
            "commit applied"
        );
        assert_eq!(runtime.editing_text(), Some("abに"), "commit folded the text in");
        assert_eq!(runtime.composition_preedit(), None, "composition cleared");
        assert_eq!(&*mirror.borrow(), "abに", "EVENT_INPUT fired with the committed text");
    }

    #[test]
    fn ime_cancel_after_update_drops_the_preedit_with_no_buffer_change() {
        // (b) Cancel after an Update discards the preedit and leaves the buffer
        // untouched: "ab" in, "ab" out, no preedit.
        let (mut runtime, mirror) = focused_editable("ab");
        runtime.handle_ime(CompositionEvent::Start);
        runtime.handle_ime(CompositionEvent::Update(obscur_text::ime::Preedit::new("ni")));
        assert_eq!(runtime.composition_preedit(), Some("ni"));

        assert!(runtime.handle_ime(CompositionEvent::Cancel), "cancel applied");
        assert_eq!(runtime.editing_text(), Some("ab"), "buffer unchanged by cancel");
        assert_eq!(runtime.composition_preedit(), None, "preedit dropped");
        assert_eq!(&*mirror.borrow(), "", "cancel fires no EVENT_INPUT");
        assert_eq!(composed_text(&mut runtime, "ab"), "ab", "display back to buffer text");
    }

    #[test]
    fn ime_with_no_active_session_is_a_noop() {
        // (c) With no focused editable there is no session, so every composition
        // event is a no-op returning false — and no session springs into being.
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        assert!(!runtime.handle_ime(CompositionEvent::Start), "start no-op with no session");
        assert!(
            !runtime.handle_ime(CompositionEvent::Update(obscur_text::ime::Preedit::new("x"))),
            "update no-op with no session"
        );
        assert!(
            !runtime.handle_ime(CompositionEvent::Commit(Box::from("x"))),
            "commit no-op with no session"
        );
        assert!(runtime.editing_text().is_none(), "still no editing session");
        assert!(runtime.composition_preedit().is_none(), "no preedit");
    }

    #[test]
    fn ime_preedit_renders_an_underline_over_its_x_range() {
        // (d) An active preedit paints a thin underline (height ~1) beneath the
        // composing glyphs, spanning the preedit's x-range in the spliced display
        // text: "ab" (caret at 2) + preedit "ni" underlines x∈[2·adv, 4·adv) past
        // the text origin. The caret (width ~1) is a separate fill.
        let (mut runtime, _mirror) = focused_editable("ab");
        runtime.handle_ime(CompositionEvent::Start);
        runtime.handle_ime(CompositionEvent::Update(obscur_text::ime::Preedit::new("ni")));

        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let result = runtime.render_now(&input_reactive_view("ab"), sheets);
        let list = result.snapshot().display_list();

        let advance = 16.0_f32 * obscur_text::shaping::MockTextShaper::ADVANCE_RATIO;
        let text_origin_x = list
            .commands()
            .iter()
            .find_map(|c| match c {
                obscur_scene::command::PaintCommand::DrawText { origin, .. } => Some(origin.x),
                _ => None,
            })
            .expect("a text run");
        // The underline: a thin (height ~1) horizontal fill spanning two advances.
        let underline = list
            .commands()
            .iter()
            .find_map(|c| match c {
                obscur_scene::command::PaintCommand::DrawPath { path, .. } => path
                    .bounds()
                    .filter(|r| (r.size.height - 1.0).abs() < 1e-6 && r.size.width > 1.0),
                _ => None,
            })
            .expect("a thin preedit underline fill");
        assert!(
            (underline.origin.x - (text_origin_x + advance * 2.0)).abs() < 1e-3,
            "underline starts at the preedit's left edge (2·advance)"
        );
        assert!(
            (underline.size.width - advance * 2.0).abs() < 1e-3,
            "underline spans the two preedit glyphs"
        );
        assert!(list.is_balanced(), "the display list stays balanced");
    }

    #[test]
    fn no_ime_keystroke_path_is_byte_identical() {
        // (e) The ordinary (no-composition) keystroke path is unaffected: a runtime
        // that types 'x' produces the exact same display list as one that runs a
        // full Start→Cancel composition cycle (leaving no preedit) before typing.
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);

        let (mut plain, _m1) = focused_editable("hi");
        plain.handle_key(char_press('x'));
        let plain_cmds: Vec<_> = plain
            .render_now(&input_reactive_view("hi"), sheets)
            .snapshot()
            .display_list()
            .commands()
            .to_vec();

        let (mut cycled, _m2) = focused_editable("hi");
        cycled.handle_ime(CompositionEvent::Start);
        cycled.handle_ime(CompositionEvent::Cancel);
        cycled.handle_key(char_press('x'));
        let cycled_cmds: Vec<_> = cycled
            .render_now(&input_reactive_view("hi"), sheets)
            .snapshot()
            .display_list()
            .commands()
            .to_vec();

        assert_eq!(plain_cmds, cycled_cmds, "a cancelled composition leaves paint byte-identical");
    }

    // ---- Mouse caret positioning + drag selection (this step) ----------------

    /// The per-glyph advance the mock shaper emits at the default 16px font size
    /// (`font_size * ADVANCE_RATIO` = 9.6): every ASCII glyph in "hello" is this
    /// wide, so the caret x of boundary `i` is exactly `i * ADVANCE`.
    const ADVANCE: f32 = 16.0 * obscur_text::shaping::MockTextShaper::ADVANCE_RATIO;

    /// The layout-box origin x of the focused editable's text leaf — the left edge
    /// the click-caret local x is measured from.
    fn editing_text_origin_x(runtime: &UiRuntime) -> f32 {
        let text_node = runtime
            .tree()
            .iter()
            .find_map(|(id, node)| node.kind().text().map(|_| id))
            .expect("a text node after a frame");
        runtime
            .layout_boxes()
            .get(&text_node)
            .expect("the text leaf has a box")
            .origin()
            .x
    }

    #[test]
    fn click_places_caret_at_nearest_character_boundary() {
        // (a) A pointer click in the focused field places the caret at the nearest
        // cluster boundary: mid-way between offsets 2 and 3 snaps to 2, a click at
        // the start snaps to 0, and a click past the last glyph snaps to the end.
        let (mut runtime, _mirror) = focused_editable("hello");
        let ox = editing_text_origin_x(&runtime);

        runtime.handle_pointer(mouse(PointerEventKind::Down, ox + ADVANCE * 2.5, 25.0));
        runtime.handle_pointer(mouse(PointerEventKind::Up, ox + ADVANCE * 2.5, 25.0));
        let sel = runtime.editing_selection().expect("session active");
        assert!(sel.is_collapsed(), "a plain click collapses to a caret");
        assert_eq!(sel.focus(), 2, "click at 2.5·advance snaps to boundary 2");

        runtime.handle_pointer(mouse(PointerEventKind::Down, ox, 25.0));
        runtime.handle_pointer(mouse(PointerEventKind::Up, ox, 25.0));
        assert_eq!(runtime.editing_selection().unwrap().focus(), 0, "click at the start → 0");

        runtime.handle_pointer(mouse(PointerEventKind::Down, ox + ADVANCE * 6.0, 25.0));
        runtime.handle_pointer(mouse(PointerEventKind::Up, ox + ADVANCE * 6.0, 25.0));
        assert_eq!(runtime.editing_selection().unwrap().focus(), 5, "click past the end → 5");
    }

    #[test]
    fn click_up_on_a_field_fires_no_activation_command() {
        // A Down+Up on a text field sets a caret but must NOT spuriously fire a
        // click/change command: the field binds only `on-input`, so activation's
        // click/change lookup finds nothing.
        let (mut runtime, _mirror) = focused_editable("hello");
        let ox = editing_text_origin_x(&runtime);
        runtime.handle_pointer(mouse(PointerEventKind::Down, ox + ADVANCE * 3.0, 25.0));
        let up = runtime.handle_pointer(mouse(PointerEventKind::Up, ox + ADVANCE * 3.0, 25.0));
        assert_eq!(up.command(), None, "a click on a text field resolves no activation command");
        assert_eq!(runtime.editing_selection().unwrap().focus(), 3, "the caret still moved to 3");
    }

    #[test]
    fn drag_selects_a_range_from_press_anchor_to_pointer() {
        // (b) Press at offset 1 (anchor), drag to the offset-4 position → the
        // selection is the range [1, 4) anchored at the press; releasing clears the
        // drag so a later move no longer extends it.
        let (mut runtime, _mirror) = focused_editable("hello");
        let ox = editing_text_origin_x(&runtime);

        runtime.handle_pointer(mouse(PointerEventKind::Down, ox + ADVANCE * 1.0, 25.0));
        assert_eq!(runtime.editing_selection().unwrap().focus(), 1, "press caret at 1");

        runtime.handle_pointer(mouse(PointerEventKind::Move, ox + ADVANCE * 4.0, 25.0));
        let sel = runtime.editing_selection().unwrap();
        assert!(!sel.is_collapsed(), "the drag opened a selection");
        assert_eq!((sel.anchor(), sel.focus()), (1, 4), "range [1,4) anchored at the press");

        runtime.handle_pointer(mouse(PointerEventKind::Up, ox + ADVANCE * 4.0, 25.0));
        runtime.handle_pointer(mouse(PointerEventKind::Move, ox + ADVANCE * 2.0, 25.0));
        let sel = runtime.editing_selection().unwrap();
        assert_eq!(
            (sel.anchor(), sel.focus()),
            (1, 4),
            "no drag armed after Up → the selection is unchanged"
        );
    }

    #[test]
    fn click_on_a_non_editable_does_not_open_a_session() {
        // (c) A click on a NON-editable node opens no editing session and never
        // panics — the caret/drag path is inert when nothing editable is clicked.
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&panel_view(), sheets);
        let (_panel, a, _b) = panel_nodes(&runtime);

        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
        runtime.handle_pointer(mouse(PointerEventKind::Up, 50.0, 25.0));
        assert_eq!(runtime.focused(), Some(a));
        assert!(runtime.editing_text().is_none(), "no editable clicked → no session");
    }

    #[test]
    fn click_offset_is_the_inverse_of_the_render_caret_x() {
        // (d) The click-caret offset is the exact inverse of the render caret x:
        // the rendered caret x of boundary `i` is `i·advance` past the text origin,
        // and clicking there returns `i`.
        let (mut runtime, _mirror) = focused_editable("hello");
        let ox = editing_text_origin_x(&runtime);
        for i in 0..=5usize {
            #[allow(clippy::cast_precision_loss)]
            let x = ox + ADVANCE * i as f32;
            runtime.handle_pointer(mouse(PointerEventKind::Down, x, 25.0));
            runtime.handle_pointer(mouse(PointerEventKind::Up, x, 25.0));
            assert_eq!(
                runtime.editing_selection().unwrap().focus(),
                i,
                "click at the caret x of offset {i} returns {i}",
            );
        }
    }

    #[test]
    fn space_key_inserts_a_space_in_a_focused_field() {
        // Regression: the platform delivers Space as `NamedKey::Space` (a control
        // that is NOT being edited uses it for activation), NOT a `Character`, so
        // the editing key path must itself turn it into an inserted `' '` —
        // otherwise a focused field silently swallows every space. Type "ab",
        // press Space, type "cd": the value is "ab cd".
        let (mut runtime, mirror) = focused_editable("");
        for c in ['a', 'b'] {
            runtime.handle_key(char_press(c));
        }
        assert!(
            runtime.handle_key(named_press(NamedKey::Space)).is_handled(),
            "Space is consumed as an edit in a focused field",
        );
        for c in ['c', 'd'] {
            runtime.handle_key(char_press(c));
        }
        assert_eq!(runtime.editing_text(), Some("ab cd"));
        assert_eq!(mirror.borrow().as_str(), "ab cd", "the on-input mirror sees the space");
    }

    /// A mouse event at `(x, y)` carrying an explicit logical timestamp (µs), so a
    /// test drives the multi-click detector's time window deterministically.
    fn mouse_t(kind: PointerEventKind, x: f32, y: f32, t: u64) -> PointerEvent {
        mouse(kind, x, y).with_timestamp(t)
    }

    #[test]
    fn double_click_selects_the_word_under_the_pointer() {
        // (a) Fix 1 (§5.11): a DOUBLE click — two presses on the same point within
        // the multi-click window — selects the WORD under the pointer via
        // `NavigationUnit::Word`. Clicking inside "ab" of "ab cd" selects "ab".
        let (mut runtime, _mirror) = focused_editable("ab cd");
        let ox = editing_text_origin_x(&runtime);
        let x = ox + ADVANCE * 1.0; // inside the first word "ab"
        let t = 1_000_000;
        runtime.handle_pointer(mouse_t(PointerEventKind::Down, x, 25.0, t));
        runtime.handle_pointer(mouse_t(PointerEventKind::Up, x, 25.0, t));
        // The second press within the window is the double click.
        runtime.handle_pointer(mouse_t(PointerEventKind::Down, x, 25.0, t + 1_000));
        let sel = runtime.editing_selection().expect("session active");
        assert!(!sel.is_collapsed(), "the double click opened a word selection");
        assert_eq!(
            (sel.start(), sel.end()),
            (0, 2),
            "the word 'ab' under the pointer is selected"
        );
    }

    #[test]
    fn triple_click_selects_the_line_under_the_pointer() {
        // (b) Fix 1 (§5.11): a TRIPLE click selects the whole LINE under the pointer
        // via `NavigationUnit::LineBoundary` — for a single-line field that is the
        // whole value "ab cd".
        let (mut runtime, _mirror) = focused_editable("ab cd");
        let ox = editing_text_origin_x(&runtime);
        let x = ox + ADVANCE * 1.0;
        let t = 1_000_000;
        runtime.handle_pointer(mouse_t(PointerEventKind::Down, x, 25.0, t));
        runtime.handle_pointer(mouse_t(PointerEventKind::Up, x, 25.0, t));
        runtime.handle_pointer(mouse_t(PointerEventKind::Down, x, 25.0, t + 1_000));
        runtime.handle_pointer(mouse_t(PointerEventKind::Up, x, 25.0, t + 1_000));
        // The third press within the window is the triple click.
        runtime.handle_pointer(mouse_t(PointerEventKind::Down, x, 25.0, t + 2_000));
        let sel = runtime.editing_selection().expect("session active");
        assert_eq!(
            (sel.start(), sel.end()),
            (0, 5),
            "the whole single line 'ab cd' is selected"
        );
    }

    #[test]
    fn single_click_still_collapses_the_caret_byte_identically() {
        // (c) Fix 1 byte-identical guard: a lone click (outside any prior press's
        // multi-click window) still collapses to a plain caret at the clicked
        // boundary — unchanged from the pre-fix single-click behaviour.
        let (mut runtime, _mirror) = focused_editable("ab cd");
        let ox = editing_text_origin_x(&runtime);
        let x = ox + ADVANCE * 4.0; // inside "cd"
        runtime.handle_pointer(mouse_t(PointerEventKind::Down, x, 25.0, 1_000_000));
        runtime.handle_pointer(mouse_t(PointerEventKind::Up, x, 25.0, 1_000_000));
        let sel = runtime.editing_selection().expect("session active");
        assert!(sel.is_collapsed(), "a single click collapses to a caret");
        assert_eq!(sel.focus(), 4, "the caret sits at the clicked boundary");
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
