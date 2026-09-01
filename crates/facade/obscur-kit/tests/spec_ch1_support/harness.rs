#![doc = include_str!("harness.doc/_module.md")]
#![allow(dead_code, unreachable_pub)]

use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::diagnostics::{Diagnostic, DiagnosticSink, DiagnosticSubsystem, RecoveryAction};
use obscur_kit::geometry::{Point, Size};
use obscur_kit::identity::{ComponentId, InternedSymbol, NodeId, RuntimeId};
use obscur_kit::interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};
use obscur_kit::resources::ResourceKind;
use obscur_kit::resources::port::{DecodedResource, LoadedBytes, ResourceDecoder, ResourceLoader};
use obscur_kit::resources::source::ResourceSource;
use obscur_kit::resources::state::ResourceError;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::task::BlockingExecutor;
use obscur_kit::view::boundary::ErrorBoundaryPolicy;
use obscur_kit::view::component::{
    Component, ComponentContext, ComponentContract, ComponentFactory, ComponentInstance,
    ComponentRegistration, ComponentVersion, ErasedComponent, FromProps, InstanceId, PropsError,
};
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

// ---------------------------------------------------------------------------
// CSS / view / runtime plumbing
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

#[doc = include_str!("harness.doc/runtime_with_id.md")]
#[must_use]
pub fn runtime_with_id(id: u64, w: f32, h: f32) -> UiRuntime {
    UiRuntime::new(RuntimeId::from_raw(id), Size::new(w, h))
}

#[doc = include_str!("harness.doc/runtime.md")]
#[must_use]
pub fn runtime(w: f32, h: f32) -> UiRuntime {
    runtime_with_id(1, w, h)
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

#[doc = include_str!("harness.doc/try_geom.md")]
#[must_use]
pub fn try_geom(rt: &UiRuntime, node: NodeId) -> Option<(f32, f32, f32, f32)> {
    let b = rt.layout_boxes().get(&node).copied()?;
    Some((b.origin().x, b.origin().y, b.size().width, b.size().height))
}

#[doc = include_str!("harness.doc/geom.md")]
#[must_use]
pub fn geom(rt: &UiRuntime, node: NodeId) -> (f32, f32, f32, f32) {
    try_geom(rt, node).expect("the node was arranged")
}

#[doc = include_str!("harness.doc/approx.md")]
pub fn approx(actual: f32, expected: f32, what: &str) {
    assert!(
        (actual - expected).abs() < 1e-2,
        "{what}: expected {expected}, got {actual}"
    );
}

#[doc = include_str!("harness.doc/all_text.md")]
#[must_use]
pub fn all_text(rt: &UiRuntime) -> Vec<String> {
    rt.tree()
        .iter()
        .filter_map(|(_, node)| node.kind().text().map(str::to_owned))
        .collect()
}

#[doc = include_str!("harness.doc/child_text.md")]
#[must_use]
pub fn child_text(rt: &UiRuntime, node: NodeId) -> Option<String> {
    let child = *rt.tree().get(node)?.child_nodes().first()?;
    rt.tree().get(child)?.kind().text().map(str::to_owned)
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

// ---------------------------------------------------------------------------
// Diagnostics capture
// ---------------------------------------------------------------------------

#[doc = include_str!("harness.doc/Collected.md")]
#[derive(Debug, Default, Clone)]
pub struct Collected(pub Rc<RefCell<Vec<Diagnostic>>>);

impl DiagnosticSink for Collected {
    fn emit(&self, diagnostic: &Diagnostic) {
        self.0.borrow_mut().push(diagnostic.clone());
    }
}

impl Collected {
    #[doc = include_str!("harness.doc/install.md")]
    #[must_use]
    pub fn install(rt: &mut UiRuntime) -> Self {
        let collected = Self::default();
        rt.set_diagnostic_sink(Box::new(collected.clone()));
        collected
    }

    #[doc = include_str!("harness.doc/all.md")]
    #[must_use]
    pub fn all(&self) -> Vec<Diagnostic> {
        self.0.borrow().clone()
    }

    #[doc = include_str!("harness.doc/from.md")]
    #[must_use]
    pub fn from(&self, subsystem: DiagnosticSubsystem) -> Vec<Diagnostic> {
        self.0
            .borrow()
            .iter()
            .filter(|d| d.subsystem == subsystem)
            .cloned()
            .collect()
    }

    #[doc = include_str!("harness.doc/codes.md")]
    #[must_use]
    pub fn codes(&self) -> Vec<&'static str> {
        self.0.borrow().iter().map(|d| d.code.as_str()).collect()
    }

    #[doc = include_str!("harness.doc/boundary_activations.md")]
    #[must_use]
    pub fn boundary_activations(&self) -> Vec<Diagnostic> {
        self.0
            .borrow()
            .iter()
            .filter(|d| {
                matches!(
                    d.recovery,
                    RecoveryAction::Retried | RecoveryAction::BoundaryReplaced
                )
            })
            .cloned()
            .collect()
    }

    #[doc = include_str!("harness.doc/clear.md")]
    pub fn clear(&self) {
        self.0.borrow_mut().clear();
    }
}

// ---------------------------------------------------------------------------
// A scripted component: fails on demand, and reports whether it was remounted
// ---------------------------------------------------------------------------

#[doc = include_str!("harness.doc/P_LABEL.md")]
pub const P_LABEL: InternedSymbol = InternedSymbol::from_raw(0x0C01_0001);
#[doc = include_str!("harness.doc/P_FAIL.md")]
pub const P_FAIL: InternedSymbol = InternedSymbol::from_raw(0x0C01_0002);
#[doc = include_str!("harness.doc/P_CHILD.md")]
pub const P_CHILD: InternedSymbol = InternedSymbol::from_raw(0x0C01_0003);
#[doc = include_str!("harness.doc/P_CHILD_FAIL.md")]
pub const P_CHILD_FAIL: InternedSymbol = InternedSymbol::from_raw(0x0C01_0004);
#[doc = include_str!("harness.doc/P_IMG.md")]
pub const P_IMG: InternedSymbol = InternedSymbol::from_raw(0x0C01_0005);

#[doc = include_str!("harness.doc/ScriptProps.md")]
pub struct ScriptProps {
    #[doc = include_str!("harness.doc/label.md")]
    pub label: String,
    #[doc = include_str!("harness.doc/child.md")]
    pub child: Option<u64>,
    #[doc = include_str!("harness.doc/child_fail.md")]
    pub child_fail: bool,
    #[doc = include_str!("harness.doc/image.md")]
    pub image: Option<String>,
}

impl FromProps for ScriptProps {
    fn from_props(props: &[ViewProperty]) -> Result<Self, PropsError> {
        for property in props {
            if property.name() == P_FAIL
                && matches!(property.value(), ViewValue::Text(t) if &**t == "yes")
            {
                return Err(PropsError::wrong_type(
                    P_FAIL,
                    "fail",
                    "a component that can render",
                ));
            }
        }
        let text_of = |name: InternedSymbol| {
            props
                .iter()
                .find(|p| p.name() == name)
                .and_then(|p| match p.value() {
                    ViewValue::Text(t) => Some(t.to_string()),
                    _ => None,
                })
        };
        let child = props
            .iter()
            .find(|p| p.name() == P_CHILD)
            .and_then(|p| match p.value() {
                ViewValue::Int(i) => u64::try_from(*i).ok(),
                _ => None,
            });
        Ok(Self {
            label: text_of(P_LABEL).unwrap_or_default(),
            child,
            child_fail: text_of(P_CHILD_FAIL).as_deref() == Some("yes"),
            image: text_of(P_IMG),
        })
    }
}

#[doc = include_str!("harness.doc/Scripted.md")]
pub struct Scripted {
    seed: u32,
    policy: ErrorBoundaryPolicy,
    fallback: Option<View>,
}

impl Component for Scripted {
    type Props = ScriptProps;

    fn render(&mut self, _ctx: &mut ComponentContext<'_>, props: &Self::Props) -> View {
        let mut nodes = vec![txt(&format!("{}:{}", props.label, self.seed))];
        if let Some(src) = props.image.as_deref() {
            nodes.push(
                obscur_kit::primitives::Image::new(src)
                    .into_view_node()
                    .with_prop(ViewProperty::new(CLASS, ViewValue::Text("cell".into()))),
            );
        }
        if let Some(child) = props.child {
            nodes.push(component_node(
                ComponentId::from_raw(child),
                "cell",
                "child",
                props.child_fail,
            ));
        }
        view(nodes)
    }

    fn error_boundary(&self) -> ErrorBoundaryPolicy {
        self.policy
    }

    fn fallback(&self) -> Option<View> {
        self.fallback.clone()
    }
}

#[doc = include_str!("harness.doc/ScriptedFactory.md")]
#[derive(Debug)]
pub struct ScriptedFactory {
    id: ComponentId,
    policy: ErrorBoundaryPolicy,
    fallback: Option<View>,
    created: Rc<Cell<u32>>,
}

impl ScriptedFactory {
    #[doc = include_str!("harness.doc/new.md")]
    #[must_use]
    pub fn new(id: ComponentId) -> Self {
        Self {
            id,
            policy: ErrorBoundaryPolicy::default(),
            fallback: None,
            created: Rc::new(Cell::new(0)),
        }
    }

    #[doc = include_str!("harness.doc/with_policy.md")]
    #[must_use]
    pub fn with_policy(mut self, policy: ErrorBoundaryPolicy) -> Self {
        self.policy = policy;
        self
    }

    #[doc = include_str!("harness.doc/with_fallback.md")]
    #[must_use]
    pub fn with_fallback(mut self, fallback: View) -> Self {
        self.fallback = Some(fallback);
        self
    }

    #[doc = include_str!("harness.doc/created_counter.md")]
    #[must_use]
    pub fn created_counter(&self) -> Rc<Cell<u32>> {
        Rc::clone(&self.created)
    }
}

impl ComponentFactory for ScriptedFactory {
    fn component_id(&self) -> ComponentId {
        self.id
    }

    fn contract(&self) -> ComponentContract {
        ComponentContract::new(self.id)
    }

    fn create_instance(&self, instance: InstanceId) -> ComponentInstance {
        ComponentInstance::new(instance, self.id)
    }

    fn create_component(&self) -> Box<dyn ErasedComponent> {
        self.created.set(self.created.get() + 1);
        Box::new(Scripted {
            seed: self.created.get(),
            policy: self.policy,
            fallback: self.fallback.clone(),
        })
    }
}

#[doc = include_str!("harness.doc/register.md")]
pub fn register(rt: &mut UiRuntime, factory: ScriptedFactory) -> Rc<Cell<u32>> {
    let counter = factory.created_counter();
    rt.register_component(ComponentRegistration::new(
        Box::new(factory),
        ComponentVersion::INITIAL,
    ))
    .expect("first registration of this component id");
    counter
}

#[doc = include_str!("harness.doc/component_node.md")]
#[must_use]
pub fn component_node(id: ComponentId, class: &str, label: &str, fail: bool) -> ViewNode {
    let mut node = ViewNode::element(id)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
        .with_prop(ViewProperty::new(P_LABEL, ViewValue::Text(label.into())));
    if fail {
        node = node.with_prop(ViewProperty::new(P_FAIL, ViewValue::Text("yes".into())));
    }
    node
}

#[doc = include_str!("harness.doc/nesting_node.md")]
#[must_use]
pub fn nesting_node(
    id: ComponentId,
    class: &str,
    label: &str,
    child: u64,
    child_fail: bool,
) -> ViewNode {
    let mut node = component_node(id, class, label, false).with_prop(ViewProperty::new(
        P_CHILD,
        ViewValue::Int(i64::try_from(child).expect("a component id that fits an i64")),
    ));
    if child_fail {
        node = node.with_prop(ViewProperty::new(
            P_CHILD_FAIL,
            ViewValue::Text("yes".into()),
        ));
    }
    node
}

#[doc = include_str!("harness.doc/imaging_node.md")]
#[must_use]
pub fn imaging_node(id: ComponentId, class: &str, label: &str, src: &str) -> ViewNode {
    component_node(id, class, label, false)
        .with_prop(ViewProperty::new(P_IMG, ViewValue::Text(src.into())))
}

// ---------------------------------------------------------------------------
// Resource ports: a loader that never finds anything, and one that always does
// ---------------------------------------------------------------------------

#[doc = include_str!("harness.doc/MissingLoader.md")]
#[derive(Debug, Default)]
pub struct MissingLoader;

impl ResourceLoader for MissingLoader {
    fn can_load(&self, _source: &ResourceSource) -> bool {
        true
    }

    fn load(&self, _source: &ResourceSource) -> Result<LoadedBytes, ResourceError> {
        Err(ResourceError::NotFound)
    }
}

#[doc = include_str!("harness.doc/NoDecoder.md")]
#[derive(Debug, Default)]
pub struct NoDecoder;

impl ResourceDecoder for NoDecoder {
    fn supports(&self, _kind: ResourceKind) -> bool {
        false
    }

    fn decode(&self, _kind: ResourceKind, _bytes: &[u8]) -> Result<DecodedResource, ResourceError> {
        Err(ResourceError::Unsupported("test decoder".into()))
    }
}

#[doc = include_str!("harness.doc/install_missing_resources.md")]
pub fn install_missing_resources(rt: &mut UiRuntime) {
    rt.set_resource_providers(Arc::new(MissingLoader), Arc::new(NoDecoder));
}

// ---------------------------------------------------------------------------
// Blocking executors: the two §1.5 adapter answers
// ---------------------------------------------------------------------------

#[doc = include_str!("harness.doc/InlineExecutor.md")]
#[derive(Debug, Default)]
pub struct InlineExecutor;

impl BlockingExecutor for InlineExecutor {
    fn run_blocking(&self, job: Box<dyn FnOnce() + Send + 'static>) {
        job();
    }
}

#[doc = include_str!("harness.doc/ThreadedExecutor.md")]
#[derive(Debug, Default)]
pub struct ThreadedExecutor;

impl BlockingExecutor for ThreadedExecutor {
    fn run_blocking(&self, job: Box<dyn FnOnce() + Send + 'static>) {
        std::thread::spawn(job)
            .join()
            .expect("the blocking job did not panic");
    }
}

#[doc = include_str!("harness.doc/QueuedExecutor.md")]
#[derive(Default)]
pub struct QueuedExecutor {
    jobs: std::sync::Mutex<Vec<Box<dyn FnOnce() + Send + 'static>>>,
}

impl std::fmt::Debug for QueuedExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueuedExecutor").finish_non_exhaustive()
    }
}

impl QueuedExecutor {
    #[doc = include_str!("harness.doc/pending.md")]
    #[must_use]
    pub fn pending(&self) -> usize {
        self.jobs.lock().expect("queue lock").len()
    }

    #[doc = include_str!("harness.doc/run_at.md")]
    pub fn run_at(&self, index: usize) {
        let job = {
            let mut jobs = self.jobs.lock().expect("queue lock");
            assert!(index < jobs.len(), "no queued job at index {index}");
            jobs.remove(index)
        };
        job();
    }

    #[doc = include_str!("harness.doc/run_all.md")]
    pub fn run_all(&self) {
        loop {
            let job = {
                let mut jobs = self.jobs.lock().expect("queue lock");
                if jobs.is_empty() {
                    return;
                }
                jobs.remove(0)
            };
            job();
        }
    }
}

impl BlockingExecutor for QueuedExecutor {
    fn run_blocking(&self, job: Box<dyn FnOnce() + Send + 'static>) {
        self.jobs.lock().expect("queue lock").push(job);
    }
}

// ---------------------------------------------------------------------------
// Repository layout, for the §1.1–§1.3 build/API-surface cases
// ---------------------------------------------------------------------------

#[doc = include_str!("harness.doc/workspace_root.md")]
#[must_use]
pub fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
}

#[doc = include_str!("harness.doc/rust_files.md")]
#[must_use]
pub fn rust_files(dir: &std::path::Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(rust_files(&path));
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
    out.sort();
    out
}

#[doc = include_str!("harness.doc/without_test_modules.md")]
#[must_use]
pub fn without_test_modules(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut rest = source;
    while let Some(pos) = rest.find("#[cfg(test)]") {
        out.push_str(&rest[..pos]);
        let after = &rest[pos..];
        let Some(open) = after.find('{') else {
            break;
        };
        let mut depth = 0i32;
        let mut end = None;
        for (i, ch) in after[open..].char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(open + i + 1);
                        break;
                    }
                }
                _ => {}
            }
        }
        match end {
            Some(e) => rest = &after[e..],
            None => return out,
        }
    }
    out.push_str(rest);
    out
}
