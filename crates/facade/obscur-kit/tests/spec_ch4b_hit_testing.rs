#![doc = include_str!("spec_ch4b_hit_testing.doc/_module.md")]

mod spec_ch4b_support;

use obscur_kit::identity::NodeId;
use obscur_kit::primitives::Portal;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::computed::ComputedValue;
use obscur_kit::style::property::{PropertyId, StyleColor};
use obscur_kit::view::view::{ID, ViewProperty, ViewValue};
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
            el(4, "overlay", vec![])
                .with_prop(ViewProperty::new(ID, ViewValue::Text("overlay".into()))),
        ],
    )]);

    let mut rt = drive(800.0, 600.0, css, &v);
    let root = at(&rt, &[0]);
    let panel_a = at(&rt, &[0, 0]);
    let trigger = at(&rt, &[0, 0, 0]);
    let portal_node = at(&rt, &[0, 0, 0, 0]);
    let p = at(&rt, &[0, 0, 0, 0, 0]);
    let overlay = at(&rt, &[0, 1]);

    assert_eq!(rt.tree().visual_parent_of(portal_node), Some(overlay));

    // §4.14 keeps the logical parent's style scope across visual projection.
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

#[test]
fn geometric_hit_chain_matches_pointer_landing_and_reports_local_points() {
    use obscur_kit::geometry::Point;

    let v = view(vec![el(1, "parent", vec![el(2, "child", vec![])])]);
    let mut rt = drive(
        300.0,
        300.0,
        ".parent { width: 100px; height: 100px }
         .child { width: 30px; height: 20px; transform: translateX(40px) }",
        &v,
    );
    rt.set_device_scale(2.0);
    rt.set_content_offset(5.0, 0.0);
    let point = Point::new(80.0, 12.0);
    let child = at(&rt, &[0, 0]);
    let parent = at(&rt, &[0]);
    let chain = rt.hit_test(point);

    assert_eq!(chain.nodes(), [child, parent, at(&rt, &[])]);
    assert_eq!(chain.target().unwrap().local_point(), Point::new(5.0, 6.0));
    assert_eq!(chain.hits()[1].local_point(), Point::new(45.0, 6.0));
    assert_eq!(hit(&mut rt, point.x, point.y), Some(child));
}

#[test]
fn a_child_clipped_by_a_rounded_visual_ancestor_cannot_be_hit() {
    let css = "
        .under { position: absolute; width: 100px; height: 100px; }
        .round { position: absolute; width: 100px; height: 100px; border-radius: 75%; }
        .child { width: 100px; height: 100px; background-color: red; }
    ";
    let v = view(vec![
        el(1, "under", vec![]),
        el(2, "round", vec![el(3, "child", vec![])]),
    ]);
    let mut rt = drive(300.0, 300.0, css, &v);
    let under = at(&rt, &[0]);
    let child = at(&rt, &[1, 0]);
    assert_eq!(hit(&mut rt, 3.0, 3.0), Some(under));
    assert_eq!(hit(&mut rt, 50.0, 50.0), Some(child));
}

#[test]
fn reordered_grid_and_flex_subtrees_paint_and_hit_in_the_same_order() {
    use obscur_kit::scene::command::PaintCommand;

    for display in ["grid", "flex"] {
        let css = format!(
            ".wrap{{display:{display};width:100px;height:100px}}
             .a,.b{{grid-area:1 / 1;height:60px;flex-shrink:0}}
             .a{{order:3;width:70px;background-color:red}}
             .b{{order:-1;width:80px;margin-right:-80px}}
             .inner{{width:60px;height:50px;background-color:blue}}"
        );
        let v = view(vec![el(
            9,
            "wrap",
            vec![el(1, "a", vec![]), el(2, "b", vec![el(3, "inner", vec![])])],
        )]);
        let mut rt = drive(800.0, 600.0, &css, &v);
        let a = at(&rt, &[0, 0]);
        let snapshot = rt.snapshot();
        let widths: Vec<_> = snapshot
            .display_list()
            .commands()
            .iter()
            .filter_map(|command| match command {
                PaintCommand::DrawPath { path, .. } => {
                    path.bounds().map(|bounds| bounds.size.width)
                }
                _ => None,
            })
            .collect();
        assert_eq!(widths, [60.0, 70.0], "{display}");
        assert_eq!(
            hit_chain(&mut rt, 20.0, 20.0).first().copied(),
            Some(a),
            "{display}"
        );
    }
}

#[test]
fn boxless_wrappers_interleave_grid_and_flex_items_by_order() {
    use obscur_kit::scene::command::PaintCommand;

    for display in ["grid", "flex"] {
        let css = format!(
            ".wrap{{display:{display};width:100px;height:100px}}
             .group{{display:contents}}
             .a,.b,.c{{grid-area:1 / 1;height:60px;flex-shrink:0}}
             .a{{width:70px;margin-right:-70px;background-color:red;order:0}}
             .b{{width:80px;margin-right:-80px;background-color:blue;order:-1}}
             .c{{width:60px;background-color:green;order:1}}"
        );
        let v = view(vec![el(
            9,
            "wrap",
            vec![
                el(1, "a", vec![]),
                el(
                    2,
                    "group",
                    vec![el(3, "b", vec![]), el(4, "group", vec![el(5, "c", vec![])])],
                ),
            ],
        )]);
        let mut rt = drive(800.0, 600.0, &css, &v);
        let group = at(&rt, &[0, 1]);
        let target = at(&rt, &[0, 1, 1, 0]);
        let snapshot = rt.snapshot();
        let widths: Vec<_> = snapshot
            .display_list()
            .commands()
            .iter()
            .filter_map(|command| match command {
                PaintCommand::DrawPath { path, .. } => {
                    path.bounds().map(|bounds| bounds.size.width)
                }
                _ => None,
            })
            .collect();
        assert_eq!(widths, [80.0, 70.0, 60.0], "{display}");
        let chain = hit_chain(&mut rt, 20.0, 20.0);
        assert_eq!(chain.first().copied(), Some(target), "{display}");
        assert!(chain.contains(&group), "{display}");
    }
}

#[test]
fn static_grid_and_flex_items_use_z_index_before_order() {
    use obscur_kit::scene::command::PaintCommand;

    for display in ["grid", "flex"] {
        let css = format!(
            ".wrap{{display:{display};width:100px;height:100px}}
             .group{{display:contents}}
             .a,.b{{grid-area:1 / 1;height:60px;flex-shrink:0}}
             .a{{order:-1;z-index:2;width:70px;margin-right:-70px;background-color:red}}
             .b{{order:3;z-index:1;width:80px;background-color:blue}}"
        );
        let v = view(vec![el(
            9,
            "wrap",
            vec![el(1, "group", vec![el(2, "a", vec![])]), el(3, "b", vec![])],
        )]);
        let mut rt = drive(800.0, 600.0, &css, &v);
        let a = at(&rt, &[0, 0, 0]);
        let snapshot = rt.snapshot();
        let widths: Vec<_> = snapshot
            .display_list()
            .commands()
            .iter()
            .filter_map(|command| match command {
                PaintCommand::DrawPath { path, .. } => {
                    path.bounds().map(|bounds| bounds.size.width)
                }
                _ => None,
            })
            .collect();
        assert_eq!(widths, [80.0, 70.0], "{display}");
        assert_eq!(
            hit_chain(&mut rt, 20.0, 20.0).first().copied(),
            Some(a),
            "{display}"
        );
    }
}

#[test]
fn nested_z_index_stays_inside_its_parent_context_for_paint_and_hits() {
    use obscur_kit::scene::command::PaintCommand;

    for (parent_z, child_z, expected_widths, target_path) in [
        (1, 100, [70.0, 80.0], &[0, 1][..]),
        (3, -100, [80.0, 70.0], &[0, 0, 0][..]),
    ] {
        let css = format!(
            ".parent,.child,.sibling{{position:absolute;left:0;top:0;height:60px}}
             .parent{{width:100px;z-index:{parent_z};overflow:hidden}}
             .child{{width:70px;z-index:{child_z};background-color:red}}
             .sibling{{width:80px;z-index:2;background-color:blue}}"
        );
        let v = view(vec![el(
            9,
            "wrap",
            vec![
                el(1, "parent", vec![el(2, "child", vec![])]),
                el(3, "sibling", vec![]),
            ],
        )]);
        let mut rt = drive(800.0, 600.0, &css, &v);
        let target = at(&rt, target_path);
        let snapshot = rt.snapshot();
        let widths: Vec<_> = snapshot
            .display_list()
            .commands()
            .iter()
            .filter_map(|command| match command {
                PaintCommand::DrawPath { path, .. } => {
                    path.bounds().map(|bounds| bounds.size.width)
                }
                _ => None,
            })
            .collect();
        assert_eq!(widths, expected_widths);
        assert_eq!(
            hit_chain(&mut rt, 20.0, 20.0).first().copied(),
            Some(target)
        );
    }
}

#[test]
fn compositing_effects_isolate_descendant_z_index() {
    use obscur_kit::scene::command::PaintCommand;

    for (effect, isolated) in [
        ("opacity:0.5", true),
        ("transform:translateX(0px)", true),
        ("transform:none", false),
        ("filter:blur(2px)", true),
        ("filter:blur(0)", true),
        ("filter:none", false),
        ("backdrop-filter:blur(2px)", true),
        ("backdrop-filter:none", false),
        ("mix-blend-mode:multiply", true),
        ("mix-blend-mode:normal", false),
        ("isolation:isolate", true),
        ("isolation:auto", false),
    ] {
        let css = format!(
            ".parent,.child,.sibling{{position:absolute;left:0;top:0;width:80px;height:60px}}
             .parent{{{effect}}}
             .child{{width:70px;z-index:100;background-color:red}}
             .sibling{{z-index:1;background-color:blue}}"
        );
        let v = view(vec![el(
            9,
            "wrap",
            vec![
                el(1, "parent", vec![el(2, "child", vec![])]),
                el(3, "sibling", vec![]),
            ],
        )]);
        let mut rt = drive(800.0, 600.0, &css, &v);
        let snapshot = rt.snapshot();
        if effect.starts_with("isolation:") {
            assert_eq!(
                snapshot.display_list().commands().iter().any(
                    |command| matches!(command, PaintCommand::PushLayer(layer) if layer.is_isolated())
                ),
                isolated,
            );
        }
        let widths: Vec<_> = snapshot
            .display_list()
            .commands()
            .iter()
            .filter_map(|command| match command {
                PaintCommand::DrawPath { path, .. } => {
                    path.bounds().map(|bounds| bounds.size.width)
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            widths,
            if isolated { [70.0, 80.0] } else { [80.0, 70.0] },
            "{effect}"
        );
        let expected = if isolated {
            at(&rt, &[0, 1])
        } else {
            at(&rt, &[0, 0, 0])
        };
        assert_eq!(
            hit_chain(&mut rt, 20.0, 20.0).first().copied(),
            Some(expected),
            "{effect}"
        );
    }
}

#[test]
fn fixed_and_sticky_auto_z_index_isolate_their_descendants() {
    use obscur_kit::scene::command::PaintCommand;

    for (position, isolated) in [
        ("fixed", true),
        ("sticky", true),
        ("absolute", false),
        ("relative", false),
    ] {
        let css = format!(
            ".parent,.child,.sibling{{position:absolute;left:0;top:0;width:80px;height:60px}}
             .parent{{position:{position};z-index:auto}}
             .child{{width:70px;z-index:100;background-color:red}}
             .sibling{{z-index:1;background-color:blue}}"
        );
        let v = view(vec![el(
            9,
            "wrap",
            vec![
                el(1, "parent", vec![el(2, "child", vec![])]),
                el(3, "sibling", vec![]),
            ],
        )]);
        let mut rt = drive(800.0, 600.0, &css, &v);
        let expected = if isolated {
            at(&rt, &[0, 1])
        } else {
            at(&rt, &[0, 0, 0])
        };
        let snapshot = rt.snapshot();
        let widths: Vec<_> = snapshot
            .display_list()
            .commands()
            .iter()
            .filter_map(|command| match command {
                PaintCommand::DrawPath { path, .. } => {
                    path.bounds().map(|bounds| bounds.size.width)
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            widths,
            if isolated { [70.0, 80.0] } else { [80.0, 70.0] },
            "{position}"
        );
        assert_eq!(
            hit_chain(&mut rt, 20.0, 20.0).first().copied(),
            Some(expected),
            "{position}"
        );
    }
}

#[doc = include_str!("spec_ch4b_hit_testing.doc/visibility_preserves_layout_and_visible_descendants.md")]
#[test]
fn visibility_preserves_layout_and_visible_descendants() {
    use obscur_kit::scene::command::PaintCommand;
    use spec_ch4b_support::harness::geom;

    let css = "
        .under, .hidden { position: absolute; width: 100px; height: 100px; }
        .under { background-color: red; }
        .hidden { visibility: hidden; background-color: blue; }
        .inherited { width: 30px; height: 30px; background-color: blue; }
        .visible { visibility: visible; width: 40px; height: 40px; background-color: green; }
    ";
    let tree = view(vec![
        el(1, "under", vec![]),
        el(
            2,
            "hidden",
            vec![el(3, "inherited", vec![]), el(4, "visible", vec![])],
        ),
    ]);
    let mut rt = drive(800.0, 600.0, css, &tree);
    let under = at(&rt, &[0]);
    let hidden = at(&rt, &[1]);
    let inherited = at(&rt, &[1, 0]);
    let visible = at(&rt, &[1, 1]);
    assert_eq!(geom(&rt, hidden), (0.0, 0.0, 100.0, 100.0));
    assert_eq!(geom(&rt, inherited), (0.0, 0.0, 30.0, 30.0));
    assert_eq!(geom(&rt, visible), (0.0, 30.0, 40.0, 40.0));
    let snapshot = rt.snapshot();
    let widths: Vec<_> = snapshot
        .display_list()
        .commands()
        .iter()
        .filter_map(|command| match command {
            PaintCommand::DrawPath { path, .. } => path.bounds().map(|bounds| bounds.size.width),
            _ => None,
        })
        .collect();
    assert_eq!(widths, [100.0, 40.0]);
    assert_eq!(hit(&mut rt, 10.0, 10.0), Some(under));
    assert_eq!(hit(&mut rt, 10.0, 40.0), Some(visible));
    assert_eq!(hit(&mut rt, 80.0, 80.0), Some(under));
}

#[doc = include_str!("spec_ch4b_hit_testing.doc/visibility_controls_focus_and_accessibility.md")]
#[test]
fn visibility_controls_focus_and_accessibility() {
    use obscur_kit::view::view::LOCAL_NAME;
    use spec_ch4b_support::harness::compile;

    let control = |id, class, label| {
        el(id, class, vec![txt(label)]).with_prop(ViewProperty::new(
            LOCAL_NAME,
            ViewValue::Text("button".into()),
        ))
    };
    let tree = view(vec![el(
        1,
        "panel",
        vec![
            control(2, "inherited", "Hidden control"),
            control(3, "override", "Visible control"),
        ],
    )]);
    let mut rt = drive(800.0, 600.0, "", &tree);
    let panel = at(&rt, &[0]);
    let hidden = at(&rt, &[0, 0]);
    let visible = at(&rt, &[0, 1]);
    assert!(rt.focus_node(Some(hidden)));

    let sheet = compile(".panel { visibility: hidden } .override { visibility: visible }");
    let _ = rt.render_now(&tree, &[sheet]);

    assert_eq!(rt.focused(), None);
    assert!(!rt.focus_node(Some(hidden)));
    assert!(rt.focus_first_window_tab_stop());
    assert_eq!(rt.focused(), Some(visible));

    let semantics = rt.semantic_tree();
    assert!(!semantics.contains(panel));
    assert!(!semantics.contains(hidden));
    assert_eq!(semantics.children(semantics.root().unwrap()), &[visible]);
    assert!(semantics.get(visible).unwrap().state().focused);
}
