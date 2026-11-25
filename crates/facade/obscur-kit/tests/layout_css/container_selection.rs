use super::*;

#[test]
fn container_queries_follow_named_ancestors_and_viewport_resizes() {
    let item = tag(el(42), "item");
    let inner = tag(el(41), "inner").with_children(ViewFragment::from_nodes(vec![item]));
    let outer = tag(el(40), "outer").with_children(ViewFragment::from_nodes(vec![inner]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![outer]));
    let mut rt = drive(
        ".outer { container: sidebar / inline-size; width: 100vw; \
                   box-sizing: border-box; padding: 20px; } \
         .inner { container-type: inline-size; width: 100px; } \
         .item { width: 20px; height: 10px; } \
         @container (min-width: 250px) { .item { width: 40px; } } \
         @container sidebar (min-width: 250px) { .item { height: 30px; } } \
         @container sidebar (min-width: 270px) { .item { width: 60px; } }",
        &view,
    );
    let outer = container_node(&rt);
    let inner = rt.tree().get(outer).unwrap().child_nodes().to_vec()[0];
    let item = rt.tree().get(inner).unwrap().child_nodes().to_vec()[0];

    assert!(rt.run_due_frame(&view, &[]).is_some());
    let (_, _, width, height) = geom(&rt, item);
    approx(
        width,
        20.0,
        "nearest unnamed container and named content width",
    );
    approx(height, 30.0, "matching named ancestor");
    assert!(rt.run_due_frame(&view, &[]).is_none());

    rt.set_viewport(Size::new(240.0, VIEWPORT));
    assert!(rt.render_now(&view, &[]).relaid_out());
    assert!(rt.run_due_frame(&view, &[]).is_some());
    approx(geom(&rt, item).3, 10.0, "named query after resize");
    assert!(rt.run_due_frame(&view, &[]).is_none());
}

#[test]
fn container_shorthand_substitutes_variables_before_applying_each_longhand() {
    let child = tag(el(172), "child");
    let owner = tag(el(171), "owner").with_children(ViewFragment::from_nodes(vec![child]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![owner]));
    let queries = ".child { width: 10px; } \
         @container Alias (width > 0px) { .child { width: 30px; } } \
         @container Override (width > 0px) { .child { width: 40px; } } \
         @container Old (width > 0px) { .child { width: 50px; } }";
    for (decls, expected) in [
        (
            "--preset: Main Alias / inline-size; --link: var(--preset); container: var(--link)",
            30.0,
        ),
        (
            "container: var(--missing, Main Alias / inline-size); container-name: Override",
            40.0,
        ),
        (
            "container: var(--missing, Main Alias / inline-size) !important; container-type: normal",
            30.0,
        ),
        (
            "container: Old / inline-size; --bad: Name / nope; container: var(--bad, Alias / size)",
            10.0,
        ),
        (
            "container: Old / inline-size; container: var(--missing)",
            10.0,
        ),
        (
            "--kind: inline-size; container: Main Alias / var(--kind)",
            30.0,
        ),
        (
            "--names: Main Alias; container: var(--names) / inline-size",
            30.0,
        ),
        (
            "--names: Alias; container: Main var(--names) / var(--kind, inline-size)",
            30.0,
        ),
        ("container: Alias var(--absent,) / inline-size", 30.0),
        (
            "--kind: inline-size; container: var(--missing, Main Alias / var(--kind, size))",
            30.0,
        ),
        (
            "container: Old / inline-size; --kind: nope; container: Alias / var(--kind, size)",
            10.0,
        ),
        (
            "--kind: nope; container: Alias / var(--kind); container-type: inline-size",
            10.0,
        ),
        (
            "--kind: inline-size; container: Main Alias / var(--kind); container-name: Override",
            40.0,
        ),
        (
            "--kind: inline-size; container: Main Alias / var(--kind) !important; container-type: normal",
            30.0,
        ),
    ] {
        let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
        rt.set_custom_value_parser(std::rc::Rc::new(
            obscur_kit::css::compiler::CustomValueCache::new(),
        ));
        let sheet = compile(&format!(".owner {{ width: 100px; {decls}; }} {queries}"));
        let _ = rt.render_now(&view, &[sheet]);
        let owner = container_node(&rt);
        let child = rt.tree().get(owner).unwrap().child_nodes()[0];
        let _ = rt.run_due_frame(&view, &[]);
        approx(geom(&rt, child).2, expected, decls);
        assert!(rt.run_due_frame(&view, &[]).is_none(), "{decls}");
    }
}

#[test]
fn container_names_alias_the_same_ancestor_for_size_and_style_queries() {
    let item = tag(el(163), "item");
    let inner = tag(el(162), "inner").with_children(ViewFragment::from_nodes(vec![item]));
    let outer = tag(el(161), "outer").with_children(ViewFragment::from_nodes(vec![inner]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![outer]));
    let source = ".outer { container: Layout Wide / inline-size; width: 200px; --mode: wide; } \
         .inner { --names: Layout Compact; --kind: inline-size; \
                  container-name: var(--names); container-type: var(--kind); width: 80px; --mode: compact; } \
         .item { width: 10px; height: 10px; } \
         @container Layout (width > 100px) { .item { height: 99px !important; } } \
         @container Compact (width < 100px) { .item { width: 60px; } } \
         @container Wide (width > 100px) and style(--mode: wide) { .item { height: 30px; } } \
         @container Compact style(--mode: compact) { .item { margin-left: 5px; } } \
         @container wide (width > 100px) { .item { width: 100px; } }";
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    rt.set_custom_value_parser(std::rc::Rc::new(
        obscur_kit::css::compiler::lower_custom_value,
    ));
    let _ = rt.render_now(&view, &[compile(source)]);
    let outer = container_node(&rt);
    let inner = rt.tree().get(outer).unwrap().child_nodes()[0];
    let item = rt.tree().get(inner).unwrap().child_nodes()[0];

    assert!(rt.run_due_frame(&view, &[]).is_some());
    approx(
        geom(&rt, item).2,
        60.0,
        "secondary name selects nearest size container",
    );
    approx(
        geom(&rt, item).0,
        5.0,
        "secondary name selects nearest style container",
    );
    approx(
        geom(&rt, item).3,
        30.0,
        "secondary name selects size and style on the outer container",
    );
    assert!(rt.run_due_frame(&view, &[]).is_none());
}

#[test]
fn style_container_queries_use_the_selected_ancestors_custom_properties() {
    let item = tag(el(143), "item");
    let inner = tag(el(142), "inner").with_children(ViewFragment::from_nodes(vec![item]));
    let outer = tag(el(141), "outer").with_children(ViewFragment::from_nodes(vec![inner]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![outer]));
    let stylesheet = |mode| {
        format!(
            ".outer {{ container-name: Layout; --mode: {mode}; --expected: compact; }} \
             .inner {{ --mode: wide; --present: var(--mode); }} \
             .item {{ width: 10px; height: 10px; }} \
             @container Layout style(--mode: var(--expected)) {{ .item {{ width: 40px; }} }} \
             @container style(--mode: compact) {{ .item {{ width: 99px; }} }} \
             @container style(--present) {{ .item {{ height: 20px; }} }}"
        )
    };
    let mut rt = drive(&stylesheet("compact"), &view);
    let outer = container_node(&rt);
    let inner = rt.tree().get(outer).unwrap().child_nodes()[0];
    let item = rt.tree().get(inner).unwrap().child_nodes()[0];

    assert!(rt.run_due_frame(&view, &[]).is_some());
    approx(
        geom(&rt, item).2,
        40.0,
        "named query sees a non-size container",
    );
    approx(
        geom(&rt, item).3,
        20.0,
        "unnamed presence query sees nearest parent",
    );
    assert!(rt.run_due_frame(&view, &[]).is_none());

    let updated = compile(&stylesheet("expanded"));
    let _ = rt.render_now(&view, &[updated]);
    assert!(rt.run_due_frame(&view, &[]).is_some());
    approx(
        geom(&rt, item).2,
        10.0,
        "the custom-property change invalidates the result",
    );
    approx(
        geom(&rt, item).3,
        20.0,
        "the nearest parent still has the property",
    );
    assert!(rt.run_due_frame(&view, &[]).is_none());
}

#[test]
fn container_logic_selects_one_ancestor_before_evaluating_or_and_not() {
    let item = tag(el(183), "item");
    let inner = tag(el(182), "inner").with_children(ViewFragment::from_nodes(vec![item]));
    let outer = tag(el(181), "outer").with_children(ViewFragment::from_nodes(vec![inner]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![outer]));
    let mut rt = drive(
        ".outer { container: Layout / inline-size; width: 180px; --mode: wide; } \
         .inner { --mode: compact; } \
         .item { width: 10px; height: 10px; } \
         @container style(--mode: wide) or style(--mode: compact) { .item { width: 40px; } } \
         @container Layout not style(--mode: compact) { .item { height: 30px; } } \
         @container Missing not style(--mode) { .item { height: 99px; } } \
         @container not style(--mode) { .item { height: 98px; } } \
         @container style(--mode: compact) or (width > 200px) { .item { width: 97px; } } \
         @container Layout not (orientation: landscape) { .item { width: 96px; } }",
        &view,
    );
    let outer = container_node(&rt);
    let inner = rt.tree().get(outer).unwrap().child_nodes()[0];
    let item = rt.tree().get(inner).unwrap().child_nodes()[0];

    assert!(rt.run_due_frame(&view, &[]).is_some());
    approx(
        geom(&rt, item).2,
        40.0,
        "or uses the ancestor eligible for every branch",
    );
    approx(
        geom(&rt, item).3,
        30.0,
        "not applies only after a container was selected",
    );
    assert!(rt.run_due_frame(&view, &[]).is_none());
}

#[test]
fn custom_style_ranges_compare_numeric_dimensions_on_the_selected_container() {
    let child = tag(el(212), "child");
    let owner = tag(el(211), "owner").with_children(ViewFragment::from_nodes(vec![child]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![owner]));
    let stylesheet = |gap| {
        format!(
            ".owner {{ container-name: Panel; --distance: {gap}px; --gap: var(--distance); --ratio: 12%; --count: 2; --zero: 0; }} \
         .child {{ width: 10px; height: 10px; }} \
         @container Panel style(--gap >= 1cm) {{ .child {{ width: 40px; }} }} \
         @container Panel style((--ratio < 15%) and (--count = 2)) {{ .child {{ height: 30px; }} }} \
         @container Panel style((--zero = 0px) and (not (--count > 3px))) {{ .child {{ margin-left: 5px; }} }} \
         @container Panel not style(--gap > 1em) {{ .child {{ width: 99px; }} }}"
        )
    };
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    rt.set_custom_value_parser(std::rc::Rc::new(
        obscur_kit::css::compiler::CustomValueCache::new(),
    ));
    let _ = rt.render_now(&view, &[compile(&stylesheet(40))]);
    let owner = container_node(&rt);
    let child = rt.tree().get(owner).unwrap().child_nodes()[0];
    assert!(rt.run_due_frame(&view, &[]).is_some());
    approx(
        geom(&rt, child).2,
        40.0,
        "absolute units normalize before comparison",
    );
    approx(
        geom(&rt, child).3,
        30.0,
        "numbers and percentages compare within their dimensions",
    );
    approx(
        geom(&rt, child).0,
        5.0,
        "unitless zero compares with length and incompatible types are false",
    );
    assert!(rt.run_due_frame(&view, &[]).is_none());

    let _ = rt.render_now(&view, &[compile(&stylesheet(30))]);
    assert!(rt.run_due_frame(&view, &[]).is_some());
    approx(
        geom(&rt, child).2,
        10.0,
        "a custom-property change updates the numeric range",
    );
    assert!(rt.run_due_frame(&view, &[]).is_none());
}

#[test]
fn nested_style_query_fallbacks_use_the_first_resolved_value() {
    let child = tag(el(302), "child");
    let owner = tag(el(301), "owner").with_children(ViewFragment::from_nodes(vec![child]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![owner]));
    let stylesheet = |overrides| {
        format!(
            ".owner {{ container-name: Panel; --gap: 20px; --unset: initial; color: red; {overrides} }} \
         .child {{ width: 10px; height: 10px; }} \
         @container Panel style(--gap >= var(--limit, var(--unset, var(--default, 12px)))) {{ \
             .child {{ width: 40px; }} }} \
         @container Panel style(color: var(--target, var(--default-color, red))) {{ \
             .child {{ height: 30px; }} }}"
        )
    };
    for (overrides, expected) in [
        (
            "--limit: 30px; --default: 10px; --default-color: blue",
            (10.0, 10.0),
        ),
        ("--default: 15px; --default-color: red", (40.0, 30.0)),
        ("", (40.0, 30.0)),
        ("--default: bogus; --default-color: bogus", (10.0, 10.0)),
    ] {
        let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
        rt.set_custom_value_parser(std::rc::Rc::new(
            obscur_kit::css::compiler::CustomValueCache::new(),
        ));
        let _ = rt.render_now(&view, &[compile(&stylesheet(overrides))]);
        let owner = container_node(&rt);
        let child = rt.tree().get(owner).unwrap().child_nodes()[0];
        assert!(rt.run_due_frame(&view, &[]).is_some());
        approx(geom(&rt, child).2, expected.0, overrides);
        approx(geom(&rt, child).3, expected.1, overrides);
        assert!(rt.run_due_frame(&view, &[]).is_none());
    }
}
