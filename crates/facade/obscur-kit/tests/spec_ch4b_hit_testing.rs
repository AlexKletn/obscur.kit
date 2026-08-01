#![doc = include_str!("spec_ch4b_hit_testing.doc/_module.md")]

mod spec_ch4b_support;

use obscur_kit::primitives::Portal;
use spec_ch4b_support::harness::{at, drive, el, hit, hit_chain, txt, view};

#[doc = include_str!("spec_ch4b_hit_testing.doc/ch4_7_a_the_hit_chain_is_the_logical_tree_not_the_portal_target.md")]
#[test]
fn ch4_7_a_the_hit_chain_is_the_logical_tree_not_the_portal_target() {
    let css = "
        .root { width: 800px; height: 600px; }
        .panel-a { width: 800px; height: 100px; }
        .trigger { width: 100px; height: 100px; }
        .overlay { position: absolute; left: 0px; top: 0px;
                   width: 800px; height: 600px; }
        .p { position: absolute; left: 400px; top: 300px;
             width: 50px; height: 50px; }
    ";
    let portal = Portal::new("overlay")
        .child(el(5, "p", vec![txt("p")]))
        .into_view_node();
    let v = view(vec![el(
        1,
        "root",
        vec![
            el(2, "panel-a", vec![el(3, "trigger", vec![portal])]),
            el(4, "overlay", vec![]),
        ],
    )]);

    let mut rt = drive(800.0, 600.0, css, &v);
    let root = at(&rt, &[0]);
    let panel_a = at(&rt, &[0, 0]);
    let trigger = at(&rt, &[0, 0, 0]);
    let overlay = at(&rt, &[0, 1]);

    let chain = hit_chain(&mut rt, 420.0, 320.0);
    assert!(
        !chain.is_empty(),
        "the portal content P must be hit at (420, 320)"
    );
    let index_of = |n| {
        chain
            .iter()
            .position(|&c| c == n)
            .unwrap_or_else(|| panic!("expected {n:?} in the hit chain {chain:?}"))
    };
    // P's logical ancestors, in order: [P, ..., Trigger, PanelA, Root].
    let (i_trigger, i_panel, i_root) = (index_of(trigger), index_of(panel_a), index_of(root));
    assert!(
        0 < i_trigger && i_trigger < i_panel && i_panel < i_root,
        "the hit chain must run P -> Trigger -> PanelA -> Root: got {chain:?}"
    );
    assert!(
        !chain.contains(&overlay),
        "the hit chain must be logical, not the visual [P, Overlay, Root]: \
         got {chain:?}"
    );
}

#[doc = include_str!("spec_ch4b_hit_testing.doc/ch4_7_b_a_rotated_box_is_missed_inside_its_bounding_box.md")]
#[test]
fn ch4_7_b_a_rotated_box_is_missed_inside_its_bounding_box() {
    let css = "
        .box { position: absolute; left: 0px; top: 0px;
               width: 100px; height: 40px; transform: rotate(45deg); }
    ";
    let v = view(vec![el(1, "box", vec![])]);
    let mut rt = drive(800.0, 600.0, css, &v);
    let boxed = at(&rt, &[0]);

    assert_eq!(
        hit(&mut rt, 50.0, 20.0),
        Some(boxed),
        "the rotated box's centre must hit it"
    );
    let chain = hit_chain(&mut rt, 2.0, -28.0);
    assert!(
        !chain.contains(&boxed),
        "(2, -28) is inside the rotated box's AABB but outside the box itself, \
         so it must miss: got chain {chain:?}"
    );
}

#[doc = include_str!("spec_ch4b_hit_testing.doc/ch4_7_c_a_clipped_out_child_is_not_hit.md")]
#[test]
fn ch4_7_c_a_clipped_out_child_is_not_hit() {
    let css = "
        .parent { position: absolute; left: 0px; top: 0px;
                  width: 100px; height: 100px; overflow: hidden; }
        .child { width: 300px; height: 300px; }
    ";
    let v = view(vec![el(1, "parent", vec![el(2, "child", vec![])])]);
    let mut rt = drive(800.0, 600.0, css, &v);
    let parent = at(&rt, &[0]);
    let child = at(&rt, &[0, 0]);

    let chain = hit_chain(&mut rt, 150.0, 50.0);
    assert!(
        !chain.contains(&child),
        "the child is clipped away at (150, 50): got chain {chain:?}"
    );
    assert!(
        !chain.contains(&parent),
        "the parent's own box ends at x=100, so (150, 50) is outside it too: \
         got chain {chain:?}"
    );
}

#[doc = include_str!("spec_ch4b_hit_testing.doc/ch4_7_d_z_index_beats_document_order.md")]
#[test]
fn ch4_7_d_z_index_beats_document_order() {
    let css = "
        .a { position: absolute; left: 0px; top: 0px;
             width: 100px; height: 100px; z-index: 2; }
        .b { position: absolute; left: 0px; top: 0px;
             width: 100px; height: 100px; z-index: 1; }
    ";
    let v = view(vec![el(
        9,
        "wrap",
        vec![el(1, "a", vec![]), el(2, "b", vec![])],
    )]);
    let mut rt = drive(800.0, 600.0, css, &v);
    let a = at(&rt, &[0, 0]);

    let chain = hit_chain(&mut rt, 50.0, 50.0);
    assert_eq!(
        chain.first().copied(),
        Some(a),
        "the z-index: 2 sibling A must be the topmost hit: got chain {chain:?}"
    );
}

#[doc = include_str!("spec_ch4b_hit_testing.doc/ch4_7_e_border_radius_shapes_the_hit_region.md")]
#[test]
fn ch4_7_e_border_radius_shapes_the_hit_region() {
    let css = "
        .bg { position: absolute; left: 0px; top: 0px;
              width: 800px; height: 600px; }
        .circle { position: absolute; left: 0px; top: 0px;
                  width: 100px; height: 100px; border-radius: 50px; }
    ";
    let v = view(vec![el(
        9,
        "wrap",
        vec![el(1, "bg", vec![]), el(2, "circle", vec![])],
    )]);
    let mut rt = drive(800.0, 600.0, css, &v);
    let bg = at(&rt, &[0, 0]);
    let circle = at(&rt, &[0, 1]);

    assert_eq!(
        hit(&mut rt, 50.0, 50.0),
        Some(circle),
        "the circle's centre must hit it"
    );
    let chain = hit_chain(&mut rt, 5.0, 5.0);
    assert!(
        !chain.contains(&circle),
        "(5, 5) is outside the inscribed circle: got chain {chain:?}"
    );
    assert_eq!(
        chain.first().copied(),
        Some(bg),
        "(5, 5) must fall through to the background beneath: got chain {chain:?}"
    );
}

#[doc = include_str!("spec_ch4b_hit_testing.doc/ch4_7_f_scrolled_content_hits_and_pointer_events_none_is_transparent.md")]
#[test]
fn ch4_7_f_scrolled_content_hits_and_pointer_events_none_is_transparent() {
    let css = "
        .scroller { position: absolute; left: 0px; top: 0px;
                    width: 200px; height: 200px; overflow: scroll; }
        .item { width: 200px; height: 100px; }
        .overlay { position: absolute; left: 0px; top: 0px;
                   width: 200px; height: 200px; pointer-events: none; }
    ";
    let items: Vec<_> = (0..20).map(|i| el(100 + i, "item", vec![])).collect();

    // --- without the overlay.
    let v = view(vec![el(9, "wrap", vec![el(1, "scroller", items.clone())])]);
    let mut rt = drive(400.0, 400.0, css, &v);
    let scroller = at(&rt, &[0, 0]);
    let item5 = at(&rt, &[0, 0, 5]);
    rt.set_scroll_top(scroller, 500.0);
    let _ = rt.render_now(&v, &[]);
    assert_eq!(
        hit(&mut rt, 50.0, 50.0),
        Some(item5),
        "at scroll offset 500, content y 550 is item 5"
    );

    // --- with a `pointer-events: none` overlay exactly over the container.
    let v2 = view(vec![el(
        9,
        "wrap",
        vec![el(1, "scroller", items), el(2, "overlay", vec![])],
    )]);
    let mut rt2 = drive(400.0, 400.0, css, &v2);
    let scroller2 = at(&rt2, &[0, 0]);
    let item5b = at(&rt2, &[0, 0, 5]);
    let overlay = at(&rt2, &[0, 1]);
    rt2.set_scroll_top(scroller2, 500.0);
    let _ = rt2.render_now(&v2, &[]);

    let chain = hit_chain(&mut rt2, 50.0, 50.0);
    assert!(
        !chain.contains(&overlay),
        "a `pointer-events: none` node must never appear in the hit chain: \
         got chain {chain:?}"
    );
    assert_eq!(
        chain.first().copied(),
        Some(item5b),
        "the overlay must not block item 5 beneath it: got chain {chain:?}"
    );
}
