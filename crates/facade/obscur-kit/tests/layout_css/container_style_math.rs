use super::*;

#[test]
fn unitless_style_range_math_preserves_number_types() {
    let child = tag(el(292), "child");
    let owner = tag(el(291), "owner").with_children(ViewFragment::from_nodes(vec![child]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![owner]));
    let css = ".owner { container-name: Panel; --level: calc((2 + 1) * max(2, 1)); --zero: calc(3 - 3); } \
         .child { width: 10px; height: 10px; } \
         @container Panel style(--level = calc(12 / 2)) { .child { width: 40px; } } \
         @container Panel style(min(4, 5) <= --level < var(--missing, clamp(6, 7, 8))) { \
             .child { height: 30px; } } \
         @container Panel style(--zero = 0px) { .child { margin-left: 5px; } } \
         @container Panel style(--level = 6px) { .child { width: 98px; } } \
         @container Panel not style(--level = calc(12 / 0)) { .child { width: 99px; } }";
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    rt.set_custom_value_parser(std::rc::Rc::new(
        obscur_kit::css::compiler::CustomValueCache::new(),
    ));
    let _ = rt.render_now(&view, &[compile(css)]);
    let owner = container_node(&rt);
    let child = rt.tree().get(owner).unwrap().child_nodes()[0];

    assert!(rt.run_due_frame(&view, &[]).is_some());
    approx(
        geom(&rt, child).2,
        40.0,
        "unitless equality rejects lengths and invalid division",
    );
    approx(
        geom(&rt, child).3,
        30.0,
        "unitless chain with a computed fallback",
    );
    approx(
        geom(&rt, child).0,
        5.0,
        "unitless zero still compares to zero length",
    );
    assert!(rt.run_due_frame(&view, &[]).is_none());
}

#[test]
fn custom_style_range_math_compares_expressions_and_fallbacks() {
    let child = tag(el(282), "child");
    let owner = tag(el(281), "owner").with_children(ViewFragment::from_nodes(vec![child]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![owner]));
    let stylesheet = |font| {
        format!(
            ".owner {{ container-name: Panel; font-size: {font}px; --gap: calc(1em + 8px); --progress: max(13%, 7%); }} \
         .child {{ width: 10px; height: 10px; }} \
         @container Panel style(--gap >= calc(1em + 4px)) {{ .child {{ width: 40px; }} }} \
         @container Panel style(min(35px, 10vw) <= --gap < var(--missing, clamp(30px, 2em, 34px))) {{ \
             .child {{ height: 30px; }} }} \
         @container Panel style(--progress = 13%) {{ .child {{ margin-left: 5px; }} }} \
         @container Panel style(--gap = calc(10% + 20px)) {{ .child {{ width: 99px; }} }}"
        )
    };
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    rt.set_custom_value_parser(std::rc::Rc::new(
        obscur_kit::css::compiler::CustomValueCache::new(),
    ));
    let _ = rt.render_now(&view, &[compile(&stylesheet(20))]);
    let owner = container_node(&rt);
    let child = rt.tree().get(owner).unwrap().child_nodes()[0];

    assert!(rt.run_due_frame(&view, &[]).is_some());
    approx(geom(&rt, child).2, 40.0, "actual and threshold expressions");
    approx(
        geom(&rt, child).3,
        10.0,
        "the gap starts below the chained lower bound",
    );
    approx(
        geom(&rt, child).0,
        5.0,
        "pure percentage math keeps its dimension",
    );
    assert!(rt.run_due_frame(&view, &[]).is_none());

    let _ = rt.render_now(&view, &[compile(&stylesheet(24))]);
    assert!(rt.run_due_frame(&view, &[]).is_some());
    approx(
        geom(&rt, child).3,
        30.0,
        "font change puts the gap inside both expression bounds",
    );
    assert!(rt.run_due_frame(&view, &[]).is_none());
}

#[test]
fn relative_style_ranges_use_the_selected_containers_font_and_available_sizes() {
    let child = tag(el(243), "child");
    let owner = tag(el(242), "owner").with_children(ViewFragment::from_nodes(vec![child]));
    let root = tag(el(241), "root").with_children(ViewFragment::from_nodes(vec![owner]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![root]));
    let css = ".root { font-size: 20px; container-type: inline-size; width: 100vw; } \
         .owner { font-size: 30px; container-name: Panel; --gap: 2em; --limit: 3rem; --glyph: 1ch; } \
         .child { width: 10px; height: 10px; } \
         @container Panel style((--gap = var(--limit)) and (--glyph = 1ch)) { .child { width: 40px; } } \
         @container Panel style(--gap >= 20vw) { .child { height: 30px; } } \
         @container Panel style(--gap >= 20cqw) { .child { margin-left: 5px; } }";
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    rt.set_custom_value_parser(std::rc::Rc::new(
        obscur_kit::css::compiler::CustomValueCache::new(),
    ));
    let _ = rt.render_now(&view, &[compile(css)]);
    let root = container_node(&rt);
    let owner = rt.tree().get(root).unwrap().child_nodes()[0];
    let child = rt.tree().get(owner).unwrap().child_nodes()[0];

    assert!(rt.run_due_frame(&view, &[]).is_some());
    approx(
        geom(&rt, child).2,
        40.0,
        "em, rem and ch resolve in the selected font context",
    );
    approx(geom(&rt, child).3, 30.0, "vw uses the viewport");
    approx(
        geom(&rt, child).0,
        5.0,
        "cqw uses the selected container's eligible ancestor",
    );
    assert!(rt.run_due_frame(&view, &[]).is_none());

    rt.set_viewport(Size::new(400.0, VIEWPORT));
    let _ = rt.render_now(&view, &[]);
    assert!(rt.run_due_frame(&view, &[]).is_some());
    approx(
        geom(&rt, child).2,
        40.0,
        "font comparison remains stable after resize",
    );
    approx(geom(&rt, child).3, 10.0, "vw is recomputed after resize");
    approx(
        geom(&rt, child).0,
        0.0,
        "cqw is recomputed after ancestor resize",
    );
    assert!(rt.run_due_frame(&view, &[]).is_none());
}

#[test]
fn style_range_variables_read_and_refresh_bounds_on_the_selected_container() {
    let item = tag(el(233), "item");
    let inner = tag(el(232), "inner").with_children(ViewFragment::from_nodes(vec![item]));
    let outer = tag(el(231), "outer").with_children(ViewFragment::from_nodes(vec![inner]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![outer]));
    let stylesheet = |max| {
        format!(
            ".outer {{ container-name: Panel; --gap: 20px; --min: 10px; --limit: {max}; --max: var(--limit); }} \
         .inner {{ --max: 1px; }} \
         .item {{ width: 10px; height: 10px; }} \
         @container Panel style(var(--min) <= --gap < var(--max, 40px)) {{ .item {{ width: 30px; }} }} \
         @container Panel style(--gap > var(--missing, 12px)) {{ .item {{ height: 40px; }} }}"
        )
    };
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    rt.set_custom_value_parser(std::rc::Rc::new(
        obscur_kit::css::compiler::CustomValueCache::new(),
    ));
    let _ = rt.render_now(&view, &[compile(&stylesheet("30px"))]);
    let outer = container_node(&rt);
    let inner = rt.tree().get(outer).unwrap().child_nodes()[0];
    let item = rt.tree().get(inner).unwrap().child_nodes()[0];
    assert!(rt.run_due_frame(&view, &[]).is_some());
    approx(
        geom(&rt, item).2,
        30.0,
        "both variables come from the named ancestor",
    );
    approx(
        geom(&rt, item).3,
        40.0,
        "missing variable uses the literal fallback",
    );
    assert!(rt.run_due_frame(&view, &[]).is_none());

    for max in ["20px", "invalid", "50%"] {
        let _ = rt.render_now(&view, &[compile(&stylesheet(max))]);
        assert!(rt.run_due_frame(&view, &[]).is_some());
        approx(
            geom(&rt, item).2,
            10.0,
            "strict bound, invalid value or incompatible dimension",
        );
        assert!(rt.run_due_frame(&view, &[]).is_none());
    }
}

#[test]
fn reversed_and_chained_style_ranges_honor_both_boundaries() {
    let child = tag(el(222), "child");
    let owner = tag(el(221), "owner").with_children(ViewFragment::from_nodes(vec![child]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![owner]));
    for (gap, expected) in [(11, (10.0, 10.0)), (12, (30.0, 40.0)), (40, (10.0, 40.0))] {
        let css = format!(
            ".owner {{ container-name: Panel; --gap: {gap}px; --zero: 0; }} \
             .child {{ width: 10px; height: 10px; }} \
             @container Panel style(40px > --gap >= 12px) {{ .child {{ width: 30px; }} }} \
             @container Panel style(12px <= --gap) {{ .child {{ height: 40px; }} }} \
             @container Panel style(12px <= --gap < 40px) {{ .child {{ margin-left: 5px; }} }} \
             @container Panel style(-1 < --zero < 1px) {{ .child {{ width: 99px; }} }}"
        );
        let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
        rt.set_custom_value_parser(std::rc::Rc::new(
            obscur_kit::css::compiler::CustomValueCache::new(),
        ));
        let _ = rt.render_now(&view, &[compile(&css)]);
        let owner = container_node(&rt);
        let child = rt.tree().get(owner).unwrap().child_nodes()[0];
        assert!(rt.run_due_frame(&view, &[]).is_some());
        let (left, _, width, height) = geom(&rt, child);
        approx(width, expected.0, "descending chain");
        approx(height, expected.1, "reversed comparison");
        approx(left, if gap == 12 { 5.0 } else { 0.0 }, "ascending chain");
        assert!(rt.run_due_frame(&view, &[]).is_none());
    }
}

#[test]
fn standard_style_query_math_uses_the_same_bases_as_declarations() {
    let item = tag(el(273), "item");
    let owner = tag(el(272), "owner").with_children(ViewFragment::from_nodes(vec![item]));
    let root = tag(el(271), "root").with_children(ViewFragment::from_nodes(vec![owner]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![root]));
    let css = ".root { font-size: 20px; width: 100vw; height: 200px; container-type: inline-size; } \
         .owner { container-name: Panel; font-size: 30px; width: 100px; height: calc(50% - 1em); \
                  --expected: clamp(50px, max(90px, 4em), 100px); } \
         .item { width: 10px; height: 10px; } \
         @container Panel style((width: calc(3em + 10px)) and (width: var(--expected))) { \
             .item { min-width: 40px; } } \
         @container Panel style(font-size: min(40px, calc(100% + 10px))) { .item { min-height: 30px; } } \
         @container Panel style(height: calc(50% - 30px)) { .item { margin-top: 7px; } } \
         @container Panel style(width: calc(20cqw + 40px)) { .item { margin-left: 5px; } }";
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    rt.set_custom_value_parser(std::rc::Rc::new(
        obscur_kit::css::compiler::CustomValueCache::new(),
    ));
    let _ = rt.render_now(&view, &[compile(css)]);
    let root = container_node(&rt);
    let owner = rt.tree().get(root).unwrap().child_nodes()[0];
    let item = rt.tree().get(owner).unwrap().child_nodes()[0];
    assert!(rt.run_due_frame(&view, &[]).is_some());
    let (left, top, width, height) = geom(&rt, item);
    approx(
        width,
        40.0,
        "calc, max and clamp resolve against selected font",
    );
    approx(
        height,
        30.0,
        "font-size percentages inside math use the parent font",
    );
    approx(
        left,
        5.0,
        "container units inside calc resolve against the eligible ancestor",
    );
    approx(
        top,
        7.0,
        "percentage formulas compare after resolving their length components",
    );
    assert!(rt.run_due_frame(&view, &[]).is_none());

    rt.set_viewport(Size::new(400.0, VIEWPORT));
    let _ = rt.render_now(&view, &[]);
    assert!(rt.run_due_frame(&view, &[]).is_some());
    approx(
        geom(&rt, item).0,
        0.0,
        "the resized ancestor changes the calc result",
    );
    assert!(rt.run_due_frame(&view, &[]).is_none());
}

#[test]
fn font_size_style_queries_use_parent_metrics_and_standard_fallbacks() {
    let child = tag(el(263), "child");
    let owner = tag(el(262), "owner").with_children(ViewFragment::from_nodes(vec![child]));
    let root = tag(el(261), "root").with_children(ViewFragment::from_nodes(vec![owner]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![root]));
    let css = ".root { container-name: Root; font-size: 20px; } \
         .owner { container-name: Panel; font-size: 150%; width: 100px; color: red; --bad: bogus; } \
         .child { width: 10px; height: 10px; } \
         @container Panel style(font-size: 1.5em) { .child { min-width: 40px; } } \
         @container Panel style(font-size: var(--missing, 150%)) { .child { height: 30px; } } \
         @container Panel style(color: var(--missing, red)) { .child { margin-left: 5px; } } \
         @container Panel style(width: var(--bad, 100px)) { .child { height: 99px; } } \
         @container Root style(font-size: 1.25rem) { .child { margin-top: 7px; } }";
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    rt.set_custom_value_parser(std::rc::Rc::new(
        obscur_kit::css::compiler::CustomValueCache::new(),
    ));
    let _ = rt.render_now(&view, &[compile(css)]);
    let root = container_node(&rt);
    let owner = rt.tree().get(root).unwrap().child_nodes()[0];
    let child = rt.tree().get(owner).unwrap().child_nodes()[0];

    assert!(rt.run_due_frame(&view, &[]).is_some());
    let (left, top, width, height) = geom(&rt, child);
    approx(width, 40.0, "font-size em is based on the parent font");
    approx(
        height,
        30.0,
        "fallback percentage uses the parent font and invalid width does not take its fallback",
    );
    approx(
        left,
        5.0,
        "missing color variable uses its literal fallback",
    );
    approx(top, 7.0, "rem in the root font-size uses the initial size");
    assert!(rt.run_due_frame(&view, &[]).is_none());
}

#[test]
fn standard_style_query_values_resolve_relative_lengths_and_custom_references() {
    let child = tag(el(252), "child");
    let owner = tag(el(251), "owner").with_children(ViewFragment::from_nodes(vec![child]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![owner]));
    let stylesheet = |target| {
        format!(
            ".owner {{ container-name: Panel; width: 100px; font-size: 25px; color: #ff0000; --target: {target}; --color: red; }} \
         .child {{ width: 10px; height: 10px; }} \
         @container Panel style(width: 4em) {{ .child {{ min-width: 40px; }} }} \
         @container Panel style(width: var(--target)) {{ .child {{ margin-left: 5px; }} }} \
         @container Panel style(color: var(--color)) {{ .child {{ height: 30px; }} }} \
         @container Panel not style(width: var(--missing)) {{ .child {{ height: 99px; }} }}"
        )
    };
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    rt.set_custom_value_parser(std::rc::Rc::new(
        obscur_kit::css::compiler::CustomValueCache::new(),
    ));
    let _ = rt.render_now(&view, &[compile(&stylesheet("4em"))]);
    let owner = container_node(&rt);
    let child = rt.tree().get(owner).unwrap().child_nodes()[0];

    assert!(rt.run_due_frame(&view, &[]).is_some());
    approx(geom(&rt, child).2, 40.0, "literal relative length");
    approx(geom(&rt, child).0, 5.0, "relative variable lowers as width");
    approx(geom(&rt, child).3, 30.0, "color variable lowers as color");
    assert!(rt.run_due_frame(&view, &[]).is_none());

    let _ = rt.render_now(&view, &[compile(&stylesheet("5em"))]);
    assert!(rt.run_due_frame(&view, &[]).is_some());
    approx(
        geom(&rt, child).0,
        0.0,
        "changed variable updates just its comparison",
    );
    approx(geom(&rt, child).2, 40.0, "literal comparison still matches");
    assert!(rt.run_due_frame(&view, &[]).is_none());
}

#[test]
fn standard_style_queries_read_computed_values_and_follow_paint_only_changes() {
    let child = tag(el(202), "child");
    let owner = tag(el(201), "owner").with_children(ViewFragment::from_nodes(vec![child]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![owner]));
    let stylesheet = |opacity| {
        format!(
            ".owner {{ container-name: Panel; width: calc(90px + 10px); color: #ff0000; opacity: {opacity}; }} \
         .child {{ width: 10px; height: 10px; }} \
         @container Panel style((width: 100px) and (color: red) and (display: block)) {{ .child {{ min-width: 40px; }} }} \
         @container Panel style(opacity) {{ .child {{ height: 30px; }} }} \
         @container Panel style(opacity: initial) {{ .child {{ height: 20px; }} }} \
         @container Panel not style(letter-spacing: 1px) {{ .child {{ width: 99px; }} }}"
        )
    };
    let mut rt = drive(&stylesheet(0.5), &view);
    let owner = container_node(&rt);
    let child = rt.tree().get(owner).unwrap().child_nodes()[0];
    assert!(rt.run_due_frame(&view, &[]).is_some());
    approx(
        geom(&rt, child).2,
        40.0,
        "query compares computed length and color",
    );
    approx(geom(&rt, child).3, 30.0, "opacity differs from initial");
    assert!(rt.run_due_frame(&view, &[]).is_none());

    let frame = rt.render_now(&view, &[compile(&stylesheet(1.0))]);
    assert!(!frame.relaid_out(), "changing only opacity needs paint");
    assert!(rt.run_due_frame(&view, &[]).is_some());
    approx(
        geom(&rt, child).3,
        20.0,
        "the follow-up observes initial opacity",
    );
    assert!(rt.run_due_frame(&view, &[]).is_none());
}

#[test]
fn nested_container_groups_preserve_selection_and_unknown_values() {
    let child = tag(el(192), "child");
    let owner = tag(el(191), "owner").with_children(ViewFragment::from_nodes(vec![child]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![owner]));
    let mut rt = drive(
        ".owner { container: Layout / inline-size; width: 180px; --density: compact; } \
         .child { width: 10px; height: 10px; } \
         @container Layout (width > 100px) and (style(--density: wide) or style(--density: compact)) { \
             .child { width: 40px; } } \
         @container Layout style(((--density: compact) or (--density: dense)) and (not (--disabled))) { \
             .child { height: 30px; } } \
         @container Layout not ((orientation: landscape) or (width > 200px)) { \
             .child { width: 99px; } } \
         @container Layout not ((orientation: landscape) and (width > 200px)) { \
             .child { margin-left: 5px; } }",
        &view,
    );
    let owner = container_node(&rt);
    let child = rt.tree().get(owner).unwrap().child_nodes()[0];

    assert!(rt.run_due_frame(&view, &[]).is_some());
    approx(
        geom(&rt, child).2,
        40.0,
        "nested or matches but negated unknown does not",
    );
    approx(geom(&rt, child).3, 30.0, "nested logic inside style()");
    approx(
        geom(&rt, child).0,
        5.0,
        "false is decisive in and even with unknown",
    );
    assert!(rt.run_due_frame(&view, &[]).is_none());
}

#[test]
fn mixed_container_queries_match_size_and_style_on_the_same_ancestor() {
    let item = tag(el(153), "item");
    let inner = tag(el(152), "inner").with_children(ViewFragment::from_nodes(vec![item]));
    let outer = tag(el(151), "outer").with_children(ViewFragment::from_nodes(vec![inner]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![outer]));
    let mut rt = drive(
        ".outer { container-type: inline-size; width: 200px; --mode: wide; } \
         .inner { --mode: compact; } \
         .item { width: 10px; } \
         @container style(--mode: wide) and (width > 100px) { .item { width: 40px; } } \
         @container (width > 100px) and style(--mode: compact) { .item { width: 99px; } }",
        &view,
    );
    let outer = container_node(&rt);
    let inner = rt.tree().get(outer).unwrap().child_nodes()[0];
    let item = rt.tree().get(inner).unwrap().child_nodes()[0];
    assert!(rt.run_due_frame(&view, &[]).is_some());
    approx(
        geom(&rt, item).2,
        40.0,
        "style and size come from the eligible outer box",
    );
}

#[test]
fn container_shorthand_cascades_with_both_longhands() {
    let child = tag(el(112), "child");
    let owner = tag(el(111), "owner").with_children(ViewFragment::from_nodes(vec![child]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![owner]));
    let queries = ".child { width: 20px; } \
                   @container Sidebar (width > 0px) { .child { width: 40px; } }";
    for (declarations, expected) in [
        ("container-type: size; container: Sidebar", 20.0),
        ("container: Sidebar; container-type: size", 40.0),
        ("container: Sidebar / size; container-name: sidebar", 20.0),
        (
            "container: Sidebar / size !important; container-name: sidebar; container-type: normal",
            40.0,
        ),
    ] {
        let mut rt = drive(
            &format!(".owner {{ width: 100px; {declarations}; }} {queries}"),
            &view,
        );
        let owner = container_node(&rt);
        let child = rt.tree().get(owner).unwrap().child_nodes()[0];
        let _ = rt.run_due_frame(&view, &[]);
        approx(geom(&rt, child).2, expected, declarations);
    }
}
