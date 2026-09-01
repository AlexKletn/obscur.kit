#![doc = include_str!("error_boundaries.doc/_module.md")]
#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::rc::Rc;

use obscur_diagnostics::{
    ChannelSink, Diagnostic, DiagnosticBufferPolicy, DiagnosticChannel, DiagnosticFilter,
    DiagnosticSeverity, DiagnosticSink, DiagnosticSubsystem, RecoveryAction,
};
use obscur_identity::NodeId;
use obscur_runtime::UiRuntime;
use obscur_runtime::component::COMPONENT_PROPS_FAILED;
use obscur_testkit::fixture::TestFixture;
use obscur_testkit::host::{EnclosingComponent, GuardedComponent};
use obscur_testkit::selector::{NodePath, SemanticSelector};
use obscur_view::boundary::ErrorBoundaryPolicy;
use obscur_view::view::ViewValue;

/// A [`DiagnosticSink`] that collects into a shared buffer, so a test can
/// read what the runtime reported after installing the sink by value.
#[derive(Debug, Default, Clone)]
struct SharedDiagnostics(Rc<RefCell<Vec<Diagnostic>>>);

impl DiagnosticSink for SharedDiagnostics {
    fn emit(&self, diagnostic: &Diagnostic) {
        self.0.borrow_mut().push(diagnostic.clone());
    }
}

/// The shipped [`ChannelSink`], shared: `set_diagnostic_sink` takes the
/// sink by value, so a test that wants to drain it needs a handle.
struct SharedChannel(Rc<ChannelSink>);

impl DiagnosticSink for SharedChannel {
    fn emit(&self, diagnostic: &Diagnostic) {
        self.0.emit(diagnostic);
    }
}

/// A runtime with `<Guarded>` registered under `policy`, plus a sink to
/// read the diagnostics back from.
fn guarded_runtime(
    policy: ErrorBoundaryPolicy,
    fallback: Option<obscur_view::view::View>,
) -> (UiRuntime, SharedDiagnostics) {
    let mut runtime = TestFixture::runtime();
    runtime
        .register_component(GuardedComponent::registration(policy, fallback))
        .expect("first registration");
    let diagnostics = SharedDiagnostics::default();
    runtime.set_diagnostic_sink(Box::new(diagnostics.clone()));
    (runtime, diagnostics)
}

/// The node the component instance is mounted on.
fn component_node(runtime: &UiRuntime) -> NodeId {
    NodePath::new(&[0]).resolve(runtime)
}

/// The node the first diagnostic named — the failing child, which the
/// containment has since unmounted, so the tree can no longer name it.
fn outer_child(recorded: &[Diagnostic]) -> NodeId {
    recorded[0].context.node.expect("anchored to a node")
}

#[test]
fn a_failed_render_is_reported_with_every_field_1_6_asks_for() {
    let (mut runtime, diagnostics) = guarded_runtime(
        ErrorBoundaryPolicy::default(),
        Some(GuardedComponent::text_view("sorry")),
    );
    runtime.render_now(&GuardedComponent::view(ViewValue::Text("2".into())), &[]);

    let node = component_node(&runtime);
    let recorded = diagnostics.0.borrow();
    assert_eq!(recorded.len(), 1, "the failure reached the channel");
    let diagnostic = &recorded[0];
    assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
    assert_eq!(diagnostic.subsystem, DiagnosticSubsystem::View);
    assert_eq!(diagnostic.code, COMPONENT_PROPS_FAILED);
    assert_eq!(
        diagnostic.context.node,
        Some(node),
        "anchored to the node that failed"
    );
    assert_eq!(
        diagnostic.recovery,
        RecoveryAction::Retried,
        "the boundary's budget bought one more frame, and says so"
    );
    assert!(!diagnostic.message.is_empty());
}

#[test]
fn the_report_names_the_property_in_words_not_only_by_number() {
    // Audit 1-B. `InternedSymbol` is an FNV hash with no reverse table, so a
    // message carrying only the symbol read `property #2147483911 could not be
    // read as u32` — true, anchored, severity-correct, and of no use whatever
    // to the author of the component that refused. The name reaches here as a
    // literal from `#[derive(Contract)]`, which is the last place it exists.
    //
    // The number stays alongside it: it is what every other report about this
    // property prints, and the two must correlate.
    let (mut runtime, diagnostics) = guarded_runtime(ErrorBoundaryPolicy::default(), None);
    runtime.render_now(&GuardedComponent::view(ViewValue::Text("2".into())), &[]);

    let recorded = diagnostics.0.borrow();
    let message = &recorded[0].message;
    assert!(
        message.contains("`step`"),
        "the failing property is named: {message}"
    );
    assert!(
        message.contains("u32"),
        "and so is the type it wanted: {message}"
    );
}

#[test]
fn an_exhausted_boundary_replaces_the_subtree_with_the_authors_fallback() {
    let (mut runtime, diagnostics) = guarded_runtime(
        ErrorBoundaryPolicy::default(),
        Some(GuardedComponent::text_view("sorry")),
    );
    let bad = GuardedComponent::view(ViewValue::Text("2".into()));
    runtime.render_now(&bad, &[]);
    let node = component_node(&runtime);
    assert_eq!(
        SemanticSelector::child_text(&runtime, node),
        None,
        "nothing rendered yet"
    );

    runtime.render_now(&bad, &[]);
    assert_eq!(
        SemanticSelector::child_text(&runtime, node),
        Some("sorry".to_string()),
        "the fallback UI is what the failed subtree shows"
    );
    let recorded = diagnostics.0.borrow();
    assert_eq!(
        recorded.iter().map(|d| d.recovery).collect::<Vec<_>>(),
        vec![RecoveryAction::Retried, RecoveryAction::BoundaryReplaced],
        "both decisions are reported, in the order they were taken"
    );
}

#[test]
fn a_disabled_boundary_neither_retries_nor_falls_back() {
    // The same failing component as above, differing ONLY in its policy.
    let (mut runtime, diagnostics) = guarded_runtime(
        ErrorBoundaryPolicy::disabled(),
        Some(GuardedComponent::text_view("sorry")),
    );
    let bad = GuardedComponent::view(ViewValue::Text("2".into()));
    runtime.render_now(&bad, &[]);
    runtime.render_now(&bad, &[]);

    let node = component_node(&runtime);
    assert_eq!(
        SemanticSelector::child_text(&runtime, node),
        None,
        "its own fallback is not used: it opted out of containment"
    );
    let recorded = diagnostics.0.borrow();
    assert_eq!(
        recorded.iter().map(|d| d.recovery).collect::<Vec<_>>(),
        vec![RecoveryAction::None],
        "reported once, unrecovered, and not re-fired every frame"
    );
}

#[test]
fn a_failure_escaping_a_disabled_boundary_is_contained_by_the_enclosing_one() {
    // Outer (enabled, with a fallback) renders Guarded (disabled), whose
    // props cannot decode. The failure has nowhere to be contained but
    // the enclosing boundary.
    let (mut runtime, diagnostics) = guarded_runtime(
        ErrorBoundaryPolicy::disabled(),
        Some(GuardedComponent::text_view("sorry")),
    );
    runtime
        .register_component(EnclosingComponent::registration())
        .expect("second registration");
    runtime.render_now(&EnclosingComponent::view(), &[]);

    let outer = component_node(&runtime);
    assert_eq!(
        SemanticSelector::child_text(&runtime, outer),
        Some("outer fallback".to_string()),
        "the enclosing boundary's subtree is what gets replaced"
    );
    assert_eq!(
        runtime.mounted_component_count(),
        1,
        "replacing the subtree unmounted the component that failed"
    );
    let recorded = diagnostics.0.borrow();
    assert_eq!(
        recorded
            .iter()
            .map(|d| (d.context.node, d.recovery))
            .collect::<Vec<_>>(),
        vec![
            (Some(outer_child(&recorded)), RecoveryAction::None),
            (Some(outer), RecoveryAction::BoundaryReplaced),
        ],
        "the failing node reports no recovery; the boundary that answered \
         reports the replacement"
    );
}

#[test]
fn a_component_recovers_when_its_props_decode_again() {
    let (mut runtime, diagnostics) = guarded_runtime(
        ErrorBoundaryPolicy::default(),
        Some(GuardedComponent::text_view("sorry")),
    );
    let bad = GuardedComponent::view(ViewValue::Text("2".into()));
    runtime.render_now(&bad, &[]);
    runtime.render_now(&bad, &[]);
    let node = component_node(&runtime);
    assert_eq!(
        SemanticSelector::child_text(&runtime, node),
        Some("sorry".to_string())
    );

    // The author fixes the attribute: the boundary is not a tombstone.
    runtime.render_now(&GuardedComponent::view(ViewValue::Int(2)), &[]);
    assert_eq!(
        SemanticSelector::child_text(&runtime, node),
        Some("healthy".to_string())
    );
    assert_eq!(
        diagnostics.0.borrow().len(),
        2,
        "recovering is not itself a failure"
    );
}

#[test]
fn the_failure_reaches_a_real_subscriber_through_the_shipped_channel() {
    // Not a bespoke test sink: the §7.8 ChannelSink/DiagnosticChannel a
    // host or DevTools backend actually subscribes with. This is the
    // whole of §1-B — a Diagnostic built by production code arriving at
    // a subscriber that filtered for it.
    let sink = Rc::new(ChannelSink::new(DiagnosticChannel::new()));
    let subscription = sink.channel_mut().subscribe(
        DiagnosticFilter::min(DiagnosticSeverity::Error),
        DiagnosticBufferPolicy::unbounded(),
    );

    let mut runtime = TestFixture::runtime();
    runtime
        .register_component(GuardedComponent::registration(
            ErrorBoundaryPolicy::default().with_max_retries(0),
            Some(GuardedComponent::text_view("sorry")),
        ))
        .expect("first registration");
    runtime.set_diagnostic_sink(Box::new(SharedChannel(Rc::clone(&sink))));
    runtime.render_now(&GuardedComponent::view(ViewValue::Text("2".into())), &[]);

    let drained = sink.channel_mut().drain(subscription);
    assert_eq!(drained.len(), 1, "the subscriber received the failure");
    assert_eq!(drained[0].code, COMPONENT_PROPS_FAILED);
    assert_eq!(drained[0].recovery, RecoveryAction::BoundaryReplaced);
    assert_eq!(drained[0].context.node, Some(component_node(&runtime)));
}

#[test]
fn an_uncontained_propagated_failure_leaves_the_last_output_but_says_so() {
    // A disabled boundary with no enclosing one: the failure escapes the
    // component tree. Nothing recovers it, so the last good output does
    // stand — the honest floor is that it is reported, once, rather than
    // re-fired every frame.
    let (mut runtime, diagnostics) = guarded_runtime(
        ErrorBoundaryPolicy::disabled(),
        Some(GuardedComponent::text_view("sorry")),
    );
    runtime.render_now(&GuardedComponent::view(ViewValue::Int(2)), &[]);
    let node = component_node(&runtime);
    assert_eq!(
        SemanticSelector::child_text(&runtime, node),
        Some("healthy".to_string())
    );

    let bad = GuardedComponent::view(ViewValue::Text("2".into()));
    runtime.render_now(&bad, &[]);
    runtime.render_now(&bad, &[]);
    assert_eq!(
        SemanticSelector::child_text(&runtime, node),
        Some("healthy".to_string()),
        "uncontained: the stale output stands, which is what disabling every \
         boundary above it asked for"
    );
    assert_eq!(
        diagnostics
            .0
            .borrow()
            .iter()
            .map(|d| (d.context.node, d.recovery))
            .collect::<Vec<_>>(),
        vec![(Some(node), RecoveryAction::None)],
        "reported once and unrecovered — never silent, never a per-frame storm"
    );
}

#[test]
fn a_runtime_with_no_sink_still_takes_the_failure_path() {
    // The default NullDiagnosticSink must not be a second code path: the
    // boundary decides and acts identically with nobody listening.
    let mut runtime = TestFixture::runtime();
    runtime
        .register_component(GuardedComponent::registration(
            ErrorBoundaryPolicy::default().with_max_retries(0),
            Some(GuardedComponent::text_view("sorry")),
        ))
        .expect("first registration");
    runtime.render_now(&GuardedComponent::view(ViewValue::Text("2".into())), &[]);
    let node = component_node(&runtime);
    assert_eq!(
        SemanticSelector::child_text(&runtime, node),
        Some("sorry".to_string())
    );
}
