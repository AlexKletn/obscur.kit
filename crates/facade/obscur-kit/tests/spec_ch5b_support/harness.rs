//! Shared harness for the `spec_ch5b_*` **specification-derived** test files.
//!
//! These tests are NOT implementation-derived. Every expected value in the
//! `spec_ch5b_*.rs` files was written by an agent reading the specification
//! (obscur.kit §5.4 "Overlay host mode", §5.5, §5.9, §5.13) **without opening
//! any source in this repository**. This module exists only to give those cases
//! a compiling home; it contains no expectations of its own.
//!
//! Nothing here may be "fixed" to make a case pass. A failing `spec_ch5b_*` test
//! is a claim about the implementation, not about the test.
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

/// A mock external engine that owns the window, the event loop, the render graph,
/// the lifecycle and the frame callback, and hands the kit only what §5.4 says
/// the kit receives: viewport, input, clock, render target, and frame scheduling
/// hooks.
///
/// It deliberately exposes no way to create a window and never enters a loop:
/// [`windows_created`](Self::windows_created) and
/// [`event_loop_entered`](Self::event_loop_entered) are the observations the
/// §5.4 cases assert on.
pub struct MockOverlayHost {
    /// The kit instance the host drives. Constructed with a viewport only — no
    /// window, no surface, no event loop.
    runtime: UiRuntime,
    /// The viewport the host supplies.
    viewport: Size,
    /// The host's clock, in milliseconds. The kit must read time from here.
    clock_ms: f64,
    /// The clock reading the kit's motion was last driven to.
    motion_at_ms: f64,
    /// Stylesheets installed through the kit's own CSS frontend.
    sheets: Vec<Stylesheet>,
    /// The host's input queue. The §5.4 cases keep it empty.
    input: Vec<InputEvent>,
    /// How many OS windows this host created. Always incremented by the host, so
    /// a non-zero value would mean the host itself made one.
    windows_created: usize,
    /// Whether the host's event loop was ever entered.
    event_loop_entered: bool,
    /// How many times the kit asked the host to schedule another frame, observed
    /// after each host-driven frame.
    frame_hook_calls: usize,
}

impl MockOverlayHost {
    /// Creates an overlay-mode kit against a host supplying a `w x h` viewport at
    /// scale 1.0, an empty input queue and a clock reading 0 ms.
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

    /// Installs authored CSS through the kit's own frontend.
    pub fn install_css(&mut self, css: &str) {
        let mut pipeline = CssPipeline::new();
        pipeline.install(&mut self.runtime, &[CssSource::anonymous(css)]);
        self.sheets = pipeline.sheets().to_vec();
    }

    /// The host-driven frame entry: sets the host clock to `host_clock_ms` and
    /// asks the kit for one frame's worth of render commands. It returns rather
    /// than blocking, and never enters a loop.
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

    /// The host clock's current reading, in milliseconds.
    #[must_use]
    pub const fn clock_ms(&self) -> f64 {
        self.clock_ms
    }

    /// How many OS windows exist for this host.
    #[must_use]
    pub const fn windows_created(&self) -> usize {
        self.windows_created
    }

    /// Whether the host's event loop was ever entered.
    #[must_use]
    pub const fn event_loop_entered(&self) -> bool {
        self.event_loop_entered
    }

    /// How many times the kit asked the host to schedule a frame.
    #[must_use]
    pub const fn frame_hook_calls(&self) -> usize {
        self.frame_hook_calls
    }

    /// How many frames the kit currently has queued for the host to drive.
    #[must_use]
    pub fn pending_frame_requests(&self) -> usize {
        self.runtime.pending_frames()
    }

    /// The kit instance, for reads the host does not wrap.
    #[must_use]
    pub const fn runtime(&self) -> &UiRuntime {
        &self.runtime
    }

    /// The last returned batch of render commands, in paint order.
    #[must_use]
    pub fn commands(&self) -> Vec<PaintCommand> {
        self.runtime.snapshot().display_list().commands().to_vec()
    }

    /// The bounding rectangles of every solid fill in the returned batch.
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

    /// The opacity of every group layer pushed in the returned batch.
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

    /// Rasterizes the returned batch onto a host-supplied render target,
    /// producing the texture §5.4 says the kit may hand back.
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

    /// A cheap identity for the currently published batch, so a test can assert
    /// that nothing repainted while the host drove nothing.
    #[must_use]
    pub fn batch_fingerprint(&self) -> (usize, Vec<String>) {
        let commands = self.commands();
        let shapes = commands.iter().map(|c| format!("{c:?}")).collect();
        (commands.len(), shapes)
    }

    /// The number of queued but undelivered host input events.
    #[must_use]
    pub fn pending_input(&self) -> usize {
        self.input.len()
    }
}

/// An element node carrying `class`, with `kids` as its children.
#[must_use]
pub fn el(id: u64, class: &str, kids: Vec<ViewNode>) -> ViewNode {
    ViewNode::element(obscur_kit::identity::ComponentId::from_raw(id))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
        .with_children(ViewFragment::from_nodes(kids))
}

/// A view whose root fragment is `nodes`.
#[must_use]
pub fn view(nodes: Vec<ViewNode>) -> View {
    View::from_fragment(ViewFragment::from_nodes(nodes))
}

/// Asserts `actual ≈ expected` within `tolerance`.
pub fn approx(actual: f32, expected: f32, tolerance: f32, what: &str) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{what}: expected {expected}, got {actual}"
    );
}
