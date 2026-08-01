#![doc = include_str!("math_function_lengths.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::scene::path::PathSegment;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{ID, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

fn view() -> View {
    View::from_fragment(ViewFragment::from_nodes(vec![
        ViewNode::element(ComponentId::from_raw(1))
            .with_prop(ViewProperty::new(ID, ViewValue::Text("box".into())))
            .with_children(ViewFragment::from_nodes(vec![
                ViewNode::element(ComponentId::from_raw(2))
                    .with_prop(ViewProperty::new(ID, ViewValue::Text("child".into()))),
            ])),
    ]))
}

#[doc = include_str!("math_function_lengths.doc/curves.md")]
fn curves(css: &str) -> usize {
    let sheet = compile(css);
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(400.0, 300.0));
    let result = rt.render_now(&view(), std::slice::from_ref(&sheet));
    result
        .snapshot()
        .display_list()
        .commands()
        .iter()
        .filter_map(|command| match command {
            PaintCommand::DrawPath { path, .. } => Some(path),
            _ => None,
        })
        .flat_map(|path| path.segments().iter())
        .filter(|segment| {
            matches!(
                segment,
                PathSegment::QuadTo { .. } | PathSegment::CubicTo { .. }
            )
        })
        .count()
}

#[doc = include_str!("math_function_lengths.doc/a_plain_length_radius_rounds_the_corners.md")]
#[test]
fn a_plain_length_radius_rounds_the_corners() {
    let painted = curves(
        "#box { width: 200px; height: 100px; background-color: #123; \
                          border-radius: 12px }",
    );
    assert!(painted > 0, "a 12px radius paints curved corners");
}

#[doc = include_str!("math_function_lengths.doc/a_math_function_radius_rounds_the_corners_too.md")]
#[test]
fn a_math_function_radius_rounds_the_corners_too() {
    let painted = curves(
        "#box { width: 200px; height: 100px; background-color: #123; \
                          border-radius: min(3vmin, 12px) }",
    );
    assert!(
        painted > 0,
        "a math function is a length: the corners must round"
    );
}

#[doc = include_str!("math_function_lengths.doc/one_math_component_does_not_drop_the_whole_shorthand.md")]
#[test]
fn one_math_component_does_not_drop_the_whole_shorthand() {
    let painted = curves(
        "#box { width: 200px; height: 100px; background-color: #123; \
                          border-radius: min(3vmin, 12px) 4px }",
    );
    assert!(painted > 0, "the mixed shorthand still rounds");
}

#[doc = include_str!("math_function_lengths.doc/a_math_function_padding_reserves_space.md")]
#[test]
fn a_math_function_padding_reserves_space() {
    let css = "#box { width: 200px; height: 100px; padding: min(4vmin, 30px) } \
               #child { width: 10px; height: 10px }";
    let sheet = compile(css);
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(400.0, 300.0));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![
        ViewNode::element(ComponentId::from_raw(1)).with_children(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(2))
                .with_prop(ViewProperty::new(ID, ViewValue::Text("box".into())))
                .with_children(ViewFragment::from_nodes(vec![
                    ViewNode::element(ComponentId::from_raw(3))
                        .with_prop(ViewProperty::new(ID, ViewValue::Text("child".into()))),
                ])),
        ])),
    ]));
    let _ = rt.render_now(&view, std::slice::from_ref(&sheet));
    let root = rt.tree().root().expect("a root after a frame");
    let box_node = rt.tree().get(root).expect("live").child_nodes()[0];
    let padded = rt.layout_boxes().get(&box_node).expect("an arranged box");
    // 4vmin of a 400×300 viewport is 12px, under the 30px ceiling, so the 100px
    // content box grows to a 124px border box. Before the fix the whole
    // declaration was dropped and it stayed 100.
    assert_eq!(padded.bounds.size.height, 124.0);
}
