#![doc = include_str!("canvas_paints.doc/_module.md")]
#![forbid(unsafe_code)]

use std::cell::Cell;
use std::rc::Rc;

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::{Point, Rect, Size};
use obscur_kit::identity::{ComponentId, InternedSymbol, RuntimeId};
use obscur_kit::interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};
use obscur_kit::primitives::Canvas;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::scene::color::Color;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::scene::paint::Brush;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

const CHART: InternedSymbol = InternedSymbol::from_raw(0x0C01);

const CSS: &str = ".panel{width:300px;height:300px} .chart{width:100px;height:60px}";

fn compile(css: &str) -> Stylesheet {
    let parsed = StylesheetParser::new(&CssSource::anonymous(css)).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("canvas_paints.doc/staged.md")]
fn staged() -> UiRuntime {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(400.0, 400.0));
    let _ = rt.render_now(&staged_view(), std::slice::from_ref(&compile(CSS)));
    rt
}

#[doc = include_str!("canvas_paints.doc/target_at.md")]
fn target_at(rt: &mut UiRuntime, x: f32, y: f32) -> Option<obscur_kit::identity::NodeId> {
    rt.handle_pointer(
        PointerEvent::new(
            PointerId::MOUSE,
            PointerDevice::Mouse,
            PointerEventKind::Move,
            Point::new(x, y),
        )
        .as_trusted(),
    )
    .target()
}

fn red() -> Brush {
    Brush::solid(Color::rgba(1.0, 0.0, 0.0, 1.0))
}

#[doc = include_str!("canvas_paints.doc/drawn_paths.md")]
fn drawn_paths(rt: &UiRuntime) -> usize {
    rt.snapshot()
        .display_list()
        .commands()
        .iter()
        .filter(|c| matches!(c, PaintCommand::DrawPath { .. }))
        .count()
}

#[doc = include_str!("canvas_paints.doc/a_registered_painter_reaches_the_display_list.md")]
#[test]
fn a_registered_painter_reaches_the_display_list() {
    // The gap this closes: `Canvas` had a `ComponentId` and nothing anywhere in
    // the runtime or the adapters referenced it. It laid out and clipped like
    // any element and could not draw.
    let mut rt = staged();
    let before = drawn_paths(&rt);

    rt.register_canvas(
        CHART,
        Box::new(|region, _paint| {
            let size = region.size();
            region
                .painter()
                .fill_rect(Rect::new(Point::new(0.0, 0.0), size), red());
            false
        }),
    );
    let _ = rt.render_now(&staged_view(), std::slice::from_ref(&compile(CSS)));

    assert!(
        drawn_paths(&rt) > before,
        "the painter's fill reached the scene"
    );
}

#[doc = include_str!("canvas_paints.doc/staged_view.md")]
fn staged_view() -> View {
    let chart = Canvas::new(100.0, 60.0)
        .named(CHART)
        .into_view_node()
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("chart".into())));
    View::single(
        ViewNode::element(ComponentId::from_raw(0x00C0_0001))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into())))
            .with_children(ViewFragment::from_nodes(vec![chart])),
    )
}

#[doc = include_str!("canvas_paints.doc/the_painter_is_told_its_laid_out_size.md")]
#[test]
fn the_painter_is_told_its_laid_out_size() {
    // A painter must not have to be told what CSS gave it. `.chart` is
    // 100x60 in the sheet, and that — not the 100x60 constructor hint — is what
    // the region reports, because layout is what decides.
    let seen: Rc<Cell<Size>> = Rc::new(Cell::new(Size::new(0.0, 0.0)));
    let sink = Rc::clone(&seen);
    let mut rt = staged();
    rt.register_canvas(
        CHART,
        Box::new(move |region, _paint| {
            sink.set(region.size());
            region
                .painter()
                .fill_rect(Rect::new(Point::new(0.0, 0.0), Size::new(1.0, 1.0)), red());
            false
        }),
    );
    let _ = rt.render_now(&staged_view(), std::slice::from_ref(&compile(CSS)));

    assert_eq!(
        seen.get(),
        Size::new(100.0, 60.0),
        "the region reports the laid-out box, which is what the sheet decided"
    );
}

#[doc = include_str!("canvas_paints.doc/an_unregistered_canvas_paints_nothing_and_does_not_panic.md")]
#[test]
fn an_unregistered_canvas_paints_nothing_and_does_not_panic() {
    let mut rt = staged();
    let _ = rt.render_now(&staged_view(), std::slice::from_ref(&compile(CSS)));
    assert!(
        rt.snapshot().display_list().is_balanced(),
        "a canvas nobody registered leaves the list untouched and balanced"
    );
}

#[doc = include_str!("canvas_paints.doc/an_unbalanced_painter_is_contained_and_named.md")]
#[test]
fn an_unbalanced_painter_is_contained_and_named() {
    // The containment that matters: a painter which opens a clip and never
    // closes it would otherwise swallow every node painted after it.
    let mut rt = staged();
    rt.register_canvas(
        CHART,
        Box::new(|region, _paint| {
            region
                .painter()
                .push_clip(obscur_kit::scene::clip::Clip::rect(Rect::new(
                    Point::new(0.0, 0.0),
                    Size::new(10.0, 10.0),
                )));
            region
                .painter()
                .fill_rect(Rect::new(Point::new(0.0, 0.0), Size::new(5.0, 5.0)), red());
            false
        }),
    );
    let _ = rt.render_now(&staged_view(), std::slice::from_ref(&compile(CSS)));

    assert!(
        rt.snapshot().display_list().is_balanced(),
        "the scene is balanced even though the painter was not"
    );
    assert_eq!(
        rt.take_unbalanced_canvases(),
        vec![CHART],
        "and the painter that did it is named"
    );
}

#[doc = include_str!("canvas_paints.doc/a_declared_hit_region_narrows_the_canvas_to_its_own_shape.md")]
#[test]
fn a_declared_hit_region_narrows_the_canvas_to_its_own_shape() {
    // §4.13. A canvas that draws a triangle should not be hit in the corners of
    // its box — that space belongs to whatever is behind it.
    let mut rt = staged();
    rt.register_canvas(
        CHART,
        Box::new(|region, _paint| {
            let size = region.size();
            let mut shape = obscur_kit::scene::path::Path::new();
            shape.move_to(Point::new(0.0, 0.0));
            shape.line_to(Point::new(size.width, 0.0));
            shape.line_to(Point::new(size.width / 2.0, size.height));
            shape.close();
            region
                .painter()
                .fill_rect(Rect::new(Point::new(0.0, 0.0), Size::new(1.0, 1.0)), red());
            region.hit_region(shape);
            false
        }),
    );
    let _ = rt.render_now(&staged_view(), std::slice::from_ref(&compile(CSS)));

    // The canvas is laid out at the panel's origin, 100x60.
    let inside = target_at(&mut rt, 50.0, 10.0);
    let corner = target_at(&mut rt, 2.0, 55.0);
    assert_ne!(
        inside, corner,
        "a point inside the triangle and one in the box's corner cannot resolve \
         to the same node: {inside:?} vs {corner:?}"
    );
}

#[doc = include_str!("canvas_paints.doc/a_canvas_that_declares_nothing_is_hit_anywhere_in_its_box.md")]
#[test]
fn a_canvas_that_declares_nothing_is_hit_anywhere_in_its_box() {
    // The default, and it is the one every other element already has.
    let mut rt = staged();
    rt.register_canvas(
        CHART,
        Box::new(|region, _paint| {
            region
                .painter()
                .fill_rect(Rect::new(Point::new(0.0, 0.0), Size::new(1.0, 1.0)), red());
            false
        }),
    );
    let _ = rt.render_now(&staged_view(), std::slice::from_ref(&compile(CSS)));
    let inside = target_at(&mut rt, 50.0, 10.0);
    let corner = target_at(&mut rt, 2.0, 55.0);
    assert_eq!(
        inside, corner,
        "with no declared region both points land on the same node"
    );
}
