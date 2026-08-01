#![doc = include_str!("spec_ch5_svg.doc/_module.md")]
#![forbid(unsafe_code)]
#![cfg(all(feature = "resource-fs", feature = "render-cpu"))]

#[path = "spec_ch5_support/mod.rs"]
mod support;

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

use obscur_kit::bootstrap::KitBootstrap;
use obscur_kit::geometry::{Point, Rect, Size};
use obscur_kit::identity::{ComponentId, NodeId, RuntimeId};
use obscur_kit::interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};
use obscur_kit::primitives::Image;
use obscur_kit::resources::port::VectorDecoder;
use obscur_kit::resources::state::ResourceError;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::scene::color::Color;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::scene::paint::{Brush, Paint, SceneVector, VectorPaint, VectorShape};
use obscur_kit::scene::path::Path;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewProperty, ViewValue};

use support::{at, compile, rgb8_of};

#[doc = include_str!("spec_ch5_svg.doc/VW.md")]
const VW: f32 = 400.0;

#[doc = include_str!("spec_ch5_svg.doc/COUNTER.md")]
static COUNTER: AtomicU32 = AtomicU32::new(0);

#[doc = include_str!("spec_ch5_svg.doc/ICON.md")]
const ICON: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20"
  viewBox="0 0 20 20"><circle cx="10" cy="10" r="10" fill="currentColor"/></svg>"##;

#[doc = include_str!("spec_ch5_svg.doc/CountingDecoder.md")]
struct CountingDecoder {
    parses: Arc<AtomicUsize>,
}

impl VectorDecoder for CountingDecoder {
    fn decode_vector(&self, _bytes: &[u8]) -> Result<SceneVector, ResourceError> {
        self.parses.fetch_add(1, Ordering::Relaxed);
        // A 20x20 document holding one `currentColor` disc, approximated as a
        // closed path so the assertion is about command KIND, not curve fidelity.
        let mut path = Path::new();
        path.move_to(Point::new(10.0, 0.0));
        path.cubic_to(
            Point::new(20.0, 0.0),
            Point::new(20.0, 20.0),
            Point::new(10.0, 20.0),
        );
        path.cubic_to(
            Point::new(0.0, 20.0),
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
        );
        path.close();
        Ok(SceneVector::new(
            Size::new(20.0, 20.0),
            vec![VectorShape {
                path,
                fill: Some(VectorPaint::CurrentColor),
                stroke: None,
            }],
        ))
    }
}

#[doc = include_str!("spec_ch5_svg.doc/asset_dir.md")]
fn asset_dir() -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("obscur-kit-ch5-svg-{}-{n}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    std::fs::write(dir.join("icon.svg"), ICON).expect("write svg");
    dir
}

#[doc = include_str!("spec_ch5_svg.doc/icon.md")]
fn icon(class: &str) -> obscur_kit::view::view::ViewNode {
    Image::new("icon.svg")
        .into_view_node()
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
}

#[doc = include_str!("spec_ch5_svg.doc/fills.md")]
fn fills(runtime: &UiRuntime) -> Vec<String> {
    runtime
        .snapshot()
        .display_list()
        .commands()
        .iter()
        .filter_map(|c| match c {
            PaintCommand::DrawPath {
                paint: Paint::Fill(Brush::Solid(solid)),
                ..
            } => Some(rgb8_of(solid.color())),
            _ => None,
        })
        .collect()
}

#[doc = include_str!("spec_ch5_svg.doc/sv_01_current_color_resolves_per_node_without_reparsing.md")]
#[test]
fn sv_01_current_color_resolves_per_node_without_reparsing() {
    let dir = asset_dir();
    let parses = Arc::new(AtomicUsize::new(0));

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VW, VW));
    KitBootstrap::new().install_image_resources(&mut runtime, &dir);
    runtime.set_vector_decoder(Arc::new(CountingDecoder {
        parses: Arc::clone(&parses),
    }));

    let sheet = compile(
        ".a{position:absolute;left:0;top:0;width:20px;height:20px;color:rgb(255,0,0)}\
         .a:hover{color:rgb(0,0,255)}\
         .b{position:absolute;left:40px;top:0;width:20px;height:20px;color:rgb(0,128,0)}",
    );
    let view = View::from_fragment(ViewFragment::from_nodes(vec![icon("a"), icon("b")]));

    // Frame 1 — no hover. One parsed document, two different rendered colours.
    // With no blocking executor the vector load is deferred out of the frame
    // phase (§3.4/§5.17); drive it, then re-render so the parsed document composes.
    let _ = runtime.render_now(&view, std::slice::from_ref(&sheet));
    let _ = runtime.poll_resource_tasks();
    let _ = runtime.render_now(&view, std::slice::from_ref(&sheet));
    let frame1 = fills(&runtime);
    assert!(
        frame1.contains(&"rgb(255,0,0)".to_owned()),
        "node A fills with its own computed color; fills were {frame1:?}"
    );
    assert!(
        frame1.contains(&"rgb(0,128,0)".to_owned()),
        "node B fills with its own computed color; fills were {frame1:?}"
    );

    // Frame 2 — the pointer over node A.
    let a: NodeId = at(&runtime, &[0]);
    let b_box = runtime.layout_boxes().get(&a).copied().expect("arranged");
    runtime.handle_pointer(PointerEvent::new(
        PointerId::MOUSE,
        PointerDevice::Mouse,
        PointerEventKind::Move,
        Point::new(
            b_box.origin().x + b_box.size().width / 2.0,
            b_box.origin().y + b_box.size().height / 2.0,
        ),
    ));
    let _ = runtime.render_now(&view, std::slice::from_ref(&sheet));
    let frame2 = fills(&runtime);
    assert!(
        frame2.contains(&"rgb(0,0,255)".to_owned()),
        "the :hover rule changes node A's fill without reparsing; fills were {frame2:?}"
    );
    assert!(
        frame2.contains(&"rgb(0,128,0)".to_owned()),
        "node B is unchanged; fills were {frame2:?}"
    );

    assert_eq!(
        parses.load(Ordering::Relaxed),
        1,
        "the document is parsed EXACTLY once across both frames and both colours"
    );
}

#[doc = include_str!("spec_ch5_svg.doc/sv_02_an_svg_under_scale_stays_vector.md")]
#[test]
fn sv_02_an_svg_under_scale_stays_vector() {
    let dir = asset_dir();
    let parses = Arc::new(AtomicUsize::new(0));
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VW, VW));
    KitBootstrap::new().install_image_resources(&mut runtime, &dir);
    runtime.set_vector_decoder(Arc::new(CountingDecoder {
        parses: Arc::clone(&parses),
    }));

    let sheet = compile(
        ".scaled{position:absolute;left:0;top:0;transform:scale(8);transform-origin:0 0}\
         .icon{width:20px;height:20px;color:rgb(255,0,0)}",
    );
    let group = obscur_kit::view::view::ViewNode::element(ComponentId::from_raw(0xC1))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("scaled".into())))
        .with_children(ViewFragment::from_nodes(vec![icon("icon")]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![group]));

    // No executor: the vector load is deferred out of the frame phase
    // (§3.4/§5.17); drive it, then re-render so the vector reaches the scene.
    let _ = runtime.render_now(&view, std::slice::from_ref(&sheet));
    let _ = runtime.poll_resource_tasks();
    let _ = runtime.render_now(&view, std::slice::from_ref(&sheet));
    let list = runtime.snapshot().display_list().commands().to_vec();

    let paths = list
        .iter()
        .filter(|c| matches!(c, PaintCommand::DrawPath { .. }))
        .count();
    let images = list
        .iter()
        .filter(|c| matches!(c, PaintCommand::DrawImage { .. }))
        .count();
    assert!(
        paths > 0,
        "the icon reaches the scene as path fills/strokes"
    );
    assert_eq!(
        images, 0,
        "and as ZERO image/texture draws — a pre-scaled raster reads as an \
         anti-aliasing difference, which §5.2 permits, so only this assertion \
         separates them"
    );

    // The factor 8 is carried by a transform command, not baked into geometry:
    // the path stays in the node's own 20x20 local units.
    assert!(
        list.iter()
            .any(|c| matches!(c, PaintCommand::PushTransform(_))),
        "the scale is a Transform command"
    );
    let bounds = list.iter().find_map(|c| match c {
        PaintCommand::DrawPath { path, .. } => path.bounds(),
        _ => None,
    });
    let bounds: Rect = bounds.expect("the painted path has bounds");
    assert!(
        bounds.size.width <= 20.0 + 1e-3 && bounds.size.height <= 20.0 + 1e-3,
        "the geometry stays in the node's local 20x20 units, got {bounds:?}"
    );

    // Sanity: a real colour was substituted, so this is not a vacuously empty scene.
    assert!(
        fills(&runtime).contains(&rgb8_of(Color::rgb(1.0, 0.0, 0.0))),
        "the disc is filled with the computed color"
    );
}
