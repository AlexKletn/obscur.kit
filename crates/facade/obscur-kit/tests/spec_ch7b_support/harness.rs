#![doc = include_str!("harness.doc/_module.md")]
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

#[doc = include_str!("harness.doc/VW.md")]
pub const VW: f32 = 400.0;

#[doc = include_str!("harness.doc/CMD.md")]
pub const CMD: InternedSymbol = InternedSymbol::from_raw(0x0B11);

// ---------------------------------------------------------------------------
// Authoring
// ---------------------------------------------------------------------------

#[doc = include_str!("harness.doc/compile.md")]
#[must_use]
pub fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("harness.doc/el.md")]
#[must_use]
pub fn el(id: u64, class: &str, kids: Vec<ViewNode>) -> ViewNode {
    ViewNode::element(obscur_kit::identity::ComponentId::from_raw(id))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
        .with_children(ViewFragment::from_nodes(kids))
}

#[doc = include_str!("harness.doc/txt.md")]
#[must_use]
pub fn txt(content: &str) -> ViewNode {
    ViewNode::text(content)
}

#[doc = include_str!("harness.doc/view.md")]
#[must_use]
pub fn view(nodes: Vec<ViewNode>) -> View {
    View::from_fragment(ViewFragment::from_nodes(nodes))
}

#[doc = include_str!("harness.doc/runtime.md")]
#[must_use]
pub fn runtime() -> UiRuntime {
    UiRuntime::new(RuntimeId::from_raw(1), Size::new(VW, VW))
}

// ---------------------------------------------------------------------------
// Power vocabulary, as an author would spell it
// ---------------------------------------------------------------------------

#[doc = include_str!("harness.doc/full_performance_profile.md")]
#[must_use]
pub fn full_performance_profile() -> PowerProfile {
    PowerProfile::new()
        .on_source(PowerSource::Ac)
        .with_battery_level(1.0)
        .low_power_mode(false)
        .with_thermal(ThermalState::Nominal)
}

#[doc = include_str!("harness.doc/maximum_saving_profile.md")]
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

#[doc = include_str!("harness.doc/TelemetryLog.md")]
#[derive(Debug, Default)]
pub struct TelemetryLog {
    pub spans: Vec<PerformanceSpan>,
    pub counters: Vec<PerformanceCounter>,
    pub violations: Vec<BudgetViolation>,
}

#[doc = include_str!("harness.doc/SharedTelemetry.md")]
#[derive(Debug, Default, Clone)]
pub struct SharedTelemetry(Rc<RefCell<TelemetryLog>>);

impl SharedTelemetry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("harness.doc/counters.md")]
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

    #[doc = include_str!("harness.doc/span_names.md")]
    #[must_use]
    pub fn span_names(&self) -> Vec<&'static str> {
        self.0.borrow().spans.iter().map(|s| s.name).collect()
    }

    #[doc = include_str!("harness.doc/violations.md")]
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

#[doc = include_str!("harness.doc/effects_css.md")]
#[must_use]
pub fn effects_css() -> &'static str {
    ".backdrop { width: 400px; height: 400px; backdrop-filter: blur(8px) } \
     .card { width: 200px; height: 60px; box-shadow: 0 2px 8px rgba(0, 0, 0, 0.4); \
             opacity: 0.5; transition: opacity 200ms linear } \
     .list { width: 200px; height: 120px; overflow: scroll } \
     .row { width: 200px; height: 20px }"
}

#[doc = include_str!("harness.doc/effects_view.md")]
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

#[doc = include_str!("harness.doc/drive_effect_frames.md")]
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

#[doc = include_str!("harness.doc/TICK.md")]
pub const TICK: f32 = 1.0 / 60.0;

// ---------------------------------------------------------------------------
// The focused-text-field app, driven tick by tick
// ---------------------------------------------------------------------------

#[doc = include_str!("harness.doc/Field.md")]
pub struct Field {
    pub runtime: UiRuntime,
    pub sheets: Vec<Stylesheet>,
    pub view: View,
    pub node: NodeId,
}

impl Field {
    #[doc = include_str!("harness.doc/focused.md")]
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

    #[doc = include_str!("harness.doc/queue_char.md")]
    pub fn queue_char(&mut self, ch: char) {
        let logical = Some(LogicalKey::character(ch));
        self.runtime
            .queue_input(InputEvent::Key(KeyEvent::press(PhysicalKey::KeyA, logical)));
        self.runtime.queue_input(InputEvent::Key(KeyEvent::release(
            PhysicalKey::KeyA,
            logical,
        )));
    }

    #[doc = include_str!("harness.doc/tick.md")]
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

    #[doc = include_str!("harness.doc/text.md")]
    #[must_use]
    pub fn text(&self) -> String {
        self.runtime.editing_text().unwrap_or_default().to_owned()
    }
}

#[doc = include_str!("harness.doc/KeystrokeRun.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeystrokeRun {
    #[doc = include_str!("harness.doc/delivered_at.md")]
    pub delivered_at: usize,
    #[doc = include_str!("harness.doc/reflected_at.md")]
    pub reflected_at: Option<usize>,
    #[doc = include_str!("harness.doc/final_text.md")]
    pub final_text: String,
}

impl KeystrokeRun {
    #[doc = include_str!("harness.doc/ticks_to_reflect.md")]
    #[must_use]
    pub fn ticks_to_reflect(&self) -> Option<usize> {
        self.reflected_at.map(|t| t - self.delivered_at)
    }
}

#[doc = include_str!("harness.doc/run_keystroke.md")]
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

#[doc = include_str!("harness.doc/child_at.md")]
#[must_use]
pub fn child_at(rt: &UiRuntime, path: &[usize]) -> NodeId {
    let mut node = rt.tree().root().expect("a root after a frame");
    for &i in path {
        let kids = rt
            .tree()
            .get(node)
            .expect("a live node")
            .child_nodes()
            .to_vec();
        assert!(i < kids.len(), "child index {i} out of range");
        node = kids[i];
    }
    node
}
