use super::*;

#[test]
fn style_queries_compare_keyword_defaults_and_follow_changed_values() {
    let child = tag(el(202), "child");
    let owner = tag(el(201), "owner").with_children(ViewFragment::from_nodes(vec![child]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![owner]));
    let css = |isolation| {
        format!(
            ".owner {{ container-name: Panel; isolation: {isolation}; --expected: isolate; }}
             .child {{ width: 10px; height: 10px; }}
             @container Panel style((position: static) and (box-sizing: content-box) and
                 (direction: ltr) and (writing-mode: horizontal-tb) and
                 (pointer-events: auto) and (visibility: visible) and (mix-blend-mode: normal)) {{
                 .child {{ width: 20px; }}
             }}
             @container Panel style(isolation: initial) {{ .child {{ height: 20px; }} }}
             @container Panel style((isolation) and (isolation: var(--expected))) {{
                 .child {{ height: 30px; }}
             }}"
        )
    };
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    rt.set_custom_value_parser(std::rc::Rc::new(
        obscur_kit::css::compiler::CustomValueCache::new(),
    ));
    let _ = rt.render_now(&view, &[compile(&css("auto"))]);
    let child = rt.tree().get(container_node(&rt)).unwrap().child_nodes()[0];
    assert!(rt.run_due_frame(&view, &[]).is_some());
    approx(geom(&rt, child).2, 20.0, "canonical defaults match");
    approx(geom(&rt, child).3, 20.0, "initial isolation matches");
    assert!(rt.run_due_frame(&view, &[]).is_none());

    let frame = rt.render_now(&view, &[compile(&css("isolate"))]);
    assert!(!frame.relaid_out(), "only the paint property changed");
    assert!(rt.run_due_frame(&view, &[]).is_some());
    approx(
        geom(&rt, child).3,
        30.0,
        "computed isolation matches the variable",
    );
    assert!(rt.run_due_frame(&view, &[]).is_none());
}

#[test]
fn style_queries_compare_box_lengths_in_the_selected_containers_context() {
    let child = tag(el(202), "child");
    let owner = tag(el(201), "owner").with_children(ViewFragment::from_nodes(vec![child]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![owner]));
    let css = |font_size| {
        format!(
            ".owner {{ container-name: Panel; font-size: {font_size}px;
                       padding-left: 20px; margin-top: 10%; min-width: 60px; }}
             .child {{ width: 10px; height: 10px; }}
             @container Panel style((padding-left: 1em) and (margin-top: 10%) and
                 (min-width: calc(2em + 20px)) and (top: initial)) {{
                 .child {{ width: 30px; }}
             }}"
        )
    };
    let mut rt = drive(&css(20), &view);
    let child = rt.tree().get(container_node(&rt)).unwrap().child_nodes()[0];
    assert!(rt.run_due_frame(&view, &[]).is_some());
    approx(
        geom(&rt, child).2,
        30.0,
        "relative and percentage comparisons match",
    );
    let _ = rt.render_now(&view, &[compile(&css(24))]);
    assert!(rt.run_due_frame(&view, &[]).is_some());
    approx(
        geom(&rt, child).2,
        10.0,
        "relative query follows the container font",
    );
}

#[test]
fn style_queries_see_none_reset_a_maximum_size() {
    let child = tag(el(202), "child");
    let owner = tag(el(201), "owner").with_children(ViewFragment::from_nodes(vec![child]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![owner]));
    let css = |reset| {
        format!(
            ".owner {{ container-name: Panel; width: 140px; height: 90px;
                       max-width: 60px; max-height: 40px; {reset} }}
             .child {{ width: 10px; height: 10px; }}
             @container Panel style((max-width: 60px) and (max-height: 40px)) {{
                 .child {{ width: 20px; }}
             }}
             @container Panel style((max-width: none) and (max-height: initial)) {{
                 .child {{ width: 30px; }}
             }}"
        )
    };
    let mut rt = drive(&css(""), &view);
    let owner = container_node(&rt);
    let child = rt.tree().get(owner).unwrap().child_nodes()[0];
    assert!(rt.run_due_frame(&view, &[]).is_some());
    approx(geom(&rt, owner).2, 60.0, "width is capped");
    approx(geom(&rt, owner).3, 40.0, "height is capped");
    approx(geom(&rt, child).2, 20.0, "finite limits match");

    let _ = rt.render_now(
        &view,
        &[compile(&css("max-width: none; max-height: none;"))],
    );
    assert!(rt.run_due_frame(&view, &[]).is_some());
    approx(geom(&rt, owner).2, 140.0, "width cap is reset");
    approx(geom(&rt, owner).3, 90.0, "height cap is reset");
    approx(geom(&rt, child).2, 30.0, "both unbounded limits match");
}

#[test]
fn style_queries_compare_intrinsic_sizing_keywords_as_computed_values() {
    let child = tag(el(202), "child");
    let owner = tag(el(201), "owner").with_children(ViewFragment::from_nodes(vec![child]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![owner]));
    let css = |width| {
        format!(
            ".owner {{ container-name: Panel; width: {width}; min-height: min-content;
                       max-width: fit-content; }}
             .child {{ width: 10px; height: 10px; }}
             @container Panel style((width: max-content) and (min-height: min-content)
                 and (max-width: fit-content)) {{ .child {{ height: 20px; }} }}
             @container Panel style(width: min-content) {{ .child {{ height: 30px; }} }}
             @container Panel not style(padding-top: max-content) {{
                 .child {{ height: 99px; }}
             }}"
        )
    };
    let mut rt = drive(&css("max-content"), &view);
    let child = rt.tree().get(container_node(&rt)).unwrap().child_nodes()[0];
    assert!(rt.run_due_frame(&view, &[]).is_some());
    approx(geom(&rt, child).3, 20.0, "compares computed keywords");
    let _ = rt.render_now(&view, &[compile(&css("min-content"))]);
    assert!(rt.run_due_frame(&view, &[]).is_some());
    approx(
        geom(&rt, child).3,
        30.0,
        "different intrinsic keyword matches",
    );
}

#[test]
fn style_queries_compare_numeric_layout_values_and_z_index_auto() {
    let owner =
        tag(el(201), "owner").with_children(ViewFragment::from_nodes(vec![tag(el(202), "child")]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![owner]));
    let css = |values| {
        format!(
            ".owner {{ container-name: Panel; {values} }}
             .child {{ width: 10px; height: 10px; }}
             @container Panel style((order: 0) and (flex-grow: initial)
                 and (flex-shrink: 1) and (z-index: auto)) {{ .child {{ width: 20px; }} }}
             @container Panel style((order: -2) and (flex-grow: 2.5)
                 and (flex-shrink: 0) and (z-index: -3)) {{ .child {{ width: 30px; }} }}
             @container Panel not style(order: 1.5) {{ .child {{ width: 99px; }} }}"
        )
    };
    let mut rt = drive(&css(""), &view);
    let child = rt.tree().get(container_node(&rt)).unwrap().child_nodes()[0];
    assert!(rt.run_due_frame(&view, &[]).is_some());
    approx(geom(&rt, child).2, 20.0, "number and auto defaults match");

    let sheet = compile(&css(
        "order: -2; flex-grow: 2.5; flex-shrink: 0; z-index: -3;",
    ));
    let _ = rt.render_now(&view, &[sheet]);
    assert!(rt.run_due_frame(&view, &[]).is_some());
    approx(geom(&rt, child).2, 30.0, "authored numbers match");
}

#[test]
fn style_queries_map_logical_lengths_using_the_selected_container_axes() {
    let child = tag(el(202), "child");
    let owner = tag(el(201), "owner").with_children(ViewFragment::from_nodes(vec![child]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![owner]));
    let css = |axes| {
        format!(
            ".owner {{ container-name: Panel; {axes}; margin-left: 15px; margin-bottom: 20px;
                       padding-right: 10px; inset-inline-end: 30px; }}
             .child {{ width: 10px; height: 10px; direction: ltr; writing-mode: horizontal-tb; }}
             @container Panel style((margin-inline-start: 15px) and (inset-inline-end: 30px)
                 and (padding-block-start: initial)) {{ .child {{ width: 20px; }} }}
             @container Panel style((margin-inline-start: 20px) and (inset-inline-end: 30px)
                 and (padding-block-start: 10px)) {{ .child {{ width: 30px; }} }}"
        )
    };
    let mut rt = drive(&css("direction: ltr"), &view);
    let child = rt.tree().get(container_node(&rt)).unwrap().child_nodes()[0];
    assert!(rt.run_due_frame(&view, &[]).is_some());
    approx(geom(&rt, child).2, 20.0, "horizontal logical sides");

    let sheet = compile(&css("direction: rtl; writing-mode: vertical-rl"));
    let _ = rt.render_now(&view, &[sheet]);
    assert!(rt.run_due_frame(&view, &[]).is_some());
    approx(geom(&rt, child).2, 30.0, "vertical RTL logical sides");
}

#[test]
fn style_queries_compare_alignment_defaults_and_baseline_preferences() {
    let owner =
        tag(el(201), "owner").with_children(ViewFragment::from_nodes(vec![tag(el(202), "child")]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![owner]));
    let css = |alignment| {
        format!(
            ".owner {{ container-name: Panel; {alignment} }}
             .child {{ width: 10px; height: 10px; }}
             @container Panel style((align-items: normal) and (justify-items: legacy) and
                 (align-self: auto) and (justify-self: initial) and (align-content: normal)
                 and (justify-content: normal) and (flex-direction: row) and (flex-wrap: nowrap)) {{
                 .child {{ width: 20px; }}
             }}
             @container Panel style((align-items: last baseline) and (justify-items: center)
                 and (flex-direction: column) and (flex-wrap: wrap)) {{ .child {{ width: 30px; }} }}"
        )
    };
    let mut rt = drive(&css(""), &view);
    let child = rt.tree().get(container_node(&rt)).unwrap().child_nodes()[0];
    assert!(rt.run_due_frame(&view, &[]).is_some());
    approx(
        geom(&rt, child).2,
        20.0,
        "alignment and flex defaults match",
    );
    let sheet = compile(&css(
        "place-items: last baseline center; flex-direction: column; flex-wrap: wrap;",
    ));
    let _ = rt.render_now(&view, &[sheet]);
    assert!(rt.run_due_frame(&view, &[]).is_some());
    approx(
        geom(&rt, child).2,
        30.0,
        "two-word baseline comparison matches",
    );
}

#[test]
fn style_query_variables_validate_the_complete_expected_value() {
    let owner =
        tag(el(201), "owner").with_children(ViewFragment::from_nodes(vec![tag(el(202), "child")]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![owner]));
    let css = ".owner { container-name: Panel; z-index: 3; align-items: last baseline;
                        --bad-index: 3 trailing; --baseline: last baseline; }
               .child { width: 10px; height: 10px; }
               @container Panel style(z-index: var(--missing, 3)) { .child { width: 20px; } }
               @container Panel style(align-items: var(--baseline)) { .child { height: 20px; } }
               @container Panel style(z-index: var(--bad-index, 3)) { .child { width: 99px; } }
               @container Panel not style(z-index: var(--bad-index, 3)) { .child { width: 98px; } }";
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    rt.set_custom_value_parser(std::rc::Rc::new(
        obscur_kit::css::compiler::CustomValueCache::new(),
    ));
    let _ = rt.render_now(&view, &[compile(css)]);
    assert!(rt.run_due_frame(&view, &[]).is_some());
    let child = rt.tree().get(container_node(&rt)).unwrap().child_nodes()[0];
    approx(
        geom(&rt, child).2,
        20.0,
        "missing variable falls back, invalid value stays unknown",
    );
    approx(geom(&rt, child).3, 20.0, "valid two-word variable matches");
}

#[test]
fn style_queries_compare_paired_overflow_axes_and_text_keywords() {
    let owner =
        tag(el(201), "owner").with_children(ViewFragment::from_nodes(vec![tag(el(202), "child")]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![owner]));
    let css = |values| {
        format!(
            ".owner {{ container-name: Panel; {values} }}
             .child {{ width: 10px; height: 10px; }}
             @container Panel style((overflow-x: visible) and (overflow-y: initial)
                 and (overscroll-behavior-x: auto) and (text-overflow: clip) and
                 (text-align: start) and (white-space: normal) and (word-break: normal)
                 and (overflow-wrap: normal) and (text-transform: none)) {{
                 .child {{ width: 20px; }}
             }}
             @container Panel style((overflow-x: auto) and (overflow-y: scroll)
                 and (overscroll-behavior-x: contain) and (white-space: nowrap)
                 and (word-break: break-all)) {{ .child {{ width: 30px; }} }}"
        )
    };
    let mut rt = drive(&css(""), &view);
    let child = rt.tree().get(container_node(&rt)).unwrap().child_nodes()[0];
    assert!(rt.run_due_frame(&view, &[]).is_some());
    approx(
        geom(&rt, child).2,
        20.0,
        "overflow and text keywords default",
    );
    let sheet = compile(&css(
        "overflow-y: scroll; overscroll-behavior-x: contain; white-space: nowrap; word-break: break-all;",
    ));
    let _ = rt.render_now(&view, &[sheet]);
    assert!(rt.run_due_frame(&view, &[]).is_some());
    approx(
        geom(&rt, child).2,
        30.0,
        "vertical scroll computes horizontal overflow to auto",
    );
}
