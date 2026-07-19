//! §4.9 Virtualization and collections — spec-derived conformance cases.
//!
//! Expected values come from the specification text quoted on each test. They
//! must not be edited to make a case pass.

mod spec_ch4b_support;

use obscur_kit::identity::{ComponentId, NodeId};
use obscur_kit::layout::virtualization::VirtualLayout;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};
use spec_ch4b_support::harness::{approx, at, compile, el, geom, runtime, txt};

/// A keyed row: the materialized node backing item `key`.
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

/// §4.9: "Virtualized layout supports:" / "- stable item identity;"
///
/// A virtualized list keyed `["b","c","d","e","f"]` has the row for key `"d"`
/// toggled on. Prepending key `"a"` moves `"d"` from index 2 to index 3. The
/// row for `"d"` must still be the toggled one and must be backed by the **same
/// node identity** — recycling keyed by viewport slot or item index would move
/// the toggle to `"c"`.
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

/// §4.9: "Virtualized layout supports:" / "- scroll anchoring;" /
/// "- estimated geometry;"
///
/// 100 items of 50px (content extent 5000px), viewport 500px, scrolled to 2500
/// so item 50 is flush with the viewport top. Growing item 10 (above the
/// viewport, at content y 500..550) from 50px to 150px must keep item 50 flush:
/// the offset becomes 2600, the extent 5100, and the visible items stay 50..59.
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

/// §4.9: "Virtualized layout supports:" / "- materialization;" / "- overscan;"
///
/// 1000 items of 50px, viewport 500px, offset 0. Items 0..9 intersect the
/// viewport. The materialized range must cover them, extend **strictly past**
/// index 9 in the scroll direction (or blank rows pop on every scroll), and
/// stay far below 1000 (no virtualization at all).
#[test]
fn ch4_9_c_materialization_overscans_past_the_visible_range() {
    let layout = VirtualLayout::with_default_extent(50.0);
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
