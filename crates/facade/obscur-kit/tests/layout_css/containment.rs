use super::*;

#[test]
fn container_units_resolve_each_axis_and_follow_content_box_resize() {
    let item = tag(el(63), "item");
    let inner = tag(el(62), "inner").with_children(ViewFragment::from_nodes(vec![item]));
    let outer = tag(el(61), "outer").with_children(ViewFragment::from_nodes(vec![inner]));
    let fallback = tag(el(64), "fallback");
    let view = View::from_fragment(ViewFragment::from_nodes(vec![outer, fallback]));
    let mut rt = drive(
        ".outer { container-type: size; width: 100vw; height: 100vw; \
                   box-sizing: border-box; padding: 20px; } \
         .inner { container-type: inline-size; width: 50%; } \
         .item { width: 50cqw; height: calc(10cqh + 10cqi); } \
         .inner::before { content: ''; height: 10cqw; } \
         .fallback { width: 10cqw; height: 10cqh; }",
        &view,
    );
    let root = rt.tree().root().unwrap();
    let siblings = rt.tree().get(root).unwrap().child_nodes();
    let (outer, fallback) = (siblings[0], siblings[1]);
    let inner = rt.tree().get(outer).unwrap().child_nodes()[0];
    let item = rt.tree().get(inner).unwrap().child_nodes()[0];

    assert!(rt.run_due_frame(&view, &[]).is_some());
    approx(geom(&rt, item).2, 65.0, "nearest eligible width");
    approx(
        geom(&rt, item).3,
        39.0,
        "height and inline use different containers",
    );
    approx(
        geom(&rt, item).1 - geom(&rt, inner).1,
        13.0,
        "pseudo-element uses its owner",
    );
    approx(geom(&rt, fallback).2, 30.0, "viewport width fallback");
    approx(geom(&rt, fallback).3, 30.0, "viewport height fallback");
    assert!(rt.run_due_frame(&view, &[]).is_none());

    rt.set_viewport(Size::new(240.0, VIEWPORT));
    assert!(rt.render_now(&view, &[]).relaid_out());
    assert!(rt.run_due_frame(&view, &[]).is_some());
    approx(geom(&rt, item).2, 50.0, "nearest width after resize");
    approx(geom(&rt, item).3, 30.0, "height and inline after resize");
    approx(
        geom(&rt, item).1 - geom(&rt, inner).1,
        10.0,
        "pseudo-element after resize",
    );
    approx(
        geom(&rt, fallback).2,
        24.0,
        "viewport width fallback after resize",
    );
    assert!(rt.run_due_frame(&view, &[]).is_none());
}

#[test]
fn size_containment_keeps_layout_of_overflowing_children() {
    for display in ["block", "flex", "grid"] {
        let child = tag(el(72), "child");
        let owner = tag(el(71), "owner").with_children(ViewFragment::from_nodes(vec![child]));
        let view = View::from_fragment(ViewFragment::from_nodes(vec![owner]));
        let css = format!(
            ".owner {{ display: {display}; container-type: size; width: max-content; \
                       padding: 5px; border: 2px solid black; box-sizing: border-box; min-height: 20px; }} \
             .child {{ width: 60px; height: 40px; flex-shrink: 0; }} \
             @container (width = 0px) {{ .child {{ width: 80px; }} }}"
        );
        let mut rt = drive(&css, &view);
        let owner = container_node(&rt);
        let child = rt.tree().get(owner).unwrap().child_nodes()[0];

        assert!(rt.run_due_frame(&view, &[]).is_some(), "{display}");
        approx(geom(&rt, owner).2, 14.0, display);
        approx(geom(&rt, owner).3, 20.0, display);
        approx(geom(&rt, child).2, 80.0, display);
        approx(geom(&rt, child).3, 40.0, display);
        approx(geom(&rt, child).0 - geom(&rt, owner).0, 7.0, display);
        assert!(rt.run_due_frame(&view, &[]).is_none(), "{display}");
    }
}

#[test]
fn contained_flow_preserves_the_natural_height_of_nested_flex() {
    let item = tag(el(93), "item");
    let child = tag(el(92), "child").with_children(ViewFragment::from_nodes(vec![item]));
    let owner = tag(el(91), "owner").with_children(ViewFragment::from_nodes(vec![child]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![owner]));
    let mut rt = drive(
        ".owner { container-type: size; } \
         .child { display: flex; flex-direction: column; } \
         .item { height: 40px; }",
        &view,
    );
    let owner = container_node(&rt);
    let child = rt.tree().get(owner).unwrap().child_nodes()[0];
    approx(geom(&rt, owner).2, VIEWPORT, "auto inline size stretches");
    approx(geom(&rt, owner).3, 0.0, "auto contained block size");
    approx(
        geom(&rt, child).3,
        40.0,
        "the overflowing flex column keeps its height",
    );

    let sheet = compile(
        ".owner { container-type: normal; } \
         .child { display: flex; flex-direction: column; } \
         .item { height: 40px; }",
    );
    assert!(rt.render_now(&view, &[sheet]).relaid_out());
    approx(
        geom(&rt, owner).3,
        40.0,
        "removing containment restores content sizing",
    );
}

#[test]
fn contained_flex_item_uses_parent_assignment_for_child_percentages() {
    let child = tag(el(103), "child");
    let item = tag(el(102), "item").with_children(ViewFragment::from_nodes(vec![child]));
    let row = tag(el(101), "row").with_children(ViewFragment::from_nodes(vec![item]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![row]));
    let rt = drive(
        ".row { display: flex; width: 200px; height: 60px; } \
         .item { container-type: size; flex-grow: 1; } \
         .child { width: 50%; height: 50%; }",
        &view,
    );
    let row = container_node(&rt);
    let item = rt.tree().get(row).unwrap().child_nodes()[0];
    let child = rt.tree().get(item).unwrap().child_nodes()[0];
    approx(
        geom(&rt, item).2,
        200.0,
        "flex grow assigns the contained width",
    );
    approx(
        geom(&rt, item).3,
        60.0,
        "flex stretch assigns the contained height",
    );
    approx(geom(&rt, child).2, 100.0, "child uses the assigned width");
    approx(geom(&rt, child).3, 30.0, "child uses the assigned height");
}

#[test]
fn intrinsic_containment_sizes_feed_layout_and_container_queries() {
    let child = tag(el(122), "child");
    let owner = tag(el(121), "owner").with_children(ViewFragment::from_nodes(vec![child]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![owner]));
    let mut rt = drive(
        ".owner { container: Box / size; width: max-content; padding: 5px; \
                   box-sizing: border-box; contain-intrinsic-size: 4rem calc(20px + 10px); } \
         .child { width: 5px; height: 80px; } \
         @container Box (height = 30px) { .child { width: 50cqw; } }",
        &view,
    );
    let owner = container_node(&rt);
    let child = rt.tree().get(owner).unwrap().child_nodes()[0];
    assert!(rt.run_due_frame(&view, &[]).is_some());
    approx(geom(&rt, owner).2, 74.0, "intrinsic width plus padding");
    approx(geom(&rt, owner).3, 40.0, "intrinsic height plus padding");
    approx(
        geom(&rt, child).2,
        32.0,
        "query uses the intrinsic content box",
    );
    approx(
        geom(&rt, child).3,
        80.0,
        "child retains its overflowing height",
    );
    assert!(rt.run_due_frame(&view, &[]).is_none());
}

#[test]
fn logical_intrinsic_size_cascades_onto_the_contained_axis() {
    let child = tag(el(132), "child");
    let owner = tag(el(131), "owner").with_children(ViewFragment::from_nodes(vec![child]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![owner]));
    let rt = drive(
        ".owner { container-type: inline-size; writing-mode: vertical-lr; height: max-content; \
                   contain-intrinsic-size: 100px 120px; contain-intrinsic-inline-size: 30px; } \
         .child { width: 60px; height: 80px; }",
        &view,
    );
    let owner = container_node(&rt);
    approx(
        geom(&rt, owner).2,
        60.0,
        "uncontained axis uses real content",
    );
    approx(
        geom(&rt, owner).3,
        30.0,
        "logical inline overrides physical height",
    );
}

#[test]
fn inline_size_containment_leaves_block_axis_content_sized() {
    for (writing_mode, intrinsic, expected) in [
        ("horizontal-tb", "width", (10.0, 50.0)),
        ("vertical-lr", "height", (70.0, 10.0)),
    ] {
        let child = tag(el(82), "child");
        let owner = tag(el(81), "owner").with_children(ViewFragment::from_nodes(vec![child]));
        let view = View::from_fragment(ViewFragment::from_nodes(vec![owner]));
        let css = format!(
            ".owner {{ container-type: inline-size; writing-mode: {writing_mode}; \
                       {intrinsic}: max-content; padding: 5px; }} \
             .child {{ width: 60px; height: 40px; }}"
        );
        let rt = drive(&css, &view);
        let owner = container_node(&rt);
        let child = rt.tree().get(owner).unwrap().child_nodes()[0];
        approx(geom(&rt, owner).2, expected.0, writing_mode);
        approx(geom(&rt, owner).3, expected.1, writing_mode);
        approx(geom(&rt, child).2, 60.0, writing_mode);
        approx(geom(&rt, child).3, 40.0, writing_mode);
    }
}

#[test]
fn generated_boxes_join_layout_and_paint_in_child_order() {
    use obscur_kit::scene::command::PaintCommand;
    use obscur_kit::style::selector::PseudoState;

    let child =
        tag(el(52), "child").with_children(ViewFragment::from_nodes(vec![ViewNode::text("child")]));
    let owner = tag(el(51), "owner").with_children(ViewFragment::from_nodes(vec![child]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![owner]));
    let mut rt = drive(
        r#".owner { width: 100px; }
           .owner::before { content: "before"; height: 30px; background-color: red; }
           .child { height: 25px; }
           .owner::AFTER { content: "after"; height: 35px; }"#,
        &view,
    );
    let owner = container_node(&rt);
    let children = rt.tree().get(owner).unwrap().child_nodes();
    assert_eq!(children.len(), 1);
    let child = children[0];
    approx(
        geom(&rt, child).1,
        30.0,
        "generated before box shifts the child",
    );
    approx(
        geom(&rt, owner).3,
        90.0,
        "both boxes contribute to auto height",
    );

    let snapshot = rt.snapshot();
    let texts: Vec<_> = snapshot
        .display_list()
        .commands()
        .iter()
        .filter_map(|cmd| match cmd {
            PaintCommand::DrawText { run, .. } => Some(run.text()),
            _ => None,
        })
        .collect();
    assert_eq!(texts, ["before", "child", "after"]);

    let sheet = compile(
        r#".owner::before { content: ""; height: 30px; }
           .child { height: 25px; }
           .owner:hover::before { content: none; }"#,
    );
    assert!(rt.render_now(&view, &[sheet]).relaid_out());
    approx(
        geom(&rt, child).1,
        30.0,
        "empty generated box outside hover",
    );
    rt.set_pseudo_state(owner, PseudoState::HOVER);
    assert!(rt.render_now(&view, &[]).relaid_out());
    approx(geom(&rt, child).1, 0.0, "content none removes the box");
    approx(
        geom(&rt, owner).3,
        25.0,
        "removed decoration no longer takes space",
    );
}

#[test]
fn container_units_combine_independently_selected_logical_axes() {
    let item = tag(el(3), "item");
    let inner = tag(el(2), "inner").with_children(ViewFragment::from_nodes(vec![item]));
    let outer = tag(el(1), "outer").with_children(ViewFragment::from_nodes(vec![inner]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![outer]));
    let mut rt = drive(
        ".outer { container-type: size; width: 240px; height: 160px; }
         .inner { container-type: inline-size; width: 100px; }
         .item { width: calc(10cqb + 10cqmin); height: 10cqmax; }",
        &view,
    );
    assert!(rt.run_due_frame(&view, &[]).is_some());
    let outer = container_node(&rt);
    let inner = rt.tree().get(outer).unwrap().child_nodes()[0];
    let item = rt.tree().get(inner).unwrap().child_nodes()[0];

    approx(
        geom(&rt, item).2,
        26.0,
        "block basis 160px and minimum basis 100px",
    );
    approx(geom(&rt, item).3, 16.0, "maximum basis 160px");
    assert!(rt.run_due_frame(&view, &[]).is_none());
}
