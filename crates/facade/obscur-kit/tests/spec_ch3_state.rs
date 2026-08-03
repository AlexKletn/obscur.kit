#![doc = include_str!("spec_ch3_state.doc/_module.md")]
#![forbid(unsafe_code)]

#[path = "spec_ch3_support/mod.rs"]
mod support;

use std::sync::atomic::{AtomicUsize, Ordering};

use obscur_kit::geometry::Size;
use obscur_kit::identity::{RuntimeId, SignalId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::ui_macros::Contract;
use obscur_kit::view::component::{
    Component, ComponentContext, ComponentContract, ComponentFactory, ComponentInstance,
    ComponentRegistration, ComponentVersion, ErasedComponent, InstanceId,
};
use obscur_kit::view::view::{View, ViewFragment, ViewNode, ViewProperty, ViewValue};
use support::{drain_frames, view};

const SIGNAL_A: SignalId = SignalId::from_raw(0xA);
const SIGNAL_B: SignalId = SignalId::from_raw(0xB);

static X_RENDERS: AtomicUsize = AtomicUsize::new(0);
static Y_RENDERS: AtomicUsize = AtomicUsize::new(0);
static CHILD_LOCAL: AtomicUsize = AtomicUsize::new(0);

#[doc = include_str!("spec_ch3_state.doc/COUNTERS.md")]
static COUNTERS: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[doc = include_str!("spec_ch3_state.doc/XProps.md")]
#[derive(Contract)]
#[contract(name = "SpecX")]
struct XProps {}

struct X;

impl Component for X {
    type Props = XProps;

    fn render(&mut self, ctx: &mut ComponentContext<'_>, _props: &Self::Props) -> View {
        X_RENDERS.fetch_add(1, Ordering::SeqCst);
        ctx.track(SIGNAL_A);
        View::from_fragment(ViewFragment::from_nodes(vec![ViewNode::text("x")]))
    }
}

#[derive(Debug)]
struct XFactory;

impl ComponentFactory for XFactory {
    fn component_id(&self) -> obscur_kit::identity::ComponentId {
        XProps::contract().id
    }
    fn contract(&self) -> ComponentContract {
        XProps::contract()
    }
    fn create_instance(&self, instance: InstanceId) -> ComponentInstance {
        ComponentInstance::new(instance, XProps::contract().id)
    }
    fn create_component(&self) -> Box<dyn ErasedComponent> {
        Box::new(X)
    }
}

#[doc = include_str!("spec_ch3_state.doc/YProps.md")]
#[derive(Contract)]
#[contract(name = "SpecY")]
struct YProps {}

struct Y;

impl Component for Y {
    type Props = YProps;

    fn render(&mut self, ctx: &mut ComponentContext<'_>, _props: &Self::Props) -> View {
        Y_RENDERS.fetch_add(1, Ordering::SeqCst);
        ctx.track(SIGNAL_B);
        View::from_fragment(ViewFragment::from_nodes(vec![ViewNode::text("y")]))
    }
}

#[derive(Debug)]
struct YFactory;

impl ComponentFactory for YFactory {
    fn component_id(&self) -> obscur_kit::identity::ComponentId {
        YProps::contract().id
    }
    fn contract(&self) -> ComponentContract {
        YProps::contract()
    }
    fn create_instance(&self, instance: InstanceId) -> ComponentInstance {
        ComponentInstance::new(instance, YProps::contract().id)
    }
    fn create_component(&self) -> Box<dyn ErasedComponent> {
        Box::new(Y)
    }
}

#[doc = include_str!("spec_ch3_state.doc/ChildProps.md")]
#[derive(Contract)]
#[contract(name = "SpecChild")]
struct ChildProps {
    #[doc = include_str!("spec_ch3_state.doc/value.md")]
    value: i64,
}

struct Child;

impl Component for Child {
    type Props = ChildProps;

    fn render(&mut self, _ctx: &mut ComponentContext<'_>, props: &Self::Props) -> View {
        // The child writes 9 into its OWN local view state derived from the prop.
        CHILD_LOCAL.store(9, Ordering::SeqCst);
        View::from_fragment(ViewFragment::from_nodes(vec![ViewNode::text(format!(
            "{}",
            props.value
        ))]))
    }
}

#[derive(Debug)]
struct ChildFactory;

impl ComponentFactory for ChildFactory {
    fn component_id(&self) -> obscur_kit::identity::ComponentId {
        ChildProps::contract().id
    }
    fn contract(&self) -> ComponentContract {
        ChildProps::contract()
    }
    fn create_instance(&self, instance: InstanceId) -> ComponentInstance {
        ComponentInstance::new(instance, ChildProps::contract().id)
    }
    fn create_component(&self) -> Box<dyn ErasedComponent> {
        Box::new(Child)
    }
}

fn host() -> UiRuntime {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 300.0));
    rt.register_component(ComponentRegistration::new(
        Box::new(XFactory),
        ComponentVersion::INITIAL,
    ))
    .expect("X registers");
    rt.register_component(ComponentRegistration::new(
        Box::new(YFactory),
        ComponentVersion::INITIAL,
    ))
    .expect("Y registers");
    rt.register_component(ComponentRegistration::new(
        Box::new(ChildFactory),
        ComponentVersion::INITIAL,
    ))
    .expect("Child registers");
    rt
}

fn both_mounted() -> View {
    view(vec![
        ViewNode::element(XProps::contract().id),
        ViewNode::element(YProps::contract().id),
    ])
}

fn only_y() -> View {
    view(vec![ViewNode::element(YProps::contract().id)])
}

fn node_ids(rt: &UiRuntime) -> Vec<u64> {
    let mut ids: Vec<u64> = rt.tree().iter().map(|(id, _)| id.raw()).collect();
    ids.sort_unstable();
    ids
}

#[doc = include_str!("spec_ch3_state.doc/a_signal_write_re_runs_only_its_own_consumer.md")]
#[test]
fn a_signal_write_re_runs_only_its_own_consumer() {
    let _guard = COUNTERS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut rt = host();
    let v = both_mounted();
    let _ = rt.render_now(&v, &[]);
    let _ = drain_frames(&mut rt, &v, &[]);

    let x_before = X_RENDERS.load(Ordering::SeqCst);
    let y_before = Y_RENDERS.load(Ordering::SeqCst);
    let ids_before = node_ids(&rt);
    assert!(x_before > 0 && y_before > 0, "both components mounted");

    let invalidated = rt.notify_signal(SIGNAL_A);
    assert_eq!(invalidated, 1, "exactly one consumer depends on signal A");

    let _ = drain_frames(&mut rt, &v, &[]);

    assert_eq!(
        X_RENDERS.load(Ordering::SeqCst) - x_before,
        1,
        "X's view function is invoked exactly once in that frame"
    );
    assert_eq!(
        Y_RENDERS.load(Ordering::SeqCst) - y_before,
        0,
        "Y reads a different signal and must not be re-rendered"
    );
    assert_eq!(
        node_ids(&rt),
        ids_before,
        "Y's retained subtree keeps identical node ids"
    );
}

#[doc = include_str!("spec_ch3_state.doc/removing_a_consumer_prunes_its_subscription.md")]
#[test]
fn removing_a_consumer_prunes_its_subscription() {
    let _guard = COUNTERS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut rt = host();
    let mounted = both_mounted();
    let _ = rt.render_now(&mounted, &[]);
    let _ = drain_frames(&mut rt, &mounted, &[]);

    // Remove X's subtree via a commit and present a frame.
    let removed = only_y();
    rt.request_frame(obscur_kit::runtime::frame::FrameReason::UiCommit);
    let _ = drain_frames(&mut rt, &removed, &[]);

    let x_before = X_RENDERS.load(Ordering::SeqCst);

    let mut presented = 0usize;
    for _ in 0..100 {
        assert_eq!(
            rt.notify_signal(SIGNAL_A),
            0,
            "signal A reports 0 live subscribers once its only consumer unmounted"
        );
        presented += drain_frames(&mut rt, &removed, &[]);
    }

    assert_eq!(
        presented, 0,
        "100 writes to a signal nobody reads present 0 frames"
    );
    assert_eq!(
        X_RENDERS.load(Ordering::SeqCst),
        x_before,
        "the removed component's view function is invoked 0 additional times"
    );
}

#[doc = include_str!("spec_ch3_state.doc/a_child_writing_local_state_does_not_reach_the_parents_value.md")]
#[test]
fn a_child_writing_local_state_does_not_reach_the_parents_value() {
    let mut rt = host();
    // The parent holds V = 5 and passes it as a typed prop.
    let parent_value: i64 = 5;
    let v = view(vec![
        ViewNode::element(ChildProps::contract().id).with_prop(ViewProperty::new(
            obscur_kit::ui_ir::name_binding::attr_symbol("value"),
            ViewValue::Int(parent_value),
        )),
    ]);

    let _ = rt.render_now(&v, &[]);
    let _ = drain_frames(&mut rt, &v, &[]);

    assert_eq!(
        CHILD_LOCAL.load(Ordering::SeqCst),
        9,
        "the child really did write 9 into its own local state"
    );
    assert_eq!(
        parent_value, 5,
        "the parent's V is still 5 — a prop is not a shared mutable cell"
    );
    assert_eq!(
        support::tree_text(&rt).as_deref(),
        Some("5"),
        "and the value the child renders from its prop is the parent's 5"
    );
}

#[doc = include_str!("spec_ch3_state.doc/no_core_crate_depends_on_tokio.md")]
#[test]
fn no_core_crate_depends_on_tokio() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("the workspace root")
        .to_path_buf();

    let mut offenders: Vec<String> = Vec::new();
    let mut inspected = 0usize;
    for layer in ["foundation", "domain", "application"] {
        let dir = root.join("crates").join(layer);
        let entries =
            std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("reading {}: {e}", dir.display()));
        for entry in entries.flatten() {
            let manifest = entry.path().join("Cargo.toml");
            let Ok(text) = std::fs::read_to_string(&manifest) else {
                continue;
            };
            inspected += 1;
            if text.contains("tokio") {
                offenders.push(manifest.display().to_string());
            }
        }
    }

    assert!(
        inspected > 0,
        "the crate layers were found — otherwise this case proves nothing"
    );
    assert!(
        offenders.is_empty(),
        "`tokio` must appear 0 times in the core crates' dependency graph, found: {offenders:?}"
    );
}
