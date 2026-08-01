#![doc = include_str!("measure.doc/_module.md")]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::RuntimeId;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::ua_stylesheet;
use obscur_kit::view::view::View;

#[doc = include_str!("measure.doc/MeasuredNode.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct MeasuredNode {
    #[doc = include_str!("measure.doc/class.md")]
    pub class: String,
    #[doc = include_str!("measure.doc/local_name.md")]
    pub local_name: String,
    #[doc = include_str!("measure.doc/depth.md")]
    pub depth: u32,
    #[doc = include_str!("measure.doc/text.md")]
    pub text: String,
    #[doc = include_str!("measure.doc/width.md")]
    pub width: f32,
    #[doc = include_str!("measure.doc/height.md")]
    pub height: f32,
}

#[doc = include_str!("measure.doc/stylesheet.md")]
#[must_use]
pub fn stylesheet(css: &str) -> Stylesheet {
    let source = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&source).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("measure.doc/runtime.md")]
#[must_use]
pub fn runtime(width: f32, height: f32) -> UiRuntime {
    UiRuntime::new(RuntimeId::from_raw(1), Size::new(width, height))
}

#[doc = include_str!("measure.doc/measure.md")]
#[must_use]
pub fn measure(view: &View, css: &str, width: f32, height: f32) -> Vec<MeasuredNode> {
    measure_in(&mut runtime(width, height), view, css)
}

#[doc = include_str!("measure.doc/measure_in.md")]
#[must_use]
pub fn measure_in(runtime: &mut UiRuntime, view: &View, css: &str) -> Vec<MeasuredNode> {
    let authored = stylesheet(css);
    runtime.render_now(view, &[ua_stylesheet(), authored]);
    runtime
        .inspect_snapshot()
        .nodes
        .into_iter()
        .map(|node| MeasuredNode {
            class: node.classes.join(" "),
            local_name: node.local_name.unwrap_or_default(),
            depth: node.depth,
            text: node.text.unwrap_or_default(),
            width: node.bounds.map_or(0.0, |rect| rect.size.width),
            height: node.bounds.map_or(0.0, |rect| rect.size.height),
        })
        .collect()
}

#[doc = include_str!("measure.doc/find.md")]
#[must_use]
pub fn find<'a>(nodes: &'a [MeasuredNode], class: &str) -> Option<&'a MeasuredNode> {
    nodes
        .iter()
        .find(|node| node.class.split_whitespace().any(|name| name == class))
}
