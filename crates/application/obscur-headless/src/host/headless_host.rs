#![doc = include_str!("headless_host.doc/_module.md")]

use std::fmt;
use std::sync::Arc;

use obscur_identity::{RuntimeId, WindowId};
use obscur_runtime::UiRuntime;
use obscur_runtime::frame::FrameResult;
use obscur_scene::scene::{Scene, SceneSnapshot};
use obscur_style::matching::Stylesheet;
use obscur_task::{Clock, Instant};
use obscur_tree::tree::RetainedTree;
use obscur_view::view::View;

use crate::deterministic::{DeterministicClock, DeterministicExecutor};

use super::{HeadlessConfig, VirtualWindow};

#[doc = include_str!("headless_host.doc/NANOS_PER_SECOND.md")]
const NANOS_PER_SECOND: f32 = 1_000_000_000.0;

#[doc = include_str!("headless_host.doc/HeadlessHost.md")]
pub struct HeadlessHost {
    #[doc = include_str!("headless_host.doc/config.md")]
    config: HeadlessConfig,
    #[doc = include_str!("headless_host.doc/window.md")]
    window: VirtualWindow,
    #[doc = include_str!("headless_host.doc/runtime.md")]
    runtime: UiRuntime,
    #[doc = include_str!("headless_host.doc/clock.md")]
    clock: DeterministicClock,
    #[doc = include_str!("headless_host.doc/executor.md")]
    executor: Arc<DeterministicExecutor>,
    #[doc = include_str!("headless_host.doc/motion_at.md")]
    motion_at: Instant,
}

impl HeadlessHost {
    #[doc = include_str!("headless_host.doc/WINDOW_ID.md")]
    const WINDOW_ID: WindowId = WindowId::from_raw(0);

    #[doc = include_str!("headless_host.doc/new.md")]
    #[must_use]
    pub fn new(runtime_id: RuntimeId) -> Self {
        Self::with_config(runtime_id, HeadlessConfig::default())
    }

    #[doc = include_str!("headless_host.doc/with_config.md")]
    #[must_use]
    pub fn with_config(runtime_id: RuntimeId, config: HeadlessConfig) -> Self {
        let window = VirtualWindow::new(Self::WINDOW_ID, config.viewport(), config.scale());
        let executor = Arc::new(DeterministicExecutor::new());
        let mut runtime = UiRuntime::new(runtime_id, config.viewport());
        // The whole point of the deterministic executor: with it installed the
        // runtime hands every resource load to it instead of running the load on
        // the frame, so `run_tasks` — and nothing else — decides when a load
        // lands (§7.4 "controlled async completion").
        runtime.set_blocking_executor(executor.clone());
        Self {
            config,
            window,
            runtime,
            clock: DeterministicClock::new(),
            executor,
            motion_at: Instant::from_nanos(0),
        }
    }

    #[doc = include_str!("headless_host.doc/render_styled.md")]
    pub fn render_styled(&mut self, desired: &View, sheets: &[Stylesheet]) -> FrameResult {
        self.runtime.render_now(desired, sheets)
    }

    #[doc = include_str!("headless_host.doc/snapshot.md")]
    #[must_use]
    pub fn snapshot(&self) -> SceneSnapshot {
        self.runtime.snapshot()
    }

    #[doc = include_str!("headless_host.doc/advance_time.md")]
    pub fn advance_time(&mut self, nanos: u64) {
        self.clock.advance(nanos);
        let now = self.clock.now();
        let delta = now.as_nanos().saturating_sub(self.motion_at.as_nanos());
        self.motion_at = now;
        // Motion is the one stage that reads time, so this is where the clock
        // reaches paint: the next frame's overlay samples every timeline at the
        // progress this advance produced (§4.17 style -> motion -> paint).
        self.runtime.advance_motion(delta as f32 / NANOS_PER_SECOND);
    }

    #[doc = include_str!("headless_host.doc/run_tasks.md")]
    pub fn run_tasks(&mut self) -> usize {
        self.executor.run();
        self.runtime.poll_resource_tasks()
    }

    #[doc = include_str!("headless_host.doc/tree.md")]
    #[must_use]
    pub fn tree(&self) -> &RetainedTree {
        self.runtime.tree()
    }

    #[doc = include_str!("headless_host.doc/scene.md")]
    #[must_use]
    pub fn scene(&self) -> &Scene {
        self.runtime.scene()
    }

    #[doc = include_str!("headless_host.doc/runtime_2.md")]
    #[must_use]
    pub fn runtime(&self) -> &UiRuntime {
        &self.runtime
    }

    #[doc = include_str!("headless_host.doc/runtime_mut.md")]
    pub fn runtime_mut(&mut self) -> &mut UiRuntime {
        &mut self.runtime
    }

    #[doc = include_str!("headless_host.doc/clock_2.md")]
    #[must_use]
    pub fn clock(&self) -> &DeterministicClock {
        &self.clock
    }

    #[doc = include_str!("headless_host.doc/executor_2.md")]
    #[must_use]
    pub fn executor(&self) -> &DeterministicExecutor {
        &self.executor
    }

    #[doc = include_str!("headless_host.doc/config_2.md")]
    #[must_use]
    pub fn config(&self) -> HeadlessConfig {
        self.config
    }

    #[doc = include_str!("headless_host.doc/window_2.md")]
    #[must_use]
    pub fn window(&self) -> VirtualWindow {
        self.window
    }

    #[doc = include_str!("headless_host.doc/runtime_id.md")]
    #[must_use]
    pub fn runtime_id(&self) -> RuntimeId {
        self.runtime.id()
    }
}

impl fmt::Debug for HeadlessHost {
    #[doc = include_str!("headless_host.doc/fmt.md")]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HeadlessHost")
            .field("runtime_id", &self.runtime_id())
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::{ScaleFactor, Size};
    use obscur_identity::{ComponentId, NodeId};
    use obscur_resources::handle::ResourceKind;
    use obscur_resources::port::{DecodedResource, LoadedBytes, ResourceDecoder, ResourceLoader};
    use obscur_resources::source::ResourceSource;
    use obscur_resources::state::{ResourceError, ResourceMetadata};
    use obscur_runtime::frame::FrameId;
    use obscur_scene::command::{DisplayList, PaintCommand};
    use obscur_style::matching::Stylesheet;
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
    fn with_config_sizes_window_and_runtime_to_config() {
        let config = HeadlessConfig::with_viewport(Size::new(320.0, 240.0))
            .with_scale(ScaleFactor::new(2.0));
        let host = HeadlessHost::with_config(runtime_id(), config);
        assert_eq!(host.window().size(), Size::new(320.0, 240.0));
        assert_eq!(host.window().scale().get(), 2.0);
        assert_eq!(host.window().physical_size(), Size::new(640.0, 480.0));
        assert_eq!(host.runtime().viewport(), Size::new(320.0, 240.0));
    }

    #[test]
    fn render_runs_the_pipeline() {
        let mut host = HeadlessHost::new(runtime_id());
        let result = host.render_styled(&sample_view(), &[]);

        assert!(result.reconciled(), "first frame populates the tree");
        assert!(result.composed(), "a scene is always composed");
        assert!(result.is_balanced(), "the display list is balanced");
        assert_eq!(result.id(), FrameId::FIRST);
    }

    #[test]
    fn render_is_deterministic_across_calls() {
        let mut host = HeadlessHost::new(runtime_id());
        let view = sample_view();

        let first = host.render_styled(&view, &[]);
        let first_commands: Vec<_> = first.snapshot().display_list().commands().to_vec();

        let second = host.render_styled(&view, &[]);
        let second_commands: Vec<_> = second.snapshot().display_list().commands().to_vec();

        assert_eq!(
            first_commands, second_commands,
            "the same view + sheets yields the same display list"
        );
        // The host snapshot reflects the latest committed scene identically.
        let host_commands: Vec<_> = host.snapshot().display_list().commands().to_vec();
        assert_eq!(second_commands, host_commands);
    }

    #[test]
    fn frame_ids_advance_across_renders() {
        let mut host = HeadlessHost::new(runtime_id());
        let view = sample_view();

        let first = host.render_styled(&view, &[]);
        let second = host.render_styled(&view, &[]);
        let third = host.render_styled(&view, &[]);

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
        assert_eq!(host.clock().now().as_nanos(), before.as_nanos() + 1_000);
    }

    // ---- the clock reaches a real frame (§7.4 deterministic clock) ----------

    /// A view whose single top-level element carries the `counter` class.
    fn counter_view() -> View {
        use obscur_view::view::{CLASS, ViewProperty, ViewValue};
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(10))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("counter".into()))),
        ]))
    }

    /// `.counter { background-color: red; width: 100px; height: 50px }`.
    fn counter_stylesheet() -> Stylesheet {
        use obscur_style::cascade::CascadeOrigin;
        use obscur_style::matching::{BlockDeclaration, StyleRule};
        use obscur_style::property::{Length, PropertyId, PropertyValue, StyleColor};

        Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![StyleRule::new(
                class_selector("counter"),
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

    /// `.counter { height: 50px }` — no authored width, so an animated `width`
    /// is the only thing that can size the box.
    fn height_only_stylesheet() -> Stylesheet {
        use obscur_style::cascade::CascadeOrigin;
        use obscur_style::matching::{BlockDeclaration, StyleRule};
        use obscur_style::property::{Length, PropertyId, PropertyValue};

        Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![StyleRule::new(
                class_selector("counter"),
                vec![BlockDeclaration::normal(
                    PropertyId::Height,
                    PropertyValue::Length(Length::px(50.0)),
                )],
            )],
        )
    }

    /// A one-class selector list (`.name`).
    fn class_selector(name: &str) -> obscur_style::selector::SelectorList {
        use obscur_style::selector::{
            ComplexSelector, CompoundSelector, SelectorList, SimpleSelector,
        };
        SelectorList::new(vec![ComplexSelector::new(
            CompoundSelector::new(vec![SimpleSelector::Class(name.into())]),
            Vec::new(),
        )])
    }

    /// The first child of the retained root — the `.counter` element.
    fn counter_child(host: &HeadlessHost) -> NodeId {
        let tree = host.tree();
        let root = tree.root().expect("a root");
        *tree
            .get(root)
            .expect("root node")
            .child_nodes()
            .first()
            .expect("a child")
    }

    /// The laid-out width of `node` on the host's last frame.
    fn box_width(host: &HeadlessHost, node: NodeId) -> f32 {
        host.runtime()
            .layout_boxes()
            .get(&node)
            .expect("a layout box")
            .size()
            .width
    }

    /// A scalar track running `from -> to`.
    fn scalar_track(from: f32, to: f32) -> obscur_motion::track::KeyframeTrack {
        use obscur_motion::track::{Keyframe, KeyframeTrack};
        use obscur_motion::value::MotionValue;
        KeyframeTrack::from_keyframes(vec![
            Keyframe::new(0.0, MotionValue::scalar(from)),
            Keyframe::new(1.0, MotionValue::scalar(to)),
        ])
    }

    /// Registers `width: 0 -> 100` over one logical second on the host's counter
    /// node and returns it.
    fn animate_width(host: &mut HeadlessHost, view: &View, sheets: &[Stylesheet]) -> NodeId {
        use obscur_motion::transition::TransitionInterruption;
        use obscur_runtime::orchestration::AnimatedKind;
        use obscur_style::property::PropertyId;

        // A first frame populates the tree so the node id exists to animate.
        let _ = host.render_styled(view, sheets);
        let node = counter_child(host);
        host.runtime_mut().animate(
            node,
            PropertyId::Width,
            scalar_track(0.0, 100.0),
            1.0,
            AnimatedKind::LengthPx,
            TransitionInterruption::Blend,
        );
        node
    }

    #[test]
    fn advance_time_drives_an_animation_into_the_next_frames_layout() {
        // THE §7.4 point: the clock the host owns is the clock a rendered frame
        // follows. `advance_time` is the ONLY thing moving this animation, so a
        // host that did not hand its clock to the runtime's motion would leave
        // the width pinned at 0 forever.
        let mut host = HeadlessHost::new(runtime_id());
        let view = counter_view();
        let sheet = height_only_stylesheet();
        let sheets = std::slice::from_ref(&sheet);

        let node = animate_width(&mut host, &view, sheets);
        assert_eq!(host.runtime().active_animations(), 1);

        // t = 0: the animation has not been driven, so width is still ~0.
        let _ = host.render_styled(&view, sheets);
        assert!(box_width(&host, node) < 0.5, "width starts at ~0");

        // t = 0.5s on the host's clock: the sampled value reached the layout box.
        host.advance_time(500_000_000);
        let _ = host.render_styled(&view, sheets);
        assert!(
            (box_width(&host, node) - 50.0).abs() < 0.5,
            "half a clock-second in, the animated width is ~50"
        );

        // t = 1.0s: the animation lands and the timeline deactivates.
        host.advance_time(500_000_000);
        let _ = host.render_styled(&view, sheets);
        assert!(
            (box_width(&host, node) - 100.0).abs() < 0.5,
            "a full clock-second in, the animated width is ~100"
        );
        assert_eq!(
            host.runtime().active_animations(),
            0,
            "the finished timeline deactivated"
        );
    }

    #[test]
    fn without_advance_time_a_frame_never_moves() {
        // The other half of determinism: time does not pass on its own. Three
        // renders with an animation registered and the clock untouched are
        // identical — a host that sampled a wall clock would drift here.
        let mut host = HeadlessHost::new(runtime_id());
        let view = counter_view();
        let sheet = height_only_stylesheet();
        let sheets = std::slice::from_ref(&sheet);

        let node = animate_width(&mut host, &view, sheets);
        let mut widths = Vec::new();
        for _ in 0..3 {
            let _ = host.render_styled(&view, sheets);
            widths.push(box_width(&host, node));
        }

        assert_eq!(host.clock().now().as_nanos(), 0, "the clock never moved");
        assert!(
            widths
                .windows(2)
                .all(|w| (w[0] - w[1]).abs() < f32::EPSILON),
            "no advance_time means no motion: {widths:?}"
        );
    }

    // ---- the executor reaches a real frame (§7.4 controlled completion) -----

    /// A [`ResourceLoader`] yielding four canned bytes for any file source.
    #[derive(Debug)]
    struct MockLoader;

    impl ResourceLoader for MockLoader {
        fn can_load(&self, source: &ResourceSource) -> bool {
            source.is_file()
        }

        fn load(&self, _source: &ResourceSource) -> Result<LoadedBytes, ResourceError> {
            let bytes: Arc<[u8]> = Arc::from(vec![1u8, 2, 3, 4]);
            Ok(LoadedBytes::new(
                bytes,
                ResourceMetadata::new().with_byte_size(4),
            ))
        }
    }

    /// A decoder turning any image into a canned 2x2 opaque RGBA buffer.
    #[derive(Debug)]
    struct MockDecoder;

    impl ResourceDecoder for MockDecoder {
        fn supports(&self, kind: ResourceKind) -> bool {
            kind == ResourceKind::Image
        }

        fn decode(
            &self,
            _kind: ResourceKind,
            _bytes: &[u8],
        ) -> Result<DecodedResource, ResourceError> {
            let rgba: Arc<[u8]> = Arc::from(vec![255u8; 2 * 2 * 4]);
            Ok(DecodedResource::new(
                rgba,
                ResourceMetadata::new().with_intrinsic_size(Size::new(2.0, 2.0)),
            ))
        }
    }

    /// `.img { background-image: url("x"); width: 40px; height: 30px }`.
    fn img_stylesheet() -> Stylesheet {
        use obscur_style::cascade::CascadeOrigin;
        use obscur_style::matching::{BlockDeclaration, StyleRule};
        use obscur_style::property::{Length, PropertyId, PropertyValue};

        Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![StyleRule::new(
                class_selector("img"),
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

    fn img_view() -> View {
        use obscur_view::view::{CLASS, ViewProperty, ViewValue};
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(11))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("img".into()))),
        ]))
    }

    fn draw_image_count(list: &DisplayList) -> usize {
        list.commands()
            .iter()
            .filter(|c| matches!(c, PaintCommand::DrawImage { .. }))
            .count()
    }

    /// A host wired to the mock resource providers.
    fn image_host() -> HeadlessHost {
        let mut host = HeadlessHost::new(runtime_id());
        host.runtime_mut()
            .set_resource_providers(Arc::new(MockLoader), Arc::new(MockDecoder));
        host
    }

    #[test]
    fn a_frames_resource_load_is_queued_on_the_host_executor_not_run_inline() {
        // THE §7.4 point for background work: the host installed its executor as
        // the runtime's blocking port, so a frame hands its load over instead of
        // running it. A host that never installed the executor would paint the
        // image on this first frame and queue nothing.
        let mut host = image_host();
        let sheet = img_stylesheet();

        let first = host.render_styled(&img_view(), std::slice::from_ref(&sheet));
        assert_eq!(
            draw_image_count(first.snapshot().display_list()),
            0,
            "the frame did not block on the load"
        );
        assert_eq!(
            host.executor().pending(),
            1,
            "the load was handed to the host's deterministic executor"
        );
    }

    #[test]
    fn run_tasks_lands_a_queued_load_into_the_next_frame() {
        // `run_tasks` runs the queued job AND drains the completion into the
        // runtime; without the drain the job would run and its result would never
        // reach a frame.
        let mut host = image_host();
        let sheet = img_stylesheet();
        let _ = host.render_styled(&img_view(), std::slice::from_ref(&sheet));

        assert_eq!(host.run_tasks(), 1, "one completion was applied");
        assert!(host.executor().is_idle(), "the queue drained");

        let second = host.render_styled(&img_view(), std::slice::from_ref(&sheet));
        assert_eq!(
            draw_image_count(second.snapshot().display_list()),
            1,
            "the completed load paints on the next frame"
        );
    }

    #[test]
    fn a_load_never_lands_until_run_tasks_is_called() {
        // Controlled completion: rendering repeatedly does not sneak the load in.
        let mut host = image_host();
        let sheet = img_stylesheet();
        let sheets = std::slice::from_ref(&sheet);

        for _ in 0..3 {
            let result = host.render_styled(&img_view(), sheets);
            assert_eq!(
                draw_image_count(result.snapshot().display_list()),
                0,
                "no frame lands the load on its own"
            );
        }
        assert_eq!(host.run_tasks(), 1);
        let result = host.render_styled(&img_view(), sheets);
        assert_eq!(draw_image_count(result.snapshot().display_list()), 1);
    }

    #[test]
    fn run_tasks_with_nothing_queued_applies_nothing() {
        let mut host = HeadlessHost::new(runtime_id());
        let _ = host.render_styled(&sample_view(), &[]);
        assert_eq!(host.run_tasks(), 0);
    }

    // ---- styled rendering ---------------------------------------------------

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
        assert!(rendered.contains("config"));
    }
}
