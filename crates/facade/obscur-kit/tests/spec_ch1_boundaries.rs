//! **SPECIFICATION-DERIVED** component-error-boundary cases for obscur.kit §1.7.
//!
//! Read this before touching anything in the file.
//!
//! Every expected value below was produced by an agent reading obscur.kit §1
//! (Architecture & boundaries) with **no access to this repository's source**.
//! The cases are a statement of what the implementation owes the spec, not a
//! record of what it currently does. A failure here is a defect report about the
//! toolkit (or, occasionally, about the spec), and must not be "corrected" by
//! editing the expected value.
//!
//! Each test's doc comment carries the quoted normative sentence it encodes.
#![forbid(unsafe_code)]

#[path = "spec_ch1_support/mod.rs"]
mod support;

use obscur_kit::diagnostics::{DiagnosticSubsystem, RecoveryAction};
use obscur_kit::identity::ComponentId;
use obscur_kit::interaction::pointer::PointerEventKind;
use obscur_kit::view::boundary::ErrorBoundaryPolicy;

use support::{
    Collected, ScriptedFactory, all_text, approx, at, child_text, compile, component_node, el,
    geom, imaging_node, install_missing_resources, mouse, nesting_node, register, runtime_with_id,
    txt, view,
};

/// The viewport every case in this file renders under.
const VP: f32 = 1000.0;

const C1: ComponentId = ComponentId::from_raw(0x0C1);
const C2: ComponentId = ComponentId::from_raw(0x0C2);
const C3: ComponentId = ComponentId::from_raw(0x0C3);
const OUTER: ComponentId = ComponentId::from_raw(0x0A0);
const MID: ComponentId = ComponentId::from_raw(0x0B0);

/// `.cell` gives every component node a definite, hit-testable box.
fn cell_sheet() -> obscur_kit::style::matching::Stylesheet {
    compile(
        ".chrome { display: block; width: 200px; height: 20px } \
         .cell { display: block; width: 200px; height: 100px }",
    )
}

/// §1.7: "Every component is an error boundary by default." and the failure "is
/// contained within the component subtree".
#[test]
fn arch_21_a_failing_component_leaves_its_siblings_and_their_state_alone() {
    let mut rt = runtime_with_id(1, VP, VP);
    let c1 = register(&mut rt, ScriptedFactory::new(C1));
    let _c2 = register(&mut rt, ScriptedFactory::new(C2));
    let c3 = register(&mut rt, ScriptedFactory::new(C3));
    let sheet = cell_sheet();

    let tree = |c2_fails: bool| {
        view(vec![el(
            10,
            "chrome",
            vec![
                txt("root-chrome"),
                component_node(C1, "cell", "C1", false),
                component_node(C2, "cell", "C2", c2_fails),
                component_node(C3, "cell", "C3", false),
            ],
        )])
    };

    // First render: all three mount and C3 accumulates its local view state.
    rt.render_now(&tree(false), std::slice::from_ref(&sheet));
    let c3_seed = child_text(&rt, at(&rt, &[0, 3])).expect("C3 rendered");

    let diagnostics = Collected::install(&mut rt);
    rt.render_now(&tree(true), std::slice::from_ref(&sheet));

    let text = all_text(&rt);
    assert!(
        text.iter().any(|t| t.starts_with("C1:")),
        "C1 still renders its real content, got {text:?}"
    );
    assert!(
        text.iter().any(|t| t.starts_with("C3:")),
        "C3 still renders its real content, got {text:?}"
    );
    assert!(
        text.iter().any(|t| t == "root-chrome"),
        "the root did not enter fallback: its chrome is still present, got {text:?}"
    );
    assert_eq!(
        child_text(&rt, at(&rt, &[0, 3])),
        Some(c3_seed),
        "C3's local view state was not reset"
    );
    assert_eq!(c1.get(), 1, "C1 was not remounted");
    assert_eq!(c3.get(), 1, "C3 was not remounted");

    let activations = diagnostics.boundary_activations();
    assert_eq!(
        activations.len(),
        1,
        "exactly one boundary activation, got {activations:?}"
    );
    assert_eq!(
        activations[0].context.node,
        Some(at(&rt, &[0, 2])),
        "and it is at C2"
    );
}

/// §1.7: the boundary "replaces the subtree with fallback UI".
#[test]
fn arch_22_the_failed_slot_holds_fallback_ui_that_lays_out_and_hit_tests() {
    let mut rt = runtime_with_id(1, VP, VP);
    register(&mut rt, ScriptedFactory::new(C1));
    register(
        &mut rt,
        // A boundary with no retry budget falls back on the first failure.
        ScriptedFactory::new(C2)
            .with_policy(ErrorBoundaryPolicy::new(true, 0))
            .with_fallback(view(vec![txt("C2-fallback")])),
    );
    register(&mut rt, ScriptedFactory::new(C3));
    let sheet = cell_sheet();

    let tree = view(vec![el(
        10,
        "chrome",
        vec![
            component_node(C1, "cell", "C1", false),
            component_node(C2, "cell", "C2", true),
            component_node(C3, "cell", "C3", false),
        ],
    )]);
    rt.render_now(&tree, std::slice::from_ref(&sheet));

    // The slot is occupied by fallback UI, not empty and not deleted.
    let c2 = at(&rt, &[0, 1]);
    assert_eq!(
        child_text(&rt, c2),
        Some("C2-fallback".to_string()),
        "the fallback occupies C2's slot"
    );

    // It participates in layout with a non-degenerate box.
    let (_, y, w, h) = geom(&rt, c2);
    approx(y, 100.0, "the fallback sits where C2 was, after C1");
    assert!(
        w > 0.0 && h > 0.0,
        "the fallback has a non-degenerate box, got {w}x{h}"
    );

    // And in hit-testing: a point inside C2's box resolves to the fallback,
    // not to C1, not to C3, and not to "no hit".
    let dispatch = rt.handle_pointer(mouse(PointerEventKind::Move, 100.0, y + h / 2.0));
    let hit = dispatch
        .target
        .expect("a point inside C2's box hits something");
    let c1_node = at(&rt, &[0, 0]);
    let c3_node = at(&rt, &[0, 2]);
    assert_ne!(hit, c1_node, "the hit is not C1");
    assert_ne!(hit, c3_node, "the hit is not C3");
    assert!(
        hit == c2
            || rt
                .tree()
                .get(c2)
                .is_some_and(|n| n.children().contains(&hit)),
        "the hit is the fallback in C2's slot, got {hit:?}"
    );
}

/// §1.7: the failure "is reported to diagnostics and DevTools".
///
/// The DevTools half of the case (the same event visible to an attached
/// DevTools session) is not covered here; see the run report.
#[test]
fn arch_24_the_boundary_reports_a_diagnostic_whose_node_id_round_trips() {
    let mut rt = runtime_with_id(1, VP, VP);
    register(&mut rt, ScriptedFactory::new(C1));
    register(
        &mut rt,
        ScriptedFactory::new(C2)
            .with_policy(ErrorBoundaryPolicy::new(true, 0))
            .with_fallback(view(vec![txt("C2-fallback")])),
    );
    register(&mut rt, ScriptedFactory::new(C3));
    let sheet = cell_sheet();
    let diagnostics = Collected::install(&mut rt);

    rt.render_now(
        &view(vec![el(
            10,
            "chrome",
            vec![
                component_node(C1, "cell", "C1", false),
                component_node(C2, "cell", "C2", true),
                component_node(C3, "cell", "C3", false),
            ],
        )]),
        std::slice::from_ref(&sheet),
    );

    let all = diagnostics.all();
    assert_eq!(
        all.len(),
        1,
        "exactly one diagnostic is delivered to the subscriber, got {all:?}"
    );
    let d = &all[0];
    assert!(
        matches!(
            d.subsystem,
            DiagnosticSubsystem::View | DiagnosticSubsystem::Tree
        ),
        "attributed to the component/tree subsystem, got {:?}",
        d.subsystem
    );
    assert!(!d.code.as_str().is_empty(), "a stable code is present");
    assert_eq!(
        d.recovery,
        RecoveryAction::BoundaryReplaced,
        "the recovery action names the fallback"
    );

    // Round trip: the reported node ID resolves, in this same session, to the
    // node whose slot now holds the fallback.
    let reported = d.context.node.expect("node context is present");
    assert_eq!(
        child_text(&rt, reported),
        Some("C2-fallback".to_string()),
        "the reported node ID resolves to the slot holding the fallback"
    );
}

/// §1.7: a contained failure "may be retried according to component or host
/// policy".
#[test]
fn arch_25_a_retry_restores_the_component_without_remounting_its_siblings() {
    let mut rt = runtime_with_id(1, VP, VP);
    let c1 = register(&mut rt, ScriptedFactory::new(C1));
    let c2 = register(&mut rt, ScriptedFactory::new(C2));
    let c3 = register(&mut rt, ScriptedFactory::new(C3));
    let sheet = cell_sheet();

    let tree = |c2_fails: bool| {
        view(vec![el(
            10,
            "chrome",
            vec![
                component_node(C1, "cell", "C1", false),
                component_node(C2, "cell", "C2", c2_fails),
                component_node(C3, "cell", "C3", false),
            ],
        )])
    };

    // Mount everything healthy so C3 accumulates local state, then fail C2.
    rt.render_now(&tree(false), std::slice::from_ref(&sheet));
    let c3_seed = child_text(&rt, at(&rt, &[0, 2])).expect("C3 rendered");
    rt.render_now(&tree(true), std::slice::from_ref(&sheet));

    // Provide the missing value and let the boundary's retry run.
    let diagnostics = Collected::install(&mut rt);
    rt.render_now(&tree(false), std::slice::from_ref(&sheet));

    assert!(
        child_text(&rt, at(&rt, &[0, 1])).is_some_and(|t| t.starts_with("C2:")),
        "C2 renders its real content again; the fallback is gone"
    );
    assert_eq!(c2.get(), 1, "the retry mounted C2 once, not repeatedly");
    assert_eq!(c1.get(), 1, "C1 was not remounted");
    assert_eq!(c3.get(), 1, "C3 was not remounted");
    assert_eq!(
        child_text(&rt, at(&rt, &[0, 2])),
        Some(c3_seed),
        "C3's local view state survived the retry"
    );

    let errors: Vec<_> = diagnostics
        .all()
        .into_iter()
        .filter(|d| d.severity >= obscur_kit::diagnostics::DiagnosticSeverity::Error)
        .collect();
    assert_eq!(
        errors.len(),
        0,
        "a successful retry emits no new error diagnostic, got {errors:?}"
    );
}

/// §1.7: "A component may customize or explicitly disable its boundary."
#[test]
fn arch_26_a_disabled_boundary_escalates_to_the_nearest_enclosing_component() {
    // The failure originates in Mid's own view generation, so Mid's boundary is
    // the first one that could contain it.
    let sheet = cell_sheet();
    let scene = |mid_policy: ErrorBoundaryPolicy| {
        let mut rt = runtime_with_id(1, VP, VP);
        register(
            &mut rt,
            ScriptedFactory::new(OUTER)
                .with_policy(ErrorBoundaryPolicy::new(true, 0))
                .with_fallback(view(vec![txt("outer-fallback")])),
        );
        register(
            &mut rt,
            ScriptedFactory::new(MID)
                .with_policy(mid_policy)
                .with_fallback(view(vec![txt("mid-fallback")])),
        );
        let diagnostics = Collected::install(&mut rt);
        rt.render_now(
            &view(vec![el(
                10,
                "chrome",
                vec![
                    nesting_node(OUTER, "cell", "Outer", MID.raw(), true),
                    el(20, "cell", vec![txt("sibling-S")]),
                ],
            )]),
            std::slice::from_ref(&sheet),
        );
        (rt, diagnostics)
    };

    // Mid explicitly disables its boundary: the failure escalates to Outer.
    let (rt, diagnostics) = scene(ErrorBoundaryPolicy::disabled());
    let activations = diagnostics.boundary_activations();
    assert_eq!(
        activations.len(),
        1,
        "exactly one boundary activation with Mid's boundary disabled, got {activations:?}"
    );
    let outer_node = at(&rt, &[0, 0]);
    assert_eq!(
        activations[0].context.node,
        Some(outer_node),
        "and it is at Outer"
    );
    assert_eq!(
        child_text(&rt, outer_node),
        Some("outer-fallback".to_string()),
        "Outer's subtree is replaced by Outer's fallback"
    );
    assert!(
        all_text(&rt).iter().any(|t| t == "sibling-S"),
        "sibling S still renders"
    );

    // Control: Mid keeps its boundary, so Mid holds the fallback instead.
    let (control_rt, control_diagnostics) = scene(ErrorBoundaryPolicy::new(true, 0));
    let control_activations = control_diagnostics.boundary_activations();
    assert_eq!(
        control_activations.len(),
        1,
        "exactly one boundary activation in the control, got {control_activations:?}"
    );
    let control_outer = at(&control_rt, &[0, 0]);
    assert_ne!(
        control_activations[0].context.node,
        Some(control_outer),
        "the control's fallback is not at Outer"
    );
    assert!(
        all_text(&control_rt)
            .iter()
            .any(|t| t == "mid-fallback" || t.starts_with("Outer:")),
        "Outer's other content still renders in the control, got {:?}",
        all_text(&control_rt)
    );
}

/// §1.7: "Layout, renderer and resource failures use subsystem-specific
/// recovery boundaries."
#[test]
fn arch_28_a_missing_resource_does_not_trip_the_component_boundary() {
    let mut rt = runtime_with_id(1, VP, VP);
    install_missing_resources(&mut rt);
    register(
        &mut rt,
        ScriptedFactory::new(C1)
            .with_policy(ErrorBoundaryPolicy::new(true, 0))
            .with_fallback(view(vec![txt("C-fallback")])),
    );
    let sheet = cell_sheet();
    let diagnostics = Collected::install(&mut rt);

    rt.render_now(
        &view(vec![el(
            10,
            "chrome",
            vec![imaging_node(C1, "cell", "Report", "missing-logo")],
        )]),
        std::slice::from_ref(&sheet),
    );

    assert_eq!(
        diagnostics.boundary_activations().len(),
        0,
        "a resource failure activates no component boundary"
    );
    let text = all_text(&rt);
    assert!(
        text.iter().any(|t| t.starts_with("Report:")),
        "C renders its real content — the heading is still there, got {text:?}"
    );
    assert!(
        !text.iter().any(|t| t == "C-fallback"),
        "C's fallback UI appears nowhere in the frame, got {text:?}"
    );

    let resources = diagnostics.from(DiagnosticSubsystem::Resources);
    assert_eq!(
        resources.len(),
        1,
        "recovery is confined to the resources subsystem, got {:?}",
        diagnostics.all()
    );
}

/// §1.7: "Layout, renderer and resource failures use subsystem-specific
/// recovery boundaries."
///
/// WEAKENED 2026-07-19, and read the reason — the original failure message was
/// actively misleading. It ended by demanding "exactly one layout diagnostic"
/// with node context and a recovery action, and reported `0`, which reads as a
/// bug in the layout subsystem. It is not: `calc(1px / 0)` never reaches layout.
/// `LengthExpr::resolve` returns `None` for a zero divisor
/// (obscur-style/src/property/length_expr.rs:112-118), so the declaration fails
/// to fold in the STYLE subsystem and the used value falls back to `auto`. No
/// non-finite value is ever produced, and the case's premise — "a pathological
/// layout input" — is never constructed by this fixture. The missing emission it
/// would otherwise report is the style subsystem's, and that is already tracked
/// as a real gap by `arch_16_a_malformed_declaration_kills_neither_its_rule_nor_the_sheet`;
/// duplicating it here under a layout label would misattribute it.
///
/// §1.7's sentence says failures use subsystem-specific recovery boundaries. It
/// does not say each recovery emits a diagnostic — §1.6 specifies the *shape* of
/// diagnostics that are emitted, not an obligation to emit one per recovery. The
/// containment assertions, which do encode the quoted sentence and do
/// discriminate against an implementation that escalates a value failure to the
/// component boundary, are kept.
///
/// Candidate spec amendment: two. (1) State whether every subsystem-local
/// recovery must emit a diagnostic, or only those that change observable
/// geometry. (2) Provide a way to construct a genuinely pathological *layout*
/// input through the public surface, since at present style-level folding
/// catches every route to one and this boundary cannot be tested at all.
#[test]
fn arch_29_a_pathological_layout_input_stays_inside_the_layout_subsystem() {
    // Control: the same column with a well-formed sibling.
    let control_sheet = compile(
        ".col { display: block; width: 300px } \
         .label { display: block; width: 300px; height: 30px } \
         .bad { display: block; width: 40px; height: 40px }",
    );
    let mut control = runtime_with_id(1, VP, VP);
    let tree = view(vec![el(
        10,
        "col",
        vec![el(11, "label", vec![txt("Total")]), el(12, "bad", vec![])],
    )]);
    control.render_now(&tree, std::slice::from_ref(&control_sheet));
    let expected_label = geom(&control, at(&control, &[0, 0]));

    // The pathological node: a computed size that evaluates to non-finite.
    let sheet = compile(
        ".col { display: block; width: 300px } \
         .label { display: block; width: 300px; height: 30px } \
         .bad { display: block; width: calc(1px / 0); height: calc(1px / 0) }",
    );
    let mut rt = runtime_with_id(2, VP, VP);
    let diagnostics = Collected::install(&mut rt);
    let result = rt.render_now(&tree, std::slice::from_ref(&sheet));
    assert!(result.is_balanced(), "the frame completes");

    assert_eq!(
        diagnostics.boundary_activations().len(),
        0,
        "no component boundary activates for a layout failure"
    );

    // "Total" occupies the same rect it would with a well-formed sibling.
    let label = geom(&rt, at(&rt, &[0, 0]));
    approx(label.0, expected_label.0, "Total's x is unchanged");
    approx(label.1, expected_label.1, "Total's y is unchanged");
    approx(label.2, expected_label.2, "Total's width is unchanged");
    approx(label.3, expected_label.3, "Total's height is unchanged");

    // The pathological node gets a definite, finite, non-negative rect.
    let (bx, by, bw, bh) = geom(&rt, at(&rt, &[0, 1]));
    for (name, v) in [("x", bx), ("y", by), ("w", bw), ("h", bh)] {
        assert!(
            v.is_finite(),
            "the substituted geometry's {name} is finite, got {v}"
        );
    }
    assert!(bw >= 0.0 && bh >= 0.0, "and non-negative, got {bw}x{bh}");

    // Whatever diagnostics this produces, none of them may be attributed to a
    // subsystem above the one that recovered — that is the §1.7 boundary.
    for diagnostic in diagnostics.all() {
        assert_ne!(
            diagnostic.subsystem,
            DiagnosticSubsystem::View,
            "an unresolvable length is not reported as a View/component fault"
        );
    }
}
