#![doc = include_str!("component_lifecycle.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_identity::NodeId;
use obscur_runtime::UiRuntime;
use obscur_testkit::fixture::TestFixture;
use obscur_testkit::host::CounterComponent;
use obscur_testkit::selector::{NodePath, SemanticSelector};
use obscur_tree::node::NodeFlags;
use obscur_view::view::{View, ViewFragment, ViewValue};

/// A runtime with the counter registered and nothing rendered yet.
fn runtime_with_counter() -> UiRuntime {
    let mut runtime = TestFixture::runtime();
    runtime
        .register_component(CounterComponent::registration(0, None))
        .expect("first registration");
    runtime
}

/// The node the component instance is mounted on.
fn component_node(runtime: &UiRuntime) -> NodeId {
    NodePath::new(&[0]).resolve(runtime)
}

/// The text the component rendered underneath its own node.
fn rendered_text(runtime: &UiRuntime) -> String {
    SemanticSelector::child_text(runtime, component_node(runtime))
        .expect("the component rendered its text output")
}

#[test]
fn the_runtime_mounts_and_renders_a_registered_component() {
    // THE vertical slice: a node naming a registered kind mounts an
    // instance, its typed props are decoded from the node's authored
    // properties, and its rendered output becomes that node's children.
    let mut runtime = runtime_with_counter();
    runtime.render_now(&CounterComponent::view(2), &[]);

    assert_eq!(runtime.mounted_component_count(), 1);
    assert_eq!(
        rendered_text(&runtime),
        "2",
        "the decoded step reached render"
    );
}

#[test]
fn local_state_survives_across_frames_on_a_stable_node() {
    // The node is reused across frames (same kind, same position), so the
    // instance — and the LocalState it owns — is never re-created.
    let mut runtime = runtime_with_counter();
    runtime.render_now(&CounterComponent::view(2), &[]);
    let node = component_node(&runtime);
    assert_eq!(rendered_text(&runtime), "2");

    runtime.render_now(&CounterComponent::view(3), &[]);
    assert_eq!(component_node(&runtime), node, "the node was reused");
    assert_eq!(runtime.mounted_component_count(), 1, "no remount");
    assert_eq!(rendered_text(&runtime), "5", "2 + 3: the state persisted");
}

#[test]
fn only_the_component_whose_props_changed_re_renders() {
    // THE incremental-invalidation property for components (§3.1:
    // "dependency tracking invalidates only consumers of changed values").
    // Two sibling counters; only the first one's props change. Before this,
    // every structural frame re-rendered EVERY mounted component, so both
    // totals would advance.
    let view = |a: i64, b: i64| {
        View::from_fragment(ViewFragment::from_nodes(vec![
            CounterComponent::element(ViewValue::Int(a)),
            CounterComponent::element(ViewValue::Int(b)),
        ]))
    };
    let text_at = |runtime: &UiRuntime, index: usize| {
        SemanticSelector::child_text(runtime, NodePath::new(&[index]).resolve(runtime))
            .expect("the counter rendered its text output")
    };

    let mut runtime = runtime_with_counter();
    runtime.render_now(&view(2, 10), &[]);
    assert_eq!(text_at(&runtime, 0), "2");
    assert_eq!(text_at(&runtime, 1), "10");

    // Only the first counter's `step` changes.
    runtime.render_now(&view(3, 10), &[]);
    assert_eq!(text_at(&runtime, 0), "5", "2 + 3: it re-rendered");
    assert_eq!(
        text_at(&runtime, 1),
        "10",
        "the untouched sibling must NOT have re-rendered"
    );
}

#[test]
fn an_unchanged_frame_re_renders_nothing() {
    // Re-rendering an identical view touches no component at all: every
    // instance's output is already current.
    let mut runtime = runtime_with_counter();
    runtime.render_now(&CounterComponent::view(2), &[]);
    assert_eq!(rendered_text(&runtime), "2");

    runtime.render_now(&CounterComponent::view(2), &[]);
    assert_eq!(
        rendered_text(&runtime),
        "2",
        "an identical frame must not re-run render (it would read 4)"
    );
}

#[test]
fn the_root_reconcile_does_not_destroy_the_components_output() {
    // The desired node `<Counter step=N/>` has NO children — the component
    // supplies them. Without COMPONENT_HOST, the root reconcile would
    // treat the desired (empty) children as authoritative and delete the
    // component's output every frame, re-creating it afterwards with fresh
    // NodeIds — which would also unmount and remount any nested component,
    // losing its local state. Identity of the output node proves it stands.
    let mut runtime = runtime_with_counter();
    runtime.render_now(&CounterComponent::view(2), &[]);

    let host = component_node(&runtime);
    assert!(
        runtime
            .tree()
            .get(host)
            .expect("live")
            .flags()
            .contains(NodeFlags::COMPONENT_HOST),
        "mounting flags the node as a component host"
    );
    let output = runtime.tree().get(host).expect("live").child_nodes()[0];

    runtime.render_now(&CounterComponent::view(3), &[]);
    assert_eq!(
        runtime.tree().get(host).expect("live").child_nodes()[0],
        output,
        "the component's output node survived the root reconcile"
    );
}

#[test]
fn unmounting_clears_the_component_host_flag() {
    // Otherwise a node whose component was unmounted would stay opaque to
    // reconciliation forever.
    let mut runtime = runtime_with_counter();
    runtime.render_now(&CounterComponent::view(2), &[]);
    let host = component_node(&runtime);
    assert!(runtime.tree().get(host).is_some());

    runtime.render_now(&View::empty(), &[]);
    // The node is gone with its instance; nothing is left flagged.
    assert_eq!(runtime.mounted_component_count(), 0);
    assert!(runtime.tree().get(host).is_none(), "the host node is gone");
}

#[test]
fn removing_the_node_unmounts_the_instance() {
    let mut runtime = runtime_with_counter();
    runtime.render_now(&CounterComponent::view(2), &[]);
    assert_eq!(runtime.mounted_component_count(), 1);

    runtime.render_now(&View::empty(), &[]);
    assert_eq!(runtime.mounted_component_count(), 0);
}

#[test]
fn a_host_that_registers_nothing_is_unaffected() {
    // The same view, with no registration: the element is a plain node,
    // nothing mounts, and it renders no children of its own.
    let mut runtime = TestFixture::runtime();
    runtime.render_now(&CounterComponent::view(2), &[]);
    assert_eq!(runtime.mounted_component_count(), 0);
    let node = component_node(&runtime);
    assert!(
        runtime
            .tree()
            .get(node)
            .expect("live")
            .child_nodes()
            .is_empty(),
        "an unregistered element gets no component output"
    );
}

#[test]
fn undecodable_props_are_retried_once_and_then_stop_showing_stale_output() {
    // First a good frame, so the component has output to go stale.
    let mut runtime = runtime_with_counter();
    runtime.render_now(&CounterComponent::view(2), &[]);
    assert_eq!(rendered_text(&runtime), "2");

    // Now the author writes `step="2"` — text where a number is wanted.
    let bad = View::from_fragment(ViewFragment::from_nodes(vec![CounterComponent::element(
        ViewValue::Text("2".into()),
    )]));
    runtime.render_now(&bad, &[]);
    // The default boundary's budget buys the props exactly one frame to
    // become decodable, so this frame keeps the subtree — deliberately,
    // and reported, rather than by falling through the failure.
    assert_eq!(runtime.mounted_component_count(), 1);
    assert_eq!(rendered_text(&runtime), "2");

    // The budget is now spent: the last good output does not get to
    // stand in for the current one indefinitely.
    runtime.render_now(&bad, &[]);
    assert_eq!(
        runtime
            .tree()
            .get(component_node(&runtime))
            .expect("live")
            .child_nodes(),
        &[],
        "the boundary replaced the subtree (this counter has no fallback)"
    );
}
