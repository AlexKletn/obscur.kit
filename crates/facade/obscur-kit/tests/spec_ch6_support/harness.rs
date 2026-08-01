#![doc = include_str!("harness.doc/_module.md")]
#![allow(dead_code, unreachable_pub)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::geometry::{Point, Size};
use obscur_kit::identity::{ComponentId, NodeId, RuntimeId};
use obscur_kit::interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};
use obscur_kit::windowing::request::CreateWindow;
use obscur_kit::windowing::window::WindowProperties;

#[doc = include_str!("harness.doc/compile.md")]
#[must_use]
pub fn compile(css: &str) -> Stylesheet {
    let src = obscur_kit::css::source::CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("harness.doc/el.md")]
#[must_use]
pub fn el(id: u64, class: &str, kids: Vec<ViewNode>) -> ViewNode {
    ViewNode::element(ComponentId::from_raw(id))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
        .with_children(ViewFragment::from_nodes(kids))
}

#[doc = include_str!("harness.doc/view.md")]
#[must_use]
pub fn view(nodes: Vec<ViewNode>) -> View {
    View::from_fragment(ViewFragment::from_nodes(nodes))
}

#[doc = include_str!("harness.doc/drive.md")]
#[must_use]
pub fn drive(w: f32, h: f32, css: &str, v: &View) -> UiRuntime {
    let sheet = compile(css);
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(w, h));
    let _ = rt.render_now(v, std::slice::from_ref(&sheet));
    rt
}

#[doc = include_str!("harness.doc/at.md")]
#[must_use]
pub fn at(rt: &UiRuntime, path: &[usize]) -> NodeId {
    let mut node = rt.tree().root().expect("a root after a frame");
    for &i in path {
        let kids = rt
            .tree()
            .get(node)
            .expect("a live node")
            .child_nodes()
            .to_vec();
        assert!(i < kids.len(), "child index {i} out of range");
        node = kids[i];
    }
    node
}

#[doc = include_str!("harness.doc/pointer.md")]
#[must_use]
pub fn pointer(kind: PointerEventKind, x: f32, y: f32) -> PointerEvent {
    PointerEvent::new(
        PointerId::from_raw(1),
        PointerDevice::Mouse,
        kind,
        Point::new(x, y),
    )
}

#[doc = include_str!("harness.doc/create_at.md")]
#[must_use]
pub fn create_at(x: f32, y: f32, w: f32, h: f32) -> CreateWindow {
    CreateWindow::new(
        WindowProperties::default()
            .with_size(Size::new(w, h))
            .with_position(Point::new(x, y)),
    )
}
