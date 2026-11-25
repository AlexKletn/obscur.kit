use super::*;

#[test]
fn flex_flow_wraps_in_either_order_and_resets_omitted_direction() {
    let view = two_child_view();
    let css = |flow| {
        format!(
            ".row {{ display: flex; width: 50px; height: 50px; align-content: start;
                 flex-direction: column; flex-wrap: nowrap; {flow} }}
         .a, .b {{ width: 40px; height: 40px; flex-shrink: 0; }}"
        )
    };
    let column = drive(&css("flex-flow: wrap column;"), &view);
    let (_, second) = children(&column);
    approx(geom(&column, second).0, 40.0, "column wraps horizontally");
    approx(geom(&column, second).1, 0.0, "new column starts at top");

    let row = drive(&css("flex-flow: wrap;"), &view);
    let (_, second) = children(&row);
    approx(geom(&row, second).0, 0.0, "omitted direction resets to row");
    approx(geom(&row, second).1, 40.0, "row wraps vertically");
}

#[test]
fn flex_flow_substitutes_variables_before_validating_the_shorthand() {
    let view = two_child_view();
    for (flow, expected) in [
        ("--flow: wrap column; flex-flow: var(--flow);", (40.0, 0.0)),
        (
            "--wrap: wrap; flex-flow: var(--missing, column) var(--wrap);",
            (40.0, 0.0),
        ),
        ("--flow: wrap; flex-flow: var(--flow);", (0.0, 40.0)),
        (
            "--wrap: wrap; flex-flow: var(--missing, var(--also-missing, column) var(--wrap));",
            (40.0, 0.0),
        ),
        (
            "--flow: wrap; flex-flow: var(--flow, var(--missing));",
            (0.0, 40.0),
        ),
        (
            "--flow: column row; flex-flow: var(--flow, column);",
            (40.0, 0.0),
        ),
    ] {
        let css = format!(
            ".row {{ display: flex; width: 50px; height: 50px; align-content: start;
                     flex-direction: column; flex-wrap: nowrap; {flow} }}
             .a, .b {{ width: 40px; height: 40px; flex-shrink: 0; }}"
        );
        let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
        rt.set_custom_value_parser(std::rc::Rc::new(
            obscur_kit::css::compiler::CustomValueCache::new(),
        ));
        let _ = rt.render_now(&view, &[compile(&css)]);
        let (_, second) = children(&rt);
        let (x, y, _, _) = geom(&rt, second);
        approx(x, expected.0, flow);
        approx(y, expected.1, flow);
    }
}

#[test]
fn css_longhands_resolve_nested_variable_fallbacks_at_computed_value_time() {
    let view = two_child_view();
    let css = ".row { --fallback: 40px; --invalid: bogus; --actual: 60px; }
               .a { width: var(--missing, var(--fallback, 30px)); height: var(--absent, 12px); }
               .b { width: var(--actual, var(--missing)); height: 12px;
                    height: var(--invalid, 99px); }";
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    rt.set_custom_value_parser(std::rc::Rc::new(
        obscur_kit::css::compiler::CustomValueCache::new(),
    ));
    let _ = rt.render_now(&view, &[compile(css)]);
    let (a, b) = children(&rt);
    approx(geom(&rt, a).2, 40.0, "nested fallback resolves a variable");
    approx(geom(&rt, a).3, 12.0, "literal fallback resolves");
    approx(
        geom(&rt, b).2,
        60.0,
        "valid primary value bypasses fallback",
    );
    approx(geom(&rt, b).3, 0.0, "invalid primary value resets to auto");
}

#[test]
fn css_longhands_substitute_variables_inside_math_functions() {
    let view = two_child_view();
    let css = ".row { width: 200px; --base: 20px; --fraction: 25%; }
               .a { width: calc(var(--fraction) + var(--base)); height: 10px; }
               .b { width: var(--missing, min(calc(var(--base) + 10px), 50px)); height: 10px; }";
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    rt.set_custom_value_parser(std::rc::Rc::new(
        obscur_kit::css::compiler::CustomValueCache::new(),
    ));
    let _ = rt.render_now(&view, &[compile(css)]);
    let (a, b) = children(&rt);
    use obscur_kit::style::property::{LengthContext, PropertyId, PropertyValue};
    let value = rt
        .computed_style(a)
        .unwrap()
        .get(PropertyId::Width)
        .unwrap();
    let PropertyValue::Calc(expr) = value.value() else {
        panic!("expected mixed percentage expression, got {value:?}");
    };
    let context = LengthContext::new(16.0, 16.0, VIEWPORT, VIEWPORT).with_percent_basis(200.0);
    assert_eq!(expr.resolve(&context), Some(70.0));
    approx(geom(&rt, a).2, 70.0, "layout supplies the containing width");
    approx(
        geom(&rt, b).2,
        30.0,
        "fallback math resolves its own variable",
    );
}

#[test]
fn css_nonlinear_sizing_math_resolves_against_each_containing_axis() {
    let view = two_child_view();
    let css = |width| {
        format!(
            ".row {{ width: {width}px; height: 100px; }}
         .a {{ width: min(60%, 110px); height: max(30%, 20px); }}
         .b {{ width: clamp(30px, calc(50% - 20px), 120px); height: 10px; }}"
        )
    };
    let mut rt = drive(&css(150), &view);
    let (a, b) = children(&rt);
    approx(geom(&rt, a).2, 90.0, "percentage branch of min");
    approx(geom(&rt, a).3, 30.0, "max uses the containing height");
    approx(geom(&rt, b).2, 55.0, "middle branch of clamp");
    let _ = rt.render_now(&view, &[compile(&css(400))]);
    approx(geom(&rt, a).2, 110.0, "fixed branch of min after resize");
    approx(geom(&rt, b).2, 120.0, "upper branch of clamp after resize");
}

#[test]
fn percentage_margin_math_preserves_sign_and_uses_the_containing_inline_extent() {
    let view = two_child_view();
    for (display, expected_left, expected_top) in [
        ("block", -10.0, 10.0),
        ("flex", -10.0, 10.0),
        ("grid", -20.0, 5.0),
    ] {
        let css = format!(
            ".row {{ display: {display}; width: 200px; grid-template-columns: 100px 100px;
                     align-items: start; }}
             .a {{ width: 40px; height: 10px;
                   margin-left: min(10% - 30px, 0px); margin-top: max(5%, 3px); }}"
        );
        let rt = drive(&css, &view);
        let (first, _) = children(&rt);
        approx(geom(&rt, first).0, expected_left, display);
        approx(geom(&rt, first).1, expected_top, display);
    }
}

#[test]
fn percentage_gap_resolves_independently_against_flex_and_grid_axes() {
    let view = two_child_view();
    for (css, expected) in [
        ("display: flex; flex-wrap: wrap;", (0.0, 55.0)),
        (
            "display: grid; grid-template-columns: 70px 70px;",
            (95.0, 0.0),
        ),
        (
            "display: grid; grid-template-columns: 70px; grid-template-rows: 40px 40px;",
            (0.0, 55.0),
        ),
    ] {
        let rt = drive(
            &format!(
                ".row {{ {css} width: 200px; height: 100px; gap: calc(10% + 5px);
                      align-items: start; align-content: start; }}
             .a, .b {{ width: 100px; height: 40px; flex-shrink: 0; }}"
            ),
            &view,
        );
        let (_, second) = children(&rt);
        approx(geom(&rt, second).0, expected.0, css);
        approx(geom(&rt, second).1, expected.1, css);
    }
}

#[test]
fn content_sized_grid_uses_cyclic_percentage_gap_only_for_placement() {
    let view = two_child_view();
    let rt = drive(
        ".row { display: grid; grid-template-columns: 80px;
                 grid-template-rows: 20px 20px; gap: calc(10% + 5px);
                 align-content: start; }
         .a, .b { height: 20px; }",
        &view,
    );
    let (_, second) = children(&rt);
    approx(
        geom(&rt, container_node(&rt)).3,
        45.0,
        "intrinsic height includes only the 5px gap",
    );
    approx(
        geom(&rt, second).1,
        29.5,
        "placement adds 10% of the 45px content box",
    );
}

#[test]
fn gap_shorthand_and_axis_longhands_share_the_cascade() {
    let view = two_child_view();
    let grid = drive(
        ".row { display: grid; width: 200px; grid-template-columns: 40px 40px;
                 gap: 5px 10%; row-gap: 13px; }
         .a, .b { height: 20px; }",
        &view,
    );
    let (_, second) = children(&grid);
    approx(geom(&grid, second).0, 60.0, "column gap stays at 10%");

    let flex = drive(
        ".row { display: flex; flex-wrap: wrap; align-content: start;
                 width: 100px; gap: 5px 20px; row-gap: 13px; column-gap: normal; }
         .a, .b { width: 60px; height: 20px; flex-shrink: 0; }",
        &view,
    );
    let (_, second) = children(&flex);
    approx(
        geom(&flex, second).1,
        33.0,
        "later row-gap wins over the shorthand",
    );

    let zero = drive(
        ".row { display: grid; grid-template-columns: 40px 40px; column-gap: 30px; gap: normal; }",
        &view,
    );
    let (_, second) = children(&zero);
    approx(geom(&zero, second).0, 40.0, "later gap clears column-gap");
}

#[test]
fn gap_variables_supply_both_axes_before_cascade_overrides() {
    let view = two_child_view();
    for (value, x, y) in [
        ("--gutters: 7px 10%; gap: var(--gutters);", 60.0, 27.0),
        (
            "--column: 10%; gap: var(--missing, 7px var(--column)); row-gap: 9px;",
            60.0,
            29.0,
        ),
        (
            "--gutters: 7px bogus; gap: var(--gutters, 10px);",
            40.0,
            20.0,
        ),
    ] {
        let css = format!(
            ".row {{ display: grid; width: 200px; grid-template-columns: 40px 40px;
                      grid-template-rows: 20px 20px; {value} }}
             .b {{ grid-column-start: 2; grid-row-start: 2; }}"
        );
        let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
        rt.set_custom_value_parser(std::rc::Rc::new(
            obscur_kit::css::compiler::CustomValueCache::new(),
        ));
        let _ = rt.render_now(&view, &[compile(&css)]);
        let (_, second) = children(&rt);
        approx(geom(&rt, second).0, x, value);
        approx(geom(&rt, second).1, y, value);
    }
}

#[test]
fn auto_height_flex_does_not_reintroduce_cyclic_gap_during_arrange() {
    let view = two_child_view();
    let rt = drive(
        ".row { display: flex; flex-direction: column; align-content: start;
                 row-gap: calc(10% + 5px); }
         .a, .b { width: 20px; height: 20px; flex-shrink: 0; }",
        &view,
    );
    let (_, second) = children(&rt);
    approx(
        geom(&rt, container_node(&rt)).3,
        45.0,
        "measured flex height",
    );
    approx(
        geom(&rt, second).1,
        25.0,
        "arrange retains a zero percentage basis",
    );
}

#[test]
fn nested_flex_gap_distinguishes_natural_height_from_definite_flex_assignment() {
    let inner = tag(el(2), "inner").with_children(ViewFragment::from_nodes(vec![
        tag(el(3), "item"),
        tag(el(4), "item"),
    ]));
    let outer = tag(el(1), "outer").with_children(ViewFragment::from_nodes(vec![inner]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![outer]));
    for (outer_height, expected_height, expected_offset) in
        [("", 45.0, 25.0), ("height: 100px;", 100.0, 35.0)]
    {
        let css = format!(
            ".outer {{ display: flex; flex-direction: column; {outer_height} }}
             .inner {{ display: flex; flex-direction: column; flex-grow: 1;
                       row-gap: calc(10% + 5px); }}
             .item {{ width: 20px; height: 20px; flex-shrink: 0; }}"
        );
        let rt = drive(&css, &view);
        let inner = rt.tree().get(container_node(&rt)).unwrap().child_nodes()[0];
        let second = rt.tree().get(inner).unwrap().child_nodes()[1];
        approx(geom(&rt, inner).3, expected_height, outer_height);
        approx(
            geom(&rt, second).1 - geom(&rt, inner).1,
            expected_offset,
            outer_height,
        );
    }
}

#[test]
fn grid_item_flex_gap_uses_the_area_height_only_when_stretched() {
    let inner = tag(el(2), "inner").with_children(ViewFragment::from_nodes(vec![
        tag(el(3), "item"),
        tag(el(4), "item"),
    ]));
    let outer = tag(el(1), "outer").with_children(ViewFragment::from_nodes(vec![inner]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![outer]));
    for (alignment, expected_height, expected_offset) in [
        ("align-self: stretch;", 100.0, 35.0),
        ("align-self: start;", 45.0, 25.0),
        ("align-self: stretch; margin-bottom: auto;", 45.0, 25.0),
    ] {
        let css = format!(
            ".outer {{ display: grid; grid-template-rows: 100px; }}
             .inner {{ display: flex; flex-direction: column; row-gap: calc(10% + 5px);
                       {alignment} }}
             .item {{ width: 20px; height: 20px; flex-shrink: 0; }}"
        );
        let rt = drive(&css, &view);
        let inner = rt.tree().get(container_node(&rt)).unwrap().child_nodes()[0];
        let second = rt.tree().get(inner).unwrap().child_nodes()[1];
        approx(geom(&rt, inner).3, expected_height, alignment);
        approx(
            geom(&rt, second).1 - geom(&rt, inner).1,
            expected_offset,
            alignment,
        );
    }
}

#[test]
fn percentage_padding_math_uses_the_parent_inline_size_after_resize() {
    let child = tag(el(3), "child");
    let inner = tag(el(2), "inner").with_children(ViewFragment::from_nodes(vec![child]));
    let outer = tag(el(1), "outer").with_children(ViewFragment::from_nodes(vec![inner]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![outer]));
    let css = |width| {
        format!(
            ".outer {{ width: {width}px; }}
         .inner {{ width: 80px; padding-left: calc(10% + 5px); padding-top: 5%; }}
         .child {{ width: 20px; height: 10px; }}"
        )
    };
    let mut rt = drive(&css(200), &view);
    let inner = rt.tree().get(container_node(&rt)).unwrap().child_nodes()[0];
    let child = rt.tree().get(inner).unwrap().child_nodes()[0];
    approx(geom(&rt, child).0, 25.0, "left padding uses parent width");
    approx(
        geom(&rt, child).1,
        10.0,
        "top padding also uses parent width",
    );
    let _ = rt.render_now(&view, &[compile(&css(300))]);
    approx(
        geom(&rt, child).0,
        35.0,
        "left padding follows parent resize",
    );
    approx(
        geom(&rt, child).1,
        15.0,
        "top padding follows parent resize",
    );
}

#[test]
fn flex_reserves_percentage_padding_in_each_items_border_box() {
    let view = two_child_view();
    for (sizing, expected) in [("content-box", 70.0), ("border-box", 40.0)] {
        let css = format!(
            ".row {{ display: flex; width: 200px; align-items: start; }}
             .a {{ width: 40px; height: 20px; box-sizing: {sizing}; flex-shrink: 0;
                   padding-left: 10%; padding-right: 5%; }}
             .b {{ width: 20px; height: 20px; flex-shrink: 0; }}"
        );
        let rt = drive(&css, &view);
        let (first, second) = children(&rt);
        approx(geom(&rt, first).2, expected, sizing);
        approx(geom(&rt, second).0, expected, sizing);
    }
}

#[test]
fn percentage_padding_keeps_text_wrapping_and_paint_alignment_in_sync() {
    use obscur_kit::scene::command::PaintCommand;

    let inner = tag(el(2), "inner").with_children(ViewFragment::from_nodes(vec![ViewNode::text(
        "word word word",
    )]));
    let outer = tag(el(1), "outer").with_children(ViewFragment::from_nodes(vec![inner]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![outer]));
    let lines = |padding| {
        let css = format!(
            ".outer {{ width: 200px; }}
             .inner {{ width: 100px; box-sizing: border-box; text-align: center;
                       padding-left: {padding}; padding-right: {padding}; }}"
        );
        let rt = drive(&css, &view);
        rt.snapshot()
            .display_list()
            .commands()
            .iter()
            .filter_map(|command| match command {
                PaintCommand::DrawText { origin, run, .. } => {
                    Some((*origin, run.text().to_owned()))
                }
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    let fixed = lines("20px");
    assert!(fixed.len() >= 2, "fixture text wraps");
    assert_eq!(lines("10%"), fixed);
}

#[doc = include_str!("../layout_css.doc/layout_depth_limit_reports_the_first_omitted_node.md")]
#[test]
fn layout_depth_limit_reports_the_first_omitted_node() {
    use std::cell::RefCell;
    use std::rc::Rc;

    use obscur_kit::diagnostics::{Diagnostic, DiagnosticSink, DiagnosticSubsystem};

    #[derive(Clone, Default)]
    struct Recorder(Rc<RefCell<Vec<Diagnostic>>>);

    impl DiagnosticSink for Recorder {
        fn emit(&self, diagnostic: &Diagnostic) {
            self.0.borrow_mut().push(diagnostic.clone());
        }
    }

    let recorder = Recorder::default();
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    runtime.set_diagnostic_sink(Box::new(recorder.clone()));
    let mut node = tag(el(1), "leaf");
    for id in 2..=256 {
        node = el(id).with_children(ViewFragment::from_nodes(vec![node]));
    }
    let view = View::single(node);
    let sheet = compile(".leaf { width: 10px; height: 10px; }");
    runtime.render_now(&view, &[sheet]);
    let diagnostics = recorder.0.borrow();
    let mut limits = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code.as_str() == "layout.depth_limit");
    let limit = limits.next().expect("layout depth limit was reported");
    assert_eq!(limit.subsystem, DiagnosticSubsystem::Layout);
    let node = limit
        .context
        .node
        .expect("diagnostic identifies the omitted node");
    assert!(runtime.tree().get(node).is_some());
    assert!(!runtime.layout_boxes().contains_key(&node));
    assert!(limits.next().is_none(), "one report per layout pass");
}

#[doc = include_str!("../layout_css.doc/clamped_grid_placement_reports_the_affected_item.md")]
#[test]
fn clamped_grid_placement_reports_the_affected_item() {
    use std::cell::RefCell;
    use std::rc::Rc;

    use obscur_kit::diagnostics::{Diagnostic, DiagnosticSink};

    struct Recorder(Rc<RefCell<Vec<NodeId>>>);
    impl DiagnosticSink for Recorder {
        fn emit(&self, diagnostic: &Diagnostic) {
            if diagnostic.code.as_str() == "layout.grid_track_limit" {
                self.0.borrow_mut().extend(diagnostic.context.node);
            }
        }
    }

    let reports = Rc::new(RefCell::new(Vec::new()));
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    runtime.set_diagnostic_sink(Box::new(Recorder(Rc::clone(&reports))));
    let view = View::single(
        tag(el(1), "grid").with_children(ViewFragment::from_nodes(vec![tag(el(2), "item")])),
    );

    for line in [1025, 1027] {
        let sheet = compile(&format!(
            ".grid {{ display: grid; grid-template-columns: 1px; grid-auto-columns: 1px; }}
             .item {{ grid-column: {line}; height: 1px; }}"
        ));
        runtime.render_now(&view, &[sheet]);
        let item = runtime
            .tree()
            .get(container_node(&runtime))
            .unwrap()
            .child_nodes()[0];
        assert_eq!(geom(&runtime, item).0, 1024.0);
        if line == 1025 {
            assert!(reports.borrow().is_empty());
        } else {
            assert_eq!(&*reports.borrow(), &[item]);
        }
    }
}
