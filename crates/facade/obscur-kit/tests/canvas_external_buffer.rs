#![doc = include_str!("canvas_external_buffer.doc/_module.md")]
#![forbid(unsafe_code)]
#![cfg(feature = "render-cpu")]

use std::cell::Cell;
use std::rc::Rc;

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, InternedSymbol, RuntimeId};
use obscur_kit::render::frame::{FrameSubmission, RenderFrame};
use obscur_kit::render::target::{BitmapTarget, RenderTarget};
use obscur_kit::render::{Renderer, RendererId};
use obscur_kit::render_cpu::renderer::CpuRenderer;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::scene::color::AlphaMode;
use obscur_kit::scene::paint::{BufferFit, BufferFormat, ExternalBuffer};
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{
    LOCAL_NAME, View, ViewNode, ViewProperty, ViewValue, canvas_painter_prop,
};

const VIEWPORT: f32 = 64.0;
const CAMERA: InternedSymbol = InternedSymbol::from_raw(0x0E01);
const PANEL: ComponentId = ComponentId::from_raw(0x00C0_000A);

/// A 2×2 BGRA frame whose rows are padded to three pixels — what a real capture
/// hands over. Two rows, deliberately: with one row a stride cannot go wrong,
/// and a test that cannot fail proves nothing.
///
/// ```text
///   red    blue   | black padding
///   green  white  | black padding
/// ```
const CAPTURE: [u8; 24] = [
    0, 0, 255, 255, // b=0 g=0 r=255 → red
    255, 0, 0, 255, // b=255 g=0 r=0 → blue
    0, 0, 0, 255, // padding, must never be drawn
    0, 255, 0, 255, // green
    255, 255, 255, 255, // white
    0, 0, 0, 255, // padding
];

const CSS: &str = "camera{width:40px;height:20px;position:absolute;left:0;top:0}";

fn compile(css: &str) -> Stylesheet {
    let parsed = StylesheetParser::new(&CssSource::anonymous(css)).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("canvas_external_buffer.doc/rasterized.md")]
fn rasterized(fit: BufferFit) -> (obscur_kit::render_cpu::renderer::CpuRenderer, Rc<Cell<u32>>) {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    let frames = Rc::new(Cell::new(0));
    let counter = Rc::clone(&frames);
    rt.register_canvas(
        CAMERA,
        Box::new(move |region, _paint| {
            counter.set(counter.get() + 1);
            let buffer =
                ExternalBuffer::new(&CAPTURE, 2, 2, 12, BufferFormat::Bgra8, AlphaMode::Straight)
                    .expect("a valid frame");
            region.draw_buffer(&buffer, fit);
            // A live source keeps asking for frames; this one is a still.
            false
        }),
    );

    let view = View::single(
        ViewNode::element(PANEL)
            .with_prop(ViewProperty::new(
                LOCAL_NAME,
                ViewValue::Text("camera".into()),
            ))
            .with_prop(canvas_painter_prop(CAMERA)),
    );
    let snapshot = rt
        .render_now(&view, std::slice::from_ref(&compile(CSS)))
        .snapshot()
        .clone();

    let mut renderer = CpuRenderer::new(RendererId::from_raw(2));
    let submission =
        FrameSubmission::new(RenderFrame::new(snapshot, Size::new(VIEWPORT, VIEWPORT)));
    let target = RenderTarget::bitmap(BitmapTarget::new(Size::new(VIEWPORT, VIEWPORT)));
    renderer.render(&submission, &target).expect("cpu render");
    (renderer, frames)
}

#[doc = include_str!("canvas_external_buffer.doc/a_foreign_frame_reaches_real_pixels_with_its_colours_intact.md")]
#[test]
fn a_foreign_frame_reaches_real_pixels_with_its_colours_intact() {
    // The whole road: an application's own bytes, in the layout a camera
    // actually produces, through a painter, through the display list, through a
    // real rasterizer, onto pixels.
    let (renderer, frames) = rasterized(BufferFit::Stretch);
    assert!(frames.get() >= 1, "the painter ran");
    let pixmap = renderer.pixmap().expect("a rasterized frame");

    // The 2×2 frame is stretched across a 40×20 box, so each source pixel owns
    // one quarter of it.
    let top_left = pixmap.pixel(9, 4).expect("inside the box");
    let top_right = pixmap.pixel(30, 4).expect("inside the box");
    assert!(
        top_left.red() > 200 && top_left.blue() < 60,
        "the first pixel is RED — the channels were not swapped: {top_left:?}"
    );
    assert!(
        top_right.blue() > 200 && top_right.red() < 60,
        "and the second BLUE: {top_right:?}"
    );
}

#[doc = include_str!("canvas_external_buffer.doc/row_padding_never_reaches_the_screen.md")]
#[test]
fn row_padding_never_reaches_the_screen() {
    // The trap this whole type exists to close. Read the padded buffer as a
    // tight one and the second row starts four bytes early: it becomes
    // black-then-green instead of green-then-white, and every row after it
    // slides further sideways.
    let (renderer, _) = rasterized(BufferFit::Stretch);
    let pixmap = renderer.pixmap().expect("a rasterized frame");

    let bottom_left = pixmap.pixel(9, 15).expect("inside the box");
    let bottom_right = pixmap.pixel(30, 15).expect("inside the box");
    assert!(
        bottom_left.green() > 200 && bottom_left.red() < 60,
        "the second row starts at its own pixel, not at the padding: {bottom_left:?}"
    );
    assert!(
        bottom_right.red() > 200 && bottom_right.green() > 200 && bottom_right.blue() > 200,
        "and ends on the white one: {bottom_right:?}"
    );
}

#[doc = include_str!("canvas_external_buffer.doc/a_contained_frame_is_letterboxed_inside_its_box.md")]
#[test]
fn a_contained_frame_is_letterboxed_inside_its_box() {
    // A square frame in a 40×20 box: `Contain` scales it to 20×20 and centres
    // it, so the picture occupies x 10..30 and the two ends of the box stay
    // empty. That emptiness is the assertion — a `Stretch` would have filled it.
    let (renderer, _) = rasterized(BufferFit::Contain);
    let pixmap = renderer.pixmap().expect("a rasterized frame");

    assert!(
        pixmap.pixel(14, 4).expect("inside the picture").red() > 200,
        "the picture is where the letterbox puts it"
    );
    assert!(pixmap.pixel(25, 4).expect("inside the picture").blue() > 200);
    assert_eq!(
        pixmap.pixel(3, 10).expect("inside the box").alpha(),
        0,
        "and the left bar is empty rather than stretched into"
    );
}
