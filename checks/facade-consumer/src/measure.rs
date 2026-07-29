//! A headless measurement harness an application can build from the facade.
//!
//! `docs/external-consumer-audit.md` recommends lifting this into the repository
//! on its own merits: it answers a layout question with numbers instead of an
//! argument, and needs no display, window or GPU.

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::RuntimeId;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::ua_stylesheet;
use obscur_kit::view::view::View;

/// One laid-out node: the identifiers an application can ask about, and its box.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasuredNode {
    /// The node's classes, joined — how an application names it in CSS.
    pub class: String,
    /// The node's CSS type (local name), if it has one.
    pub local_name: String,
    /// Depth below the root.
    pub depth: u32,
    /// Border-box width in logical pixels, or `0` if the node was not laid out.
    pub width: f32,
    /// Border-box height in logical pixels, or `0` if the node was not laid out.
    pub height: f32,
}

/// Compiles authored CSS the way an application does, from text it owns.
#[must_use]
pub fn stylesheet(css: &str) -> Stylesheet {
    let source = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&source).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

/// Lays `view` out at `width`×`height` under `css` and returns the boxes.
#[must_use]
pub fn measure(view: &View, css: &str, width: f32, height: f32) -> Vec<MeasuredNode> {
    let authored = stylesheet(css);
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(width, height));
    runtime.render_now(view, &[ua_stylesheet(), authored]);
    runtime
        .inspect_snapshot()
        .nodes
        .into_iter()
        .map(|node| MeasuredNode {
            class: node.classes.join(" "),
            local_name: node.local_name.unwrap_or_default(),
            depth: node.depth,
            width: node.bounds.map_or(0.0, |rect| rect.size.width),
            height: node.bounds.map_or(0.0, |rect| rect.size.height),
        })
        .collect()
}

/// The measured node whose class list contains `class`, if the tree has one.
#[must_use]
pub fn find<'a>(nodes: &'a [MeasuredNode], class: &str) -> Option<&'a MeasuredNode> {
    nodes
        .iter()
        .find(|node| node.class.split_whitespace().any(|name| name == class))
}
