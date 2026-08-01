#![doc = include_str!("harness.doc/_module.md")]
#![allow(dead_code, unreachable_pub)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, NodeId, RuntimeId};
use obscur_kit::render::frame::{FrameSubmission, RenderFrame};
use obscur_kit::render::target::{BitmapTarget, RenderTarget};
use obscur_kit::render::{Renderer, RendererId};
use obscur_kit::render_cpu::renderer::CpuRenderer;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::scene::color::Color;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::scene::paint::{Brush, Paint};
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("harness.doc/compile.md")]
#[must_use]
pub fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
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

#[doc = include_str!("harness.doc/txt.md")]
#[must_use]
pub fn txt(content: &str) -> ViewNode {
    ViewNode::text(content)
}

#[doc = include_str!("harness.doc/view.md")]
#[must_use]
pub fn view(nodes: Vec<ViewNode>) -> View {
    View::from_fragment(ViewFragment::from_nodes(nodes))
}

#[doc = include_str!("harness.doc/runtime.md")]
#[must_use]
pub fn runtime(w: f32, h: f32) -> UiRuntime {
    UiRuntime::new(RuntimeId::from_raw(1), Size::new(w, h))
}

#[doc = include_str!("harness.doc/drive.md")]
#[must_use]
pub fn drive(w: f32, h: f32, css: &str, v: &View) -> UiRuntime {
    let sheet = compile(css);
    let mut rt = runtime(w, h);
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
        assert!(
            i < kids.len(),
            "child index {i} out of range: node has {} children",
            kids.len()
        );
        node = kids[i];
    }
    node
}

#[doc = include_str!("harness.doc/geom.md")]
#[must_use]
pub fn geom(rt: &UiRuntime, node: NodeId) -> (f32, f32, f32, f32) {
    let b = rt
        .layout_boxes()
        .get(&node)
        .copied()
        .expect("the node was arranged");
    (b.origin().x, b.origin().y, b.size().width, b.size().height)
}

#[doc = include_str!("harness.doc/approx.md")]
pub fn approx(actual: f32, expected: f32, what: &str) {
    assert!(
        (actual - expected).abs() < 1e-2,
        "{what}: expected {expected}, got {actual}"
    );
}

#[doc = include_str!("harness.doc/commands.md")]
#[must_use]
pub fn commands(rt: &UiRuntime) -> Vec<PaintCommand> {
    rt.snapshot().display_list().commands().to_vec()
}

#[doc = include_str!("harness.doc/leaves.md")]
#[must_use]
pub fn leaves(rt: &UiRuntime) -> Vec<PaintCommand> {
    commands(rt)
        .into_iter()
        .filter(|c| !c.is_push() && !c.is_pop())
        .collect()
}

#[doc = include_str!("harness.doc/describe.md")]
#[must_use]
pub fn describe(command: &PaintCommand) -> String {
    match command {
        PaintCommand::DrawPath {
            paint: Paint::Fill(Brush::Solid(solid)),
            ..
        } => format!("fill {}", rgb8_of(solid.color())),
        PaintCommand::DrawPath {
            paint: Paint::Fill(brush),
            ..
        } => format!("fill {brush:?}"),
        PaintCommand::DrawPath {
            paint: Paint::Stroke(stroke),
            ..
        } => match stroke.brush() {
            Brush::Solid(solid) => format!("stroke {}", rgb8_of(solid.color())),
            other => format!("stroke {other:?}"),
        },
        PaintCommand::DrawText { run, .. } => format!("text {:?}", run.text()),
        PaintCommand::DrawShadow { color, inset, .. } => {
            let kind = if *inset {
                "inset shadow"
            } else {
                "drop shadow"
            };
            format!("{kind} {}", rgb8_of(*color))
        }
        PaintCommand::DrawImage { .. } => "image".to_owned(),
        PaintCommand::BackdropFilter { .. } => "backdrop-filter".to_owned(),
        PaintCommand::PushTransform(_) => "push-transform".to_owned(),
        PaintCommand::PopTransform => "pop-transform".to_owned(),
        PaintCommand::PushClip(_) => "push-clip".to_owned(),
        PaintCommand::PopClip => "pop-clip".to_owned(),
        PaintCommand::PushLayer(_) => "push-layer".to_owned(),
        PaintCommand::PopLayer => "pop-layer".to_owned(),
    }
}

#[doc = include_str!("harness.doc/rgb8_of.md")]
#[must_use]
pub fn rgb8_of(color: Color) -> String {
    let q = |c: f32| (c.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("rgb({},{},{})", q(color.r), q(color.g), q(color.b))
}

#[doc = include_str!("harness.doc/fill_color.md")]
#[must_use]
pub fn fill_color(command: &PaintCommand) -> Option<Color> {
    match command {
        PaintCommand::DrawPath {
            paint: Paint::Fill(Brush::Solid(solid)),
            ..
        } => Some(solid.color()),
        _ => None,
    }
}

#[doc = include_str!("harness.doc/Raster.md")]
pub struct Raster {
    width: u32,
    height: u32,
    #[doc = include_str!("harness.doc/bytes.md")]
    bytes: Vec<u8>,
}

impl Raster {
    #[doc = include_str!("harness.doc/at_2.md")]
    #[must_use]
    pub fn at(&self, x: u32, y: u32) -> (u8, u8, u8, u8) {
        assert!(
            x < self.width && y < self.height,
            "probe ({x},{y}) is outside the {}x{} raster",
            self.width,
            self.height
        );
        let i = ((y * self.width + x) * 4) as usize;
        let (r, g, b, a) = (
            self.bytes[i],
            self.bytes[i + 1],
            self.bytes[i + 2],
            self.bytes[i + 3],
        );
        if a == 0 || a == 255 {
            return (r, g, b, a);
        }
        let un = |c: u8| ((f32::from(c) * 255.0 / f32::from(a)).round()).clamp(0.0, 255.0) as u8;
        (un(r), un(g), un(b), a)
    }

    #[doc = include_str!("harness.doc/width.md")]
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    #[doc = include_str!("harness.doc/height.md")]
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }
}

#[doc = include_str!("harness.doc/rasterize.md")]
#[must_use]
pub fn rasterize(rt: &UiRuntime, w: f32, h: f32) -> Raster {
    let mut renderer = CpuRenderer::new(RendererId::from_raw(9));
    let frame = RenderFrame::new(rt.snapshot(), Size::new(w, h));
    let submission = FrameSubmission::new(frame);
    let target = RenderTarget::bitmap(BitmapTarget::new(Size::new(w, h)));
    renderer.render(&submission, &target).expect("a CPU render");
    let pixmap = renderer.pixmap().expect("a rasterized frame");
    Raster {
        width: pixmap.width(),
        height: pixmap.height(),
        bytes: pixmap.data().to_vec(),
    }
}
