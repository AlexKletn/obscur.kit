//! The deterministic headless driver: [`HeadlessHost`].

use std::fmt;

use obscur_identity::{RuntimeId, WindowId};
use obscur_runtime::frame::{FrameContext, FrameId, FramePipeline, FrameReason, FrameResult};
use obscur_runtime::orchestration::StyleInputs;
use obscur_scene::scene::{Scene, SceneSnapshot};
use obscur_style::matching::Stylesheet;
use obscur_tree::tree::RetainedTree;
use obscur_view::view::View;

use crate::deterministic::{DeterministicClock, DeterministicExecutor};

use super::{HeadlessConfig, VirtualWindow};

/// The official windowless, GPU-less entry point for reproducible tests
/// (spec §7.4).
///
/// A `HeadlessHost` owns all cross-frame state — the [`FramePipeline`], the
/// retained [`RetainedTree`], the mutable [`Scene`], a [`DeterministicClock`],
/// and a [`DeterministicExecutor`] — and drives frames with no OS window and no
/// GPU. It is the headless analogue of the real runtime: construct it,
/// [`render`](Self::render) desired view trees, and inspect the resulting scene
/// and tree.
///
/// Every step is deterministic. [`render`](Self::render) is the headless
/// equivalent of a frame: it runs the whole `View -> reconcile -> style ->
/// layout -> scene` pipeline synchronously. The same sequence of `render` calls
/// with the same inputs always produces the same results, because the pipeline
/// is deterministic and both time and background work only advance when the test
/// drives them (via [`advance_time`](Self::advance_time) and
/// [`run_tasks`](Self::run_tasks)).
///
/// # Example flow
///
/// 1. `let mut host = HeadlessHost::new(runtime_id);`
/// 2. `let result = host.render(&desired, &inputs);` — one deterministic frame.
/// 3. Inspect `result` (phase flags, snapshot) or [`snapshot`](Self::snapshot) /
///    [`tree`](Self::tree) for the current committed state.
pub struct HeadlessHost {
    /// Identity of the host this driver stands in for.
    runtime_id: RuntimeId,
    /// The viewport/DPI configuration frames are produced under.
    config: HeadlessConfig,
    /// The windowless mapping of the host's single window.
    window: VirtualWindow,
    /// The synchronous frame driver; stateless across frames.
    pipeline: FramePipeline,
    /// The retained tree carried across frames.
    tree: RetainedTree,
    /// The mutable scene composed into each frame.
    scene: Scene,
    /// The manually-advanced virtual clock.
    clock: DeterministicClock,
    /// The manually-driven deterministic job runner.
    executor: DeterministicExecutor,
    /// The id assigned to the next produced frame.
    next_frame: FrameId,
}

impl HeadlessHost {
    /// The identity of the single virtual window a host owns.
    const WINDOW_ID: WindowId = WindowId::from_raw(0);

    /// Creates a headless host with the default [`HeadlessConfig`].
    ///
    /// The host starts with a fresh pipeline, an empty retained tree and scene, a
    /// clock at time zero, an idle executor, and a virtual window sized to the
    /// configuration's viewport. The first produced frame will be
    /// [`FrameId::FIRST`].
    #[must_use]
    pub fn new(runtime_id: RuntimeId) -> Self {
        Self::with_config(runtime_id, HeadlessConfig::default())
    }

    /// Creates a headless host with an explicit [`HeadlessConfig`].
    ///
    /// The virtual window is sized to the configuration's viewport and scale, and
    /// all cross-frame state is fresh, exactly as in [`new`](Self::new).
    #[must_use]
    pub fn with_config(runtime_id: RuntimeId, config: HeadlessConfig) -> Self {
        let window = VirtualWindow::new(Self::WINDOW_ID, config.viewport(), config.scale());
        Self {
            runtime_id,
            config,
            window,
            pipeline: FramePipeline::new(),
            tree: RetainedTree::new(),
            scene: Scene::new(),
            clock: DeterministicClock::new(),
            executor: DeterministicExecutor::new(),
            next_frame: FrameId::FIRST,
        }
    }

    /// Renders one deterministic frame for the `desired` view tree and returns
    /// its [`FrameResult`].
    ///
    /// This is the core of the host and the headless equivalent of a frame. It
    /// builds a [`FrameContext`] for the next [`FrameId`] with reason
    /// [`FrameReason::UiCommit`] under the configured viewport, runs the whole
    /// pipeline (`View -> reconcile -> style -> layout -> scene`) over the
    /// host-owned tree and scene, advances the frame counter, and returns the
    /// result. No OS or GPU is involved and the call never panics.
    ///
    /// Calling `render` twice with the same `desired` tree and `inputs` produces
    /// the same snapshot display list: reconciliation preserves identity, and
    /// every stage is deterministic.
    pub fn render(&mut self, desired: &View, inputs: &StyleInputs) -> FrameResult {
        let ctx = FrameContext::new(self.next_frame, FrameReason::UiCommit, self.config.viewport());
        let result = self
            .pipeline
            .run(&mut self.tree, &mut self.scene, ctx, desired, inputs);
        self.next_frame = self.next_frame.next();
        result
    }

    /// Renders one deterministic frame that **matches authored stylesheets**
    /// against the reconciled tree, and returns its [`FrameResult`].
    ///
    /// This is the styled counterpart of [`render`](Self::render): instead of
    /// taking pre-cascaded [`StyleInputs`], it hands `sheets` to the pipeline's
    /// [`run_matched`](FramePipeline::run_matched), which matches and cascades
    /// them against the just-reconciled tree so authored CSS reaches paint. It
    /// builds the same [`FrameContext`] for the next [`FrameId`], advances the
    /// frame counter identically, and never panics.
    ///
    /// `sheets` must be ordered by ascending cascade origin (system-default
    /// first, inline last). An empty `sheets` slice matches nothing and is
    /// equivalent to [`render`](Self::render) with empty inputs.
    pub fn render_styled(&mut self, desired: &View, sheets: &[Stylesheet]) -> FrameResult {
        let ctx = FrameContext::new(self.next_frame, FrameReason::UiCommit, self.config.viewport());
        let result =
            self.pipeline
                .run_matched(&mut self.tree, &mut self.scene, ctx, desired, sheets);
        self.next_frame = self.next_frame.next();
        result
    }

    /// Returns an immutable snapshot of the current scene.
    ///
    /// This is the scene as of the most recent [`render`](Self::render); a
    /// renderer (or a test) consumes the snapshot's display list without touching
    /// the mutable scene.
    #[must_use]
    pub fn snapshot(&self) -> SceneSnapshot {
        self.scene.snapshot()
    }

    /// Advances the deterministic clock by `nanos` nanoseconds.
    ///
    /// Time in a headless run only moves when the test asks it to, keeping any
    /// time-dependent behaviour reproducible.
    pub fn advance_time(&mut self, nanos: u64) {
        self.clock.advance(nanos);
    }

    /// Drives all queued deterministic jobs to completion.
    ///
    /// Background work never runs on its own in a headless run; it runs in FIFO
    /// order exactly when the test calls this.
    pub fn run_tasks(&self) {
        self.executor.run();
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

    /// Returns the deterministic clock driving this host.
    #[must_use]
    pub fn clock(&self) -> &DeterministicClock {
        &self.clock
    }

    /// Returns the deterministic executor driving this host's background work.
    #[must_use]
    pub fn executor(&self) -> &DeterministicExecutor {
        &self.executor
    }

    /// Returns the viewport/DPI configuration this host renders under.
    #[must_use]
    pub fn config(&self) -> HeadlessConfig {
        self.config
    }

    /// Returns the host's virtual window.
    #[must_use]
    pub fn window(&self) -> VirtualWindow {
        self.window
    }

    /// Returns the identity of the host this driver stands in for.
    #[must_use]
    pub fn runtime_id(&self) -> RuntimeId {
        self.runtime_id
    }
}

impl fmt::Debug for HeadlessHost {
    /// Reports the stable identity of the host — its runtime id, configuration,
    /// and the next frame id — without touching the opaque pipeline, tree, scene,
    /// or executor internals.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HeadlessHost")
            .field("runtime_id", &self.runtime_id)
            .field("config", &self.config)
            .field("next_frame", &self.next_frame)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::{ScaleFactor, Size};
    use obscur_identity::ComponentId;
    use obscur_task::Clock;
    use obscur_view::view::{ViewFragment, ViewNode};

    fn runtime_id() -> RuntimeId {
        RuntimeId::from_raw(1)
    }

    /// A small view: two top-level element children.
    fn sample_view() -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(10)),
            ViewNode::element(ComponentId::from_raw(11)),
        ]))
    }

    #[test]
    fn new_uses_default_config_and_window_sized_to_viewport() {
        let host = HeadlessHost::new(runtime_id());
        assert_eq!(host.config(), HeadlessConfig::default());
        assert_eq!(host.runtime_id(), runtime_id());
        assert_eq!(host.window().size(), HeadlessConfig::default().viewport());
        assert_eq!(host.window().id(), WindowId::from_raw(0));
        assert!(host.window().visible());
    }

    #[test]
    fn with_config_sizes_window_to_config() {
        let config =
            HeadlessConfig::with_viewport(Size::new(320.0, 240.0)).with_scale(ScaleFactor::new(2.0));
        let host = HeadlessHost::with_config(runtime_id(), config);
        assert_eq!(host.window().size(), Size::new(320.0, 240.0));
        assert_eq!(host.window().scale().get(), 2.0);
        assert_eq!(host.window().physical_size(), Size::new(640.0, 480.0));
    }

    #[test]
    fn render_runs_the_pipeline() {
        let mut host = HeadlessHost::new(runtime_id());
        let inputs = StyleInputs::new();
        let result = host.render(&sample_view(), &inputs);

        assert!(result.reconciled(), "first frame populates the tree");
        assert!(result.composed(), "a scene is always composed");
        assert!(result.is_balanced(), "the display list is balanced");
        assert_eq!(result.id(), FrameId::FIRST);
    }

    #[test]
    fn render_is_deterministic_across_calls() {
        let mut host = HeadlessHost::new(runtime_id());
        let view = sample_view();
        let inputs = StyleInputs::new();

        let first = host.render(&view, &inputs);
        let first_commands: Vec<_> = first.snapshot().display_list().commands().to_vec();

        let second = host.render(&view, &inputs);
        let second_commands: Vec<_> = second.snapshot().display_list().commands().to_vec();

        assert_eq!(
            first_commands, second_commands,
            "the same view + inputs yields the same display list"
        );
        // The host snapshot reflects the latest committed scene identically.
        let host_commands: Vec<_> = host.snapshot().display_list().commands().to_vec();
        assert_eq!(second_commands, host_commands);
    }

    #[test]
    fn frame_ids_advance_across_renders() {
        let mut host = HeadlessHost::new(runtime_id());
        let view = sample_view();
        let inputs = StyleInputs::new();

        let first = host.render(&view, &inputs);
        let second = host.render(&view, &inputs);
        let third = host.render(&view, &inputs);

        assert_eq!(first.id(), FrameId::FIRST);
        assert_eq!(second.id(), FrameId::FIRST.next());
        assert_eq!(third.id(), FrameId::FIRST.next().next());
        assert!(first.id() < second.id() && second.id() < third.id());
    }

    #[test]
    fn advance_time_moves_the_clock() {
        let mut host = HeadlessHost::new(runtime_id());
        let before = host.clock().now();
        host.advance_time(1_000);
        let after = host.clock().now();
        assert!(after > before);
    }

    #[test]
    fn run_tasks_drains_the_executor() {
        let host = HeadlessHost::new(runtime_id());
        host.executor().spawn(|| {});
        assert_eq!(host.executor().pending(), 1);
        host.run_tasks();
        assert_eq!(host.executor().pending(), 0);
    }

    /// A view whose single top-level element carries the `counter` class.
    fn counter_view() -> View {
        use obscur_view::view::{CLASS, ViewProperty, ViewValue};
        View::from_fragment(ViewFragment::from_nodes(vec![ViewNode::element(
            ComponentId::from_raw(10),
        )
        .with_prop(ViewProperty::new(
            CLASS,
            ViewValue::Text("counter".into()),
        ))]))
    }

    /// `.counter { background-color: red; width: 100px; height: 50px }`.
    fn counter_stylesheet() -> Stylesheet {
        use obscur_style::cascade::CascadeOrigin;
        use obscur_style::matching::{BlockDeclaration, StyleRule};
        use obscur_style::property::{Length, PropertyId, PropertyValue, StyleColor};
        use obscur_style::selector::{
            ComplexSelector, CompoundSelector, SelectorList, SimpleSelector,
        };

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

    #[test]
    fn render_styled_paints_matched_background() {
        // Acceptance: authored CSS reaches paint through the host. A matched
        // `.counter` element produces a non-empty display list; no stylesheet
        // produces an empty one.
        let mut host = HeadlessHost::new(runtime_id());
        let view = counter_view();
        let sheet = counter_stylesheet();

        let styled = host.render_styled(&view, std::slice::from_ref(&sheet));
        assert!(styled.reconciled());
        assert!(styled.restyled(), "the matched node resolved styles");
        assert!(styled.relaid_out(), "the sized node arranged a box");
        assert!(
            !styled.snapshot().display_list().is_empty(),
            "matched background produced a non-empty display list"
        );
        assert!(styled.is_balanced());
        assert_eq!(styled.id(), FrameId::FIRST);

        // A second host with no stylesheet: empty display list.
        let mut bare = HeadlessHost::new(runtime_id());
        let unstyled = bare.render_styled(&view, &[]);
        assert!(
            unstyled.snapshot().display_list().is_empty(),
            "no stylesheet means no fill"
        );
    }

    #[test]
    fn render_styled_advances_frame_ids() {
        let mut host = HeadlessHost::new(runtime_id());
        let view = counter_view();
        let sheet = counter_stylesheet();

        let first = host.render_styled(&view, std::slice::from_ref(&sheet));
        let second = host.render_styled(&view, std::slice::from_ref(&sheet));
        assert_eq!(first.id(), FrameId::FIRST);
        assert_eq!(second.id(), FrameId::FIRST.next());
    }

    #[test]
    fn debug_reports_identity_fields() {
        let host = HeadlessHost::new(runtime_id());
        let rendered = format!("{host:?}");
        assert!(rendered.contains("HeadlessHost"));
        assert!(rendered.contains("runtime_id"));
        assert!(rendered.contains("next_frame"));
    }
}
