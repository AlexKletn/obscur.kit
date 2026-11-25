#![doc = include_str!("spec_ch4b_virtualization.doc/_module.md")]

mod spec_ch4b_support;

use obscur_kit::identity::{ComponentId, NodeId};
use obscur_kit::layout::virtualization::{OverscanPolicy, SizeEstimator, VirtualLayout};
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};
use spec_ch4b_support::harness::{approx, at, compile, el, geom, runtime, txt};

#[doc = include_str!("spec_ch4b_virtualization.doc/row.md")]
fn row(key: &str, toggled: bool) -> ViewNode {
    ViewNode::element(ComponentId::from_raw(0x4B_0001))
        .with_key(key)
        .with_prop(ViewProperty::new(
            CLASS,
            ViewValue::Text(if toggled {
                "row on".into()
            } else {
                "row".into()
            }),
        ))
        .with_children(ViewFragment::from_nodes(vec![txt(key)]))
}

#[doc = include_str!("spec_ch4b_virtualization.doc/ch4_9_a_item_identity_is_keyed_not_indexed.md")]
#[test]
fn ch4_9_a_item_identity_is_keyed_not_indexed() {
    // The per-row toggle is observable as geometry: a toggled row is 70px tall.
    let css = ".row { width: 100px; height: 50px; } .on { height: 70px; }";
    let sheet = compile(css);

    let keys_before = ["b", "c", "d", "e", "f"];
    let before = View::from_fragment(ViewFragment::from_nodes(vec![el(
        1,
        "list",
        keys_before
            .iter()
            .map(|k| row(k, *k == "d"))
            .collect::<Vec<_>>(),
    )]));

    let mut rt = runtime(400.0, 400.0);
    let _ = rt.render_now(&before, std::slice::from_ref(&sheet));
    let d_before = at(&rt, &[0, 2]);

    let keys_after = ["a", "b", "c", "d", "e", "f"];
    let after = View::from_fragment(ViewFragment::from_nodes(vec![el(
        1,
        "list",
        keys_after
            .iter()
            .map(|k| row(k, *k == "d"))
            .collect::<Vec<_>>(),
    )]));
    let _ = rt.render_now(&after, std::slice::from_ref(&sheet));

    let d_after = at(&rt, &[0, 3]);
    assert_eq!(
        d_before, d_after,
        "the node backing key \"d\" must survive the prepend unchanged"
    );

    // No other row may have inherited the toggle.
    let toggled: Vec<NodeId> = (0..keys_after.len())
        .map(|i| at(&rt, &[0, i]))
        .filter(|&n| (geom(&rt, n).3 - 70.0).abs() < 1e-2)
        .collect();
    assert_eq!(
        toggled,
        vec![d_after],
        "only key \"d\"'s row may be toggled on after the prepend"
    );
}

#[doc = include_str!("spec_ch4b_virtualization.doc/ch4_9_b_scroll_anchoring_compensates_growth_above_the_viewport.md")]
#[test]
fn ch4_9_b_scroll_anchoring_compensates_growth_above_the_viewport() {
    let mut layout = VirtualLayout::with_default_extent(50.0);
    approx(
        layout.total_extent(100),
        5000.0,
        "the initial content extent",
    );

    let anchor = layout.anchor_at(2500.0, 100);
    assert_eq!(
        anchor.index(),
        50,
        "at offset 2500 item 50 is flush with the viewport top"
    );

    let before = layout.visible_range(2500.0, 500.0, 100);

    layout.estimator_mut().set_measured(10, 150.0);

    approx(
        layout.total_extent(100),
        5100.0,
        "the content extent after item 10 grows by 100px",
    );
    let offset = anchor.resolve(layout.estimator(), 100);
    approx(
        offset,
        2600.0,
        "the anchored scroll offset compensates the 100px growth above the viewport",
    );

    approx(
        offset - layout.estimator().offset_of(50, 100),
        0.0,
        "item 50's viewport-relative y stays 0",
    );

    let after = layout.visible_range(offset, 500.0, 100);
    assert_eq!(
        (before.start(), before.end()),
        (50, 60),
        "the visible items before the change are 50..59"
    );
    assert_eq!(
        (after.start(), after.end()),
        (50, 60),
        "the visible items are still 50..59 — the list must not lurch"
    );
}

#[doc = include_str!("spec_ch4b_virtualization.doc/ch4_9_c_materialization_overscans_past_the_visible_range.md")]
#[test]
fn ch4_9_c_materialization_overscans_past_the_visible_range() {
    // CORRECTED 2026-07-20 by the owner: this built the layout with
    // `with_default_extent`, which is `OverscanPolicy::none` — no buffer — yet
    // asserted a materialization buffer past the visible range. It only "passed"
    // because `visible_range` used to include the item flush with the viewport
    // bottom (a half-open-interval bug now fixed, and pinned by ch4_9_b). §4.9
    // lists overscan as a supported mechanism; to exercise it the layout must
    // actually declare one.
    let layout = VirtualLayout::new(SizeEstimator::new(50.0), OverscanPolicy::symmetric(3));
    let range = layout.visible_range(0.0, 500.0, 1000);

    assert_eq!(range.start(), 0, "the materialized range starts at item 0");
    for i in 0..10 {
        assert!(
            range.contains(i),
            "item {i} intersects the viewport and must be materialized: \
             got {range:?}"
        );
    }
    assert!(
        range.end() > 10,
        "the materialized range must extend past the visible range 0..9: \
         got {range:?}"
    );
    assert!(
        range.len() > 10,
        "more than the ten visible items must be materialized: got {range:?}"
    );
    assert!(
        range.len() < 100,
        "materializing all 1000 items is not virtualization: got {range:?}"
    );
}

#[test]
fn tree_grid_virtualizes_visible_rows_with_stable_identity_and_selection() {
    use std::{num::NonZeroU32, sync::Arc};

    use obscur_kit::bootstrap::CssPipeline;
    use obscur_kit::css::source::CssSource;
    use obscur_kit::identity::InternedSymbol;
    use obscur_kit::widgets::table::Column;
    use obscur_kit::widgets::tree::{TreeGrid, TreeNode};

    let mut grid = TreeGrid::new()
        .column(Column::new("Name"))
        .column(Column::new("Value"))
        .row(TreeNode::new("Parent", 0, true), ["branch"]);
    for i in 1..120 {
        grid = grid.row(
            TreeNode::new(format!("Row {i}"), u16::from(i < 8), false),
            ["value"],
        );
    }
    assert_eq!(grid.logical_row_at(2), Some(8));
    assert!(grid.set_expanded(0, true));
    assert_eq!(grid.logical_row_at(2), Some(1));
    assert_eq!(grid.virtual_slot_of(40), Some(41));
    assert!(grid.select(40));
    let grid = Arc::new(grid);
    let list = InternedSymbol::named("treegrid-test");
    let height = NonZeroU32::new(20).unwrap();

    let mut rt = runtime(200.0, 100.0);
    let mut css = CssPipeline::new();
    css.install(
        &mut rt,
        &[CssSource::anonymous(
            "treegrid { display:block; height:100px; overflow:auto } tr { display:block }",
        )],
    );
    let v = Arc::clone(&grid).virtual_view(list, height);
    let _ = rt.render_now(&v, css.sheets());
    let scroller = at(&rt, &[0]);
    assert_eq!(
        rt.scroll_metrics(scroller).unwrap().content_extent.height,
        2420.0
    );
    let visible = rt.virtual_window(list).unwrap();
    assert!(visible.end < 40);

    assert_eq!(
        rt.handle_scroll(820.0, obscur_kit::geometry::Point::new(10.0, 10.0)),
        Some(scroller)
    );
    assert!(rt.run_due_frame(&v, css.sheets()).is_some());
    assert!(rt.virtual_window(list).unwrap().contains(&41));
    let selected = |rt: &obscur_kit::runtime::UiRuntime| {
        rt.semantic_snapshot()
            .iter()
            .find(|(_, node)| node.state().selected)
            .unwrap()
            .0
    };
    let row = selected(&rt);
    rt.set_scroll_top(scroller, 840.0);
    let _ = rt.render_now(&grid.virtual_view(list, height), css.sheets());
    assert_eq!(selected(&rt), row);
}
