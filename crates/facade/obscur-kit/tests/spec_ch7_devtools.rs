#![doc = include_str!("spec_ch7_devtools.doc/_module.md")]
#![forbid(unsafe_code)]

#[path = "spec_ch7_support/mod.rs"]
mod support;

use obscur_kit::view::view::View;
use support::{all_geom, at, click, drive, el, hover, view};

const VW: f32 = 400.0;

#[doc = include_str!("spec_ch7_devtools.doc/rotated.md")]
fn rotated() -> View {
    view(vec![el(1, "root", vec![el(2, "box", vec![])])])
}

const ROTATED_CSS: &str = ".root { width: 400px; height: 400px } \
                           .box { width: 100px; height: 40px; \
                                  transform: rotate(45deg); \
                                  background-color: #cc3333 }";

#[doc = include_str!("spec_ch7_devtools.doc/inspect_picking_uses_the_runtime_hit_test_not_the_layout_rect.md")]
#[test]
fn inspect_picking_uses_the_runtime_hit_test_not_the_layout_rect() {
    let mut rt = drive(VW, VW, ROTATED_CSS, &rotated());
    let box_node = at(&rt, &[0, 0]);
    rt.set_inspect_mode(true);

    // Q: the centre of the 100x40 box, which rotation about the centre leaves
    // inside the shape.
    hover(&mut rt, 50.0, 20.0);
    assert_eq!(
        rt.hovered(),
        Some(box_node),
        "Q is inside the rotated shape: the runtime hit-test finds the box"
    );
    assert_eq!(
        rt.hovered_inspect().map(|i| i.node),
        Some(box_node),
        "Q: inspect picks the same node the runtime hit-test did"
    );

    // P: inside the axis-aligned bounding box of the rotated shape (which spans
    // roughly x in 0.5..99.5, y in -29.5..69.5) but outside the rotated shape
    // itself — the bottom-left corner region the rotation vacated.
    hover(&mut rt, 5.0, 65.0);
    let runtime_pick = rt.hovered();
    let inspect_pick = rt.hovered_inspect().map(|i| i.node);
    assert_ne!(
        runtime_pick,
        Some(box_node),
        "P is outside the rotated shape: the runtime hit-test must miss the box"
    );
    assert_eq!(
        inspect_pick, runtime_pick,
        "P: inspect returns the same underlying node the runtime hit-test does"
    );

    // P is outside the UNTRANSFORMED 100x40 rect as well, so it does not
    // distinguish a hit test that honours the transform from one that ignores it.
    // R does: it is inside the layout rect, and only inverse-rotating it (to
    // local (3.3, 41.2), past the 40px bottom edge) shows it is off the shape.
    hover(&mut rt, 2.0, 2.0);
    let runtime_pick = rt.hovered();
    assert_ne!(
        runtime_pick,
        Some(box_node),
        "R is inside the box's LAYOUT rect but the rotation vacated it: the \
         runtime hit-test must miss the box"
    );
    assert_eq!(
        rt.hovered_inspect().map(|i| i.node),
        runtime_pick,
        "R: inspect returns the same underlying node the runtime hit-test does"
    );

    // S: the other side of the same claim — local (95, 5) is drawn at
    // (92.4, 41.2), BELOW the layout rect, and must be pickable there.
    hover(&mut rt, 92.43, 41.21);
    assert_eq!(
        rt.hovered(),
        Some(box_node),
        "S is inside the rotated shape though outside the layout rect: the \
         runtime hit-test must find the box"
    );
    assert_eq!(
        rt.hovered_inspect().map(|i| i.node),
        Some(box_node),
        "S: inspect picks the same node the runtime hit-test did"
    );
}

#[doc = include_str!("spec_ch7_devtools.doc/inspect_picking_honours_stacking_order.md")]
#[test]
fn inspect_picking_honours_stacking_order() {
    let css = ".root { width: 400px; height: 400px; position: relative } \
               .button { position: absolute; left: 0; top: 0; \
                         width: 100px; height: 40px } \
               .overlay { position: absolute; left: 0; top: 0; \
                          width: 200px; height: 100px; background-color: #000000 }";
    let v = view(vec![el(
        1,
        "root",
        vec![el(2, "button", vec![]), el(3, "overlay", vec![])],
    )]);

    let mut rt = drive(VW, VW, css, &v);
    let overlay = at(&rt, &[0, 1]);
    rt.set_inspect_mode(true);

    hover(&mut rt, 50.0, 20.0);
    assert_eq!(
        rt.hovered(),
        Some(overlay),
        "the runtime hit-test returns the overlay, not the button beneath it"
    );
    assert_eq!(
        rt.hovered_inspect().map(|i| i.node),
        Some(overlay),
        "inspect returns the overlay too"
    );
}

#[doc = include_str!("spec_ch7_devtools.doc/the_inspect_highlight_is_inert.md")]
#[test]
fn the_inspect_highlight_is_inert() {
    let css = ".root { width: 400px; height: 400px } \
               .button { width: 100px; height: 40px; background-color: #336699 }";
    let v = view(vec![el(1, "root", vec![el(2, "button", vec![])])]);

    // Baseline: no highlight.
    let mut plain = drive(VW, VW, css, &v);
    hover(&mut plain, 50.0, 20.0);
    click(&mut plain, 50.0, 20.0);
    let plain_hover = plain.hovered();
    let plain_focus = plain.focused();
    let plain_geom = all_geom(&plain);
    let plain_a11y = plain.semantic_snapshot().len();

    // The same run with inspect highlighting enabled on the button.
    let mut lit = drive(VW, VW, css, &v);
    lit.set_inspect_mode(true);
    hover(&mut lit, 50.0, 20.0);
    assert!(
        lit.hovered_inspect().is_some(),
        "the highlight is genuinely on — otherwise this proves nothing"
    );
    click(&mut lit, 50.0, 20.0);

    assert_eq!(
        lit.hovered(),
        plain_hover,
        "the highlight swallows no pointer: the same node is hit"
    );
    assert_eq!(
        lit.focused(),
        plain_focus,
        "the highlight changes no focus outcome"
    );
    assert_eq!(
        all_geom(&lit),
        plain_geom,
        "the highlight contributes no layout"
    );
    assert_eq!(
        lit.semantic_snapshot().len(),
        plain_a11y,
        "the highlight contributes no accessibility node"
    );
}
