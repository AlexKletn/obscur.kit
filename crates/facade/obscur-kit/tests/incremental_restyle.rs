#![doc = include_str!("incremental_restyle.doc/_module.md")]
#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::rc::Rc;

use obscur_kit::bootstrap::CssPipeline;
use obscur_kit::css::source::CssSource;
use obscur_kit::diagnostics::{
    BudgetViolation, PerformanceCounter, PerformanceSpan, TelemetrySink,
};
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, NodeId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::property::PropertyId;
use obscur_kit::style::selector::PseudoState;
use obscur_kit::ui_ir::name_binding::attr_symbol;
use obscur_kit::ui_macros::Contract;
use obscur_kit::view::component::{
    Component, ComponentContext, ComponentContract, ComponentFactory, ComponentInstance,
    ComponentRegistration, ComponentVersion, ErasedComponent, InstanceId,
};
use obscur_kit::view::view::{
    CLASS, View, ViewFragment, ViewKey, ViewNode, ViewProperty, ViewValue,
};

#[doc = include_str!("incremental_restyle.doc/StyleCounts.md")]
#[derive(Debug, Default)]
struct StyleCounts {
    resolved: Vec<u64>,
}

#[doc = include_str!("incremental_restyle.doc/CountingSink.md")]
struct CountingSink(Rc<RefCell<StyleCounts>>);

impl TelemetrySink for CountingSink {
    fn record_span(&mut self, _span: &PerformanceSpan) {}

    fn record_counter(&mut self, counter: &PerformanceCounter) {
        if counter.label() == "style.resolved" {
            self.0.borrow_mut().resolved.push(counter.value());
        }
    }

    fn record_violation(&mut self, _violation: &BudgetViolation) {}
}

#[doc = include_str!("incremental_restyle.doc/STYLESHEET.md")]
const STYLESHEET: &str = "\
.row { width: 40px; height: 10px; background-color: rgb(0, 0, 255); }\n\
.btn:hover { background-color: rgb(255, 0, 0); }\n\
";

#[doc = include_str!("incremental_restyle.doc/view.md")]
fn view(count: usize) -> View {
    let nodes = (0..count)
        .map(|i| {
            let class = if i + 1 == count { "row btn" } else { "row" };
            ViewNode::element(ComponentId::from_raw(i as u64 + 1))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
        })
        .collect();
    View::from_fragment(ViewFragment::from_nodes(nodes))
}

fn children(runtime: &UiRuntime) -> Vec<NodeId> {
    let root = runtime.tree().root().expect("a root after the first frame");
    runtime.tree().get(root).unwrap().child_nodes().to_vec()
}

fn background(runtime: &UiRuntime, node: NodeId) -> Option<(f32, f32, f32)> {
    runtime
        .computed_style(node)?
        .get(PropertyId::BackgroundColor)?
        .as_color()
        .map(|c| (c.r, c.g, c.b))
}

const NODES: usize = 12;
const RED: (f32, f32, f32) = (1.0, 0.0, 0.0);
const BLUE: (f32, f32, f32) = (0.0, 0.0, 1.0);

#[doc = include_str!("incremental_restyle.doc/settled.md")]
fn settled() -> (
    UiRuntime,
    Vec<NodeId>,
    Rc<RefCell<StyleCounts>>,
    Vec<obscur_kit::style::matching::Stylesheet>,
    View,
) {
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(200.0, 200.0));
    let counts = Rc::new(RefCell::new(StyleCounts::default()));
    runtime.set_telemetry_sink(Box::new(CountingSink(Rc::clone(&counts))));

    let mut css = CssPipeline::new();
    css.install(&mut runtime, &[CssSource::anonymous(STYLESHEET)]);
    let sheets = css.sheets().to_vec();
    let view = view(NODES);

    // Frame 1 builds the tree, so it resolves everything by definition.
    let _ = runtime.render_now(&view, &sheets);
    let nodes = children(&runtime);
    assert_eq!(nodes.len(), NODES);
    (runtime, nodes, counts, sheets, view)
}

#[test]
fn a_hover_resolves_only_the_nodes_it_can_reach() {
    let (mut runtime, nodes, counts, sheets, view) = settled();
    let hovered = *nodes.last().unwrap();

    // The first frame resolved the whole tree: the synthetic host plus every
    // element. That is the number a non-incremental style phase would repeat.
    let full = counts.borrow().resolved[0];
    assert!(
        full > NODES as u64,
        "the first frame resolves every node ({full} >= {NODES} elements + the host)"
    );

    // Hover the LAST sibling: its reach is itself and nothing else.
    runtime.set_pseudo_state(hovered, PseudoState::HOVER);
    let _ = runtime.render_now(&view, &sheets);

    let hover_frame = *counts.borrow().resolved.last().unwrap();
    assert_eq!(
        hover_frame, 1,
        "a hover on one leaf must resolve exactly that one node, not all {full}"
    );

    // ...and the restyle was REAL, not skipped: the rule actually applied.
    assert_eq!(background(&runtime, hovered), Some(RED), "`:hover` applied");
    // Every other node kept the style it already had.
    for &other in &nodes[..NODES - 1] {
        assert_eq!(
            background(&runtime, other),
            Some(BLUE),
            "an unhovered sibling keeps its style"
        );
    }
}

#[test]
fn un_hovering_resolves_the_node_back() {
    let (mut runtime, nodes, counts, sheets, view) = settled();
    let hovered = *nodes.last().unwrap();

    runtime.set_pseudo_state(hovered, PseudoState::HOVER);
    let _ = runtime.render_now(&view, &sheets);
    assert_eq!(background(&runtime, hovered), Some(RED));

    // Clearing the state is as much a change as setting it: the node must be
    // re-matched (it now matches no `:hover` rule) and its stale cascade dropped.
    runtime.clear_pseudo_states();
    let _ = runtime.render_now(&view, &sheets);

    assert_eq!(
        *counts.borrow().resolved.last().unwrap(),
        1,
        "un-hovering resolves the one node too"
    );
    assert_eq!(
        background(&runtime, hovered),
        Some(BLUE),
        "the `:hover` cascade did not survive the hover ending"
    );
}

#[doc = include_str!("incremental_restyle.doc/a_cascade_that_only_hover_produced_is_dropped_when_hover_ends.md")]
#[test]
fn a_cascade_that_only_hover_produced_is_dropped_when_hover_ends() {
    const SHEET: &str = ".solo:hover { background-color: rgb(255, 0, 0); }";
    let view = View::from_fragment(ViewFragment::from_nodes(vec![
        ViewNode::element(ComponentId::from_raw(1))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("solo".into()))),
    ]));

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(200.0, 200.0));
    let mut css = CssPipeline::new();
    css.install(&mut runtime, &[CssSource::anonymous(SHEET)]);
    let sheets = css.sheets().to_vec();

    let _ = runtime.render_now(&view, &sheets);
    let node = children(&runtime)[0];
    // Nothing matches at rest, so this is `background-color`'s initial value.
    let at_rest = background(&runtime, node);
    assert_ne!(at_rest, Some(RED));

    runtime.set_pseudo_state(node, PseudoState::HOVER);
    let _ = runtime.render_now(&view, &sheets);
    assert_eq!(background(&runtime, node), Some(RED));

    runtime.clear_pseudo_states();
    let _ = runtime.render_now(&view, &sheets);
    assert_eq!(
        background(&runtime, node),
        at_rest,
        "the hover-only cascade must be cleared, not left behind"
    );
}

#[test]
fn a_frame_that_changes_no_interaction_state_resolves_nothing() {
    let (mut runtime, nodes, counts, sheets, view) = settled();

    // Same view, same sheets, same viewport, no pseudo change: there is no node
    // whose style can differ, so the style phase has nothing to do.
    let _ = runtime.render_now(&view, &sheets);
    assert_eq!(*counts.borrow().resolved.last().unwrap(), 0);
    // And the retained styles survived the frame that resolved none of them.
    assert_eq!(background(&runtime, nodes[0]), Some(BLUE));
}

#[test]
fn hovering_an_ancestor_resolves_its_subtree_and_nothing_before_it() {
    // `.a:hover .leaf` — a descendant combinator, so hovering the parent DOES
    // change a child's style. The invalidation set must contain the subtree, or
    // this renders wrong rather than merely slowly.
    const SHEET: &str = "\
.item { width: 10px; height: 10px; }\n\
.leaf { background-color: rgb(0, 0, 255); }\n\
.a:hover .leaf { background-color: rgb(255, 0, 0); }\n\
";
    // <div.item.a><div.item.leaf/></div><div.item.a2><div.item.leaf/></div>
    let branch = |id: u64, class: &str| {
        ViewNode::element(ComponentId::from_raw(id))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
            .with_children(ViewFragment::from_nodes(vec![
                ViewNode::element(ComponentId::from_raw(id + 100)).with_prop(ViewProperty::new(
                    CLASS,
                    ViewValue::Text("item leaf".into()),
                )),
            ]))
    };
    let view = View::from_fragment(ViewFragment::from_nodes(vec![
        branch(1, "item a"),
        branch(2, "item b"),
    ]));

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(200.0, 200.0));
    let counts = Rc::new(RefCell::new(StyleCounts::default()));
    runtime.set_telemetry_sink(Box::new(CountingSink(Rc::clone(&counts))));
    let mut css = CssPipeline::new();
    css.install(&mut runtime, &[CssSource::anonymous(SHEET)]);
    let sheets = css.sheets().to_vec();

    let _ = runtime.render_now(&view, &sheets);
    let tops = children(&runtime);
    let (a, b) = (tops[0], tops[1]);
    let a_leaf = runtime.tree().get(a).unwrap().child_nodes()[0];
    let b_leaf = runtime.tree().get(b).unwrap().child_nodes()[0];

    // Hover `.a`. Its reach is `.a` + `.a`'s leaf, plus the following sibling
    // `.b` and its leaf (a `.a:hover + .b` rule would be legal). That is 4 of the
    // 5 nodes — everything except the synthetic host, which no selector can reach
    // from below.
    runtime.set_pseudo_state(a, PseudoState::HOVER);
    let _ = runtime.render_now(&view, &sheets);
    assert_eq!(
        *counts.borrow().resolved.last().unwrap(),
        4,
        "the hovered node's subtree and its following siblings' subtrees"
    );

    // The descendant rule bit: correctness, not just work-avoidance.
    assert_eq!(background(&runtime, a_leaf), Some(RED));
    assert_eq!(background(&runtime, b_leaf), Some(BLUE));

    // Now hover the SECOND branch: nothing precedes-sibling its way to it, so
    // `.a` and `.a`'s leaf are provably unaffected and are not resolved.
    runtime.clear_pseudo_states();
    let _ = runtime.render_now(&view, &sheets);
    runtime.set_pseudo_state(b, PseudoState::HOVER);
    let _ = runtime.render_now(&view, &sheets);
    assert_eq!(
        *counts.borrow().resolved.last().unwrap(),
        2,
        "hovering the last branch cannot reach the first one"
    );
    assert_eq!(background(&runtime, a_leaf), Some(BLUE));
}

#[doc = include_str!("incremental_restyle.doc/a_viewport_change_still_restyles_everything.md")]
#[test]
fn a_viewport_change_still_restyles_everything() {
    let (mut runtime, _nodes, counts, sheets, view) = settled();
    let full = counts.borrow().resolved[0];

    runtime.set_viewport(Size::new(100.0, 100.0));
    let _ = runtime.render_now(&view, &sheets);
    assert_eq!(
        *counts.borrow().resolved.last().unwrap(),
        full,
        "a changed viewport is not a bounded scope"
    );
}

#[doc = include_str!("incremental_restyle.doc/a_stylesheet_change_still_restyles_everything.md")]
#[test]
fn a_stylesheet_change_still_restyles_everything() {
    let (mut runtime, nodes, counts, sheets, view) = settled();
    let full = counts.borrow().resolved[0];

    let mut css = CssPipeline::new();
    css.install(
        &mut runtime,
        &[CssSource::anonymous(
            ".row { width: 40px; height: 10px; background-color: rgb(0, 255, 0); }",
        )],
    );
    let _ = runtime.render_now(&view, css.sheets());

    assert_eq!(
        *counts.borrow().resolved.last().unwrap(),
        full,
        "new rules are not a bounded scope"
    );
    assert_eq!(
        background(&runtime, nodes[0]),
        Some((0.0, 1.0, 0.0)),
        "the new sheet reached every node"
    );
    let _ = sheets;
}

// ---- A tree made of components, which is the shape a real application has ----

#[doc = include_str!("incremental_restyle.doc/KeyProps.md")]
#[derive(Contract)]
#[contract(name = "RestyleKey")]
struct KeyProps {
    #[doc = include_str!("incremental_restyle.doc/pressed.md")]
    pressed: i64,
}

#[doc = include_str!("incremental_restyle.doc/Key.md")]
struct Key;

impl Component for Key {
    type Props = KeyProps;

    fn render(&mut self, _ctx: &mut ComponentContext<'_>, props: &Self::Props) -> View {
        let face = ViewNode::element(ComponentId::from_raw(900))
            .with_prop(ViewProperty::new(
                CLASS,
                ViewValue::Text(if props.pressed == 0 { "row" } else { "row btn" }.into()),
            ))
            .with_children(ViewFragment::from_nodes(vec![ViewNode::text("k")]));
        View::from_fragment(ViewFragment::from_nodes(vec![face]))
    }
}

#[doc = include_str!("incremental_restyle.doc/KeyFactory.md")]
#[derive(Debug)]
struct KeyFactory;

impl ComponentFactory for KeyFactory {
    fn component_id(&self) -> ComponentId {
        KeyProps::contract().id
    }
    fn contract(&self) -> ComponentContract {
        KeyProps::contract()
    }
    fn create_instance(&self, instance: InstanceId) -> ComponentInstance {
        ComponentInstance::new(instance, KeyProps::contract().id)
    }
    fn create_component(&self) -> Box<dyn ErasedComponent> {
        Box::new(Key)
    }
}

#[doc = include_str!("incremental_restyle.doc/keyboard.md")]
fn keyboard(pressed: usize) -> View {
    let nodes = (0..NODES)
        .map(|i| {
            ViewNode::element(KeyProps::contract().id).with_prop(ViewProperty::new(
                attr_symbol("pressed"),
                ViewValue::Int(i64::from(i == pressed)),
            ))
        })
        .collect();
    View::from_fragment(ViewFragment::from_nodes(nodes))
}

#[test]
fn pressing_one_component_backed_key_does_not_restyle_the_whole_tree() {
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(200.0, 200.0));
    let counts = Rc::new(RefCell::new(StyleCounts::default()));
    runtime.set_telemetry_sink(Box::new(CountingSink(Rc::clone(&counts))));
    runtime
        .register_component(ComponentRegistration::new(
            Box::new(KeyFactory),
            ComponentVersion::INITIAL,
        ))
        .expect("the key component registers");

    let mut css = CssPipeline::new();
    css.install(&mut runtime, &[CssSource::anonymous(STYLESHEET)]);
    let sheets = css.sheets().to_vec();

    // `NODES` presses nothing, so this is the settled tree.
    let settled = keyboard(NODES);
    let _ = runtime.render_now(&settled, &sheets);
    while runtime.run_due_frame(&settled, &sheets).is_some() {}
    let total = runtime.tree().iter().count() as u64;
    counts.borrow_mut().resolved.clear();

    // The LAST key, so nothing follows it and the dirty set is honestly one key:
    // a restyle root takes the node's subtree and what comes after it.
    let pressed = keyboard(NODES - 1);
    let _ = runtime.render_now(&pressed, &sheets);
    while runtime.run_due_frame(&pressed, &sheets).is_some() {}

    let resolved = counts
        .borrow()
        .resolved
        .iter()
        .copied()
        .max()
        .unwrap_or_default();
    assert!(
        resolved > 0,
        "the press has to restyle something, or this test measures nothing"
    );
    assert!(
        resolved * 3 < total,
        "one component re-rendered, so the restyle must stay inside its subtree: \
         resolved {resolved} of {total}. A component sub-reconcile used to answer \
         `roots = None`, which made every interactive frame of an application \
         built out of components pay the full pass."
    );
}

#[doc = include_str!("incremental_restyle.doc/window_view.md")]
fn window_view(first: usize, width: usize) -> View {
    let nodes = (first..first + width)
        .map(|i| {
            ViewNode::element(ComponentId::from_raw(1))
                .with_key(ViewKey::from(i.to_string()))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("row".into())))
        })
        .collect();
    View::from_fragment(ViewFragment::from_nodes(nodes))
}

#[doc = include_str!("incremental_restyle.doc/a_departed_node_keeps_no_computed_style.md")]
#[test]
fn a_departed_node_keeps_no_computed_style() {
    const WIDTH: usize = 30;
    const FRAMES: usize = 200;

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(200.0, 200.0));
    let mut css = CssPipeline::new();
    css.install(&mut runtime, &[CssSource::anonymous(STYLESHEET)]);
    let sheets = css.sheets().to_vec();

    let mut seen: std::collections::BTreeSet<NodeId> = std::collections::BTreeSet::new();
    for frame in 0..FRAMES {
        let _ = runtime.render_now(&window_view(frame, WIDTH), &sheets);
        seen.extend(children(&runtime));
    }

    let live = children(&runtime);
    let dead: Vec<NodeId> = seen
        .into_iter()
        .filter(|id| !live.contains(id))
        .filter(|id| runtime.tree().get(*id).is_none())
        .collect();
    let still_styled = dead
        .iter()
        .filter(|id| runtime.computed_style(**id).is_some())
        .count();

    println!(
        "[probe] live={} tree.len()={} dead={} still_styled={}",
        live.len(),
        runtime.tree().len(),
        dead.len(),
        still_styled
    );
    assert_eq!(
        still_styled, 0,
        "a node the reconcile removed must not keep its computed style"
    );
}

#[doc = include_str!("incremental_restyle.doc/rss_kb.md")]
fn rss_kb() -> u64 {
    std::fs::read_to_string("/proc/self/statm")
        .ok()
        .and_then(|s| {
            s.split_whitespace()
                .nth(1)
                .and_then(|p| p.parse::<u64>().ok())
        })
        .map_or(0, |pages| pages * 4)
}

#[doc = include_str!("incremental_restyle.doc/a_sliding_window_costs_what_the_window_costs.md")]
#[test]
#[ignore = "a measurement, not an oracle: run with --release --ignored"]
fn a_sliding_window_costs_what_the_window_costs() {
    const WIDTH: usize = 30;
    const FRAMES: usize = 3000;

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(200.0, 200.0));
    let mut css = CssPipeline::new();
    css.install(&mut runtime, &[CssSource::anonymous(STYLESHEET)]);
    let sheets = css.sheets().to_vec();

    let _ = runtime.render_now(&window_view(0, WIDTH), &sheets);
    let base_rss = rss_kb();
    let mut early = std::time::Duration::ZERO;

    for frame in 1..=FRAMES {
        let t = std::time::Instant::now();
        let _ = runtime.render_now(&window_view(frame, WIDTH), &sheets);
        let dt = t.elapsed();
        if frame == 20 {
            early = dt;
        }
        if frame % 500 == 0 {
            println!(
                "[churn] frame {frame:>4}  rss +{:>6} kB  frame {:>7.3} ms  tree.len()={}",
                rss_kb().saturating_sub(base_rss),
                dt.as_secs_f64() * 1000.0,
                runtime.tree().len()
            );
        }
    }
    println!(
        "[churn] frame 20 = {:.3} ms; growth = {:.2} kB/frame",
        early.as_secs_f64() * 1000.0,
        (rss_kb().saturating_sub(base_rss)) as f64 / FRAMES as f64
    );
}

#[test]
fn child_edits_reuse_styles_only_when_selectors_allow_it() {
    let keyed_view = |keys: &[u64]| {
        View::from_fragment(ViewFragment::from_nodes(
            keys.iter()
                .map(|&key| {
                    ViewNode::element(ComponentId::from_raw(1))
                        .with_key(ViewKey::from(key))
                        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("row".into())))
                })
                .collect(),
        ))
    };
    let structural_css = ".row:not(:nth-last-child(odd)) { background-color: red; }";
    for structural in [false, true] {
        let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(200.0, 200.0));
        let counts = Rc::new(RefCell::new(StyleCounts::default()));
        runtime.set_telemetry_sink(Box::new(CountingSink(Rc::clone(&counts))));
        let source = if structural {
            format!("{STYLESHEET}\n{structural_css}")
        } else {
            STYLESHEET.to_owned()
        };
        let mut css = CssPipeline::new();
        css.install(&mut runtime, &[CssSource::anonymous(source)]);
        let sheets = css.sheets();

        let _ = runtime.render_now(&keyed_view(&[1, 2, 3, 4]), sheets);
        for (keys, narrow_count) in [
            (&[1, 5, 2, 3, 4][..], Some(1)),
            (&[1, 5, 2, 4][..], Some(0)),
            (&[5, 1, 2, 4][..], None),
        ] {
            counts.borrow_mut().resolved.clear();
            let view = keyed_view(keys);
            let _ = runtime.render_now(&view, sheets);
            let actual: Vec<_> = children(&runtime)
                .into_iter()
                .map(|node| background(&runtime, node))
                .collect();

            let mut fresh = UiRuntime::new(RuntimeId::from_raw(2), Size::new(200.0, 200.0));
            let _ = fresh.render_now(&view, sheets);
            let expected: Vec<_> = children(&fresh)
                .into_iter()
                .map(|node| background(&fresh, node))
                .collect();
            assert_eq!(actual, expected);
            if !structural && let Some(count) = narrow_count {
                assert_eq!(
                    counts.borrow().resolved.iter().sum::<u64>(),
                    count,
                    "existing siblings should retain their styles"
                );
            }
        }
    }
}

#[test]
fn a_sibling_rule_only_widens_child_edits_where_its_target_can_match() {
    let leaf = |key: u64, class: &str| {
        ViewNode::element(ComponentId::from_raw(1))
            .with_key(ViewKey::from(key))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
    };
    let branch = |key: u64, children| {
        ViewNode::element(ComponentId::from_raw(2))
            .with_key(ViewKey::from(key))
            .with_children(ViewFragment::from_nodes(children))
    };
    let view = |insert_elsewhere, insert_between| {
        let mut unrelated = vec![leaf(10, "row"), leaf(11, "row")];
        if insert_elsewhere {
            unrelated.insert(0, leaf(12, "row"));
        }
        let mut related = vec![
            leaf(23, "row"),
            leaf(24, "row"),
            leaf(20, "lead"),
            leaf(21, "row target"),
        ];
        if insert_between {
            related.insert(3, leaf(22, "row"));
        }
        View::from_fragment(ViewFragment::from_nodes(vec![
            branch(1, unrelated),
            branch(2, related),
        ]))
    };
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(200.0, 200.0));
    let counts = Rc::new(RefCell::new(StyleCounts::default()));
    runtime.set_telemetry_sink(Box::new(CountingSink(Rc::clone(&counts))));
    let mut css = CssPipeline::new();
    css.install(
        &mut runtime,
        &[CssSource::anonymous(format!(
            "{STYLESHEET}\n.lead + .target {{ background-color: red; }}\n.row:not(:indeterminate) {{ opacity: 1; }}"
        ))],
    );
    let sheets = css.sheets();
    let target_color = |runtime: &UiRuntime| {
        let related = children(runtime)[1];
        let target = *runtime
            .tree()
            .get(related)
            .unwrap()
            .child_nodes()
            .last()
            .unwrap();
        background(runtime, target)
    };
    let _ = runtime.render_now(&view(false, false), sheets);
    assert_eq!(target_color(&runtime), Some(RED));

    counts.borrow_mut().resolved.clear();
    let _ = runtime.render_now(&view(true, false), sheets);
    assert_eq!(counts.borrow().resolved.iter().sum::<u64>(), 1);
    assert_eq!(target_color(&runtime), Some(RED));

    for insert_between in [true, false] {
        let view = view(true, insert_between);
        counts.borrow_mut().resolved.clear();
        let _ = runtime.render_now(&view, sheets);
        assert_eq!(
            counts.borrow().resolved.iter().sum::<u64>(),
            if insert_between { 2 } else { 1 }
        );
        let mut fresh = UiRuntime::new(RuntimeId::from_raw(2), Size::new(200.0, 200.0));
        let _ = fresh.render_now(&view, sheets);
        assert_eq!(target_color(&runtime), target_color(&fresh));
        assert_eq!(
            target_color(&runtime),
            Some(if insert_between { BLUE } else { RED })
        );
    }
}

#[doc = include_str!("incremental_restyle.doc/focus_within_follows_the_focused_branch.md")]
#[test]
fn focus_within_follows_the_focused_branch() {
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(200.0, 200.0));
    let mut css = CssPipeline::new();
    css.install(
        &mut runtime,
        &[CssSource::anonymous(
            ".branch { background-color: blue; }
             .branch:focus-within { background-color: red; }",
        )],
    );
    let sheets = css.sheets().to_vec();
    let branch = |id, has_child| {
        let children = if has_child {
            vec![ViewNode::element(ComponentId::from_raw(id + 10))]
        } else {
            Vec::new()
        };
        ViewNode::element(ComponentId::from_raw(id))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("branch".into())))
            .with_children(ViewFragment::from_nodes(children))
    };
    let view = View::from_fragment(ViewFragment::from_nodes(vec![
        branch(1, true),
        branch(2, true),
    ]));
    let _ = runtime.render_now(&view, &sheets);
    let branches = children(&runtime);
    let leaf = |runtime: &UiRuntime, branch| runtime.tree().get(branch).unwrap().child_nodes()[0];
    let left = leaf(&runtime, branches[0]);
    let right = leaf(&runtime, branches[1]);

    for (focused, highlighted, other) in [
        (left, branches[0], branches[1]),
        (right, branches[1], branches[0]),
    ] {
        runtime.focus_node(Some(focused));
        let _ = runtime.render_now(&view, &sheets);
        assert!(
            runtime
                .pseudo_state(focused)
                .contains(PseudoState::FOCUS_WITHIN)
        );
        assert!(
            !runtime
                .pseudo_state(highlighted)
                .contains(PseudoState::FOCUS)
        );
        assert_eq!(background(&runtime, highlighted), Some((1.0, 0.0, 0.0)));
        assert_eq!(background(&runtime, other), Some((0.0, 0.0, 1.0)));
    }

    let without_right = View::from_fragment(ViewFragment::from_nodes(vec![
        branch(1, true),
        branch(2, false),
    ]));
    let _ = runtime.render_now(&without_right, &sheets);
    assert_eq!(runtime.focused(), None);
    assert_eq!(background(&runtime, branches[1]), Some((0.0, 0.0, 1.0)));
}

#[doc = include_str!("incremental_restyle.doc/hover_follows_a_stationary_cursor.md")]
#[test]
fn hover_follows_a_stationary_cursor() {
    use obscur_kit::geometry::Point;
    use obscur_kit::interaction::pointer::{
        PointerDevice, PointerEvent, PointerEventKind, PointerId,
    };

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(200.0, 300.0));
    let mut css = CssPipeline::new();
    css.install(
        &mut runtime,
        &[CssSource::anonymous(
            ".row { width: 100px; height: 50px; background-color: blue }
             .row:hover { background-color: red }
             .tall { height: 150px }
             .port { width: 100px; height: 150px; overflow: auto }",
        )],
    );
    let sheets = css.sheets().to_vec();
    let rows = |count, tall| {
        (0..count)
            .map(|index| {
                ViewNode::element(ComponentId::from_raw(10 + index)).with_prop(ViewProperty::new(
                    CLASS,
                    ViewValue::Text(
                        if tall && index == 0 {
                            "row tall"
                        } else {
                            "row"
                        }
                        .into(),
                    ),
                ))
            })
            .collect()
    };
    let point = Point::new(10.0, 120.0);
    let move_cursor = |runtime: &mut UiRuntime| {
        runtime.handle_pointer(PointerEvent::new(
            PointerId::MOUSE,
            PointerDevice::Mouse,
            PointerEventKind::Move,
            point,
        ));
    };
    let initial = View::from_fragment(ViewFragment::from_nodes(rows(3, false)));
    let _ = runtime.render_now(&initial, &sheets);
    let nodes = children(&runtime);
    move_cursor(&mut runtime);
    assert_eq!(runtime.hovered(), Some(nodes[2]));

    let grown = View::from_fragment(ViewFragment::from_nodes(rows(3, true)));
    let frame = runtime.render_now(&grown, &sheets);
    assert_eq!(runtime.hovered(), Some(nodes[0]));
    use obscur_kit::scene::command::PaintCommand;
    use obscur_kit::scene::paint::{Brush, Paint};
    let red: Vec<_> = frame
        .snapshot()
        .display_list()
        .commands()
        .iter()
        .filter_map(|command| match command {
            PaintCommand::DrawPath {
                path,
                paint: Paint::Fill(Brush::Solid(brush)),
                ..
            } if brush.color().r == 1.0 => path.bounds(),
            _ => None,
        })
        .collect();
    assert_eq!(red.len(), 1);
    assert_eq!(red[0].origin.y, 0.0);
    assert_eq!(red[0].size.height, 150.0);
    assert_eq!(background(&runtime, nodes[0]), Some((1.0, 0.0, 0.0)));
    assert_eq!(background(&runtime, nodes[2]), Some((0.0, 0.0, 1.0)));

    let empty = View::from_fragment(ViewFragment::from_nodes(Vec::new()));
    let _ = runtime.render_now(&empty, &sheets);
    assert_eq!(runtime.hovered(), runtime.tree().root());

    let scrollable = View::single(
        ViewNode::element(ComponentId::from_raw(1))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("port".into())))
            .with_children(ViewFragment::from_nodes(rows(7, false))),
    );
    let _ = runtime.render_now(&scrollable, &sheets);
    let port = children(&runtime)[0];
    let rows = runtime.tree().get(port).unwrap().child_nodes().to_vec();
    move_cursor(&mut runtime);
    assert_eq!(runtime.hovered(), Some(rows[2]));
    assert_eq!(runtime.handle_scroll(100.0, point), Some(port));
    assert_eq!(runtime.hovered(), Some(rows[4]));
    let _ = runtime.run_due_frame(&scrollable, &sheets);
    assert_eq!(background(&runtime, rows[4]), Some((1.0, 0.0, 0.0)));
}

#[doc = include_str!("incremental_restyle.doc/animated_transform_updates_hover_before_paint.md")]
#[test]
fn animated_transform_updates_hover_before_paint() {
    use obscur_kit::geometry::Point;
    use obscur_kit::interaction::pointer::{
        PointerDevice, PointerEvent, PointerEventKind, PointerId,
    };
    use obscur_kit::scene::command::PaintCommand;
    use obscur_kit::scene::paint::{Brush, Paint};

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(200.0, 100.0));
    let mut css = CssPipeline::new();
    css.install(
        &mut runtime,
        &[CssSource::anonymous(
            ".moving { width: 30px; height: 30px; background-color: blue;
                       animation: slide 1s linear both }
             .moving:hover { background-color: red }
             @keyframes slide {
                 from { transform: translateX(0px) }
                 to { transform: translateX(120px) }
             }",
        )],
    );
    let view = View::single(
        ViewNode::element(ComponentId::from_raw(42))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("moving".into()))),
    );
    let _ = runtime.render_now(&view, css.sheets());
    let node = children(&runtime)[0];
    runtime.handle_pointer(PointerEvent::new(
        PointerId::MOUSE,
        PointerDevice::Mouse,
        PointerEventKind::Move,
        Point::new(70.0, 10.0),
    ));
    assert_ne!(runtime.hovered(), Some(node));

    for (red, expected) in [(true, Some(node)), (false, runtime.tree().root())] {
        runtime.advance_motion(0.5);
        let frame = runtime.render_now(&view, css.sheets());
        assert_eq!(runtime.hovered(), expected);
        assert!(!frame.relaid_out());
        assert_eq!(
            frame
                .snapshot()
                .display_list()
                .commands()
                .iter()
                .any(|command| {
                    matches!(command, PaintCommand::DrawPath {
                    paint: Paint::Fill(Brush::Solid(brush)), ..
                } if brush.color().r == 1.0)
                }),
            red,
        );
    }
}
