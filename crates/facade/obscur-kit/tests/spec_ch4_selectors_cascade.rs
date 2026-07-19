//! **SPECIFICATION-DERIVED** selector, specificity and cascade cases for
//! obscur.kit §4.13 ("complex selectors", ":nth-*") and §4.14 (CSS cascade,
//! `!important`), plus `display: contents`.
//!
//! Every expected value below was produced by an agent reading the specification
//! — obscur.kit §4 and the W3C selectors-4 / css-display-3 text it normatively
//! cites — with **no access to this repository's source**. A failure here is a
//! defect report about the engine, and must not be "corrected" by editing the
//! expected value.
//!
//! Translation note: this workspace's view tree has no element *tag names* that a
//! CSS type selector could match, so where the deriver wrote `em`, `li`, `p`,
//! `div` or `strong` as the **matching** part of a selector, a class of the same
//! name is authored instead (`.em`, `.li`, …) and the node carries that class.
//! The specificity arithmetic each case turns on is preserved or made *harder*,
//! never easier — see each test. Type selectors that are only there to *not*
//! match (the `:not(em, strong#foo)` arguments) are left verbatim.
#![forbid(unsafe_code)]

#[path = "spec_ch4_support/mod.rs"]
mod support;

use obscur_kit::identity::{ComponentId, NodeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::cascade::CascadeOrigin;
use obscur_kit::style::property::StyleColor;
use obscur_kit::view::view::{CLASS, ID, View, ViewFragment, ViewNode, ViewProperty, ViewValue};
use support::{approx, at, color_of, compile, compile_in, drive, drive_sheets, el, geom, view};

const VW: f32 = 1000.0;

const RED: StyleColor = StyleColor {
    r: 1.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};
const BLUE: StyleColor = StyleColor {
    r: 0.0,
    g: 0.0,
    b: 1.0,
    a: 1.0,
};

/// An element carrying both an `id` and a `class`.
fn el_id(raw: u64, id: &str, class: &str, kids: Vec<ViewNode>) -> ViewNode {
    ViewNode::element(ComponentId::from_raw(raw))
        .with_prop(ViewProperty::new(ID, ViewValue::Text(id.into())))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
        .with_children(ViewFragment::from_nodes(kids))
}

fn assert_color(rt: &UiRuntime, node: NodeId, want: StyleColor, what: &str) {
    let got = color_of(rt, node);
    match got {
        Some(c) => assert!(
            (c.r - want.r).abs() < 1e-3
                && (c.g - want.g).abs() < 1e-3
                && (c.b - want.b).abs() < 1e-3,
            "{what}: expected rgb({}, {}, {}), got rgb({}, {}, {})",
            want.r,
            want.g,
            want.b,
            c.r,
            c.g,
            c.b
        ),
        None => panic!("{what}: the node has no computed color at all"),
    }
}

fn is_red(rt: &UiRuntime, node: NodeId) -> bool {
    color_of(rt, node).is_some_and(|c| c.r > 0.5 && c.g < 0.5 && c.b < 0.5)
}

/// selectors-4 §15: "The specificity of a :where() pseudo-class is replaced by
/// zero."
#[test]
fn where_contributes_zero_specificity() {
    let v = view(vec![el_id(10, "root", "", vec![el(100, "x", vec![])])]);
    let rt = drive(
        VW,
        VW,
        ":where(#root) .x { color: rgb(255,0,0) } \
         .x { color: rgb(0,0,255) }",
        &v,
    );
    assert_color(
        &rt,
        at(&rt, &[0, 0]),
        BLUE,
        "(0,1,0) vs (0,1,0): the later rule wins",
    );
}

/// selectors-4 §15: "The specificity of an :is(), :not(), or :has() pseudo-class
/// is replaced by the specificity of the most specific complex selector in its
/// selector list argument." — with the spec's examples ":is(em, #foo) has a
/// specificity of (1,0,0)" and ":not(em, strong#foo) has a specificity of (1,0,1)".
#[test]
fn is_and_not_take_the_specificity_of_their_most_specific_argument() {
    // (a) `.em` stands in for the type selector `em`; the element matches through
    // that branch, which is exactly the branch a per-branch implementation would
    // (wrongly) score, so the discrimination is preserved.
    let a = view(vec![el(10, "em c", vec![])]);
    let rt_a = drive(
        VW,
        VW,
        ":is(.em, #foo) { color: rgb(255,0,0) } .c { color: rgb(0,0,255) }",
        &a,
    );
    assert_color(&rt_a, at(&rt_a, &[0]), RED, "(1,0,0) beats (0,1,0)");

    // (b) The `:not()` arguments are left verbatim — they exist only to *not*
    // match. `div` becomes `.div`, which makes the competing rule (0,5,0) rather
    // than (0,4,1): a strictly harder bar for the `:not()` rule to clear.
    let b = view(vec![el(10, "div a b c d", vec![])]);
    let rt_b = drive(
        VW,
        VW,
        ":not(em, strong#foo) { color: rgb(255,0,0) } \
         .div.a.b.c.d { color: rgb(0,0,255) }",
        &b,
    );
    assert_color(
        &rt_b,
        at(&rt_b, &[0]),
        RED,
        "(1,0,1) beats a class-only rule",
    );
}

/// selectors-4 §13.3.1: "Note: For these purposes, the list of elements is
/// 1-indexed; that is, the first child of an element has index 1, and will be
/// matched by :nth-child(2n+1), because when n=0 the expression evaluates to 1."
#[test]
fn nth_child_is_one_indexed() {
    // The rule is scoped to the parent's children (`.p > `) and the parent is
    // given a contrasting colour. Without that, the bare `:nth-child(2n+1)` also
    // matches every ancestor up to the root — which is correct CSS — and `color`'s
    // inheritance would redden all five children whatever the indexing base is,
    // so the case could not discriminate at all. The 1-vs-0 indexing claim under
    // test is untouched.
    let kids = (0..5).map(|i| el(100 + i, "k", vec![])).collect();
    let v = view(vec![el(10, "p", kids)]);
    let rt = drive(
        VW,
        VW,
        ".p { color: rgb(0,0,255) } .p > :nth-child(2n+1) { color: rgb(255,0,0) }",
        &v,
    );
    for (i, want_red) in [true, false, true, false, true].into_iter().enumerate() {
        let node = at(&rt, &[0, i]);
        assert_eq!(
            is_red(&rt, node),
            want_red,
            "child {} (1-indexed {}) red?",
            i,
            i + 1
        );
    }
}

/// selectors-4 §13.3.1: "The :nth-child(An+B [of S]? ) pseudo-class notation
/// represents elements that are among An+Bth elements from the list composed of
/// their inclusive siblings that match the selector list S, which is a
/// <complex-real-selector-list>."
///
/// `li` is authored as the class `.li` (see the file note); the structural claim —
/// that S filters the sibling list *before* indexing — is untouched.
#[test]
fn nth_child_of_s_filters_before_counting() {
    let children = || {
        vec![
            el(100, "p", vec![]),
            el(101, "li important", vec![]),
            el(102, "li important", vec![]),
            el(103, "li important", vec![]),
            el(104, "li important", vec![]),
        ]
    };
    let v = || view(vec![el(10, "parent", children())]);

    // (a) `of S`: the first three *matching* siblings — children 2, 3 and 4.
    let a = drive(
        VW,
        VW,
        ":nth-child(-n+3 of .li.important) { color: rgb(255,0,0) }",
        &v(),
    );
    let a_hits: Vec<bool> = (0..5).map(|i| is_red(&a, at(&a, &[0, i]))).collect();
    assert_eq!(
        a_hits,
        vec![false, true, true, true, false],
        "`of S` matches children 2, 3 and 4"
    );

    // (b) The compound form: indexes over *all* siblings, so only 2 and 3.
    let b = drive(
        VW,
        VW,
        ".li.important:nth-child(-n+3) { color: rgb(255,0,0) }",
        &v(),
    );
    let b_hits: Vec<bool> = (0..5).map(|i| is_red(&b, at(&b, &[0, i]))).collect();
    assert_eq!(
        b_hits,
        vec![false, true, true, false, false],
        "the compound form matches only children 2 and 3"
    );
}

/// obscur.kit §4.14: "Normal origins, low to high: 1. system defaults; 2. theme
/// stylesheet; 3. global application CSS; 4. component/module CSS; 5. inline CSS;
/// 6. runtime override." and "Within an origin: 1. specificity; 2. source order."
#[test]
fn origin_outranks_specificity() {
    let v = view(vec![el_id(10, "panel", "", vec![el(100, "btn", vec![])])]);
    let theme = compile_in(CascadeOrigin::Theme, "#panel .btn { color: rgb(255,0,0) }");
    let global = compile_in(CascadeOrigin::GlobalApp, ".btn { color: rgb(0,0,255) }");
    let rt = drive_sheets(VW, VW, &[theme, global], &v);
    assert_color(
        &rt,
        at(&rt, &[0, 0]),
        BLUE,
        "global application CSS outranks a more specific theme rule",
    );
}

/// obscur.kit §4.14 `!important`: "runtime override is an explicit runtime layer
/// above authored CSS, including authored `!important`".
#[test]
fn a_runtime_override_beats_authored_important() {
    let v = view(vec![el(10, "btn", vec![])]);
    let component = compile_in(
        CascadeOrigin::ComponentModule,
        ".btn { color: rgb(255,0,0) !important }",
    );
    let over = compile_in(
        CascadeOrigin::RuntimeOverride,
        ".btn { color: rgb(0,0,255) }",
    );
    let rt = drive_sheets(VW, VW, &[component, over], &v);
    assert_color(
        &rt,
        at(&rt, &[0]),
        BLUE,
        "a non-important runtime override still beats authored !important",
    );
}

/// css-display-3 §3.3: "The element itself does not generate any boxes, but its
/// children and pseudo-elements still generate boxes and text sequences as normal.
/// For the purposes of box generation and layout, the element must be treated as
/// if it had been replaced in the element tree by its contents" and "Note: As only
/// the box tree is affected, any semantics based on the document tree, such as
/// selector-matching, event handling, and property inheritance, are not affected."
#[test]
fn display_contents_affects_the_box_tree_only() {
    let v = view(vec![el_id(
        10,
        "container",
        "",
        vec![el(
            100,
            "c",
            vec![el(200, "gc", vec![]), el(201, "gc", vec![])],
        )],
    )]);
    let sheet = compile(
        "#container { display: flex; width: 300px; height: 40px } \
         .c { display: contents } \
         .gc { flex: 1 1 0px; min-width: 0; height: 10px } \
         #container > .gc { color: rgb(255,0,0) }",
    );
    let rt = drive_sheets(VW, VW, std::slice::from_ref(&sheet), &v);

    // Layout half: the two grandchildren are the flex items.
    let g1 = at(&rt, &[0, 0, 0]);
    let g2 = at(&rt, &[0, 0, 1]);
    let (x1, _, w1, _) = geom(&rt, g1);
    let (x2, _, w2, _) = geom(&rt, g2);
    approx(x1, 0.0, "first grandchild is a flex item at x=0");
    approx(w1, 150.0, "…sharing the 300px main size");
    approx(x2, 150.0, "second grandchild at x=150");
    approx(w2, 150.0, "…sharing the 300px main size");

    // `display: contents` generates no box of its own.
    let contents_node = at(&rt, &[0, 0]);
    assert!(
        rt.layout_boxes().get(&contents_node).is_none(),
        "a `display: contents` element must generate no box"
    );

    // Selector half: the document-tree parent of each grandchild is still `.c`,
    // so the child combinator must not match.
    assert!(
        !is_red(&rt, g1) && !is_red(&rt, g2),
        "`#container > .gc` must not match across a `display: contents` element"
    );
}
