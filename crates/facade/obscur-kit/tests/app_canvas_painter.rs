#![doc = include_str!("app_canvas_painter.doc/_module.md")]
#![forbid(unsafe_code)]

use std::cell::Cell;
use std::rc::Rc;

use obscur_kit::AppControl;
use obscur_kit::bootstrap::{CssPipeline, KitBootstrap};
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::{Point, Rect, Size};
use obscur_kit::identity::{ComponentId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::scene::color::Color;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::scene::paint::Brush;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};
use obscur_kit::widgets::progress::{Progress, ProgressShape};
use obscur_kit::widgets::slider::Slider;

#[doc = include_str!("app_canvas_painter.doc/PANEL.md")]
const PANEL: ComponentId = ComponentId::from_raw(0x00C0_0001);

#[doc = include_str!("app_canvas_painter.doc/SIZED.md")]
const SIZED: &str = ".panel{width:300px;height:300px} .bar{width:200px;height:12px}";

#[doc = include_str!("app_canvas_painter.doc/runtime.md")]
fn runtime() -> UiRuntime {
    UiRuntime::new(RuntimeId::from_raw(1), Size::new(400.0, 400.0))
}

#[doc = include_str!("app_canvas_painter.doc/panel.md")]
fn panel(child: ViewNode) -> View {
    View::single(
        ViewNode::element(PANEL)
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into())))
            .with_children(ViewFragment::from_nodes(vec![child])),
    )
}

#[doc = include_str!("app_canvas_painter.doc/sized.md")]
fn sized(node: &ViewNode) -> ViewNode {
    node.clone()
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("bar".into())))
}

#[doc = include_str!("app_canvas_painter.doc/drawn_paths.md")]
fn drawn_paths(rt: &UiRuntime) -> usize {
    rt.snapshot()
        .display_list()
        .commands()
        .iter()
        .filter(|c| matches!(c, PaintCommand::DrawPath { .. }))
        .count()
}

#[doc = include_str!("app_canvas_painter.doc/install.md")]
fn install(rt: &mut UiRuntime, css: &mut CssPipeline, sheet: &str) {
    css.install(rt, &[CssSource::anonymous(sheet)]);
}

#[doc = include_str!("app_canvas_painter.doc/the_shipped_progress_bar_draws_out_of_the_box.md")]
#[test]
fn the_shipped_progress_bar_draws_out_of_the_box() {
    // `Progress` builds a `<canvas>` naming a painter by symbol and draws
    // nothing itself. Nothing registered that painter, so a progress bar an
    // application built through the shipped builder laid out, clipped and
    // reported itself to the accessibility tree — and painted no pixels.
    let mut rt = runtime();
    let mut css = CssPipeline::new();
    install(&mut rt, &mut css, SIZED);
    KitBootstrap::new()
        .install_builtin_components(&mut rt)
        .expect("nothing has claimed a built-in id on a fresh runtime");

    let bar = sized(&Progress::contents(0.5, ProgressShape::Line, true, None).nodes()[0]);
    let _ = rt.render_now(&panel(bar), css.sheets());

    // The track and the half that is filled: two stroked paths, and the second
    // is the one that only appears because a value reached the painter.
    assert_eq!(
        drawn_paths(&rt),
        2,
        "the built-in progress painter drew its track and its fill"
    );
}

#[doc = include_str!("app_canvas_painter.doc/the_shipped_slider_draws_out_of_the_box.md")]
#[test]
fn the_shipped_slider_draws_out_of_the_box() {
    let mut rt = runtime();
    let mut css = CssPipeline::new();
    install(&mut rt, &mut css, SIZED);
    KitBootstrap::new()
        .install_builtin_components(&mut rt)
        .expect("nothing has claimed a built-in id on a fresh runtime");

    let bar = sized(&Slider::new(0.5).contents().nodes()[0]);
    let _ = rt.render_now(&panel(bar), css.sheets());

    // Track, filled span, thumb.
    assert_eq!(
        drawn_paths(&rt),
        3,
        "the built-in slider painter drew its track, its fill and its thumb"
    );
}

#[doc = include_str!("app_canvas_painter.doc/opting_out_of_the_built_ins_opts_out_of_their_painters.md")]
#[test]
fn opting_out_of_the_built_ins_opts_out_of_their_painters() {
    // The other side of the default, and the proof the assertions above are
    // about the registration rather than about anything else in the frame: the
    // same tree, the same sheet, no install — no paths.
    let mut rt = runtime();
    let mut css = CssPipeline::new();
    install(&mut rt, &mut css, SIZED);
    KitBootstrap::new()
        .without_builtin_components()
        .install_builtin_components(&mut rt)
        .expect("opting out never fails");

    let bar = sized(&Progress::contents(0.5, ProgressShape::Line, true, None).nodes()[0]);
    let _ = rt.render_now(&panel(bar), css.sheets());

    assert_eq!(
        drawn_paths(&rt),
        0,
        "a canvas with no painter draws nothing"
    );
}

#[doc = include_str!("app_canvas_painter.doc/a_painter_queued_on_the_control_handle_reaches_the_runtime.md")]
#[test]
fn a_painter_queued_on_the_control_handle_reaches_the_runtime() {
    // The mechanism `App::painter` is one line of: the registration rides the
    // control queue, which the runner drains after the bootstrap has installed
    // its own. Asserted here because `App` itself needs a window and a GPU, and
    // the mechanism does not.
    let control = AppControl::new();
    let runs = Rc::new(Cell::new(0_u32));
    let counter = Rc::clone(&runs);
    control.with_runtime(move |rt| {
        rt.register_named_canvas(
            "sparkline",
            Box::new(move |region, _paint| {
                counter.set(counter.get() + 1);
                let size = region.size();
                region.painter().fill_rect(
                    Rect::new(Point::new(0.0, 0.0), size),
                    Brush::solid(Color::rgba(1.0, 0.0, 0.0, 1.0)),
                );
                false
            }),
        );
    });

    let mut rt = runtime();
    let mut css = CssPipeline::new();
    control.set_stylesheets(vec![
        ".panel{width:300px;height:300px} .bar{width:200px;height:12px;-obscur-painter:sparkline}"
            .to_owned(),
    ]);
    control.apply(&mut rt, &mut css);

    let bar = ViewNode::element(obscur_kit::primitives::Canvas::COMPONENT_ID)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("bar".into())));
    let _ = rt.render_now(&panel(bar), css.sheets());

    assert_eq!(runs.get(), 1, "the painter CSS named ran");
    assert!(drawn_paths(&rt) >= 1, "and what it drew reached the scene");
}

#[doc = include_str!("app_canvas_painter.doc/windowed.md")]
#[cfg(all(feature = "window-winit", feature = "render-vello"))]
mod windowed {
    use super::{CLASS, drawn_paths, panel, runtime};
    use obscur_kit::app::App;
    use obscur_kit::bootstrap::CssPipeline;
    use obscur_kit::geometry::{Point, Rect};
    use obscur_kit::scene::color::Color;
    use obscur_kit::scene::paint::Brush;
    use obscur_kit::view::view::{ViewNode, ViewProperty, ViewValue};

    #[doc = include_str!("app_canvas_painter.doc/SHEET.md")]
    const SHEET: &str =
        ".panel{width:300px;height:300px} .bar{width:200px;height:80px;-obscur-painter:sparkline}";

    #[doc = include_str!("app_canvas_painter.doc/an_app_registers_a_painter_by_name_and_gets_pixels.md")]
    #[test]
    fn an_app_registers_a_painter_by_name_and_gets_pixels() {
        // The claim in the form an application makes it: built through `App`
        // exactly as the documentation says to, and the painter it named reaches
        // the scene. What this cannot call is `run` — it opens a window and a
        // GPU surface — so the two things `run` does with what the builder
        // collected are done here instead, in `run`'s order: the sheets it
        // accumulated, then the control queue, against a real runtime.
        let app = App::new("charts")
            .stylesheet(SHEET)
            .painter("sparkline", |region, _paint| {
                let size = region.size();
                region.painter().fill_rect(
                    Rect::new(Point::new(0.0, 0.0), size),
                    Brush::solid(Color::rgba(0.2, 0.5, 1.0, 1.0)),
                );
                false
            });

        let mut rt = runtime();
        let mut css = CssPipeline::new();
        let control = app.control();
        control.set_stylesheets(vec![SHEET.to_owned()]);
        control.apply(&mut rt, &mut css);

        let bar = ViewNode::element(obscur_kit::primitives::Canvas::COMPONENT_ID)
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("bar".into())));
        let _ = rt.render_now(&panel(bar), css.sheets());

        assert!(
            drawn_paths(&rt) >= 1,
            "an application that never touched `UiRuntime` got pixels from its own painter"
        );
    }
}
