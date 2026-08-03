#![doc = include_str!("ui_runtime.doc/_module.md")]

use std::collections::HashMap;
use std::fmt;

use obscur_diagnostics::DiagnosticSink;
use obscur_identity::{NodeId, RuntimeId};
use obscur_interaction::command::Modifiers;
use obscur_interaction::hover::HoverChain;
use obscur_localization::locale::LanguageTag;
use obscur_scene::scene::Scene;
use obscur_style::property::EnvironmentValues;
use obscur_tree::reconciliation::Reconciler;
use obscur_tree::tree::RetainedTree;
use obscur_view::view::View;

use super::{
    AccessibilityPublisher, BackgroundWork, CanvasPainters, CascadeSheets, FramePlan,
    FrameTelemetry, HandlerRegistry, HitTesting, InputQueue, LayoutInputs, MotionDriver,
    PointerEventBindings, PointerSessions, ResourceBridge, ScrollOffsets, StyleKey,
    SurfaceGeometry, TextEditSession, TreeInspector, VirtualLists,
};
use crate::frame::FrameId;

use crate::component::ComponentStore;
use crate::orchestration::{
    ArrangeCache, ComputedStyles, DragCoordinator, DragOutcome, GeneratedContent, LayoutBoxes,
    LayoutCoordinator, MeasureCache, PseudoStates, SceneCoordinator, StyleCoordinator, StyleInputs,
    StyleMatchingCoordinator,
};
use crate::scheduler::FrameScheduler;

use crate::transaction::ReconcileEnactor;

mod accessibility;
mod canvas;
mod caret_visibility;
mod command_routing;
mod composition;
mod construction;
mod diagnostics;
mod event_listeners;
mod focus;
mod frame_classification;
mod frame_entry;
mod frame_output;
mod frame_pipeline;
mod frame_requests;
mod gestures;
mod interaction_state;
mod keyboard;
mod motion;
mod pending_work;
mod pointer;
mod pointer_dispatch;
mod queued_input;
mod resource_resolution;
mod resources;
mod scrolling;
mod styling;
mod surface;
mod text_edit_commands;
mod text_editing;
mod text_pointer;
mod text_ports;
mod view_source;
mod virtual_lists;

#[doc = include_str!("ui_runtime.doc/UiRuntime.md")]
pub struct UiRuntime {
    #[doc = include_str!("ui_runtime.doc/id.md")]
    id: RuntimeId,
    #[doc = include_str!("ui_runtime.doc/tree.md")]
    tree: RetainedTree,
    #[doc = include_str!("ui_runtime.doc/scene.md")]
    scene: Scene,
    #[doc = include_str!("ui_runtime.doc/reconciler.md")]
    reconciler: Reconciler,
    #[doc = include_str!("ui_runtime.doc/enactor.md")]
    enactor: ReconcileEnactor,
    #[doc = include_str!("ui_runtime.doc/matcher.md")]
    matcher: StyleMatchingCoordinator,
    #[doc = include_str!("ui_runtime.doc/style.md")]
    style: StyleCoordinator,
    #[doc = include_str!("ui_runtime.doc/layout.md")]
    layout: LayoutCoordinator,
    #[doc = include_str!("ui_runtime.doc/measure_cache.md")]
    measure_cache: MeasureCache,
    #[doc = include_str!("ui_runtime.doc/arrange_cache.md")]
    arrange_cache: ArrangeCache,
    #[doc = include_str!("ui_runtime.doc/scene_composer.md")]
    scene_composer: SceneCoordinator,
    #[doc = include_str!("ui_runtime.doc/locale.md")]
    locale: LanguageTag,
    #[doc = include_str!("ui_runtime.doc/scheduler.md")]
    scheduler: FrameScheduler,
    #[doc = include_str!("ui_runtime.doc/pseudo.md")]
    pseudo: PseudoStates,
    #[doc = include_str!("ui_runtime.doc/boxes.md")]
    boxes: LayoutBoxes,
    #[doc = include_str!("ui_runtime.doc/inputs.md")]
    inputs: StyleInputs,
    #[doc = include_str!("ui_runtime.doc/computed.md")]
    computed: ComputedStyles,
    #[doc = include_str!("ui_runtime.doc/generated_content.md")]
    generated_content: HashMap<NodeId, GeneratedContent>,
    #[doc = include_str!("ui_runtime.doc/overlaid.md")]
    overlaid: ComputedStyles,
    #[doc = include_str!("ui_runtime.doc/last_style.md")]
    last_style: Option<StyleKey>,
    #[doc = include_str!("ui_runtime.doc/hits.md")]
    hits: HitTesting,
    #[doc = include_str!("ui_runtime.doc/hover.md")]
    hover: HoverChain,
    #[doc = include_str!("ui_runtime.doc/last_layout_inputs.md")]
    last_layout_inputs: Option<LayoutInputs>,
    #[doc = include_str!("ui_runtime.doc/inspector.md")]
    inspector: TreeInspector,
    #[doc = include_str!("ui_runtime.doc/focused.md")]
    focused: Option<NodeId>,
    #[doc = include_str!("ui_runtime.doc/focus_visible.md")]
    focus_visible: bool,
    #[doc = include_str!("ui_runtime.doc/pointers.md")]
    pointers: PointerSessions,
    #[doc = include_str!("ui_runtime.doc/canvas_painters.md")]
    canvas_painters: CanvasPainters,
    #[doc = include_str!("ui_runtime.doc/pointer_bindings.md")]
    pointer_bindings: PointerEventBindings,
    #[doc = include_str!("ui_runtime.doc/environment.md")]
    environment: EnvironmentValues,
    #[doc = include_str!("ui_runtime.doc/routing.md")]
    routing: HandlerRegistry,
    #[doc = include_str!("ui_runtime.doc/editing.md")]
    editing: Option<TextEditSession>,
    #[doc = include_str!("ui_runtime.doc/modifiers.md")]
    modifiers: Modifiers,
    #[doc = include_str!("ui_runtime.doc/input.md")]
    input: InputQueue,
    #[doc = include_str!("ui_runtime.doc/drag.md")]
    drag: DragCoordinator,
    #[doc = include_str!("ui_runtime.doc/drag_outcome.md")]
    drag_outcome: Option<DragOutcome>,
    #[doc = include_str!("ui_runtime.doc/next_frame.md")]
    next_frame: FrameId,
    #[doc = include_str!("ui_runtime.doc/staged_plan.md")]
    staged_plan: Option<FramePlan>,
    #[doc = include_str!("ui_runtime.doc/surface.md")]
    surface: SurfaceGeometry,
    #[doc = include_str!("ui_runtime.doc/scroll.md")]
    scroll: ScrollOffsets,
    #[doc = include_str!("ui_runtime.doc/virtual_lists.md")]
    virtual_lists: VirtualLists,
    #[doc = include_str!("ui_runtime.doc/motion.md")]
    motion: MotionDriver,
    #[doc = include_str!("ui_runtime.doc/telemetry.md")]
    telemetry: FrameTelemetry,
    #[doc = include_str!("ui_runtime.doc/diagnostics.md")]
    diagnostics: Box<dyn DiagnosticSink>,
    #[doc = include_str!("ui_runtime.doc/background.md")]
    background: BackgroundWork,
    #[doc = include_str!("ui_runtime.doc/root_view.md")]
    root_view: Option<Box<dyn FnMut() -> View>>,
    #[doc = include_str!("ui_runtime.doc/components.md")]
    components: ComponentStore,
    #[doc = include_str!("ui_runtime.doc/sheets.md")]
    sheets: CascadeSheets,
    #[doc = include_str!("ui_runtime.doc/accessibility.md")]
    accessibility: AccessibilityPublisher,
    #[doc = include_str!("ui_runtime.doc/resources.md")]
    resources: ResourceBridge,
}

impl fmt::Debug for UiRuntime {
    #[doc = include_str!("ui_runtime.doc/fmt.md")]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UiRuntime")
            .field("id", &self.id)
            .field("viewport", &self.surface.viewport())
            .field("next_frame", &self.next_frame)
            .field("pending_frames", &self.scheduler.pending_count())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::ComponentId;
    // The §7.1 focus change the one accessibility test left here asserts on, now
    // minted by `accessibility_publisher.rs`, which owns the record it reports
    // `previous` from.
    use obscur_accessibility::event::AccessibilityFocus;
    // The rectangle the paint-command assertions read back, no longer named by the
    // runtime's own code: the rounded-rect hit region moved to `hit_testing.rs`.
    use obscur_geometry::Rect;
    // The paint commands the caret-scroll assertions read back, now emitted by
    // `compose_phase.rs`.
    use obscur_scene::command::PaintCommand;
    // The multi-click window the click-run test times against, now owned by
    // `pointer_sessions.rs` along with the run detection itself. It is
    // `pub(crate)`, which is why that test cannot join the rest of its theme in
    // `tests/text_mouse_caret.rs`.
    use crate::runtime::MULTI_CLICK_WINDOW_MICROS;
    // The vocabulary of the public methods that moved into this file's child
    // modules, still named by the tests that call them from here: the resource
    // ports (`ui_runtime/resources.rs`), the semantic snapshot
    // (`ui_runtime/accessibility.rs`), the focus scopes, the app-command set and
    // the routed targets (`ui_runtime/command_routing.rs`), and the host-driven
    // pseudo states (`ui_runtime/styling.rs`).
    use crate::runtime::{AppCommands, CommandTarget};
    use obscur_accessibility::tree::SemanticSnapshot;
    use obscur_interaction::focus::{FocusScope, FocusScopeId};
    use obscur_style::selector::PseudoState;
    // The frame vocabulary every render in this module spells — the sheets a
    // caller hands a frame and the result it gets back — now named by
    // `ui_runtime/frame_entry.rs` and `ui_runtime/frame_pipeline.rs` rather than
    // by this file's own code.
    use crate::frame::FrameResult;
    use obscur_style::matching::Stylesheet;
    use std::sync::Arc;
    // The last four names this file's own code used to spell before the `impl`
    // emptied into the child modules: the frame reason a caller asks for
    // (`ui_runtime/frame_requests.rs`), the surface extent
    // (`ui_runtime/surface.rs`), the list id (`ui_runtime/virtual_lists.rs`) and
    // the port a host installs (`ui_runtime/accessibility.rs`). They are named
    // here only by the tests that call those methods from this module.
    use crate::frame::FrameReason;
    use obscur_accessibility::port::AccessibilityBridge;
    use obscur_geometry::Size;
    use obscur_identity::InternedSymbol;

    use obscur_resources::state::ResourceState;
    use obscur_scene::command::DisplayList;
    use obscur_scene::paint::SceneImage;
    use obscur_style::cascade::CascadeOrigin;
    use obscur_style::matching::{BlockDeclaration, StyleRule};
    use obscur_style::property::{Length, PropertyId, PropertyValue};
    use obscur_view::view::{CLASS, ViewFragment, ViewNode, ViewProperty, ViewValue};
    use std::cell::RefCell;
    use std::rc::Rc;
    // The §2.4 harness. Fixtures, selectors, mock providers and the resolved-frame
    // normalization these tests share with the integration tests beside them live
    // in `obscur-testkit`, which this crate takes as a dev-dependency: one set of
    // fixtures for both sides of the `--test` line.
    use obscur_testkit::assertion::{PaintedImages, counting_handler};
    use obscur_testkit::fixture::{
        ClassRule, CounterPanel, EditablePanel, HoverBox, ImagePanel, InteractionPanel,
        SingleLineField, TestFixture, TextArea, char_press, mouse, mouse_at, named_press,
    };
    use obscur_testkit::mock::{
        CannedImageDecoder, CannedImageLoader, QueuedExecutor, VariantImageDecoder,
        VariantImageLoader,
    };

    /// The command symbol the two §2.6 routing tests that stayed here dispatch.
    const CMD_SAVE: InternedSymbol = InternedSymbol::from_raw(0x5A7E);

    /// The harness's runtime identity, spelled locally: it is named on nearly
    /// every line below that builds a runtime.
    fn runtime_id() -> RuntimeId {
        TestFixture::RUNTIME_ID
    }

    /// The harness's viewport, spelled locally for the same reason.
    fn viewport() -> Size {
        TestFixture::VIEWPORT
    }

    /// A view whose single top-level element carries the `counter` class.
    fn counter_view() -> View {
        CounterPanel::view()
    }

    /// `.counter { background-color: red; width: 100px; height: 50px }`.
    fn counter_stylesheet() -> Stylesheet {
        CounterPanel::sheet()
    }

    // ---- runtime-driven components (§2.1, §2.14, §3.1) ----------------------
    //
    // The lifecycle itself — mounting, props-driven re-render, the reconcile
    // that must not eat the component's output, unmounting — is
    // `tests/component_lifecycle.rs`, and what a §1.7 boundary decides and
    // reports is `tests/error_boundaries.rs`. What stays here is the half of
    // the theme that is a claim about the frame the runtime *scheduled*: six
    // cases reading the crate-private `peek_frame_reason`, because "it asked
    // for a frame" and "it did not" are not visible through the public API.

    mod components {
        use super::{runtime_id, viewport};
        use crate::frame::FrameReason;
        use crate::runtime::UiRuntime;
        use obscur_diagnostics::{Diagnostic, DiagnosticSink};
        use obscur_identity::{NodeId, SignalId};
        use obscur_testkit::host::{CounterComponent, GuardedComponent};
        use obscur_view::boundary::ErrorBoundaryPolicy;
        use obscur_view::view::{View, ViewValue};
        use std::cell::RefCell;
        use std::rc::Rc;

        /// A runtime with the counter registered, asking for `self_updates`
        /// renders of its own before it settles.
        fn runtime_with(self_updates: u32) -> UiRuntime {
            build_runtime(self_updates, None)
        }

        /// A runtime with a plain counter: no self-updates, no tracked source.
        fn runtime_with_counter() -> UiRuntime {
            build_runtime(0, None)
        }

        /// A runtime whose counter declares a dependency on `tracks`.
        fn runtime_tracking(tracks: SignalId) -> UiRuntime {
            build_runtime(0, Some(tracks))
        }

        fn build_runtime(self_updates: u32, tracks: Option<SignalId>) -> UiRuntime {
            let mut runtime = UiRuntime::new(runtime_id(), viewport());
            runtime
                .register_component(CounterComponent::registration(self_updates, tracks))
                .expect("first registration");
            runtime
        }

        /// A view with one `<Counter step=N/>` element.
        fn counter_view(step_by: i64) -> View {
            CounterComponent::view(step_by)
        }

        /// The text the component rendered underneath its own node.
        ///
        /// The harness says this as `SemanticSelector::child_text`, and these
        /// tests cannot borrow it: its signature names `UiRuntime`, which this
        /// target builds twice.
        fn rendered_text(runtime: &UiRuntime) -> String {
            let child = runtime
                .tree()
                .get(component_node(runtime))
                .expect("live")
                .child_nodes()[0];
            match runtime.tree().get(child).expect("live").kind() {
                obscur_tree::node::NodeKind::Text { content } => content.to_string(),
                other => panic!("expected the component's text output, got {other:?}"),
            }
        }

        /// The node the component instance is mounted on.
        fn component_node(runtime: &UiRuntime) -> NodeId {
            let root = runtime.tree().root().expect("a root");
            runtime.tree().get(root).expect("live").child_nodes()[0]
        }

        const STORE: SignalId = SignalId::from_raw(1);
        const OTHER: SignalId = SignalId::from_raw(2);

        #[test]
        fn notifying_a_tracked_signal_re_renders_only_its_consumers() {
            // THE reactive entry point (§3.1: "dependency tracking invalidates
            // only consumers of changed values"). The host mutates its own state
            // and names what changed; the runtime invalidates exactly the
            // components whose last render READ that source.
            let mut runtime = runtime_tracking(STORE);
            runtime.render_now(&counter_view(2), &[]);
            assert_eq!(rendered_text(&runtime), "2");

            // A source this component does not read invalidates nobody — and so
            // costs nothing, not even a frame.
            assert_eq!(runtime.notify_signal(OTHER), 0);
            assert_eq!(runtime.peek_frame_reason(), None, "no consumer, no frame");

            // The source it does read invalidates it, and schedules the frame.
            assert_eq!(runtime.notify_signal(STORE), 1);
            assert_eq!(runtime.peek_frame_reason(), Some(FrameReason::UiCommit));

            runtime
                .run_due_frame(&counter_view(2), &[])
                .expect("the requested frame is due");
            assert_eq!(rendered_text(&runtime), "4", "2 + 2: it re-rendered");
        }

        #[test]
        fn a_component_that_tracks_nothing_is_never_invalidated_by_a_signal() {
            // The asymmetry that makes this worth having: a component reading no
            // app state is untouched however much state changes.
            let mut runtime = runtime_with_counter();
            runtime.render_now(&counter_view(2), &[]);
            assert_eq!(rendered_text(&runtime), "2");

            assert_eq!(runtime.notify_signal(STORE), 0);
            assert_eq!(runtime.notify_signal(OTHER), 0);
            assert_eq!(runtime.peek_frame_reason(), None);
        }

        #[test]
        fn a_signal_notified_twice_before_a_frame_costs_one_frame() {
            // Coalescing is the scheduler's job; notifying is cheap and idempotent
            // from the caller's side.
            let mut runtime = runtime_tracking(STORE);
            runtime.render_now(&counter_view(2), &[]);

            assert_eq!(runtime.notify_signal(STORE), 1);
            // Already stale, so the second notify finds it dirty but reports it
            // honestly as still depending on a changed source.
            assert_eq!(runtime.notify_signal(STORE), 1);

            runtime
                .run_due_frame(&counter_view(2), &[])
                .expect("one frame services both");
            assert_eq!(rendered_text(&runtime), "4", "rendered once, not twice");
            assert_eq!(runtime.peek_frame_reason(), None, "and settles");
        }

        #[test]
        fn a_dependency_taken_this_render_is_seen_as_current() {
            // The generation is captured at track() time, so a component that just
            // rendered is NOT immediately stale against the value it just read.
            let mut runtime = runtime_tracking(STORE);
            runtime.render_now(&counter_view(2), &[]);
            runtime.notify_signal(STORE);
            runtime.run_due_frame(&counter_view(2), &[]).expect("due");
            assert_eq!(rendered_text(&runtime), "4");
            // Re-tracked at the new generation: nothing further is pending.
            assert_eq!(runtime.peek_frame_reason(), None);
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

        // ---- §1.7 boundaries, §1.6 diagnostics -----------------------------
        //
        // Every other case — the retry, the replacement, the disabled policy,
        // the enclosing boundary that catches what escapes, the shipped
        // diagnostic channel — is `tests/error_boundaries.rs`. This one stays
        // because it asserts the *other* half of a retry: that the frame which
        // spends the budget is actually asked for.

        /// A [`DiagnosticSink`] that collects into a shared buffer, so a test can
        /// read what the runtime reported after installing the sink by value.
        #[derive(Debug, Default, Clone)]
        struct SharedDiagnostics(Rc<RefCell<Vec<Diagnostic>>>);

        impl DiagnosticSink for SharedDiagnostics {
            fn emit(&self, diagnostic: &Diagnostic) {
                self.0.borrow_mut().push(diagnostic.clone());
            }
        }

        /// A runtime with `<Guarded>` registered under `policy`, plus a sink to
        /// read the diagnostics back from.
        fn guarded_runtime(
            policy: ErrorBoundaryPolicy,
            fallback: Option<View>,
        ) -> (UiRuntime, SharedDiagnostics) {
            let mut runtime = UiRuntime::new(runtime_id(), viewport());
            runtime
                .register_component(GuardedComponent::registration(policy, fallback))
                .expect("first registration");
            let diagnostics = SharedDiagnostics::default();
            runtime.set_diagnostic_sink(Box::new(diagnostics.clone()));
            (runtime, diagnostics)
        }

        #[test]
        fn a_retry_schedules_the_frame_that_spends_the_budget() {
            // A retry keeps the previous output, so it is only honest if the
            // retry actually arrives: nothing else here would ask for a frame,
            // and a budget that never gets spent is the stale-UI bug wearing a
            // diagnostic.
            let (mut runtime, _diagnostics) = guarded_runtime(
                ErrorBoundaryPolicy::default(),
                Some(GuardedComponent::text_view("sorry")),
            );
            runtime.render_now(&GuardedComponent::view(ViewValue::Text("2".into())), &[]);
            assert_eq!(
                runtime.peek_frame_reason(),
                Some(FrameReason::UiCommit),
                "the retried component is still stale, so the frame end asks for another"
            );
        }
    }

    /// A view whose single element carries the `img` class.
    fn img_view() -> View {
        ImagePanel::view()
    }

    /// `.img { background-image: url("x"); width: 40px; height: 30px }`.
    fn img_stylesheet() -> Stylesheet {
        ImagePanel::sheet()
    }

    /// Counts the `DrawImage` commands in a display list.
    fn draw_image_count(list: &DisplayList) -> usize {
        PaintedImages::count(list)
    }

    /// A runtime over a fake filesystem holding a 2×2 `x` and a 4×4 `x@2x`, at
    /// device scale `scale`. `x` is the url `img_stylesheet` references.
    fn variant_runtime(scale: f32, files: &[(&str, u8)]) -> (UiRuntime, VariantImageLoader) {
        let loader = VariantImageLoader::new(files);
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        runtime.set_resource_providers(Arc::new(loader.clone()), Arc::new(VariantImageDecoder));
        runtime.set_device_scale(scale);
        (runtime, loader)
    }

    /// The `1x`/`2x` filesystem: the authored `x` is 2×2, its `@2x` variant 4×4.
    const BOTH_VARIANTS: &[(&str, u8)] = VariantImageLoader::BOTH_VARIANTS;

    /// The single resident decoded image, after a frame.
    fn resident_image(runtime: &UiRuntime) -> SceneImage {
        runtime
            .resources
            .images()
            .get("x")
            .expect("the url() image resolved")
            .clone()
    }

    /// The `tile` rect of the single `DrawImage` in a display list.
    fn draw_image_tile(list: &DisplayList) -> Rect {
        PaintedImages::tile(list)
    }

    /// Renders, then drives the deferred no-executor load to completion, then
    /// renders again so the decoded image sizes and composes — returning the
    /// second frame.
    ///
    /// With no blocking executor installed a load is queued out of the frame
    /// phase and runs on the frame thread when `poll_resource_tasks` is called,
    /// after the frame (spec §3.3/§3.4). A single `render_now` therefore no longer
    /// resolves an image in-frame; this is the "render → drain → render" a real
    /// host performs across its event loop, collapsed for a test.
    ///
    /// The harness spells this too, and for the same reason as `counter_child` it
    /// cannot be borrowed here: the signature names `UiRuntime`.
    fn render_resolved(runtime: &mut UiRuntime, view: &View, sheet: &Stylesheet) -> FrameResult {
        let _ = runtime.render_now(view, std::slice::from_ref(sheet));
        let _ = runtime.poll_resource_tasks();
        runtime.render_now(view, std::slice::from_ref(sheet))
    }

    #[test]
    fn a_1x_surface_loads_the_authored_asset_and_never_asks_for_a_variant() {
        let (mut runtime, loader) = variant_runtime(1.0, BOTH_VARIANTS);
        let sheet = img_stylesheet();
        let _ = render_resolved(&mut runtime, &img_view(), &sheet);

        // At base density the ladder is one rung: the `@2x` file is not even asked
        // for, so a 1x app pays nothing for the feature.
        assert_eq!(loader.asked(), vec!["x".into()]);
        let image = resident_image(&runtime);
        assert_eq!(image.width(), 2, "the 2x2 authored asset");
        assert_eq!(image.scale(), 1.0);
    }

    #[test]
    fn a_2x_surface_picks_the_2x_asset() {
        let (mut runtime, loader) = variant_runtime(2.0, BOTH_VARIANTS);
        let sheet = img_stylesheet();
        let _ = render_resolved(&mut runtime, &img_view(), &sheet);

        // The densest rung is asked for first — and it exists, so the ladder stops
        // there and the authored path is never read.
        assert_eq!(loader.asked(), vec!["x@2x".into()]);

        // The decoded pixels are the 4×4 ones, which only `x@2x` could have
        // produced: the 2x surface really selected the 2x file.
        let image = resident_image(&runtime);
        assert_eq!(
            image.width(),
            4,
            "the 4x4 @2x asset, not the 2x2 authored one"
        );
        assert_eq!(image.height(), 4);
        assert_eq!(image.scale(), 2.0);
    }

    #[test]
    fn a_2x_surface_falls_back_to_the_authored_asset_when_no_variant_ships() {
        // The same 2x surface, but the project ships no `@2x` file.
        let (mut runtime, loader) = variant_runtime(2.0, &[("x", 2)]);
        let sheet = img_stylesheet();
        let result = render_resolved(&mut runtime, &img_view(), &sheet);

        // It asks for the variant, misses, and walks down to the authored path.
        assert_eq!(loader.asked(), vec!["x@2x".into(), "x".into()]);
        let image = resident_image(&runtime);
        assert_eq!(image.width(), 2, "the authored asset");
        assert_eq!(image.scale(), 1.0, "which is base density, not 2x");
        // A missing variant is not an error: the image still paints.
        assert_eq!(draw_image_count(result.snapshot().display_list()), 1);
    }

    #[test]
    fn a_2x_asset_paints_at_the_same_logical_size_as_the_1x_one() {
        // THE read-side proof. `background-size: auto` resolves the tile straight
        // to the image's intrinsic size, so the tile *is* the logical size the
        // rest of CSS sizes against.
        let sheet = img_stylesheet();

        let (mut base, _) = variant_runtime(1.0, BOTH_VARIANTS);
        let base_tile = draw_image_tile(
            render_resolved(&mut base, &img_view(), &sheet)
                .snapshot()
                .display_list(),
        );

        let (mut hidpi, _) = variant_runtime(2.0, BOTH_VARIANTS);
        let hidpi_tile = draw_image_tile(
            render_resolved(&mut hidpi, &img_view(), &sheet)
                .snapshot()
                .display_list(),
        );

        // The @2x asset carries twice the pixels (4×4 vs 2×2) …
        assert_eq!(resident_image(&base).width(), 2);
        assert_eq!(resident_image(&hidpi).width(), 4);
        // … yet occupies the SAME 2×2 logical box. Reading the density is what
        // makes that true: at the raw pixel count the @2x tile would be 4×4 and
        // the layout would silently double.
        assert_eq!(base_tile.size, Size::new(2.0, 2.0));
        assert_eq!(
            hidpi_tile.size,
            Size::new(2.0, 2.0),
            "the @2x asset must not paint at double size"
        );
        assert_eq!(base_tile.size, hidpi_tile.size);
    }

    #[test]
    fn the_hub_records_the_loaded_variants_density_and_pixel_size() {
        let (mut runtime, _) = variant_runtime(2.0, BOTH_VARIANTS);
        let sheet = img_stylesheet();
        let _ = render_resolved(&mut runtime, &img_view(), &sheet);

        let metadata = runtime
            .resources
            .state_of("x")
            .and_then(ResourceState::metadata)
            .expect("the load completed");
        // `intrinsic_size` is the representation's pixel count and `scale` its
        // density; the logical size is one over the other.
        assert_eq!(metadata.intrinsic_size(), Some(Size::new(4.0, 4.0)));
        assert_eq!(metadata.scale(), 2.0);
    }

    #[test]
    fn moving_to_a_denser_display_re_resolves_to_the_2x_asset() {
        let (mut runtime, _) = variant_runtime(1.0, BOTH_VARIANTS);
        let sheet = img_stylesheet();
        runtime.render_now(&img_view(), std::slice::from_ref(&sheet));
        // No executor: the load is deferred out of the frame phase; drive it so the
        // 1x asset becomes resident before the density change reads it.
        let _ = runtime.poll_resource_tasks();
        assert_eq!(
            resident_image(&runtime).width(),
            2,
            "starts on the 1x asset"
        );

        // The window is dragged to a 2x display.
        runtime.set_device_scale(2.0);
        runtime.render_now(&img_view(), std::slice::from_ref(&sheet));
        let _ = runtime.poll_resource_tasks();

        let image = resident_image(&runtime);
        assert_eq!(
            image.width(),
            4,
            "picks up the @2x asset rather than keeping the 1x one"
        );
        assert_eq!(image.scale(), 2.0);
    }

    #[test]
    fn a_scale_change_within_one_density_keeps_the_resident_image() {
        let (mut runtime, loader) = variant_runtime(2.0, BOTH_VARIANTS);
        let sheet = img_stylesheet();
        runtime.render_now(&img_view(), std::slice::from_ref(&sheet));
        // No executor: drive the deferred load so its one asked-for path is counted.
        let _ = runtime.poll_resource_tasks();
        let asked_once = loader.asked().len();

        // 2.0 -> 1.5 still selects `@2x`, so there is nothing to reload.
        runtime.set_device_scale(1.5);
        runtime.render_now(&img_view(), std::slice::from_ref(&sheet));
        let _ = runtime.poll_resource_tasks();

        assert_eq!(
            loader.asked().len(),
            asked_once,
            "a scale change that selects the same files must not re-load"
        );
        assert_eq!(resident_image(&runtime).width(), 4);
    }

    // ---- §7.1 the semantic tree's output path -------------------------------
    //
    // The derivation and three of the four publication cases are
    // `tests/semantic_tree.rs`. What stays is the focus one: it reads the
    // private `focused` field, because "the tree the platform was told agrees
    // with the focus the platform was told" is a claim about two writes to one
    // field, and only one of them is on the public surface.

    /// Returns the `button` node id after a frame has populated the tree.
    fn button_node(runtime: &UiRuntime) -> NodeId {
        let root = runtime.tree().root().expect("root after a frame");
        runtime.tree().get(root).unwrap().child_nodes()[0]
    }

    /// A bridge that keeps what it was handed. Counting pushes would let a test
    /// pass while the runtime published an empty or stale tree, so this retains
    /// the snapshot itself and the test asserts what is *in* it.
    #[derive(Default)]
    struct CapturingBridge {
        last: Option<SemanticSnapshot>,
        focus: Option<AccessibilityFocus>,
        pushes: u32,
    }

    impl AccessibilityBridge for CapturingBridge {
        fn update_tree(&mut self, snapshot: &SemanticSnapshot) {
            self.last = Some(snapshot.clone());
            self.pushes += 1;
        }
        fn apply_update(&mut self, _update: &obscur_accessibility::tree::SemanticUpdate) {}
        fn set_focus(&mut self, focus: &AccessibilityFocus) {
            self.focus = Some(*focus);
        }
        fn announce(&mut self, _message: &str, _assertive: bool) {}
        fn poll_actions(&mut self) -> Vec<obscur_accessibility::event::AccessibilityAction> {
            Vec::new()
        }
    }

    /// Installs a capturing bridge and returns the shared handle a host would
    /// keep.
    fn install_capture(runtime: &mut UiRuntime) -> Rc<RefCell<CapturingBridge>> {
        let bridge = Rc::new(RefCell::new(CapturingBridge::default()));
        runtime.set_accessibility_bridge(bridge.clone());
        bridge
    }

    /// A 100x50 `button` with class `btn`, and the sheet that sizes it, so a
    /// press can actually hit it.
    fn sized_button() -> (View, Stylesheet) {
        use obscur_view::view::LOCAL_NAME;
        let view = View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(21))
                .with_prop(ViewProperty::new(
                    LOCAL_NAME,
                    ViewValue::Text("button".into()),
                ))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("btn".into())))
                .with_children(ViewFragment::from_nodes(vec![ViewNode::text("Save")])),
        ]));
        let sheet = Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![class_rule("btn", size(100.0, 50.0))],
        );
        (view, sheet)
    }

    /// Focus is what a screen reader follows, and an adapter converts a tree
    /// against the focus it was last *told* — so a frame that only moves focus
    /// must still publish, and must report the node focus moved from.
    #[test]
    fn moving_focus_publishes_the_move_and_the_tree_that_reflects_it() {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let bridge = install_capture(&mut runtime);
        let (view, sheet) = sized_button();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&view, sheets);
        let button = button_node(&runtime);

        let settled = bridge.borrow().pushes;
        // A real press through the real input path is what moves focus.
        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
        assert_eq!(runtime.focused(), Some(button), "the press focused it");
        let _ = runtime.render_now(&view, sheets);

        let seen = bridge.borrow();
        assert!(seen.pushes > settled, "a focus move reaches the platform");
        assert_eq!(
            seen.focus.expect("focus pushed"),
            AccessibilityFocus::moved_to(button, None),
            "reported as a move from nothing to the button"
        );
        assert!(
            seen.last
                .as_ref()
                .and_then(|s| s.get(button))
                .expect("the button is in the published tree")
                .state()
                .focused,
            "and the published tree agrees about which node has it"
        );
    }

    // ---- §4.17 S4: CSS transitions / @keyframes driven through motion --------

    /// `.box { width: 100px; height: 50px } .box:hover { width: 200px }`.
    fn box_hover_sheet() -> Stylesheet {
        HoverBox::layout_hover_sheet()
    }

    /// Three sibling `.box` elements, so a hover on one has clean siblings to be
    /// about.
    fn three_box_view() -> View {
        HoverBox::row_view()
    }

    #[test]
    fn a_layout_affecting_hover_remeasures_only_the_hovered_node() {
        // §3.4 step 8: the phase is named "perform **incremental** layout", and this
        // is the half `a_layout_affecting_hover_still_relayouts` leaves open. That
        // test proves the phase RUNS; this one proves it does not re-measure the
        // whole tree to do it.
        //
        // The hover is on the LAST sibling, because a flow hands each child a
        // `remaining_height` net of its preceding siblings — so only a change to the
        // last one leaves every other child's constraints untouched. Widening past
        // that needs per-context `LayoutDependency` wiring; see
        // `LayoutCoordinator::layout_invalidated`.
        let view = three_box_view();
        let sheet = box_hover_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        runtime.render_now(&view, sheets);

        let root = runtime.tree().root().expect("a root");
        let boxes = runtime
            .tree()
            .get(root)
            .expect("root")
            .child_nodes()
            .to_vec();
        let [first, second, last] = <[NodeId; 3]>::try_from(boxes).expect("three siblings");

        // The cold frame measured all three.
        for node in [first, second, last] {
            assert!(
                runtime.measure_cache.measured(node),
                "the cold frame must measure {node:?}",
            );
        }

        runtime.set_pseudo_state(last, PseudoState::HOVER);
        runtime.request_frame(FrameReason::Input);
        let hot = runtime
            .run_due_frame(&view, sheets)
            .expect("the input frame is due");

        assert!(hot.relaid_out(), "a width change must relayout");
        assert!(
            runtime.measure_cache.measured(last),
            "the hovered node's width changed, so it must re-measure",
        );
        // The claim. Without it, "incremental layout" is just "layout".
        assert!(
            !runtime.measure_cache.measured(first),
            "an unhovered sibling must not re-measure",
        );
        assert!(
            !runtime.measure_cache.measured(second),
            "an unhovered sibling must not re-measure",
        );
        // And it is still CORRECT: the hovered box really did grow, and the ones
        // served from the cache kept the boxes a full pass gives them.
        assert_eq!(
            runtime.layout_boxes()[&last].size().width,
            200.0,
            "the hover rule applied",
        );
        for node in [first, second] {
            assert_eq!(
                runtime.layout_boxes()[&node].size().width,
                100.0,
                "a cache-served sibling keeps its box",
            );
        }
    }

    #[test]
    fn a_layout_affecting_hover_rearranges_only_the_hovered_node() {
        // The arrange twin of the test above, and the half it left open: that one
        // proves the frame does not re-MEASURE the whole tree, this one proves it
        // does not re-ARRANGE it either. Both run on the real frame path — this is
        // `UiRuntime::run_due_frame`, not the coordinator in isolation.
        //
        // The hover changes the LAST sibling's WIDTH, so in a vertical flow nothing
        // above it moves: the two clean siblings keep both their origin and their
        // constraints, which is exactly the case a served subtree must cover.
        let view = three_box_view();
        let sheet = box_hover_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        runtime.render_now(&view, sheets);

        let root = runtime.tree().root().expect("a root");
        let boxes = runtime
            .tree()
            .get(root)
            .expect("root")
            .child_nodes()
            .to_vec();
        let [first, second, last] = <[NodeId; 3]>::try_from(boxes).expect("three siblings");

        // The cold frame arranged all three, or there is nothing to skip below.
        for node in [first, second, last] {
            assert!(
                runtime.arrange_cache.arranged(node),
                "the cold frame must arrange {node:?}",
            );
        }
        let before = runtime.layout_boxes()[&first];

        runtime.set_pseudo_state(last, PseudoState::HOVER);
        runtime.request_frame(FrameReason::Input);
        let hot = runtime
            .run_due_frame(&view, sheets)
            .expect("the input frame is due");

        assert!(hot.relaid_out(), "a width change must relayout");
        assert!(
            runtime.arrange_cache.arranged(last),
            "the hovered node's width changed, so it must re-arrange",
        );
        // The claim. Without it, "incremental layout" is incremental on one half.
        assert!(
            !runtime.arrange_cache.arranged(first),
            "an unhovered sibling must not re-arrange",
        );
        assert!(
            !runtime.arrange_cache.arranged(second),
            "an unhovered sibling must not re-arrange",
        );
        assert_eq!(
            runtime.arrange_cache.arranged_count(),
            2,
            "exactly the hovered node and the root above it re-arranged",
        );
        // And it is still CORRECT: served is not the same as forgotten.
        assert_eq!(
            runtime.layout_boxes()[&last].size().width,
            200.0,
            "the hover rule applied",
        );
        assert_eq!(
            runtime.layout_boxes()[&first],
            before,
            "a served sibling keeps the exact box a full pass gives it",
        );
    }

    #[test]
    fn swapping_the_shaper_drops_the_retained_measures() {
        // The measure cache's sizes were produced by the outgoing shaping ports, and
        // a port swap is a layout input that no computed-style diff can see. If the
        // cache survived it, the next incremental frame would serve sizes measured
        // by a shaper that is gone.
        let view = three_box_view();
        let sheet = box_hover_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        runtime.render_now(&view, sheets);
        assert!(
            !runtime.measure_cache.is_empty(),
            "the cold frame retained measures",
        );

        runtime.set_text_shaper(Box::new(obscur_text::shaping::MockTextShaper::new()));
        assert!(
            runtime.measure_cache.is_empty(),
            "a shaper swap must drop every retained measure",
        );
    }
    // ---- The interaction panel, spelled for this side of the `--test` line ----
    //
    // Phase 3 step 2's own tests — hit-test → hover/focus → pseudo, the click
    // that resolves a command, the §2.3 listener walk, the keyboard and its
    // §6.7 commands — are `tests/pointer_dispatch.rs`, `tests/click_commands.rs`,
    // `tests/event_listeners.rs` and `tests/keyboard_commands.rs`, which reach
    // the harness directly. What stays here is the same panel spelled for the
    // sections below it — the §2.6 chain read from the inside, and the
    // invalidation classification — which drive it and cannot borrow a
    // `&UiRuntime` across the boundary.

    use obscur_interaction::pointer::{PointerEvent, PointerEventKind};
    // The named keys the caret-scroll and textarea tests press (`Home`, `Enter`),
    // now named by `ui_runtime/keyboard.rs`'s activation check rather than by this
    // file's own code.
    use obscur_interaction::command::NamedKey;

    /// A rule matching `.class`, carrying `decls`.
    fn class_rule(class: &str, decls: Vec<BlockDeclaration>) -> StyleRule {
        ClassRule::new(class).declaring(decls).build()
    }

    /// A `width`/`height` declaration block, both in `px`.
    fn size(width: f32, height: f32) -> Vec<BlockDeclaration> {
        vec![
            BlockDeclaration::normal(PropertyId::Width, PropertyValue::Length(Length::px(width))),
            BlockDeclaration::normal(
                PropertyId::Height,
                PropertyValue::Length(Length::px(height)),
            ),
        ]
    }

    /// A `panel` container (100x100) holding two stacked children `a` and `b`
    /// (each 100x50, so `a` occupies y∈[0,50) and `b` occupies y∈[50,100)).
    fn panel_view() -> View {
        InteractionPanel::view()
    }

    /// Sheet: `a`/`b` are *sized but unpainted*; only `:hover`/`:focus` add a
    /// background — so a paint command for `a`/`b` appears **iff** the
    /// interaction pseudo-class fired end-to-end.
    fn interaction_sheet() -> Stylesheet {
        InteractionPanel::sheet()
    }

    /// Returns `(panel, a, b)` node ids after a frame has populated the tree.
    ///
    /// The harness says this as `InteractionPanel::nodes`, and for the same
    /// reason as [`counter_child`] it cannot be borrowed here: the signature
    /// names `UiRuntime`, which this target builds twice.
    fn panel_nodes(runtime: &UiRuntime) -> (NodeId, NodeId, NodeId) {
        let root = runtime.tree().root().expect("root after a frame");
        let panel = runtime.tree().get(root).unwrap().child_nodes()[0];
        let kids = runtime.tree().get(panel).unwrap().child_nodes().to_vec();
        (panel, kids[0], kids[1])
    }

    // ---- §2.6 command routing chain, from the inside ---------------------
    //
    // "Command routing: 1. focused node; 2. focus scope; 3. ancestors;
    //  4. window; 5. application fallback."
    //
    // The chain read from *outside* is `tests/command_routing.rs`. These two
    // cases cannot be: one takes the window link away through the private
    // `routing` field, and the other dispatches from a node that is not the
    // focused one through the private `dispatch_command_from`. Widening either
    // to relocate a test is what this campaign forbids.

    /// A runtime with `panel_view` rendered and focus seated on child `a`.
    /// Returns `(runtime, panel, a)` — `panel` is `a`'s parent, i.e. the
    /// nearest ancestor link of the chain.
    fn focused_on_a(sheets: &[Stylesheet]) -> (UiRuntime, NodeId, NodeId) {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let _ = runtime.render_now(&panel_view(), sheets);
        let (panel, a, _b) = panel_nodes(&runtime);
        // A press on `a`'s box seats focus there.
        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
        let _ = runtime.run_due_frame(&panel_view(), sheets);
        assert_eq!(
            runtime.focused,
            Some(a),
            "fixture precondition: the press must seat focus on `a`"
        );
        (runtime, panel, a)
    }

    /// §2.6's chain **in order**, and that it is a chain: with one handler at
    /// every one of the five links, the innermost claims the command and no
    /// other runs. Removing that link hands the command to exactly the next one
    /// — five times, down to the application fallback.
    ///
    /// This is the property that distinguishes a chain from a set of registries:
    /// each link is reached *only* when every inner link declined.
    #[test]
    fn the_five_links_are_tried_in_the_specs_order_and_the_first_claim_stops_the_chain() {
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let (mut runtime, panel, a) = focused_on_a(sheets);

        let scope_id = FocusScopeId::from_raw(7);
        runtime.open_focus_scope(FocusScope::new(scope_id).with_member(a).trapping(true));

        // The route itself is the spec's order: focused node, scope, ancestors
        // (panel, then root), window, application.
        let root = runtime.tree().root().expect("root after a frame");
        assert_eq!(
            runtime.command_route(Some(a)),
            vec![
                CommandTarget::Node(a),         // 1. focused node
                CommandTarget::Scope(scope_id), // 2. focus scope
                CommandTarget::Node(panel),     // 3. ancestors, nearest first
                CommandTarget::Node(root),
                CommandTarget::Window, // 4. window
                CommandTarget::App,    // 5. application fallback
            ],
            "the route must be §2.6's chain, in §2.6's order"
        );

        // One handler at each link, all for the SAME symbol.
        let (node_ran, node_handler) = counting_handler();
        let (scope_ran, scope_handler) = counting_handler();
        let (anc_ran, anc_handler) = counting_handler();
        let (win_ran, win_handler) = counting_handler();
        let (app_ran, app_handler) = counting_handler();

        runtime.on_command_at(a, CMD_SAVE, node_handler);
        runtime.on_scope_command(scope_id, CMD_SAVE, scope_handler);
        runtime.on_command_at(panel, CMD_SAVE, anc_handler);
        runtime.on_command(CMD_SAVE, win_handler);
        let app = AppCommands::new();
        app.on_command(CMD_SAVE, app_handler);
        runtime.set_app_commands(app.clone());

        // (node, scope, ancestor, window, app) run-counts so far.
        let fired = || {
            (
                node_ran.get(),
                scope_ran.get(),
                anc_ran.get(),
                win_ran.get(),
                app_ran.get(),
            )
        };

        // 1. The focused node claims it; nothing else sees it.
        assert!(runtime.dispatch_command(CMD_SAVE));
        assert_eq!(fired(), (1, 0, 0, 0, 0), "link 1: the focused node");

        // 2. Without a handler at the focused node, the focus SCOPE claims it —
        //    before the ancestors, which is §2.6's ordering.
        assert_eq!(runtime.clear_node_commands(a), 1);
        assert!(runtime.dispatch_command(CMD_SAVE));
        assert_eq!(fired(), (1, 1, 0, 0, 0), "link 2: the focus scope");

        // 3. With the scope closed, the ancestor claims it.
        assert!(runtime.close_focus_scope(scope_id));
        assert!(runtime.dispatch_command(CMD_SAVE));
        assert_eq!(fired(), (1, 1, 1, 0, 0), "link 3: the ancestor");

        // 4. With no ancestor handler, the window claims it.
        assert_eq!(runtime.clear_node_commands(panel), 1);
        assert!(runtime.dispatch_command(CMD_SAVE));
        assert_eq!(fired(), (1, 1, 1, 1, 0), "link 4: the window");

        // 5. With no window handler, the application fallback claims it — the
        //    chain's last link.
        assert!(runtime.routing.remove_command(CMD_SAVE));
        assert!(runtime.dispatch_command(CMD_SAVE));
        assert_eq!(fired(), (1, 1, 1, 1, 1), "link 5: the application fallback");

        // Nothing left: the command is unclaimed and dispatch says so.
        assert!(app.remove(CMD_SAVE));
        assert!(!runtime.dispatch_command(CMD_SAVE));
        assert_eq!(
            fired(),
            (1, 1, 1, 1, 1),
            "an unclaimed command runs nothing"
        );
    }

    /// open scope is not a global level.
    #[test]
    fn a_focus_scope_link_appears_only_for_its_own_members() {
        let sheet = interaction_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let (mut runtime, _panel, a) = focused_on_a(sheets);
        let (_p, _a, b) = panel_nodes(&runtime);

        let scope_id = FocusScopeId::from_raw(3);
        runtime.open_focus_scope(FocusScope::new(scope_id).with_member(a));

        assert!(
            runtime
                .command_route(Some(a))
                .contains(&CommandTarget::Scope(scope_id)),
            "the scope must be a link for its member"
        );
        assert!(
            !runtime
                .command_route(Some(b))
                .contains(&CommandTarget::Scope(scope_id)),
            "the scope must NOT be a link for a non-member"
        );

        // A command from a non-member never reaches the scope's handler.
        let (ran, handler) = counting_handler();
        runtime.on_scope_command(scope_id, CMD_SAVE, handler);
        assert!(!runtime.dispatch_command_from(Some(b), CMD_SAVE));
        assert_eq!(ran.get(), 0);
        // ...but a command from the member does.
        assert!(runtime.dispatch_command_from(Some(a), CMD_SAVE));
        assert_eq!(ran.get(), 1);
    }

    // ---- A focused editing session, for the cases that read past the API ----
    //
    // The session's own behaviour — seeding, caret motion, selection, the
    // controlled reseed, `readonly` — is `tests/text_editing_session.rs`, and
    // the pointer's half of it is `tests/text_mouse_caret.rs`. This arrangement
    // stays because the tests below it cannot: they read the private `editing`
    // field for a scroll offset that has no accessor, or time against the
    // `pub(crate)` multi-click window.

    /// Sets up a focused editable seeded (from its text child) with `seed`, an
    /// `on-input` handler mirroring the resulting text into the returned string,
    /// and the caret at the end of the seed. Uses the explicit render path.
    fn focused_editable(seed: &str) -> (UiRuntime, Rc<RefCell<String>>) {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        let mirror = Rc::new(RefCell::new(String::new()));
        {
            let s = mirror.clone();
            runtime.on_text_input(EditablePanel::CMD_INPUT, move |resulting| {
                *s.borrow_mut() = resulting.to_owned();
            });
        }
        let sheet = EditablePanel::sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&EditablePanel::view(seed), sheets);
        // Press inside `b` to focus the editable; the runtime seeds the session.
        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
        assert!(
            runtime.focused().is_some(),
            "the press focused the editable"
        );
        (runtime, mirror)
    }

    // ---- Single-line horizontal caret scroll (CSS Overflow 3 §3.1) ------------
    //
    // All four of these read `scroll_left_of`, which reaches into the private
    // `editing` session: a field's horizontal scroll offset has no public
    // accessor, and adding one to relocate a test is what this campaign forbids.

    /// The harness's single-line field, spelled locally: a `panel` (200×100)
    /// holding one 100px-wide `field` that binds `on-input`, `white-space:
    /// nowrap` (so it scrolls rather than wraps) and `overflow: hidden` (so the
    /// scrolled-out glyphs clip).
    fn scroll_field_view(text: &str) -> View {
        SingleLineField::view(text)
    }

    /// The sheet that sizes that field.
    fn scroll_field_sheet() -> Stylesheet {
        SingleLineField::sheet()
    }

    /// Focuses the single-line `field` (seeded empty) and returns the runtime. The
    /// `on-input` handler is registered but the resulting text is not mirrored.
    fn focused_scroll_field() -> UiRuntime {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        runtime.on_text_input(EditablePanel::CMD_INPUT, move |_| {});
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
        let panel = runtime.tree().get(root).unwrap().child_nodes()[0];
        runtime.tree().get(panel).unwrap().child_nodes()[0]
    }

    /// The field's current horizontal scroll offset (reads the private session).
    fn scroll_left_of(runtime: &UiRuntime) -> f32 {
        runtime
            .editing
            .as_ref()
            .expect("active session")
            .scroll_left()
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
        let field_box = runtime
            .layout_boxes()
            .get(&field)
            .expect("a field box")
            .bounds;
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
        assert_eq!(
            runtime.editing_selection().unwrap().focus(),
            0,
            "caret at start"
        );
        assert_eq!(
            scroll_left_of(&runtime),
            0.0,
            "Home resets scroll_left to 0"
        );

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
        assert!(
            focus > 1,
            "the resolved byte is not the head character: {focus}"
        );
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
        assert!(
            relaid.relaid_out(),
            "the multiline edit relaid out the grown box"
        );
    }

    // ---- The textarea, for the compose-only test above ------------------------
    //
    // Multi-line editing itself — Enter, the line-aware caret, the vertical
    // caret scroll — is `tests/text_multiline.rs`. These four are the
    // arrangement the compose-only test needs, spelled on this side of the
    // `--test` line because two of them name `UiRuntime`.

    /// The harness's textarea, spelled locally: an editable node carrying the
    /// reserved CSS type `textarea`, so the runtime treats it MULTILINE — Enter
    /// inserts `\n` and a line can be added, which is exactly what makes it
    /// never compose-only.
    fn textarea_view(text: &str) -> View {
        TextArea::view(text)
    }

    /// The sheet that sizes it: wide enough that newline-split lines never also
    /// wrap, with an exact `line-height`.
    fn textarea_sheet() -> Stylesheet {
        TextArea::sheet()
    }

    /// Focuses the empty `textarea` and returns the runtime with an (empty) session.
    fn focused_textarea() -> UiRuntime {
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        runtime.on_text_input(EditablePanel::CMD_INPUT, move |_| {});
        let sheet = textarea_sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&textarea_view(""), sheets);
        runtime.handle_pointer(mouse(PointerEventKind::Down, 20.0, 10.0));
        assert!(
            runtime.focused().is_some(),
            "the press focused the textarea"
        );
        runtime
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

    // ---- The single-line field has no vertical scroll (CSS Overflow 3 §3.1) ---

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
        assert!(
            scroll_left_of(&runtime) > 0.0,
            "the single-line field scrolls horizontally"
        );
        assert_eq!(
            runtime.scroll_top(field),
            0.0,
            "a single-line input has no vertical scroll"
        );
    }

    // ---- The multi-click window, timed from the inside (§5.11) ---------------

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

    /// A mouse event at `(x, y)` carrying an explicit logical timestamp (µs), so a
    /// test drives the multi-click detector's time window deterministically.
    fn mouse_t(kind: PointerEventKind, x: f32, y: f32, t: u64) -> PointerEvent {
        mouse_at(kind, x, y, t)
    }

    /// The other half of the multi-click window, and the half nothing asserted: a
    /// press that lands on the same spot but **after** the window is a fresh
    /// single click, not a double one.
    ///
    /// Worth its own test because the window is the condition a caller can most
    /// easily make vacuous. `time_micros.saturating_sub(prev)` against a constant
    /// `0` timestamp is always `0`, which is always inside the window — so a host
    /// that never stamps its events (as the `App` runner did not) turns every
    /// second press on a spot into a double click, however many seconds apart.
    /// The rule reads as enforced while only the radius and the target are
    /// actually being checked.
    #[test]
    fn a_press_after_the_multi_click_window_is_a_fresh_single_click() {
        let (mut runtime, _mirror) = focused_editable("ab cd");
        let ox = editing_text_origin_x(&runtime);
        let x = ox + ADVANCE * 1.0; // inside the first word "ab"
        let t = 1_000_000;
        runtime.handle_pointer(mouse_t(PointerEventKind::Down, x, 25.0, t));
        runtime.handle_pointer(mouse_t(PointerEventKind::Up, x, 25.0, t));
        // Same target, same point — only the elapsed time differs, so this
        // isolates the window from the other two conditions.
        let late = t + MULTI_CLICK_WINDOW_MICROS + 1;
        runtime.handle_pointer(mouse_t(PointerEventKind::Down, x, 25.0, late));
        let sel = runtime.editing_selection().expect("session active");
        assert!(
            sel.is_collapsed(),
            "a press past the multi-click window seats a caret; it does not select \
             the word the way the in-window double click does"
        );
    }

    // ---- Audit S1: invalidation classification gates frame phases (§3.4 p5) ----
    //
    // The end-to-end half of this theme is `tests/frame_phases.rs`. These three
    // also assert `FramePlan::for_frame(…)` directly, and `FramePlan` is
    // crate-private — the classification read from the inside, which is what a
    // unit test is for.

    #[test]
    fn first_frame_runs_all_phases_even_when_the_reason_is_input() {
        // Classification: the first frame over an empty tree is full regardless
        // of reason, so the tree, styles, boxes and scene all exist.
        assert_eq!(
            FramePlan::for_frame(FrameReason::Input, true),
            FramePlan::full()
        );

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

    // ---- async resource loading through the blocking port (§3.3) ------------

    #[test]
    fn an_installed_executor_moves_the_load_off_the_frame() {
        // THE §3.3 point: the frame no longer waits for IO. The first frame paints
        // without the image because the load has not run yet; draining it applies
        // the result and asks for a ResourceCompletion frame, which paints it.
        let executor = Arc::new(QueuedExecutor::default());
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        runtime.set_resource_providers(
            Arc::new(CannedImageLoader::working()),
            Arc::new(CannedImageDecoder),
        );
        runtime.set_blocking_executor(executor.clone());

        let sheet = img_stylesheet();
        let first = runtime.render_now(&img_view(), std::slice::from_ref(&sheet));
        assert_eq!(
            draw_image_count(first.snapshot().display_list()),
            0,
            "the frame did not block on the load"
        );
        assert_eq!(executor.pending(), 1, "the load was handed to the port");

        // The job finishes on its own thread and posts a typed message back.
        executor.run_all();
        assert_eq!(
            runtime.poll_resource_tasks(),
            1,
            "the completion was applied"
        );
        assert_eq!(
            runtime.peek_frame_reason(),
            Some(FrameReason::ResourceCompletion),
            "a load that lands while idle asks for its own frame"
        );

        let second = runtime
            .run_due_frame(&img_view(), std::slice::from_ref(&sheet))
            .expect("the ResourceCompletion frame is due");
        assert_eq!(draw_image_count(second.snapshot().display_list()), 1);
    }

    #[test]
    fn polling_with_nothing_finished_costs_nothing() {
        let executor = Arc::new(QueuedExecutor::default());
        let mut runtime = UiRuntime::new(runtime_id(), viewport());
        runtime.set_resource_providers(
            Arc::new(CannedImageLoader::working()),
            Arc::new(CannedImageDecoder),
        );
        runtime.set_blocking_executor(executor.clone());
        let sheet = img_stylesheet();
        runtime.render_now(&img_view(), std::slice::from_ref(&sheet));

        // The job is queued but has not run: nothing to apply, no frame requested.
        assert_eq!(runtime.poll_resource_tasks(), 0);
        assert_eq!(runtime.peek_frame_reason(), None);
    }
}
