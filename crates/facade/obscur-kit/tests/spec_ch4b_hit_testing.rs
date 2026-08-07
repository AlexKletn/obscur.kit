#![doc = include_str!("spec_ch4b_hit_testing.doc/_module.md")]

mod spec_ch4b_support;

use obscur_kit::identity::NodeId;
use obscur_kit::primitives::Portal;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::computed::ComputedValue;
use obscur_kit::style::property::{PropertyId, StyleColor};
use spec_ch4b_support::harness::{at, drive, el, hit, hit_chain, txt, view};

#[doc = include_str!("spec_ch4b_hit_testing.doc/color_of.md")]
fn color_of(rt: &UiRuntime, node: NodeId) -> Option<StyleColor> {
    rt.computed_style(node)?
        .get(PropertyId::Color)
        .and_then(ComputedValue::as_color)
}

#[doc = include_str!("spec_ch4b_hit_testing.doc/ch4_7_a_the_hit_chain_is_the_logical_tree_not_the_portal_target.md")]
#[test]
fn ch4_7_a_the_hit_chain_is_the_logical_tree_not_the_portal_target() {
    // Three distinct colours, one per candidate scope, so a wrong answer names
    // itself: green = P's logical parent (correct, §4.14), red = the portal
    // TARGET (what reparenting the logical tree would give), blue = the document
    // root (what losing the inherited value entirely would give).
    let css = "
        .root { width: 800px; height: 600px; color: rgb(0,0,255); }
        .panel-a { width: 800px; height: 100px; }
        .trigger { width: 100px; height: 100px; color: rgb(0,128,0); }
        .overlay { position: absolute; left: 0px; top: 0px;
                   width: 800px; height: 600px; color: rgb(255,0,0); }
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
    let portal_node = at(&rt, &[0, 0, 0, 0]);
    let p = at(&rt, &[0, 0, 0, 0, 0]);
    let overlay = at(&rt, &[0, 1]);

    // --- §4.14: the portal preserves the LOGICAL parent's style scope.
    //
    // This half carries the test when the portal is a no-op, and it is why the
    // test is not vacuous. The chain assertions below are satisfied trivially by
    // an implementation that never projects anything — `P` simply stays where it
    // was written — so on their own they could not tell a working portal from an
    // absent one. These can: they fail the moment someone "wires" portals by
    // moving `P` in the retained tree, which is both the obvious wrong fix and
    // the one §4.14 exists to forbid.
    let trigger_colour = color_of(&rt, trigger).expect("Trigger declares a colour");
    let overlay_colour = color_of(&rt, overlay).expect("Overlay declares a colour");
    let root_colour = color_of(&rt, root).expect("Root declares a colour");
    assert!(
        trigger_colour != overlay_colour && trigger_colour != root_colour,
        "fixture error: the three scopes must be distinguishable"
    );
    let p_colour = color_of(&rt, p).expect("P must inherit a colour from its logical parent");
    assert_eq!(
        p_colour, trigger_colour,
        "§4.14: a portalled node keeps its LOGICAL parent's style scope, so P \
         inherits Trigger's green — got {p_colour:?}"
    );
    assert_ne!(
        p_colour, overlay_colour,
        "§4.14: P must NOT inherit from the portal target Overlay. Inheriting \
         its red means the portal was wired by reparenting the logical tree, \
         which moves style/theme scope along with paint"
    );

    // --- §4.7: the hit chain is the logical tree.
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
    // P's logical ancestors, in order: [P, Portal, Trigger, PanelA, Root].
    let (i_portal, i_trigger, i_panel, i_root) = (
        index_of(portal_node),
        index_of(trigger),
        index_of(panel_a),
        index_of(root),
    );
    assert!(
        0 < i_portal && i_portal < i_trigger && i_trigger < i_panel && i_panel < i_root,
        "the hit chain must run P -> Portal -> Trigger -> PanelA -> Root: got {chain:?}"
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
    // (2, -28) alone proves nothing: it is outside the UNTRANSFORMED rect too,
    // so an implementation that ignores the transform entirely also passes it.
    // (2, 2) is the falsifier — inside the layout rect, so the transform-blind
    // code reports a hit, and inverse-rotating it lands at local (3.3, 41.2),
    // past the box's 40px bottom edge, so a correct one misses.
    let chain = hit_chain(&mut rt, 2.0, 2.0);
    assert!(
        !chain.contains(&boxed),
        "(2, 2) is inside the box's LAYOUT rect but the rotation vacated it, \
         so it must miss: got chain {chain:?}"
    );
    // And the converse, which no bounding-box test can fake: local (95, 5) is
    // painted at (92.4, 41.2) — BELOW the layout rect — and must be live.
    assert_eq!(
        hit(&mut rt, 92.43, 41.21),
        Some(boxed),
        "the rotation carried the box's top-right corner below its layout rect; \
         the pixel it landed on must still be clickable"
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
        .over-none { position: absolute; left: 0px; top: 0px;
                     width: 200px; height: 200px; z-index: 1;
                     pointer-events: none; }
        .blocker { position: absolute; left: 0px; top: 0px;
                   width: 200px; height: 200px; z-index: 1; }
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

    // --- an unscrolled overlay the probe is INSIDE, and which would otherwise
    //     win the hit.
    //
    // At scroll offset 500 the folded point is (50, 550), outside the overlay's
    // 200x200 box, so the assertions above hold even for a hit test that never
    // heard of `pointer-events`. Unscrolled the folded point is (50, 50) —
    // squarely on the overlay — and `z-index: 1` puts the overlay above the
    // deeper item beneath it, so the pointer-event policy is the only thing
    // that can let the click through.
    let items2: Vec<_> = (0..20).map(|i| el(200 + i, "item", vec![])).collect();
    let v3 = view(vec![el(
        9,
        "wrap",
        vec![
            el(1, "scroller", items2.clone()),
            el(2, "over-none", vec![]),
        ],
    )]);
    let mut rt3 = drive(400.0, 400.0, css, &v3);
    let item0 = at(&rt3, &[0, 0, 0]);
    let overlay3 = at(&rt3, &[0, 1]);

    let chain = hit_chain(&mut rt3, 50.0, 50.0);
    assert!(
        !chain.contains(&overlay3),
        "(50, 50) is inside the `pointer-events: none` overlay's own box, so \
         the policy — not the geometry — must keep it out: got chain {chain:?}"
    );
    assert_eq!(
        chain.first().copied(),
        Some(item0),
        "the click must reach item 0 directly beneath the transparent overlay: \
         got chain {chain:?}"
    );

    // --- the control: the same box with no declaration swallows the click.
    let v4 = view(vec![el(
        9,
        "wrap",
        vec![el(1, "scroller", items2), el(2, "blocker", vec![])],
    )]);
    let mut rt4 = drive(400.0, 400.0, css, &v4);
    let blocker = at(&rt4, &[0, 1]);

    let chain = hit_chain(&mut rt4, 50.0, 50.0);
    assert_eq!(
        chain.first().copied(),
        Some(blocker),
        "without `pointer-events: none` the identical box must swallow the \
         click — otherwise the assertion above proves nothing: got chain \
         {chain:?}"
    );
}
