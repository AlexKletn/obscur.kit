//! Shared harness for the `spec_ch1_*` **specification-derived** test files.
//!
//! These tests are NOT implementation-derived. Every expected value in the
//! `spec_ch1_*.rs` files was written by an agent reading obscur.kit §1
//! (Architecture & boundaries) **without opening any source in this
//! repository**. This module exists only to give those cases a compiling home;
//! it contains no expectations of its own.
//!
//! Nothing here may be "fixed" to make a case pass. A failing `spec_ch1_*` test
//! is a claim about the implementation, not about the test.
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

/// Compiles authored CSS text into a matchable stylesheet through the real CSS
/// frontend (`source -> parse -> compile`).
#[must_use]
pub fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

/// An element node carrying `class`, with `kids` as its children.
#[must_use]
pub fn el(id: u64, class: &str, kids: Vec<ViewNode>) -> ViewNode {
    ViewNode::element(ComponentId::from_raw(id))
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

/// A runtime with a `w x h` viewport and the given stable id.
#[must_use]
pub fn runtime_with_id(id: u64, w: f32, h: f32) -> UiRuntime {
    UiRuntime::new(RuntimeId::from_raw(id), Size::new(w, h))
}

/// A runtime with a `w x h` viewport.
#[must_use]
pub fn runtime(w: f32, h: f32) -> UiRuntime {
    runtime_with_id(1, w, h)
}

/// Walks from the tree root down `path` (child indices) to a [`NodeId`].
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

/// The absolute border-box `(x, y, w, h)` of a node, if it was arranged.
#[must_use]
pub fn try_geom(rt: &UiRuntime, node: NodeId) -> Option<(f32, f32, f32, f32)> {
    let b = rt.layout_boxes().get(&node).copied()?;
    Some((b.origin().x, b.origin().y, b.size().width, b.size().height))
}

/// The absolute border-box `(x, y, w, h)` of a node.
#[must_use]
pub fn geom(rt: &UiRuntime, node: NodeId) -> (f32, f32, f32, f32) {
    try_geom(rt, node).expect("the node was arranged")
}

/// Asserts `actual ≈ expected` within a sub-hundredth-pixel tolerance.
pub fn approx(actual: f32, expected: f32, what: &str) {
    assert!(
        (actual - expected).abs() < 1e-2,
        "{what}: expected {expected}, got {actual}"
    );
}

/// Every text-node content in the retained tree, in document order.
#[must_use]
pub fn all_text(rt: &UiRuntime) -> Vec<String> {
    rt.tree()
        .iter()
        .filter_map(|(_, node)| node.kind().text().map(str::to_owned))
        .collect()
}

/// The text content of `node`'s single text child, if it has one.
#[must_use]
pub fn child_text(rt: &UiRuntime, node: NodeId) -> Option<String> {
    let child = *rt.tree().get(node)?.child_nodes().first()?;
    rt.tree().get(child)?.kind().text().map(str::to_owned)
}

/// A left-button pointer event of `kind` at `(x, y)`.
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

/// A [`DiagnosticSink`] that collects into a shared buffer so a test can read
/// back what the runtime reported (the runtime takes its sink by value).
#[derive(Debug, Default, Clone)]
pub struct Collected(pub Rc<RefCell<Vec<Diagnostic>>>);

impl DiagnosticSink for Collected {
    fn emit(&self, diagnostic: &Diagnostic) {
        self.0.borrow_mut().push(diagnostic.clone());
    }
}

impl Collected {
    /// Installs a fresh collector on `rt` and returns the handle.
    #[must_use]
    pub fn install(rt: &mut UiRuntime) -> Self {
        let collected = Self::default();
        rt.set_diagnostic_sink(Box::new(collected.clone()));
        collected
    }

    /// Every diagnostic collected so far.
    #[must_use]
    pub fn all(&self) -> Vec<Diagnostic> {
        self.0.borrow().clone()
    }

    /// Diagnostics from `subsystem`.
    #[must_use]
    pub fn from(&self, subsystem: DiagnosticSubsystem) -> Vec<Diagnostic> {
        self.0
            .borrow()
            .iter()
            .filter(|d| d.subsystem == subsystem)
            .cloned()
            .collect()
    }

    /// The stable codes of every collected diagnostic, in emission order.
    #[must_use]
    pub fn codes(&self) -> Vec<&'static str> {
        self.0.borrow().iter().map(|d| d.code.as_str()).collect()
    }

    /// How many diagnostics record a §1.7 boundary decision — the observable
    /// signal that a component error boundary *activated*.
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

    /// Clears the buffer.
    pub fn clear(&self) {
        self.0.borrow_mut().clear();
    }
}

// ---------------------------------------------------------------------------
// A scripted component: fails on demand, and reports whether it was remounted
// ---------------------------------------------------------------------------

/// Property naming the component's label (rendered as its text child).
pub const P_LABEL: InternedSymbol = InternedSymbol::from_raw(0x0C01_0001);
/// Property that, set to the text `"yes"`, makes the component's props fail to
/// decode — the one component-failure origin the public API exposes.
pub const P_FAIL: InternedSymbol = InternedSymbol::from_raw(0x0C01_0002);
/// Property naming a child component kind to render, as a raw [`ComponentId`].
pub const P_CHILD: InternedSymbol = InternedSymbol::from_raw(0x0C01_0003);
/// Property that makes the rendered child's props fail to decode.
pub const P_CHILD_FAIL: InternedSymbol = InternedSymbol::from_raw(0x0C01_0004);
/// Property naming an image source the component renders below its label.
pub const P_IMG: InternedSymbol = InternedSymbol::from_raw(0x0C01_0005);

/// The typed props of [`Scripted`]. Decoding fails when `fail="yes"`.
pub struct ScriptProps {
    /// The label the component renders.
    pub label: String,
    /// A child component kind to render, if any.
    pub child: Option<u64>,
    /// Whether that child's own props should fail to decode.
    pub child_fail: bool,
    /// An image source to render below the label, if any.
    pub image: Option<String>,
}

impl FromProps for ScriptProps {
    fn from_props(props: &[ViewProperty]) -> Result<Self, PropsError> {
        for property in props {
            if property.name() == P_FAIL
                && matches!(property.value(), ViewValue::Text(t) if &**t == "yes")
            {
                return Err(PropsError::Type {
                    name: P_FAIL,
                    expected: "a component that can render",
                });
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

/// A component that renders `"<label>:<seed>"`, where `seed` is unique to this
/// *instance*. A remount produces a new seed, so the rendered text is a direct
/// probe of "was this component's local state preserved?".
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

/// Factory for [`Scripted`], counting how many instances it has created — the
/// mount count a §1.7 case asserts on.
#[derive(Debug)]
pub struct ScriptedFactory {
    id: ComponentId,
    policy: ErrorBoundaryPolicy,
    fallback: Option<View>,
    created: Rc<Cell<u32>>,
}

impl ScriptedFactory {
    /// A factory for component kind `id` with the default boundary policy.
    #[must_use]
    pub fn new(id: ComponentId) -> Self {
        Self {
            id,
            policy: ErrorBoundaryPolicy::default(),
            fallback: None,
            created: Rc::new(Cell::new(0)),
        }
    }

    /// Sets the boundary policy this component kind reports.
    #[must_use]
    pub fn with_policy(mut self, policy: ErrorBoundaryPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Sets the fallback view this component kind's boundary shows.
    #[must_use]
    pub fn with_fallback(mut self, fallback: View) -> Self {
        self.fallback = Some(fallback);
        self
    }

    /// A handle to this factory's instance-creation counter, readable after the
    /// factory has been handed to the runtime.
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

/// Registers `factory` on `rt`, returning its instance-creation counter.
pub fn register(rt: &mut UiRuntime, factory: ScriptedFactory) -> Rc<Cell<u32>> {
    let counter = factory.created_counter();
    rt.register_component(ComponentRegistration::new(
        Box::new(factory),
        ComponentVersion::INITIAL,
    ))
    .expect("first registration of this component id");
    counter
}

/// A `<component id=... label=... [fail]>` element node, classed `class`.
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

/// A component node that renders a nested component of kind `child`, whose own
/// props fail to decode when `child_fail`.
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

/// A component node that renders an image sourced at `src` below its label.
#[must_use]
pub fn imaging_node(id: ComponentId, class: &str, label: &str, src: &str) -> ViewNode {
    component_node(id, class, label, false)
        .with_prop(ViewProperty::new(P_IMG, ViewValue::Text(src.into())))
}

// ---------------------------------------------------------------------------
// Resource ports: a loader that never finds anything, and one that always does
// ---------------------------------------------------------------------------

/// A [`ResourceLoader`] that reports every source as missing — the §1.6
/// "invalid content or resource" path, driven deterministically.
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

/// A [`ResourceDecoder`] that decodes nothing, honestly.
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

/// Installs the always-missing resource ports on `rt`.
pub fn install_missing_resources(rt: &mut UiRuntime) {
    rt.set_resource_providers(Arc::new(MissingLoader), Arc::new(NoDecoder));
}

// ---------------------------------------------------------------------------
// Blocking executors: the two §1.5 adapter answers
// ---------------------------------------------------------------------------

/// A [`BlockingExecutor`] that runs every job inline on the caller — the
/// "no parallelism available" adapter answer (a WASM build without workers).
#[derive(Debug, Default)]
pub struct InlineExecutor;

impl BlockingExecutor for InlineExecutor {
    fn run_blocking(&self, job: Box<dyn FnOnce() + Send + 'static>) {
        job();
    }
}

/// A [`BlockingExecutor`] that runs every job on a freshly spawned thread and
/// joins it — the "parallelism available" adapter answer, with the completion
/// order still under the caller's control.
#[derive(Debug, Default)]
pub struct ThreadedExecutor;

impl BlockingExecutor for ThreadedExecutor {
    fn run_blocking(&self, job: Box<dyn FnOnce() + Send + 'static>) {
        std::thread::spawn(job)
            .join()
            .expect("the blocking job did not panic");
    }
}

/// A [`BlockingExecutor`] that queues jobs instead of running them, so a test
/// controls exactly when — and in what order — each completes.
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
    /// How many jobs are queued.
    #[must_use]
    pub fn pending(&self) -> usize {
        self.jobs.lock().expect("queue lock").len()
    }

    /// Runs the queued job at `index`, removing it.
    pub fn run_at(&self, index: usize) {
        let job = {
            let mut jobs = self.jobs.lock().expect("queue lock");
            assert!(index < jobs.len(), "no queued job at index {index}");
            jobs.remove(index)
        };
        job();
    }

    /// Runs every queued job, in queue order.
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

/// The workspace root, derived from this crate's manifest directory.
#[must_use]
pub fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
}

/// Every `.rs` file under `dir`, recursively.
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

/// Strips every `#[cfg(test)] mod tests { ... }` block from `source` by brace
/// counting, so a scan of public API surface never reads unit-test code.
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
