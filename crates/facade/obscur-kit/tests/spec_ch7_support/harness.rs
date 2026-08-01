#![doc = include_str!("harness.doc/_module.md")]
#![allow(dead_code, unreachable_pub)]

use std::cell::RefCell;
use std::rc::Rc;

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::diagnostics::{
    BudgetViolation, Diagnostic, DiagnosticSink, PerformanceCounter, PerformanceSpan, TelemetrySink,
};
use obscur_kit::geometry::{Point, Size};
use obscur_kit::identity::{ComponentId, NodeId, RuntimeId};
use obscur_kit::interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

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
    ViewNode::element(ComponentId::from_raw(id))
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
pub fn runtime(w: f32, h: f32) -> UiRuntime {
    UiRuntime::new(RuntimeId::from_raw(1), Size::new(w, h))
}

#[doc = include_str!("harness.doc/drive.md")]
#[must_use]
pub fn drive(w: f32, h: f32, css: &str, v: &View) -> UiRuntime {
    let sheet = compile(css);
    let mut rt = runtime(w, h);
    let _ = rt.render_now(v, std::slice::from_ref(&sheet));
    rt
}

#[doc = include_str!("harness.doc/at.md")]
#[must_use]
pub fn at(rt: &UiRuntime, path: &[usize]) -> NodeId {
    let mut node = rt.tree().root().expect("a root after a frame");
    for &i in path {
        let kids = rt
            .tree()
            .get(node)
            .expect("a live node")
            .child_nodes()
            .to_vec();
        assert!(
            i < kids.len(),
            "child index {i} out of range: node has {} children",
            kids.len()
        );
        node = kids[i];
    }
    node
}

#[doc = include_str!("harness.doc/geom.md")]
#[must_use]
pub fn geom(rt: &UiRuntime, node: NodeId) -> (f32, f32, f32, f32) {
    let b = rt
        .layout_boxes()
        .get(&node)
        .copied()
        .expect("the node was arranged");
    (b.origin().x, b.origin().y, b.size().width, b.size().height)
}

#[doc = include_str!("harness.doc/all_geom.md")]
#[must_use]
pub fn all_geom(rt: &UiRuntime) -> Vec<(f32, f32, f32, f32)> {
    let mut out = Vec::new();
    let mut stack = Vec::new();
    if let Some(root) = rt.tree().root() {
        stack.push(root);
    }
    while let Some(node) = stack.pop() {
        if let Some(b) = rt.layout_boxes().get(&node).copied() {
            out.push((b.origin().x, b.origin().y, b.size().width, b.size().height));
        }
        if let Some(n) = rt.tree().get(node) {
            for &child in n.child_nodes().iter().rev() {
                stack.push(child);
            }
        }
    }
    out
}

#[doc = include_str!("harness.doc/all_nodes.md")]
#[must_use]
pub fn all_nodes(rt: &UiRuntime) -> Vec<NodeId> {
    let mut out = Vec::new();
    let mut stack = Vec::new();
    if let Some(root) = rt.tree().root() {
        stack.push(root);
    }
    while let Some(node) = stack.pop() {
        out.push(node);
        if let Some(n) = rt.tree().get(node) {
            for &child in n.child_nodes().iter().rev() {
                stack.push(child);
            }
        }
    }
    out
}

#[doc = include_str!("harness.doc/approx.md")]
pub fn approx(actual: f32, expected: f32, what: &str) {
    assert!(
        (actual - expected).abs() < 1e-2,
        "{what}: expected {expected}, got {actual}"
    );
}

#[doc = include_str!("harness.doc/mouse.md")]
#[must_use]
pub fn mouse(kind: PointerEventKind, x: f32, y: f32) -> PointerEvent {
    PointerEvent::new(
        PointerId::MOUSE,
        PointerDevice::Mouse,
        kind,
        Point::new(x, y),
    )
}

#[doc = include_str!("harness.doc/hover.md")]
pub fn hover(rt: &mut UiRuntime, x: f32, y: f32) {
    rt.handle_pointer(mouse(PointerEventKind::Move, x, y));
}

#[doc = include_str!("harness.doc/click.md")]
pub fn click(rt: &mut UiRuntime, x: f32, y: f32) {
    rt.handle_pointer(mouse(PointerEventKind::Move, x, y));
    rt.handle_pointer(mouse(PointerEventKind::Down, x, y));
    rt.handle_pointer(mouse(PointerEventKind::Up, x, y));
}

#[doc = include_str!("harness.doc/SharedDiagnostics.md")]
#[derive(Debug, Default, Clone)]
pub struct SharedDiagnostics(Rc<RefCell<Vec<Diagnostic>>>);

impl SharedDiagnostics {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn taken(&self) -> Vec<Diagnostic> {
        self.0.borrow().clone()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.borrow().len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.borrow().is_empty()
    }
}

impl DiagnosticSink for SharedDiagnostics {
    fn emit(&self, diagnostic: &Diagnostic) {
        self.0.borrow_mut().push(diagnostic.clone());
    }
}

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

    #[must_use]
    pub fn span_names(&self) -> Vec<&'static str> {
        self.0.borrow().spans.iter().map(|s| s.name).collect()
    }

    #[must_use]
    pub fn violations(&self) -> Vec<BudgetViolation> {
        self.0.borrow().violations.clone()
    }

    #[must_use]
    pub fn counter_count(&self) -> usize {
        self.0.borrow().counters.len()
    }

    #[must_use]
    pub fn span_count(&self) -> usize {
        self.0.borrow().spans.len()
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
