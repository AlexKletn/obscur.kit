#![doc = include_str!("harness.doc/_module.md")]
#![allow(dead_code, unreachable_pub)]

use obscur_kit::bootstrap::CssPipeline;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::{Rect, Size};
use obscur_kit::identity::RuntimeId;
use obscur_kit::render::frame::{FrameSubmission, RenderFrame};
use obscur_kit::render::target::{BitmapTarget, RenderTarget};
use obscur_kit::render::{Renderer, RendererId};
use obscur_kit::render_cpu::renderer::CpuRenderer;
use obscur_kit::runtime::InputEvent;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::runtime::frame::FrameResult;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::scene::paint::{Brush, Paint};
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("harness.doc/MockOverlayHost.md")]
pub struct MockOverlayHost {
    #[doc = include_str!("harness.doc/runtime.md")]
    runtime: UiRuntime,
    #[doc = include_str!("harness.doc/viewport.md")]
    viewport: Size,
    #[doc = include_str!("harness.doc/clock_ms.md")]
    clock_ms: f64,
    #[doc = include_str!("harness.doc/motion_at_ms.md")]
    motion_at_ms: f64,
    #[doc = include_str!("harness.doc/sheets.md")]
    sheets: Vec<Stylesheet>,
    #[doc = include_str!("harness.doc/input.md")]
    input: Vec<InputEvent>,
    #[doc = include_str!("harness.doc/windows_created.md")]
    windows_created: usize,
    #[doc = include_str!("harness.doc/event_loop_entered.md")]
    event_loop_entered: bool,
    #[doc = include_str!("harness.doc/frame_hook_calls.md")]
    frame_hook_calls: usize,
}

impl MockOverlayHost {
    #[doc = include_str!("harness.doc/new.md")]
    #[must_use]
    pub fn new(w: f32, h: f32) -> Self {
        Self {
            runtime: UiRuntime::new(RuntimeId::from_raw(1), Size::new(w, h)),
            viewport: Size::new(w, h),
            clock_ms: 0.0,
            motion_at_ms: 0.0,
            sheets: Vec::new(),
            input: Vec::new(),
            windows_created: 0,
            event_loop_entered: false,
            frame_hook_calls: 0,
        }
    }

    #[doc = include_str!("harness.doc/install_css.md")]
    pub fn install_css(&mut self, css: &str) {
        let mut pipeline = CssPipeline::new();
        pipeline.install(&mut self.runtime, &[CssSource::anonymous(css)]);
        self.sheets = pipeline.sheets().to_vec();
    }

    #[doc = include_str!("harness.doc/drive_frame_at.md")]
    pub fn drive_frame_at(&mut self, host_clock_ms: f64, view: &View) -> FrameResult {
        self.clock_ms = host_clock_ms;
        let delta_ms = self.clock_ms - self.motion_at_ms;
        self.motion_at_ms = self.clock_ms;
        if delta_ms != 0.0 {
            // The kit receives the clock: the only time it is told about is the
            // host's.
            self.runtime.advance_motion((delta_ms / 1000.0) as f32);
        }
        let sheets = self.sheets.clone();
        let result = self.runtime.render_now(view, &sheets);
        // The frame-scheduling hook: whatever the kit queued for the host to
        // schedule next.
        self.frame_hook_calls += self.runtime.pending_frames();
        result
    }

    #[doc = include_str!("harness.doc/clock_ms_2.md")]
    #[must_use]
    pub const fn clock_ms(&self) -> f64 {
        self.clock_ms
    }

    #[doc = include_str!("harness.doc/windows_created_2.md")]
    #[must_use]
    pub const fn windows_created(&self) -> usize {
        self.windows_created
    }

    #[doc = include_str!("harness.doc/event_loop_entered_2.md")]
    #[must_use]
    pub const fn event_loop_entered(&self) -> bool {
        self.event_loop_entered
    }

    #[doc = include_str!("harness.doc/frame_hook_calls_2.md")]
    #[must_use]
    pub const fn frame_hook_calls(&self) -> usize {
        self.frame_hook_calls
    }

    #[doc = include_str!("harness.doc/pending_frame_requests.md")]
    #[must_use]
    pub fn pending_frame_requests(&self) -> usize {
        self.runtime.pending_frames()
    }

    #[doc = include_str!("harness.doc/runtime_2.md")]
    #[must_use]
    pub const fn runtime(&self) -> &UiRuntime {
        &self.runtime
    }

    #[doc = include_str!("harness.doc/commands.md")]
    #[must_use]
    pub fn commands(&self) -> Vec<PaintCommand> {
        self.runtime.snapshot().display_list().commands().to_vec()
    }

    #[doc = include_str!("harness.doc/fill_bounds.md")]
    #[must_use]
    pub fn fill_bounds(&self) -> Vec<Rect> {
        self.commands()
            .iter()
            .filter_map(|command| match command {
                PaintCommand::DrawPath {
                    path,
                    paint: Paint::Fill(Brush::Solid(_)),
                } => path.bounds(),
                _ => None,
            })
            .collect()
    }

    #[doc = include_str!("harness.doc/layer_opacities.md")]
    #[must_use]
    pub fn layer_opacities(&self) -> Vec<f32> {
        self.commands()
            .iter()
            .filter_map(|command| match command {
                PaintCommand::PushLayer(layer) => Some(layer.opacity()),
                _ => None,
            })
            .collect()
    }

    #[doc = include_str!("harness.doc/render_to_target.md")]
    #[must_use]
    pub fn render_to_target(&self) -> Vec<u8> {
        let mut renderer = CpuRenderer::new(RendererId::from_raw(9));
        let frame = RenderFrame::new(self.runtime.snapshot(), self.viewport);
        let submission = FrameSubmission::new(frame);
        let target = RenderTarget::bitmap(BitmapTarget::new(self.viewport));
        renderer
            .render(&submission, &target)
            .expect("the host's render target accepted the frame");
        renderer
            .pixmap()
            .expect("a rasterized frame")
            .data()
            .to_vec()
    }

    #[doc = include_str!("harness.doc/batch_fingerprint.md")]
    #[must_use]
    pub fn batch_fingerprint(&self) -> (usize, Vec<String>) {
        let commands = self.commands();
        let shapes = commands.iter().map(|c| format!("{c:?}")).collect();
        (commands.len(), shapes)
    }

    #[doc = include_str!("harness.doc/pending_input.md")]
    #[must_use]
    pub fn pending_input(&self) -> usize {
        self.input.len()
    }
}

#[doc = include_str!("harness.doc/el.md")]
#[must_use]
pub fn el(id: u64, class: &str, kids: Vec<ViewNode>) -> ViewNode {
    ViewNode::element(obscur_kit::identity::ComponentId::from_raw(id))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
        .with_children(ViewFragment::from_nodes(kids))
}

#[doc = include_str!("harness.doc/view.md")]
#[must_use]
pub fn view(nodes: Vec<ViewNode>) -> View {
    View::from_fragment(ViewFragment::from_nodes(nodes))
}

#[doc = include_str!("harness.doc/approx.md")]
pub fn approx(actual: f32, expected: f32, tolerance: f32, what: &str) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{what}: expected {expected}, got {actual}"
    );
}
