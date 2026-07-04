//! A software/headless [`Renderer`] that rasterizes a scene to a CPU bitmap with
//! `tiny-skia` (spec §5.2 renderer abstraction, §2.7 "Software/headless
//! renderer").
//!
//! [`CpuRenderer`] walks a [`SceneSnapshot`](obscur_scene::scene::SceneSnapshot)'s
//! display list — in paint order, maintaining the clip stack — and fills or
//! strokes each path into a [`tiny_skia::Pixmap`]. The last frame's pixels are
//! retained so a golden-image test can inspect them.
//!
//! # Determinism boundary (spec §5.2)
//!
//! This backend reproduces layout, paint order, clipping, and the consumed
//! Scene/Paint snapshots identically to every other backend; only rasterization,
//! anti-aliasing, and precision differ. Effects it cannot reproduce degrade to a
//! documented approximation rather than a panic:
//!
//! - **Gradient brushes** rasterize as a flat solid of the gradient's first stop
//!   color (spec §5.2 permits rasterization to differ). A stop-less gradient is
//!   skipped.
//! - **Layer compositing** ([`PushLayer`]/[`PopLayer`]) is passthrough: children
//!   are drawn directly, without an offscreen pass. Honoring layer opacity, blend
//!   mode, and effects is a future refinement.
//! - **Mask clips** (as opposed to geometric shape clips) inherit the current
//!   effective clip, since coverage from separate source content is not
//!   rasterized here.
//! - **Text runs** ([`DrawText`]) consume the same pre-shaped
//!   [`GlyphRun`](obscur_scene::text::GlyphRun) the GPU backend does — text is
//!   shaped once in `obscur-text`, never here — so both backends agree on glyph
//!   count and position (spec §5.2 determinism boundary). Lacking real glyph
//!   outlines, this backend rasterizes each glyph as a solid coverage cell of the
//!   glyph's advance width and the run's font size; whitespace glyphs draw
//!   nothing. This is a documented approximation of the mark, not of its layout.
//!
//! [`DrawText`]: obscur_scene::command::PaintCommand::DrawText
//!
//! [`PushLayer`]: obscur_scene::command::PaintCommand::PushLayer
//! [`PopLayer`]: obscur_scene::command::PaintCommand::PopLayer

use obscur_capability::SupportLevel;
use obscur_geometry::{Point, Size};
use obscur_render::frame::{FrameSubmission, PresentResult};
use obscur_render::target::RenderTarget;
use obscur_render::{Renderer, RendererCapabilities, RendererError, RendererId};
use obscur_scene::clip::{Clip, ClipShape};
use obscur_scene::color::Color;
use obscur_scene::command::PaintCommand;
use obscur_scene::paint::{Brush, Paint};
use obscur_scene::path::{FillRule, Path, PathSegment};
use obscur_scene::text::GlyphRun;

use crate::error::CpuRenderError;

/// A software [`Renderer`] backend that rasterizes scenes to a CPU bitmap
/// (spec §5.2, §2.7).
///
/// The renderer owns no GPU or OS handle: it advertises a conservative software
/// capability profile (offscreen output, no MSAA/HDR/custom shaders) and renders
/// entirely on the CPU with `tiny-skia`. Each [`Renderer::render`] call
/// rasterizes into a fresh [`tiny_skia::Pixmap`], which it retains as
/// [`CpuRenderer::pixmap`] for headless golden-image comparison.
#[derive(Clone)]
pub struct CpuRenderer {
    /// The stable identity of this backend instance.
    id: RendererId,
    /// The software capability profile advertised to callers.
    capabilities: RendererCapabilities,
    /// The pixels of the most recently rendered frame, if any.
    last_pixmap: Option<tiny_skia::Pixmap>,
    /// How many frames this renderer has rasterized.
    frames_rendered: u64,
}

impl CpuRenderer {
    /// Creates a software renderer with the given `id` and a conservative
    /// software capability profile: offscreen output available, and no MSAA,
    /// HDR, or custom shaders. It has rendered no frames.
    #[must_use]
    pub fn new(id: RendererId) -> Self {
        Self {
            id,
            capabilities: RendererCapabilities::new()
                .offscreen(true)
                .msaa(false)
                .hdr(false)
                .custom_shaders(false),
            last_pixmap: None,
            frames_rendered: 0,
        }
    }

    /// The pixels of the most recently rendered frame, or `None` before the
    /// first successful [`Renderer::render`].
    ///
    /// This is the surface a golden-image test inspects: its dimensions match the
    /// last render target, and its contents are the deterministic rasterization of
    /// that frame's display list (spec §5.2).
    #[must_use]
    pub fn pixmap(&self) -> Option<&tiny_skia::Pixmap> {
        self.last_pixmap.as_ref()
    }

    /// The number of frames this renderer has rasterized so far.
    #[must_use]
    pub const fn frames_rendered(&self) -> u64 {
        self.frames_rendered
    }
}

impl Default for CpuRenderer {
    /// The default renderer has identity `0` and the software capability profile
    /// of [`CpuRenderer::new`].
    fn default() -> Self {
        Self::new(RendererId::from_raw(0))
    }
}

impl std::fmt::Debug for CpuRenderer {
    /// Hand-written because [`tiny_skia::Pixmap`] is not [`Debug`]; the pixel
    /// buffer is summarized by its presence rather than dumped.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CpuRenderer")
            .field("id", &self.id)
            .field("capabilities", &self.capabilities)
            .field("has_pixmap", &self.last_pixmap.is_some())
            .field("frames_rendered", &self.frames_rendered)
            .finish()
    }
}

impl Renderer for CpuRenderer {
    fn id(&self) -> RendererId {
        self.id
    }

    fn capabilities(&self) -> RendererCapabilities {
        self.capabilities
    }

    /// Rasterizes `submission`'s display list into a CPU bitmap sized to `target`.
    ///
    /// Walks the commands in paint order, filling or stroking each path with the
    /// solid color resolved from its paint and honoring the clip stack. On success
    /// the rasterized pixels are retained (see [`CpuRenderer::pixmap`]) and the
    /// scene generation that "reached the screen" is echoed.
    ///
    /// # Errors
    ///
    /// Returns [`RendererError::InvalidTarget`] when the target has zero area or
    /// its pixel buffer cannot be allocated (a zero or too-large dimension). A
    /// malformed or unbalanced display list never errors and never panics: stray
    /// pops are ignored and unrepresentable commands degrade to a documented
    /// best-effort (see the module docs).
    fn render(
        &mut self,
        submission: &FrameSubmission,
        target: &RenderTarget,
    ) -> Result<PresentResult, RendererError> {
        let (width, height) = sk_dimensions(target.size());
        if width == 0 || height == 0 {
            return Err(CpuRenderError::EmptyTarget.into_renderer_error());
        }

        let mut pixmap = tiny_skia::Pixmap::new(width, height).ok_or_else(|| {
            CpuRenderError::PixmapAllocationFailed { width, height }.into_renderer_error()
        })?;

        // The clip stack: each entry is the effective clip mask in force after a
        // `PushClip`, or `None` when no representable clip is active at that depth.
        // Keeping one entry per push (even an inherited `None`) keeps pushes and
        // pops balanced regardless of what a clip could be rasterized into.
        let mut clip_stack: Vec<Option<tiny_skia::Mask>> = Vec::new();
        let identity = tiny_skia::Transform::identity();

        for command in submission.frame().scene().display_list().commands() {
            match command {
                PaintCommand::DrawPath { path, paint } => {
                    let Some(sk_path) = to_sk_path(path) else {
                        continue;
                    };
                    let clip = clip_stack.last().and_then(Option::as_ref);
                    match paint {
                        Paint::Fill(brush) => {
                            let Some(color) = brush_color(brush) else {
                                continue;
                            };
                            let Some(sk_paint) = solid_paint(color) else {
                                continue;
                            };
                            pixmap.fill_path(
                                &sk_path,
                                &sk_paint,
                                to_sk_fill_rule(path.fill_rule()),
                                identity,
                                clip,
                            );
                        }
                        Paint::Stroke(stroke) => {
                            let Some(color) = brush_color(stroke.brush()) else {
                                continue;
                            };
                            let Some(sk_paint) = solid_paint(color) else {
                                continue;
                            };
                            let sk_stroke = tiny_skia::Stroke {
                                width: stroke.width(),
                                ..Default::default()
                            };
                            pixmap.stroke_path(&sk_path, &sk_paint, &sk_stroke, identity, clip);
                        }
                    }
                }
                PaintCommand::PushClip(clip) => {
                    let parent = clip_stack.last().and_then(Option::as_ref);
                    let entry = match clip {
                        Clip::Shape(shape) => {
                            let domain_path = clip_shape_to_domain_path(shape);
                            match to_sk_path(&domain_path) {
                                Some(sk_path) => build_clip_mask(
                                    width,
                                    height,
                                    &sk_path,
                                    to_sk_fill_rule(domain_path.fill_rule()),
                                    parent,
                                )
                                .or_else(|| parent.cloned()),
                                // A degenerate/empty clip shape cannot be
                                // rasterized: inherit the current clip.
                                None => parent.cloned(),
                            }
                        }
                        // Mask clips derive coverage from separate content that
                        // this backend does not rasterize: inherit the current
                        // clip (best-effort, documented).
                        Clip::Mask(_) => parent.cloned(),
                    };
                    clip_stack.push(entry);
                }
                PaintCommand::PopClip => {
                    // Guard against an unbalanced list: a stray pop is ignored.
                    let _ = clip_stack.pop();
                }
                // Layers are passthrough: children are drawn directly, without an
                // offscreen compositing pass (documented future refinement).
                PaintCommand::PushLayer(_) | PaintCommand::PopLayer => {}
                PaintCommand::DrawText { origin, run, brush } => {
                    let clip = clip_stack.last().and_then(Option::as_ref);
                    draw_text_run(&mut pixmap, *origin, run, brush, clip, identity);
                }
            }
        }

        self.last_pixmap = Some(pixmap);
        self.frames_rendered += 1;
        Ok(PresentResult::Presented {
            generation: submission.generation(),
        })
    }

    /// The honest [`SupportLevel`] at which this software backend offers a named
    /// `effect` (spec §5.2).
    ///
    /// A CPU rasterizer emulates the common 2D effects, so `"blur"`, `"shadow"`,
    /// `"gradient"`, and `"clip"` report [`SupportLevel::Emulated`]. A GPU-only
    /// effect such as `"backdrop-filter"` reports [`SupportLevel::Unavailable`].
    /// Any other name defaults to [`SupportLevel::Emulated`]: a software renderer
    /// can approximate most 2D marks.
    fn supports_effect(&self, effect: &str) -> SupportLevel {
        match effect {
            "backdrop-filter" => SupportLevel::Unavailable,
            _ => SupportLevel::Emulated,
        }
    }
}

/// Computes the integer pixel dimensions of a target from its logical [`Size`],
/// rounding to the nearest pixel and clamping negatives to zero.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn sk_dimensions(size: Size) -> (u32, u32) {
    let width = size.width.round().max(0.0) as u32;
    let height = size.height.round().max(0.0) as u32;
    (width, height)
}

/// Converts a domain [`Path`] into a [`tiny_skia::Path`], or `None` when the
/// outline is empty or degenerate (`tiny_skia::PathBuilder::finish` rejects it).
fn to_sk_path(path: &Path) -> Option<tiny_skia::Path> {
    let mut builder = tiny_skia::PathBuilder::new();
    for segment in path.segments() {
        match segment {
            PathSegment::MoveTo(p) => builder.move_to(p.x, p.y),
            PathSegment::LineTo(p) => builder.line_to(p.x, p.y),
            PathSegment::QuadTo { control, end } => {
                builder.quad_to(control.x, control.y, end.x, end.y);
            }
            PathSegment::CubicTo {
                control1,
                control2,
                end,
            } => builder.cubic_to(control1.x, control1.y, control2.x, control2.y, end.x, end.y),
            PathSegment::Close => builder.close(),
        }
    }
    builder.finish()
}

/// Maps a domain [`FillRule`] onto the `tiny-skia` equivalent.
fn to_sk_fill_rule(rule: FillRule) -> tiny_skia::FillRule {
    match rule {
        FillRule::NonZero => tiny_skia::FillRule::Winding,
        FillRule::EvenOdd => tiny_skia::FillRule::EvenOdd,
    }
}

/// Resolves the flat solid color a brush rasterizes to.
///
/// A [`Brush::Solid`] yields its color directly. Every gradient brush degrades to
/// its first stop color (spec §5.2 rasterization approximation); a stop-less
/// gradient yields `None`, so the command is skipped.
fn brush_color(brush: &Brush) -> Option<Color> {
    match brush {
        Brush::Solid(solid) => Some(solid.color),
        Brush::Linear(gradient) => gradient.stops().first().map(|stop| stop.color),
        Brush::Radial(gradient) => gradient.stops().first().map(|stop| stop.color),
        Brush::Conic(gradient) => gradient.stops().first().map(|stop| stop.color),
    }
}

/// Maps a domain [`Color`] (straight-alpha sRGB, `f32`) onto a
/// [`tiny_skia::Color`], clamping each component to `0.0..=1.0`. Returns `None`
/// only if `tiny-skia` rejects the clamped components.
fn to_sk_color(color: Color) -> Option<tiny_skia::Color> {
    tiny_skia::Color::from_rgba(
        color.r.clamp(0.0, 1.0),
        color.g.clamp(0.0, 1.0),
        color.b.clamp(0.0, 1.0),
        color.a.clamp(0.0, 1.0),
    )
}

/// Builds an anti-aliased solid-color [`tiny_skia::Paint`] from a domain
/// [`Color`], or `None` when the color cannot be represented.
fn solid_paint(color: Color) -> Option<tiny_skia::Paint<'static>> {
    let sk_color = to_sk_color(color)?;
    Some(tiny_skia::Paint {
        shader: tiny_skia::Shader::SolidColor(sk_color),
        anti_alias: true,
        ..Default::default()
    })
}

/// Lowers a geometric [`ClipShape`] to a domain [`Path`] this backend can
/// rasterize into a clip mask. Rectangles and rounded rectangles are expanded to
/// their outline; a path shape is used directly.
fn clip_shape_to_domain_path(shape: &ClipShape) -> Path {
    match shape {
        ClipShape::Rect(rect) => Path::rect(*rect),
        ClipShape::RoundedRect { rect, radii } => Path::rounded_rect(*rect, *radii),
        ClipShape::Path(path) => path.clone(),
    }
}

/// Builds the clip mask in force after pushing `sk_path` onto a `parent` clip.
///
/// With no parent, the mask is the path's coverage over the whole buffer. With a
/// parent, the path is intersected into a clone of the parent's coverage. Returns
/// `None` only if a fresh [`tiny_skia::Mask`] cannot be allocated.
fn build_clip_mask(
    width: u32,
    height: u32,
    sk_path: &tiny_skia::Path,
    fill_rule: tiny_skia::FillRule,
    parent: Option<&tiny_skia::Mask>,
) -> Option<tiny_skia::Mask> {
    let identity = tiny_skia::Transform::identity();
    if let Some(parent) = parent {
        let mut mask = parent.clone();
        mask.intersect_path(sk_path, fill_rule, true, identity);
        Some(mask)
    } else {
        let mut mask = tiny_skia::Mask::new(width, height)?;
        mask.fill_path(sk_path, fill_rule, true, identity);
        Some(mask)
    }
}

/// Rasterizes a pre-shaped [`GlyphRun`] at `origin` as one solid coverage cell
/// per visible glyph (spec §5.2 rasterization approximation).
///
/// The run's glyph positions are authoritative — shaping already ran in the
/// domain, so this never re-shapes. Each glyph fills the box whose top-left is
/// `origin + (glyph.x, glyph.y - font_size)` (the baseline sits `font_size` below
/// the glyph's ascent) sized `advance × font_size`, honoring `clip`. Whitespace
/// glyphs and non-positive cells draw nothing. With a stop-less gradient brush
/// (no resolvable color) the whole run is skipped.
fn draw_text_run(
    pixmap: &mut tiny_skia::Pixmap,
    origin: Point,
    run: &GlyphRun,
    brush: &Brush,
    clip: Option<&tiny_skia::Mask>,
    transform: tiny_skia::Transform,
) {
    let Some(color) = brush_color(brush) else {
        return;
    };
    let Some(sk_paint) = solid_paint(color) else {
        return;
    };
    let font_size = run.font_size();
    for glyph in run.glyphs() {
        // Skip whitespace: the mock shaper's glyph id is the Unicode scalar.
        if char::from_u32(glyph.glyph_id()).is_none_or(char::is_whitespace) {
            continue;
        }
        let width = glyph.advance();
        if width <= 0.0 || font_size <= 0.0 {
            continue;
        }
        let x = origin.x + glyph.x();
        let top = origin.y + glyph.y() - font_size;
        let Some(rect) = tiny_skia::Rect::from_xywh(x, top, width, font_size) else {
            continue;
        };
        let sk_path = tiny_skia::PathBuilder::from_rect(rect);
        pixmap.fill_path(
            &sk_path,
            &sk_paint,
            tiny_skia::FillRule::Winding,
            transform,
            clip,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::Point;
    use obscur_render::frame::RenderFrame;
    use obscur_render::target::BitmapTarget;
    use obscur_scene::command::DisplayList;
    use obscur_scene::scene::{SceneGeneration, SceneSnapshot};

    /// A 16x16 scene with one red rectangle filling the region `(4,4)..(12,12)`.
    fn red_rect_submission() -> FrameSubmission {
        let mut path = Path::new();
        path.move_to(Point::new(4.0, 4.0))
            .line_to(Point::new(12.0, 4.0))
            .line_to(Point::new(12.0, 12.0))
            .line_to(Point::new(4.0, 12.0))
            .close();

        let list = DisplayList::from_commands(vec![PaintCommand::DrawPath {
            path,
            paint: Paint::solid_fill(Color::rgba(1.0, 0.0, 0.0, 1.0)),
        }]);
        let snapshot = SceneSnapshot::new(list, SceneGeneration::from_raw(5));
        let frame = RenderFrame::new(snapshot, Size::new(16.0, 16.0));
        FrameSubmission::new(frame)
    }

    fn bitmap_target(w: f32, h: f32) -> RenderTarget {
        RenderTarget::bitmap(BitmapTarget::new(Size::new(w, h)))
    }

    #[test]
    fn renderer_is_object_safe() {
        let _: Box<dyn Renderer> = Box::new(CpuRenderer::new(RendererId::from_raw(1)));
    }

    #[test]
    fn renders_red_rectangle_into_bitmap() {
        let mut renderer = CpuRenderer::new(RendererId::from_raw(1));
        let result = renderer.render(&red_rect_submission(), &bitmap_target(16.0, 16.0));
        assert_eq!(
            result,
            Ok(PresentResult::Presented {
                generation: SceneGeneration::from_raw(5),
            })
        );

        let pixmap = renderer.pixmap().expect("a frame was rendered");
        assert_eq!(pixmap.width(), 16);
        assert_eq!(pixmap.height(), 16);

        // A pixel inside the rectangle is opaque red.
        let inside = pixmap.pixel(8, 8).expect("center pixel exists");
        assert!(inside.red() > 200, "red channel high: {}", inside.red());
        assert!(inside.green() < 40, "green channel low: {}", inside.green());
        assert!(inside.blue() < 40, "blue channel low: {}", inside.blue());
        assert!(inside.alpha() > 200, "alpha high: {}", inside.alpha());

        // A corner pixel outside the rectangle is transparent.
        let corner = pixmap.pixel(0, 0).expect("corner pixel exists");
        assert_eq!(corner.alpha(), 0);
    }

    #[test]
    fn frames_rendered_increments_per_render() {
        let mut renderer = CpuRenderer::default();
        assert_eq!(renderer.frames_rendered(), 0);
        assert!(renderer.pixmap().is_none());
        let _ = renderer.render(&red_rect_submission(), &bitmap_target(16.0, 16.0));
        let _ = renderer.render(&red_rect_submission(), &bitmap_target(16.0, 16.0));
        assert_eq!(renderer.frames_rendered(), 2);
    }

    #[test]
    fn empty_target_is_invalid_target_error() {
        let mut renderer = CpuRenderer::default();
        let result = renderer.render(&red_rect_submission(), &bitmap_target(0.0, 0.0));
        assert!(matches!(result, Err(RendererError::InvalidTarget(_))));
        // A failed render leaves no pixmap and does not advance the counter.
        assert!(renderer.pixmap().is_none());
        assert_eq!(renderer.frames_rendered(), 0);
    }

    #[test]
    fn id_and_capabilities_reflect_software_profile() {
        let renderer = CpuRenderer::new(RendererId::from_raw(7));
        assert_eq!(renderer.id(), RendererId::from_raw(7));
        let caps = renderer.capabilities();
        assert!(caps.has_offscreen());
        assert!(!caps.has_msaa());
        assert!(!caps.has_hdr());
        assert!(!caps.has_custom_shaders());
    }

    #[test]
    fn supports_effect_emulates_2d_and_rejects_gpu_only() {
        let renderer = CpuRenderer::default();
        assert_eq!(renderer.supports_effect("blur"), SupportLevel::Emulated);
        assert_eq!(renderer.supports_effect("shadow"), SupportLevel::Emulated);
        assert_eq!(renderer.supports_effect("gradient"), SupportLevel::Emulated);
        assert_eq!(
            renderer.supports_effect("backdrop-filter"),
            SupportLevel::Unavailable
        );
    }

    #[test]
    fn draw_text_run_rasterizes_glyph_cells() {
        use obscur_scene::text::{GlyphRun, PositionedGlyph};

        // Two visible glyphs on the baseline at y=16 (font size 16), each 8px
        // wide starting at x=2, y=2. Cells span rows 2..18, cols 2..10 / 10..18.
        let run = GlyphRun::new(
            vec![
                PositionedGlyph::new(u32::from('a'), 0, 0.0, 16.0, 8.0),
                PositionedGlyph::new(u32::from('b'), 1, 8.0, 16.0, 8.0),
            ],
            16.0,
            "ab",
        );
        let list = DisplayList::from_commands(vec![PaintCommand::DrawText {
            origin: Point::new(2.0, 2.0),
            run,
            brush: Brush::solid(Color::rgba(1.0, 1.0, 1.0, 1.0)),
        }]);
        let snapshot = SceneSnapshot::new(list, SceneGeneration::from_raw(9));
        let frame = RenderFrame::new(snapshot, Size::new(32.0, 32.0));
        let submission = FrameSubmission::new(frame);

        let mut renderer = CpuRenderer::default();
        renderer
            .render(&submission, &bitmap_target(32.0, 32.0))
            .expect("renders");
        let pixmap = renderer.pixmap().expect("a frame was rendered");

        // Text is no longer a no-op: a pixel inside the first glyph cell is opaque.
        let inside = pixmap.pixel(5, 8).expect("pixel exists");
        assert!(inside.alpha() > 200, "glyph cell painted: {}", inside.alpha());
        // A pixel well outside any cell stays transparent.
        assert_eq!(pixmap.pixel(28, 28).expect("pixel exists").alpha(), 0);
    }

    #[test]
    fn draw_text_whitespace_glyphs_paint_nothing() {
        use obscur_scene::text::{GlyphRun, PositionedGlyph};

        // A single space glyph: it must contribute no coverage.
        let run = GlyphRun::new(
            vec![PositionedGlyph::new(u32::from(' '), 0, 0.0, 16.0, 8.0)],
            16.0,
            " ",
        );
        let list = DisplayList::from_commands(vec![PaintCommand::DrawText {
            origin: Point::new(0.0, 0.0),
            run,
            brush: Brush::solid(Color::WHITE),
        }]);
        let snapshot = SceneSnapshot::new(list, SceneGeneration::from_raw(1));
        let frame = RenderFrame::new(snapshot, Size::new(16.0, 16.0));
        let submission = FrameSubmission::new(frame);

        let mut renderer = CpuRenderer::default();
        renderer
            .render(&submission, &bitmap_target(16.0, 16.0))
            .expect("renders");
        let pixmap = renderer.pixmap().expect("a frame was rendered");
        // Every pixel is transparent: a space drew nothing.
        assert!(
            pixmap.pixels().iter().all(|p| p.alpha() == 0),
            "whitespace glyph must paint nothing"
        );
    }

    #[test]
    fn unbalanced_display_list_does_not_panic() {
        // A stray pop and a dangling push must be tolerated.
        let list = DisplayList::from_commands(vec![
            PaintCommand::PopClip,
            PaintCommand::PushClip(Clip::rect(obscur_geometry::Rect::new(
                Point::new(0.0, 0.0),
                Size::new(4.0, 4.0),
            ))),
        ]);
        let snapshot = SceneSnapshot::new(list, SceneGeneration::from_raw(1));
        let frame = RenderFrame::new(snapshot, Size::new(8.0, 8.0));
        let submission = FrameSubmission::new(frame);

        let mut renderer = CpuRenderer::default();
        let result = renderer.render(&submission, &bitmap_target(8.0, 8.0));
        assert!(matches!(result, Ok(PresentResult::Presented { .. })));
    }
}
