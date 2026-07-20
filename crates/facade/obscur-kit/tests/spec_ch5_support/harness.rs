//! Shared harness for the `spec_ch5_*` **specification-derived** test files.
//!
//! These tests are NOT implementation-derived. Every expected value in the
//! `spec_ch5_*.rs` files was written by an agent reading the specification
//! (obscur.kit §5 and the W3C specs it normatively cites) **without opening any
//! source in this repository**. This module exists only to give those cases a
//! compiling home; it contains no expectations of its own.
//!
//! Nothing here may be "fixed" to make a case pass. A failing `spec_ch5_*` test
//! is a claim about the implementation, not about the test.
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

/// Compiles authored CSS text into a matchable stylesheet through the real CSS
/// frontend (`source -> parse -> compile`).
#[must_use]
pub fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

/// An element node carrying `class`, with `kids` as its children.
#[must_use]
pub fn el(id: u64, class: &str, kids: Vec<ViewNode>) -> ViewNode {
    ViewNode::element(ComponentId::from_raw(id))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
        .with_children(ViewFragment::from_nodes(kids))
}

/// A text node.
#[must_use]
pub fn txt(content: &str) -> ViewNode {
    ViewNode::text(content)
}

/// A view whose root fragment is `nodes`.
#[must_use]
pub fn view(nodes: Vec<ViewNode>) -> View {
    View::from_fragment(ViewFragment::from_nodes(nodes))
}

/// A runtime with a `w x h` viewport.
#[must_use]
pub fn runtime(w: f32, h: f32) -> UiRuntime {
    UiRuntime::new(RuntimeId::from_raw(1), Size::new(w, h))
}

/// Renders one real frame of `v` under a single authored stylesheet.
#[must_use]
pub fn drive(w: f32, h: f32, css: &str, v: &View) -> UiRuntime {
    let sheet = compile(css);
    let mut rt = runtime(w, h);
    let _ = rt.render_now(v, std::slice::from_ref(&sheet));
    rt
}

/// Walks from the tree root down `path` (child indices) to a [`NodeId`].
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

/// The absolute border-box `(x, y, w, h)` of a node.
#[must_use]
pub fn geom(rt: &UiRuntime, node: NodeId) -> (f32, f32, f32, f32) {
    let b = rt
        .layout_boxes()
        .get(&node)
        .copied()
        .expect("the node was arranged");
    (b.origin().x, b.origin().y, b.size().width, b.size().height)
}

/// Asserts `actual ≈ expected` within a sub-hundredth-pixel tolerance.
pub fn approx(actual: f32, expected: f32, what: &str) {
    assert!(
        (actual - expected).abs() < 1e-2,
        "{what}: expected {expected}, got {actual}"
    );
}

/// The last frame's display list, in paint order.
#[must_use]
pub fn commands(rt: &UiRuntime) -> Vec<PaintCommand> {
    rt.snapshot().display_list().commands().to_vec()
}

/// The last frame's display list with every push/pop bracket removed — the leaf
/// draw operations only, in paint order.
#[must_use]
pub fn leaves(rt: &UiRuntime) -> Vec<PaintCommand> {
    commands(rt)
        .into_iter()
        .filter(|c| !c.is_push() && !c.is_pop())
        .collect()
}

/// A one-line label naming a command's kind (and, for a solid fill, its colour),
/// for readable assertion messages.
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

/// A `(r, g, b)` 0..=255 triple for a scene colour.
#[must_use]
pub fn rgb8_of(color: Color) -> String {
    let q = |c: f32| (c.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("rgb({},{},{})", q(color.r), q(color.g), q(color.b))
}

/// The solid fill colour of a command, if it is one.
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

/// A rasterized frame: straight-alpha RGBA8, one pixel per physical device pixel.
pub struct Raster {
    width: u32,
    height: u32,
    /// Premultiplied RGBA8, as the CPU backend produces it.
    bytes: Vec<u8>,
}

impl Raster {
    /// The straight-alpha `(r, g, b, a)` of the pixel at `(x, y)`.
    ///
    /// The CPU backend's buffer is premultiplied, so this undoes the
    /// premultiplication before reporting a channel — otherwise every assertion
    /// about a colour under a non-opaque alpha would read the wrong number.
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

    /// The raster's width in physical pixels.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// The raster's height in physical pixels.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }
}

/// Rasterizes the runtime's last published scene through the real software
/// backend onto a `w x h` bitmap.
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
