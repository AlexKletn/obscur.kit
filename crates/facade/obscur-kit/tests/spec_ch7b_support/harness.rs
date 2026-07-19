//! Shared harness for the `spec_ch7b_*` **specification-derived** test files.
//!
//! These tests are NOT implementation-derived. Every expected value in the
//! `spec_ch7b_*.rs` files was written by an agent reading the specification
//! (obscur.kit §7.11 plus the README's core principles) rather than the source.
//! This module exists only to give those cases a compiling home; it **drives**
//! and it **records**, and it carries no expectations of its own.
//!
//! Nothing here may be "fixed" to make a case pass. A failing `spec_ch7b_*` test
//! is a claim about the implementation, not about the test.
#![allow(dead_code, unreachable_pub)]

use std::cell::RefCell;
use std::rc::Rc;

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::diagnostics::{
    BudgetViolation, PerformanceCounter, PerformanceSpan, TelemetrySink,
};
use obscur_kit::geometry::{Point, Size};
use obscur_kit::identity::{InternedSymbol, NodeId, RuntimeId};
use obscur_kit::interaction::command::{KeyEvent, LogicalKey, PhysicalKey};
use obscur_kit::interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};
use obscur_kit::runtime::InputEvent;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::runtime::frame::FrameReason;
use obscur_kit::scene::scene::SceneSnapshot;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::system::power::{PowerProfile, PowerSource, ThermalState};
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};
use obscur_kit::widgets::input::Input;

/// The viewport every §7.11 case drives.
pub const VW: f32 = 400.0;

/// The app-owned command symbol the focused field's events bind to.
pub const CMD: InternedSymbol = InternedSymbol::from_raw(0x0B11);

// ---------------------------------------------------------------------------
// Authoring
// ---------------------------------------------------------------------------

/// Compiles authored CSS text through the real CSS frontend.
#[must_use]
pub fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

/// An element node carrying `class`, with `kids` as its children.
#[must_use]
pub fn el(id: u64, class: &str, kids: Vec<ViewNode>) -> ViewNode {
    ViewNode::element(obscur_kit::identity::ComponentId::from_raw(id))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
        .with_children(ViewFragment::from_nodes(kids))
}

/// A text node.
#[must_use]
pub fn txt(content: &str) -> ViewNode {
    ViewNode::text(content)
}

/// A view whose root fragment is `nodes`.
#[must_use]
pub fn view(nodes: Vec<ViewNode>) -> View {
    View::from_fragment(ViewFragment::from_nodes(nodes))
}

/// A runtime with a `VW x VW` viewport.
#[must_use]
pub fn runtime() -> UiRuntime {
    UiRuntime::new(RuntimeId::from_raw(1), Size::new(VW, VW))
}

// ---------------------------------------------------------------------------
// Power vocabulary, as an author would spell it
// ---------------------------------------------------------------------------

/// The `PowerProfile` denoting **no power saving / full performance**: on mains,
/// fully charged, OS battery-saver off, no thermal pressure.
#[must_use]
pub fn full_performance_profile() -> PowerProfile {
    PowerProfile::new()
        .on_source(PowerSource::Ac)
        .with_battery_level(1.0)
        .low_power_mode(false)
        .with_thermal(ThermalState::Nominal)
}

/// The most restrictive `PowerProfile` the API offers: on battery, nearly flat,
/// OS battery-saver engaged, critical thermal pressure.
#[must_use]
pub fn maximum_saving_profile() -> PowerProfile {
    PowerProfile::new()
        .on_source(PowerSource::Battery)
        .with_battery_level(0.0)
        .low_power_mode(true)
        .with_thermal(ThermalState::Critical)
}

// ---------------------------------------------------------------------------
// Telemetry capture
// ---------------------------------------------------------------------------

/// Everything a [`TelemetrySink`] was handed, shareable with the caller.
#[derive(Debug, Default)]
pub struct TelemetryLog {
    pub spans: Vec<PerformanceSpan>,
    pub counters: Vec<PerformanceCounter>,
    pub violations: Vec<BudgetViolation>,
}

/// A [`TelemetrySink`] that retains everything into a shared [`TelemetryLog`].
#[derive(Debug, Default, Clone)]
pub struct SharedTelemetry(Rc<RefCell<TelemetryLog>>);

impl SharedTelemetry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Every counter, as `(subsystem, label, value)`, in the order recorded.
    #[must_use]
    pub fn counters(&self) -> Vec<(String, String, u64)> {
        self.0
            .borrow()
            .counters
            .iter()
            .map(|c| {
                (
                    format!("{:?}", c.subsystem()),
                    c.label().to_owned(),
                    c.value(),
                )
            })
            .collect()
    }

    /// Every span name, in the order recorded.
    #[must_use]
    pub fn span_names(&self) -> Vec<&'static str> {
        self.0.borrow().spans.iter().map(|s| s.name).collect()
    }

    /// Every budget violation, rendered for comparison.
    #[must_use]
    pub fn violations(&self) -> Vec<String> {
        self.0
            .borrow()
            .violations
            .iter()
            .map(|v| format!("{v:?}"))
            .collect()
    }
}

impl TelemetrySink for SharedTelemetry {
    fn record_span(&mut self, span: &PerformanceSpan) {
        self.0.borrow_mut().spans.push(*span);
    }

    fn record_counter(&mut self, counter: &PerformanceCounter) {
        self.0.borrow_mut().counters.push(counter.clone());
    }

    fn record_violation(&mut self, violation: &BudgetViolation) {
        self.0.borrow_mut().violations.push(violation.clone());
    }
}

// ---------------------------------------------------------------------------
// The effect-bearing app: box-shadow, blurred backdrop, long list, transition
// ---------------------------------------------------------------------------

/// The CSS for the effect-bearing app driven by the "off by default" case: a
/// shadowed card, a blurred backdrop, a long (virtualizable) list and a running
/// transition.
#[must_use]
pub fn effects_css() -> &'static str {
    ".backdrop { width: 400px; height: 400px; backdrop-filter: blur(8px) } \
     .card { width: 200px; height: 60px; box-shadow: 0 2px 8px rgba(0, 0, 0, 0.4); \
             opacity: 0.5; transition: opacity 200ms linear } \
     .list { width: 200px; height: 120px; overflow: scroll } \
     .row { width: 200px; height: 20px }"
}

/// The effect-bearing view: a blurred backdrop wrapping a shadowed card and a
/// long scrolling list.
#[must_use]
pub fn effects_view() -> View {
    let rows: Vec<ViewNode> = (0..200)
        .map(|i| el(1000 + i, "row", vec![txt("row")]))
        .collect();
    view(vec![el(
        1,
        "backdrop",
        vec![el(2, "card", vec![txt("card")]), el(3, "list", rows)],
    )])
}

/// Drives `frames` frames of the effect-bearing app against a deterministic
/// clock, returning one [`SceneSnapshot`] per frame.
///
/// `TICK` seconds of motion are advanced per frame so the transition runs.
#[must_use]
pub fn drive_effect_frames(rt: &mut UiRuntime, frames: usize) -> Vec<SceneSnapshot> {
    let sheets = vec![compile(effects_css())];
    let v = effects_view();
    let mut out = Vec::with_capacity(frames);
    for _ in 0..frames {
        rt.request_frame(FrameReason::Animation);
        let _ = rt.run_due_frame(&v, &sheets);
        rt.advance_motion(TICK);
        out.push(rt.scene().snapshot());
    }
    out
}

/// One deterministic clock tick, in seconds.
pub const TICK: f32 = 1.0 / 60.0;

// ---------------------------------------------------------------------------
// The focused-text-field app, driven tick by tick
// ---------------------------------------------------------------------------

/// A runtime holding a focused, empty text field, plus the sheets and view it
/// was rendered with.
pub struct Field {
    pub runtime: UiRuntime,
    pub sheets: Vec<Stylesheet>,
    pub view: View,
    pub node: NodeId,
}

impl Field {
    /// Builds and focuses an empty single-line field.
    #[must_use]
    pub fn focused() -> Self {
        let mut runtime = runtime();
        let view = Input::with_value(CMD, "").view();
        let sheets = vec![compile(
            "input { width: 200px; height: 30px; font-size: 16px }",
        )];
        let _ = runtime.render_now(&view, &sheets);
        let node = child_at(&runtime, &[0]);
        let mut field = Self {
            runtime,
            sheets,
            view,
            node,
        };
        field.click_to_focus();
        field
    }

    fn click_to_focus(&mut self) {
        let target = child_at(&self.runtime, &[0]);
        self.node = target;
        let b = self
            .runtime
            .layout_boxes()
            .get(&target)
            .copied()
            .expect("the field is arranged");
        let centre = Point::new(
            b.origin().x + b.size().width / 2.0,
            b.origin().y + b.size().height / 2.0,
        );
        for kind in [PointerEventKind::Down, PointerEventKind::Up] {
            self.runtime.handle_pointer(PointerEvent::new(
                PointerId::MOUSE,
                PointerDevice::Mouse,
                kind,
                centre,
            ));
        }
    }

    /// Queues a press-and-release of `ch` for the next frame's dispatch phase.
    pub fn queue_char(&mut self, ch: char) {
        let logical = Some(LogicalKey::character(ch));
        self.runtime
            .queue_input(InputEvent::Key(KeyEvent::press(PhysicalKey::KeyA, logical)));
        self.runtime.queue_input(InputEvent::Key(KeyEvent::release(
            PhysicalKey::KeyA,
            logical,
        )));
    }

    /// Runs one clock tick: whatever frame is due runs, motion advances by one
    /// tick. Returns whether a frame was produced.
    pub fn tick(&mut self) -> bool {
        let produced = {
            let Self {
                runtime,
                sheets,
                view,
                ..
            } = self;
            runtime.run_due_frame(view, sheets).is_some()
        };
        self.runtime.advance_motion(TICK);
        produced
    }

    /// The field's current committed text.
    #[must_use]
    pub fn text(&self) -> String {
        self.runtime.editing_text().unwrap_or_default().to_owned()
    }
}

/// What one tick-by-tick run observed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeystrokeRun {
    /// The tick index at which the keystroke was delivered.
    pub delivered_at: usize,
    /// The tick index of the first *produced frame* whose text contains the
    /// typed character, if any.
    pub reflected_at: Option<usize>,
    /// The field's text at the end of the run.
    pub final_text: String,
}

impl KeystrokeRun {
    /// Ticks from event delivery to the frame that reflects it.
    #[must_use]
    pub fn ticks_to_reflect(&self) -> Option<usize> {
        self.reflected_at.map(|t| t - self.delivered_at)
    }
}

/// Drives a focused field for `total_ticks` ticks, queueing `ch` at tick
/// `deliver_at`, and records the first produced frame that reflects it.
#[must_use]
pub fn run_keystroke(ch: char, deliver_at: usize, total_ticks: usize) -> KeystrokeRun {
    let mut field = Field::focused();
    let mut reflected_at = None;
    for tick in 0..total_ticks {
        if tick == deliver_at {
            field.queue_char(ch);
        }
        let produced = field.tick();
        if produced && reflected_at.is_none() && field.text().contains(ch) {
            reflected_at = Some(tick);
        }
    }
    KeystrokeRun {
        delivered_at: deliver_at,
        reflected_at,
        final_text: field.text(),
    }
}

// ---------------------------------------------------------------------------
// Tree walking
// ---------------------------------------------------------------------------

/// Walks from the tree root down `path` (child indices) to a [`NodeId`].
#[must_use]
pub fn child_at(rt: &UiRuntime, path: &[usize]) -> NodeId {
    let mut node = rt.tree().root().expect("a root after a frame");
    for &i in path {
        let kids = rt
            .tree()
            .get(node)
            .expect("a live node")
            .children()
            .to_vec();
        assert!(i < kids.len(), "child index {i} out of range");
        node = kids[i];
    }
    node
}
