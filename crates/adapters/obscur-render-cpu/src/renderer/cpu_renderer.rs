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
//! - **Layer compositing** ([`PushLayer`]/[`PopLayer`]) honors **group opacity**:
//!   a layer with opacity `< 1.0` (a CSS `opacity` group) rasterizes its whole
//!   subtree into a fresh offscreen [`tiny_skia::Pixmap`] and composites it back
//!   at that opacity, so overlapping opaque children blend as one group (not
//!   double-darkened per element) and nested layers multiply. A trivial layer
//!   (opacity `1.0`) stays a cheap passthrough — children draw directly, so the
//!   plain path is byte-identical. The layer's **clip, blend mode, transform, and
//!   effects** are still *not* honored here (only its opacity) — a documented
//!   approximation; `filter` / `backdrop-filter` remain absent.
//! - **Mask clips** (as opposed to geometric shape clips) inherit the current
//!   effective clip, since coverage from separate source content is not
//!   rasterized here.
//! - **Box shadows** ([`DrawShadow`]) rasterize as a flat, offset rounded rect at
//!   reduced alpha (no gaussian blur; the `blur` radius is ignored) — a documented
//!   approximation of the soft shadow the GPU backend blurs. A CSS
//!   `filter: drop-shadow(...)` reaches the same [`DrawShadow`] path (the
//!   coordinator emits a drop shadow of the border box), so it rasterizes
//!   identically; the rest of `filter` / `backdrop-filter` (content blur,
//!   color-matrix) remain absent.
//! - **Images** ([`DrawImage`]) blit the real decoded RGBA8 pixels: the buffer is
//!   premultiplied (as `tiny-skia` stores) if the [`SceneImage`] is straight-alpha,
//!   wrapped into a source [`tiny_skia::Pixmap`], and drawn with `draw_pixmap` under
//!   a transform that scales the image's intrinsic size to each copy's tile rect.
//!   This is a genuine sampled bitmap, not an approximation, so both backends show
//!   the same picture. The tile is placed (`background-position`), sized
//!   (`background-size`), clipped to the paint region (the border box), and tiled
//!   along the `repeat_x` / `repeat_y` axes (`background-repeat`) with a bounded
//!   loop; a source sub-rectangle and nine-slice are not modeled yet.
//!
//! [`DrawImage`]: obscur_scene::command::PaintCommand::DrawImage
//!
//! - **Text runs** ([`DrawText`]) consume the same pre-shaped
//!   [`GlyphRun`](obscur_scene::text::GlyphRun) the GPU backend does — text is
//!   shaped once in `obscur-text`, never here — so both backends agree on glyph
//!   count and position (spec §5.2 determinism boundary). When the run's font has
//!   been registered with [`CpuRenderer::register_font`] (mirroring the Vello
//!   backend's registry), each glyph's real outline is extracted with `skrifa`,
//!   scaled to the run's font size, placed on its baseline, and filled — honest
//!   glyph shapes, deterministic for a given font. When no font is registered for
//!   the run's handle, the backend falls back to a solid coverage cell of the
//!   glyph's advance width and the run's font size (whitespace glyphs draw
//!   nothing) — a documented approximation of the mark so device-free callers
//!   still get a defined result.
//!
//! [`DrawText`]: obscur_scene::command::PaintCommand::DrawText
//!
//! [`DrawShadow`]: obscur_scene::command::PaintCommand::DrawShadow
//!
//! [`PushLayer`]: obscur_scene::command::PaintCommand::PushLayer
//! [`PopLayer`]: obscur_scene::command::PaintCommand::PopLayer

use std::collections::HashMap;
use std::sync::Arc;

use obscur_capability::SupportLevel;
use obscur_geometry::{AffineTransform, CornerRadii, Point, Rect, Size};
use skrifa::instance::{LocationRef, Size as SkrifaSize};
use skrifa::outline::DrawSettings;
use skrifa::{FontRef, GlyphId, MetadataProvider};
use obscur_render::frame::{FrameSubmission, PresentResult};
use obscur_render::target::RenderTarget;
use obscur_render::{Renderer, RendererCapabilities, RendererError, RendererId};
use obscur_scene::clip::{Clip, ClipShape};
use obscur_scene::color::{AlphaMode, Color};
use obscur_scene::command::PaintCommand;
use obscur_scene::paint::{Brush, Paint, SceneImage};
use obscur_scene::path::{FillRule, Path, PathSegment};
use obscur_scene::text::GlyphRun;

use crate::error::CpuRenderError;
use crate::renderer::glyph_pen::GlyphPathPen;

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
    /// Host-registered font faces keyed by the run font handle (spec §5.2). A run
    /// tagged with handle `run.font()` is rasterized against the matching sfnt
    /// bytes; when no font is registered for the handle the run falls back to
    /// box-coverage cells. The bytes are shared (`Arc`) so `clone` is cheap and a
    /// borrowed [`FontRef`] is rebuilt per draw (bytes must outlive the borrow).
    fonts: HashMap<u32, Arc<[u8]>>,
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
            fonts: HashMap::new(),
        }
    }

    /// Registers (or replaces) the host-provided sfnt `font_bytes` for the run
    /// font handle `id` (spec §5.2).
    ///
    /// A [`DrawText`](obscur_scene::command::PaintCommand::DrawText) run tagged
    /// with [`GlyphRun::font`]`() == id` is then rasterized as real glyph
    /// outlines against these bytes; runs whose handle has no registered font
    /// fall back to box-coverage cells. Mirrors the Vello backend's
    /// `register_font(id, bytes)` shape so a host registers the same id↔bytes into
    /// both and the two backends draw the same faces. The bytes are stored (shared
    /// via [`Arc`]) and a borrowed [`FontRef`] is rebuilt per draw.
    pub fn register_font(&mut self, id: u32, font_bytes: Vec<u8>) {
        self.fonts.insert(id, Arc::from(font_bytes.into_boxed_slice()));
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
            .field("registered_fonts", &self.fonts.len())
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

        let root_pixmap = tiny_skia::Pixmap::new(width, height).ok_or_else(|| {
            CpuRenderError::PixmapAllocationFailed { width, height }.into_renderer_error()
        })?;

        // The layer/target stack: every fill / text / shadow op draws into the
        // *top* target. The root frame buffer is the bottom entry; a non-trivial
        // `PushLayer` (a CSS `opacity` group) pushes a fresh transparent target of
        // the same size, so the whole group rasterizes offscreen, and the matching
        // `PopLayer` composites it back onto the parent at the layer's opacity —
        // one group blend, so overlapping opaque children are NOT double-darkened
        // (real group compositing, not per-element alpha). Trivial layers stay a
        // cheap passthrough (no offscreen), keeping the plain path byte-identical.
        let mut targets: Vec<tiny_skia::Pixmap> = vec![root_pixmap];
        // One entry per open `PushLayer`: the layer's opacity when it pushed an
        // offscreen target to composite on `PopLayer`, or `None` for a trivial
        // (passthrough) layer that pushed nothing. Keeps pushes and pops balanced
        // regardless of whether a layer needed an offscreen pass.
        let mut layer_stack: Vec<Option<f32>> = Vec::new();

        // The clip stack: each entry is the effective clip mask in force after a
        // `PushClip`, or `None` when no representable clip is active at that depth.
        // Keeping one entry per push (even an inherited `None`) keeps pushes and
        // pops balanced regardless of what a clip could be rasterized into.
        let mut clip_stack: Vec<Option<tiny_skia::Mask>> = Vec::new();
        let identity = tiny_skia::Transform::identity();
        // The active CSS 2D `transform` stack: each `PushTransform` composes its
        // affine onto the running transform, and every path / glyph / shadow drawn
        // (and every clip mask built) until the matching `PopTransform` is mapped
        // by the top of the stack. Empty (identity) when no transform is active, so
        // an untransformed scene rasterizes byte-for-byte as before.
        let mut transform_stack: Vec<tiny_skia::Transform> = Vec::new();

        for command in submission.frame().scene().display_list().commands() {
            let current = transform_stack.last().copied().unwrap_or(identity);
            match command {
                PaintCommand::DrawPath { path, paint } => {
                    let Some(sk_path) = to_sk_path(path) else {
                        continue;
                    };
                    let clip = clip_stack.last().and_then(Option::as_ref);
                    let Some(pixmap) = targets.last_mut() else {
                        continue;
                    };
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
                                current,
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
                                // A non-empty dash pattern rasterizes as dashes;
                                // `StrokeDash::new` rejects an odd/degenerate array
                                // (returns `None`), which leaves a solid stroke —
                                // the same result as an un-dashed stroke. An empty
                                // pattern skips this entirely, byte-identical.
                                dash: if stroke.is_dashed() {
                                    tiny_skia::StrokeDash::new(
                                        stroke.dash().to_vec(),
                                        stroke.dash_offset(),
                                    )
                                } else {
                                    None
                                },
                                ..Default::default()
                            };
                            pixmap.stroke_path(&sk_path, &sk_paint, &sk_stroke, current, clip);
                        }
                    }
                }
                PaintCommand::PushTransform(transform) => {
                    transform_stack.push(current.pre_concat(to_sk_transform(transform)));
                }
                PaintCommand::PopTransform => {
                    let _ = transform_stack.pop();
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
                                    current,
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
                // A CSS `opacity` group layer. A layer with reduced opacity pushes
                // a fresh transparent offscreen target the whole group rasterizes
                // into; `PopLayer` composites it back at that opacity, so the group
                // blends as one (overlapping opaque children are not double-
                // darkened). A trivial layer (opacity `1.0`) stays a passthrough:
                // it pushes no offscreen, so the plain path is byte-identical. The
                // layer's own clip / blend-mode / transform / effect fields are not
                // honored here (only its opacity) — a documented approximation.
                PaintCommand::PushLayer(layer) => {
                    if layer.opacity() < 1.0 {
                        match targets.last().and_then(|top| {
                            tiny_skia::Pixmap::new(top.width(), top.height())
                        }) {
                            Some(offscreen) => {
                                targets.push(offscreen);
                                layer_stack.push(Some(layer.opacity().clamp(0.0, 1.0)));
                            }
                            // Allocation failed: fall back to passthrough so the
                            // group still paints (at full opacity) and the list
                            // stays balanced.
                            None => layer_stack.push(None),
                        }
                    } else {
                        layer_stack.push(None);
                    }
                }
                PaintCommand::PopLayer => {
                    // Balance a stray pop; composite the offscreen when this layer
                    // opened one.
                    if let Some(Some(opacity)) = layer_stack.pop() {
                        composite_top_layer(&mut targets, opacity);
                    }
                }
                PaintCommand::DrawText { origin, run, brush } => {
                    let clip = clip_stack.last().and_then(Option::as_ref);
                    let Some(pixmap) = targets.last_mut() else {
                        continue;
                    };
                    draw_text_run(pixmap, *origin, run, brush, clip, current, &self.fonts);
                }
                PaintCommand::DrawShadow {
                    rect,
                    radii,
                    color,
                    blur: _,
                    spread,
                    offset,
                    inset,
                } => {
                    let clip = clip_stack.last().and_then(Option::as_ref);
                    let Some(pixmap) = targets.last_mut() else {
                        continue;
                    };
                    draw_shadow_approx(
                        pixmap,
                        *rect,
                        *radii,
                        *color,
                        *spread,
                        *offset,
                        *inset,
                        clip,
                        current,
                    );
                }
                PaintCommand::DrawImage {
                    image,
                    tile,
                    region,
                    repeat_x,
                    repeat_y,
                } => {
                    let clip = clip_stack.last().and_then(Option::as_ref);
                    let Some(pixmap) = targets.last_mut() else {
                        continue;
                    };
                    draw_image(pixmap, image, *tile, *region, *repeat_x, *repeat_y, clip, current);
                }
                // `backdrop-filter` (frosted glass) is a GPU-class effect: it needs
                // to blur the already-composited frame in place, which the software
                // rasterizer has no pass for. The CPU backend honestly skips it (the
                // panel's own translucent background still paints over the SHARP
                // backdrop) rather than faking a blur — a documented limitation, this
                // effect is Vello-only (spec §5.2, CPU/GPU effect split).
                PaintCommand::BackdropFilter { .. } => {}
            }
        }

        // Unwind any layer left open by an unbalanced list: composite each dangling
        // offscreen back at its opacity so its content is not lost, leaving only the
        // root frame buffer.
        while let Some(entry) = layer_stack.pop() {
            if let Some(opacity) = entry {
                composite_top_layer(&mut targets, opacity);
            }
        }
        // The root frame buffer is the only remaining target (a well-formed list
        // leaves exactly one; the unwind above guarantees it for a malformed one).
        let pixmap = targets.pop().unwrap_or_else(|| {
            // Unreachable in practice (the root is never popped), but keep the
            // renderer panic-free: fall back to a fresh empty buffer.
            tiny_skia::Pixmap::new(width, height)
                .unwrap_or_else(|| tiny_skia::Pixmap::new(1, 1).expect("1x1 pixmap allocates"))
        });

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
    /// `"gradient"`, and `"clip"` report [`SupportLevel::Emulated`]. A
    /// `filter: drop-shadow(...)` renders through the same `DrawShadow` path as a
    /// `box-shadow` (a flat offset reduced-alpha rect), so `"drop-shadow"` is also
    /// [`SupportLevel::Emulated`]. The general `filter` / `backdrop-filter`
    /// (content blur, color-matrix) have no software approximation here and report
    /// [`SupportLevel::Unavailable`]. Any other name defaults to
    /// [`SupportLevel::Emulated`]: a software renderer can approximate most 2D
    /// marks.
    fn supports_effect(&self, effect: &str) -> SupportLevel {
        match effect {
            "filter" | "backdrop-filter" => SupportLevel::Unavailable,
            // A `DrawImage` blits the real decoded pixels (a genuine sampled
            // bitmap via `draw_pixmap`), not an approximation, so images are
            // natively supported here.
            "image" => SupportLevel::Native,
            _ => SupportLevel::Emulated,
        }
    }
}

/// Composites the top layer target of `targets` back onto the target beneath it
/// at `opacity`, then drops it — the group-opacity blend for a CSS `opacity`
/// layer (spec §5.1).
///
/// The offscreen (which already holds the whole group's paint at full opacity) is
/// drawn onto the parent with a [`tiny_skia::PixmapPaint`] at `opacity` and
/// `SrcOver`, so the group blends **as one**: two overlapping opaque children
/// composite once, not twice (real group compositing, not per-element alpha).
/// Does nothing when only the root target remains (a stray pop), so the root is
/// never consumed.
fn composite_top_layer(targets: &mut Vec<tiny_skia::Pixmap>, opacity: f32) {
    // Only composite an actual layer: never pop the root frame buffer.
    if targets.len() < 2 {
        return;
    }
    let Some(child) = targets.pop() else {
        return;
    };
    let Some(parent) = targets.last_mut() else {
        // Cannot happen (len was >= 2), but keep the buffer to avoid data loss.
        targets.push(child);
        return;
    };
    let paint = tiny_skia::PixmapPaint {
        opacity: opacity.clamp(0.0, 1.0),
        ..Default::default()
    };
    parent.draw_pixmap(
        0,
        0,
        child.as_ref(),
        &paint,
        tiny_skia::Transform::identity(),
        None,
    );
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

/// Converts an obscur-geometry [`AffineTransform`] into a
/// [`tiny_skia::Transform`]. The obscur coefficients `[a, b, c, d, tx, ty]` map to
/// tiny-skia's row order `(sx, ky, kx, sy, tx, ty)` one-to-one (both map a point
/// `(x, y)` to `(a·x + c·y + tx, b·x + d·y + ty)`).
fn to_sk_transform(t: &AffineTransform) -> tiny_skia::Transform {
    let m = t.coeffs;
    tiny_skia::Transform::from_row(m[0], m[1], m[2], m[3], m[4], m[5])
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

/// The alpha multiplier the software backend applies to a `box-shadow` fill to
/// fake the softness a real gaussian blur would give (this backend has no blur).
/// A documented approximation (spec §5.2): the offset rounded rect is filled at
/// half the shadow color's alpha rather than blurred.
const SHADOW_APPROX_ALPHA: f32 = 0.5;

/// Rasterizes a CSS `box-shadow` as a **flat**, offset rounded rectangle at
/// reduced alpha — the software backend's blur-free approximation (spec §5.2).
///
/// The box's border box `rect` is translated by `offset` and inflated (a drop
/// shadow) or deflated (an inset shadow) by `spread`, then filled as a rounded
/// rect (`radii`) in the shadow `color` at [`SHADOW_APPROX_ALPHA`] of its alpha.
/// The `blur` radius is **ignored** (this backend has no gaussian blur), so the
/// mark is a hard-edged, semi-transparent rounded rect rather than a soft shadow —
/// a documented approximation that keeps device-free callers deterministic. An
/// inset shadow is drawn at its deflated geometry (inside the box); a degenerate
/// (non-positive) rect draws nothing.
#[allow(clippy::too_many_arguments)]
fn draw_shadow_approx(
    pixmap: &mut tiny_skia::Pixmap,
    rect: Rect,
    radii: CornerRadii,
    color: Color,
    spread: f32,
    offset: (f32, f32),
    inset: bool,
    clip: Option<&tiny_skia::Mask>,
    transform: tiny_skia::Transform,
) {
    // Drop shadows inflate by `spread`; inset shadows deflate by it.
    let inflate = if inset { -spread } else { spread };
    let (ox, oy) = offset;
    let x = rect.origin.x + ox - inflate;
    let y = rect.origin.y + oy - inflate;
    let w = 2.0f32.mul_add(inflate, rect.size.width);
    let h = 2.0f32.mul_add(inflate, rect.size.height);
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let shadow_rect = Rect::new(Point::new(x, y), Size::new(w, h));
    let path = Path::rounded_rect(shadow_rect, radii);
    let Some(sk_path) = to_sk_path(&path) else {
        return;
    };
    let faded = Color::rgba(color.r, color.g, color.b, color.a * SHADOW_APPROX_ALPHA);
    let Some(sk_paint) = solid_paint(faded) else {
        return;
    };
    pixmap.fill_path(
        &sk_path,
        &sk_paint,
        tiny_skia::FillRule::Winding,
        transform,
        clip,
    );
}

/// The largest number of image copies drawn per axis for a tiled
/// (`background-repeat`) background — a bounded-loop guard against a pathological
/// tiny-tile / huge-region combination. The visible output is clipped to `region`
/// regardless, so this only caps wasted work (documented limitation).
const MAX_TILES_PER_AXIS: i32 = 8192;

/// The smallest tile extent (logical px) that still tiles: a tile at or below this
/// on an axis is treated as non-repeating on that axis, so a degenerate size can
/// never spin the tile loop.
const MIN_TILE_EXTENT: f32 = 0.5;

/// Blits a decoded [`SceneImage`] as one copy scaled to `tile`, clipped to
/// `region`, and optionally tiled along the `repeat_x` / `repeat_y` axes to fill
/// `region`, honoring the active `clip` and `transform` (spec §5.16).
///
/// `tiny-skia` stores **premultiplied** RGBA, so a [`AlphaMode::Straight`] image is
/// premultiplied per pixel here (a [`AlphaMode::Premultiplied`] image is used as-is)
/// before being wrapped into a source [`tiny_skia::Pixmap`], once, and reused for
/// every tile. Each copy is drawn with [`draw_pixmap`](tiny_skia::Pixmap::draw_pixmap)
/// under a transform that scales the image's intrinsic `width × height` to `tile`'s
/// size and translates it to the copy's origin (stepping by `tile`'s width/height on
/// the repeated axes), composed with the running `transform`. When tiling — or when
/// a single tile overflows `region` — the draw is confined to `region` by
/// intersecting a `region` clip mask into the active `clip`. The default nearest
/// sampling keeps the blit crisp and deterministic. A zero-sized image, tile, or
/// region, or a source pixmap that fails to allocate, draws nothing.
#[allow(clippy::cast_precision_loss, clippy::too_many_arguments)]
fn draw_image(
    pixmap: &mut tiny_skia::Pixmap,
    image: &SceneImage,
    tile: Rect,
    region: Rect,
    repeat_x: bool,
    repeat_y: bool,
    clip: Option<&tiny_skia::Mask>,
    transform: tiny_skia::Transform,
) {
    let (iw, ih) = (image.width(), image.height());
    if iw == 0
        || ih == 0
        || tile.size.width <= 0.0
        || tile.size.height <= 0.0
        || region.size.width <= 0.0
        || region.size.height <= 0.0
    {
        return;
    }
    // A tile too small to tile safely repeats as a single copy on that axis.
    let repeat_x = repeat_x && tile.size.width > MIN_TILE_EXTENT;
    let repeat_y = repeat_y && tile.size.height > MIN_TILE_EXTENT;

    // tiny-skia stores premultiplied RGBA: premultiply a straight image, pass a
    // premultiplied one through unchanged, so both backends show the same picture.
    let premultiplied = match image.alpha_mode() {
        AlphaMode::Premultiplied => image.pixels().to_vec(),
        AlphaMode::Straight => premultiply_rgba(image.pixels()),
    };
    let Some(size) = tiny_skia::IntSize::from_wh(iw, ih) else {
        return;
    };
    let Some(src) = tiny_skia::Pixmap::from_vec(premultiplied, size) else {
        return;
    };

    // The single tile fully inside `region` with no tiling needs no extra clip —
    // the exact pre-placement stretch path. Otherwise confine drawing to `region`
    // by intersecting a `region` mask into the active clip.
    let contained = !repeat_x
        && !repeat_y
        && tile.origin.x >= region.origin.x - 1e-3
        && tile.origin.y >= region.origin.y - 1e-3
        && tile.origin.x + tile.size.width <= region.origin.x + region.size.width + 1e-3
        && tile.origin.y + tile.size.height <= region.origin.y + region.size.height + 1e-3;
    let region_mask = if contained {
        None
    } else {
        let Some(region_path) = to_sk_path(&Path::rect(region)) else {
            return;
        };
        build_clip_mask(
            pixmap.width(),
            pixmap.height(),
            &region_path,
            tiny_skia::FillRule::Winding,
            clip,
            transform,
        )
    };
    let effective_clip = if contained { clip } else { region_mask.as_ref() };

    // Tile index ranges: `[i0, i1] × [j0, j1]` copies stepping by tile w/h from the
    // tile origin, covering `region`. A non-repeating axis is the single index 0.
    let (i0, i1) = tile_index_range(
        repeat_x,
        region.origin.x,
        region.origin.x + region.size.width,
        tile.origin.x,
        tile.size.width,
    );
    let (j0, j1) = tile_index_range(
        repeat_y,
        region.origin.y,
        region.origin.y + region.size.height,
        tile.origin.y,
        tile.size.height,
    );

    let paint = tiny_skia::PixmapPaint::default();
    let (sx, sy) = (tile.size.width / iw as f32, tile.size.height / ih as f32);
    for j in j0..=j1 {
        for i in i0..=i1 {
            let ox = tile.origin.x + i as f32 * tile.size.width;
            let oy = tile.origin.y + j as f32 * tile.size.height;
            // Intrinsic → tile: scale the natural `iw × ih` to the tile size, then
            // translate to this copy's origin, all under the running transform.
            let place = transform
                .pre_concat(tiny_skia::Transform::from_translate(ox, oy))
                .pre_concat(tiny_skia::Transform::from_scale(sx, sy));
            pixmap.draw_pixmap(0, 0, src.as_ref(), &paint, place, effective_clip);
        }
    }
}

/// Computes the inclusive tile index range `[start, end]` covering the span
/// `[r0, r1]` with copies at `t0 + k * ts` (`ts > 0`). A non-repeating axis
/// (`repeat == false`) is the single index `0`. The range is padded by one on each
/// side (so a partially-visible edge tile is drawn) and clamped to
/// [`MAX_TILES_PER_AXIS`] copies — the `region` clip trims any overdraw.
#[allow(clippy::cast_possible_truncation)]
fn tile_index_range(repeat: bool, r0: f32, r1: f32, t0: f32, ts: f32) -> (i32, i32) {
    if !repeat || ts <= 0.0 {
        return (0, 0);
    }
    let start_f = ((r0 - t0) / ts).floor();
    let end_f = ((r1 - t0) / ts).ceil();
    if !start_f.is_finite() || !end_f.is_finite() {
        return (0, 0);
    }
    let start = start_f as i32 - 1;
    let mut end = end_f as i32 + 1;
    // Bound the loop: never draw more than MAX_TILES_PER_AXIS copies on this axis.
    if end - start + 1 > MAX_TILES_PER_AXIS {
        end = start + MAX_TILES_PER_AXIS - 1;
    }
    (start, end)
}

/// Premultiplies straight-alpha RGBA8 bytes into the premultiplied form
/// `tiny-skia` expects: each color channel is scaled by `alpha / 255` (rounded),
/// the alpha channel kept. The input length must be a multiple of four.
#[allow(clippy::cast_possible_truncation)]
fn premultiply_rgba(straight: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(straight.len());
    for px in straight.chunks_exact(4) {
        let a = u16::from(px[3]);
        // Rounded `c * a / 255` per channel (a == 255 is an exact identity).
        // The result is always `<= 255`, so the `as u8` never truncates.
        let mul = |c: u8| -> u8 { ((u16::from(c) * a + 127) / 255) as u8 };
        out.push(mul(px[0]));
        out.push(mul(px[1]));
        out.push(mul(px[2]));
        out.push(px[3]);
    }
    out
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
    transform: tiny_skia::Transform,
) -> Option<tiny_skia::Mask> {
    if let Some(parent) = parent {
        let mut mask = parent.clone();
        mask.intersect_path(sk_path, fill_rule, true, transform);
        Some(mask)
    } else {
        let mut mask = tiny_skia::Mask::new(width, height)?;
        mask.fill_path(sk_path, fill_rule, true, transform);
        Some(mask)
    }
}

/// Rasterizes a pre-shaped [`GlyphRun`] at `origin`, drawing real glyph outlines
/// when the run's font is registered and box-coverage cells otherwise (spec §5.2).
///
/// The run's glyph positions are authoritative — shaping already ran in the
/// domain, so this never re-shapes. When [`GlyphRun::font`] names a font
/// registered via [`CpuRenderer::register_font`] and its bytes parse as an sfnt,
/// each glyph's outline is extracted with `skrifa`, scaled to `font_size`, placed
/// on its baseline pen position, and filled (see [`draw_glyph_outlines`]).
/// Otherwise — no font registered for the handle, or unparseable bytes — the run
/// degrades to the documented box-coverage fallback (see [`draw_glyph_boxes`]) so
/// device-free callers that register no font still get a defined, non-empty mark.
/// With a stop-less gradient brush (no resolvable color) the whole run is skipped.
fn draw_text_run(
    pixmap: &mut tiny_skia::Pixmap,
    origin: Point,
    run: &GlyphRun,
    brush: &Brush,
    clip: Option<&tiny_skia::Mask>,
    transform: tiny_skia::Transform,
    fonts: &HashMap<u32, Arc<[u8]>>,
) {
    let Some(color) = brush_color(brush) else {
        return;
    };
    let Some(sk_paint) = solid_paint(color) else {
        return;
    };
    let font_size = run.font_size();
    if font_size <= 0.0 {
        return;
    }

    // Real outlines when the run's font is registered and its bytes are a valid
    // sfnt; box-coverage fallback otherwise. Synthetic-style hints (spec §5.18)
    // travel with the run: an oblique run is sheared, a bold run thickened.
    let skew = if run.synth_oblique() {
        FAUX_ITALIC_SKEW
    } else {
        0.0
    };
    if let Some(bytes) = fonts.get(&run.font())
        && let Ok(font_ref) = FontRef::new(bytes)
    {
        draw_glyph_outlines(
            pixmap,
            origin,
            run,
            &font_ref,
            &sk_paint,
            clip,
            transform,
            skew,
            run.synth_bold(),
        );
        return;
    }
    draw_glyph_boxes(pixmap, origin, run, &sk_paint, clip, transform, skew);
}

/// The synthetic-oblique shear factor: the standard faux-italic skew of `0.25`
/// (`tan ≈ 14°`), applied as a rightward x-shift proportional to a point's height
/// above the baseline (spec §5.18).
const FAUX_ITALIC_SKEW: f32 = 0.25;

/// The synthetic-bold stroke width as a fraction of the run's font size: the
/// filled outline is additionally stroked at this width to thicken it, faking a
/// heavier weight (spec §5.18).
const FAUX_BOLD_STROKE_FRACTION: f32 = 0.03;

/// Fills each glyph of `run` as its real outline extracted from `font` at the
/// run's `font_size` (spec §5.2 real glyph rasterization).
///
/// For each glyph the outline is drawn (unhinted, no variation) into a
/// [`GlyphPathPen`] anchored at the baseline pen position
/// `(origin.x + glyph.x, origin.y + glyph.y)`; `skrifa` yields pixel-scaled,
/// y-up coordinates that the pen flips into the pixmap's y-down device space. A
/// glyph absent from the font, with no contours (whitespace, glyph 0), or whose
/// outline `skrifa`/`tiny-skia` rejects contributes nothing.
///
/// `skew` shears the outline to synthesize an oblique slant (`0.0` = upright);
/// when `bold` is set the filled outline is additionally stroked to thicken it,
/// synthesizing a heavier weight (spec §5.18).
#[allow(clippy::too_many_arguments)]
fn draw_glyph_outlines(
    pixmap: &mut tiny_skia::Pixmap,
    origin: Point,
    run: &GlyphRun,
    font: &FontRef<'_>,
    sk_paint: &tiny_skia::Paint<'_>,
    clip: Option<&tiny_skia::Mask>,
    transform: tiny_skia::Transform,
    skew: f32,
    bold: bool,
) {
    let outlines = font.outline_glyphs();
    let size = SkrifaSize::new(run.font_size());
    // Faux-bold: stroke the filled outline with a hairline proportional to the
    // font size so the shape gains weight without re-shaping.
    let bold_stroke = bold.then(|| tiny_skia::Stroke {
        width: run.font_size() * FAUX_BOLD_STROKE_FRACTION,
        ..Default::default()
    });
    for glyph in run.glyphs() {
        let Some(outline) = outlines.get(GlyphId::new(glyph.glyph_id())) else {
            continue;
        };
        let pen_x = origin.x + glyph.x();
        let baseline_y = origin.y + glyph.y();
        let mut pen = GlyphPathPen::with_skew(pen_x, baseline_y, skew);
        let settings = DrawSettings::unhinted(size, LocationRef::default());
        if outline.draw(settings, &mut pen).is_err() {
            continue;
        }
        let Some(sk_path) = pen.into_path() else {
            continue;
        };
        pixmap.fill_path(
            &sk_path,
            sk_paint,
            tiny_skia::FillRule::Winding,
            transform,
            clip,
        );
        if let Some(stroke) = &bold_stroke {
            pixmap.stroke_path(&sk_path, sk_paint, stroke, transform, clip);
        }
    }
}

/// Rasterizes `run` as one solid coverage cell per visible glyph — the fallback
/// used when no real font is registered for the run's handle (spec §5.2
/// rasterization approximation).
///
/// Each glyph fills the box whose top-left is `origin + (glyph.x, glyph.y -
/// font_size)` (the baseline sits `font_size` below the glyph's ascent) sized
/// `advance × font_size`, honoring `clip`. Whitespace glyphs — identified by the
/// mock shaper's convention that a glyph id is the Unicode scalar — and
/// non-positive cells draw nothing.
///
/// `skew` shears the cell into a forward-leaning parallelogram to approximate a
/// synthetic oblique (spec §5.18): the top edge (a full `font_size` above the
/// baseline) slides right by `skew * font_size` while the baseline edge stays
/// put. `skew == 0.0` keeps the upright rectangle byte-identical to before.
fn draw_glyph_boxes(
    pixmap: &mut tiny_skia::Pixmap,
    origin: Point,
    run: &GlyphRun,
    sk_paint: &tiny_skia::Paint<'_>,
    clip: Option<&tiny_skia::Mask>,
    transform: tiny_skia::Transform,
    skew: f32,
) {
    let font_size = run.font_size();
    for glyph in run.glyphs() {
        // Skip whitespace: the mock shaper's glyph id is the Unicode scalar.
        if char::from_u32(glyph.glyph_id()).is_none_or(char::is_whitespace) {
            continue;
        }
        let width = glyph.advance();
        if width <= 0.0 {
            continue;
        }
        let x = origin.x + glyph.x();
        let baseline = origin.y + glyph.y();
        let top = baseline - font_size;
        let sk_path = if skew == 0.0 {
            let Some(rect) = tiny_skia::Rect::from_xywh(x, top, width, font_size) else {
                continue;
            };
            tiny_skia::PathBuilder::from_rect(rect)
        } else {
            // Parallelogram: baseline edge unshifted, top edge slid right by
            // `skew * font_size`.
            let shift = skew * font_size;
            let mut builder = tiny_skia::PathBuilder::new();
            builder.move_to(x, baseline);
            builder.line_to(x + width, baseline);
            builder.line_to(x + width + shift, top);
            builder.line_to(x + shift, top);
            builder.close();
            let Some(path) = builder.finish() else {
                continue;
            };
            path
        };
        pixmap.fill_path(
            &sk_path,
            sk_paint,
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
    use obscur_scene::text::{GlyphRun, PositionedGlyph};

    #[test]
    fn radial_and_conic_brushes_flatten_to_first_stop() {
        use obscur_scene::paint::{ConicGradient, GradientStop, RadialGradient};

        let red = Color::rgba(1.0, 0.0, 0.0, 1.0);
        let blue = Color::rgba(0.0, 0.0, 1.0, 1.0);
        let stops = || {
            vec![
                GradientStop::new(0.0, red),
                GradientStop::new(1.0, blue),
            ]
        };
        // Both approximate to the first stop (red), per the effect-split policy.
        let radial = Brush::Radial(RadialGradient::new(Point::new(5.0, 5.0), 10.0, stops()));
        assert_eq!(brush_color(&radial), Some(red));
        let conic = Brush::Conic(ConicGradient::new(Point::new(5.0, 5.0), 0.0, stops()));
        assert_eq!(brush_color(&conic), Some(red));
        // A stop-less gradient resolves to no color (the command is skipped).
        let empty = Brush::Radial(RadialGradient::new(Point::ORIGIN, 1.0, Vec::new()));
        assert_eq!(brush_color(&empty), None);
    }

    /// Builds a hermetic sfnt (1000 upem, 2 glyphs) whose glyph 1 is a real
    /// filled square contour spanning font units `x∈[100,900]`, `y∈[0,700]`;
    /// glyph 0 is the empty `.notdef`. Unlike the harfrust adapter's test font
    /// (empty `glyf`), this one carries actual outlines so `skrifa` rasterizes a
    /// non-trivial shape. Only the tables `skrifa` needs to extract an outline by
    /// glyph id — `head`, `maxp`, `loca`, `glyf` — are emitted, plus `hhea`/`hmtx`
    /// for a well-formed face; no `cmap`, since the CPU renderer addresses glyphs
    /// by index, not codepoint.
    fn outline_font() -> Vec<u8> {
        fn head() -> Vec<u8> {
            let mut t = Vec::new();
            t.extend_from_slice(&0x0001_0000u32.to_be_bytes()); // version
            t.extend_from_slice(&0u32.to_be_bytes()); // fontRevision
            t.extend_from_slice(&0u32.to_be_bytes()); // checksumAdjustment
            t.extend_from_slice(&0x5F0F_3CF5u32.to_be_bytes()); // magicNumber
            t.extend_from_slice(&0u16.to_be_bytes()); // flags
            t.extend_from_slice(&1000u16.to_be_bytes()); // unitsPerEm
            t.extend_from_slice(&0u64.to_be_bytes()); // created
            t.extend_from_slice(&0u64.to_be_bytes()); // modified
            t.extend_from_slice(&0i16.to_be_bytes()); // xMin
            t.extend_from_slice(&0i16.to_be_bytes()); // yMin
            t.extend_from_slice(&900i16.to_be_bytes()); // xMax
            t.extend_from_slice(&700i16.to_be_bytes()); // yMax
            t.extend_from_slice(&0u16.to_be_bytes()); // macStyle
            t.extend_from_slice(&8u16.to_be_bytes()); // lowestRecPPEM
            t.extend_from_slice(&0i16.to_be_bytes()); // fontDirectionHint
            t.extend_from_slice(&0i16.to_be_bytes()); // indexToLocFormat (short)
            t.extend_from_slice(&0i16.to_be_bytes()); // glyphDataFormat
            t
        }

        fn hhea() -> Vec<u8> {
            let mut t = Vec::new();
            t.extend_from_slice(&0x0001_0000u32.to_be_bytes()); // version
            t.extend_from_slice(&800i16.to_be_bytes()); // ascender
            t.extend_from_slice(&(-200i16).to_be_bytes()); // descender
            t.extend_from_slice(&0i16.to_be_bytes()); // lineGap
            t.extend_from_slice(&[0u8; 24]); // advanceWidthMax .. metricDataFormat
            t.extend_from_slice(&2u16.to_be_bytes()); // numberOfHMetrics
            t
        }

        fn maxp() -> Vec<u8> {
            let mut t = Vec::new();
            t.extend_from_slice(&0x0000_5000u32.to_be_bytes()); // version 0.5
            t.extend_from_slice(&2u16.to_be_bytes()); // numGlyphs
            t
        }

        fn hmtx() -> Vec<u8> {
            let mut t = Vec::new();
            for (advance, lsb) in [(1000u16, 0i16), (1000, 100)] {
                t.extend_from_slice(&advance.to_be_bytes());
                t.extend_from_slice(&lsb.to_be_bytes());
            }
            t
        }

        // glyph 1: one closed contour, 4 on-curve points (a filled square).
        fn glyf() -> Vec<u8> {
            let mut t = Vec::new();
            t.extend_from_slice(&1i16.to_be_bytes()); // numberOfContours
            t.extend_from_slice(&100i16.to_be_bytes()); // xMin
            t.extend_from_slice(&0i16.to_be_bytes()); // yMin
            t.extend_from_slice(&900i16.to_be_bytes()); // xMax
            t.extend_from_slice(&700i16.to_be_bytes()); // yMax
            t.extend_from_slice(&3u16.to_be_bytes()); // endPtsOfContours[0]
            t.extend_from_slice(&0u16.to_be_bytes()); // instructionLength
            t.extend_from_slice(&[0x01, 0x01, 0x01, 0x01]); // flags: all ON_CURVE
            // x deltas: (100, 900, 900, 100) -> deltas 100, 800, 0, -800
            for dx in [100i16, 800, 0, -800] {
                t.extend_from_slice(&dx.to_be_bytes());
            }
            // y deltas: (0, 0, 700, 700) -> deltas 0, 0, 700, 0
            for dy in [0i16, 0, 700, 0] {
                t.extend_from_slice(&dy.to_be_bytes());
            }
            t
        }

        fn loca(glyf_len: usize) -> Vec<u8> {
            // Short loca stores offset/2; glyph 0 empty, glyph 1 = [0, glyf_len].
            let mut t = Vec::new();
            for off in [0usize, 0, glyf_len] {
                t.extend_from_slice(&((off / 2) as u16).to_be_bytes());
            }
            t
        }

        let glyf_bytes = glyf();
        assert_eq!(glyf_bytes.len() % 2, 0, "short loca needs even glyf length");
        let tables: [(&[u8; 4], Vec<u8>); 6] = [
            (b"glyf", glyf_bytes.clone()),
            (b"head", head()),
            (b"hhea", hhea()),
            (b"hmtx", hmtx()),
            (b"loca", loca(glyf_bytes.len())),
            (b"maxp", maxp()),
        ];

        let num_tables = tables.len() as u16;
        let mut offset = 12 + tables.len() * 16;
        let mut records = Vec::new();
        let mut body = Vec::new();
        for (tag, data) in &tables {
            records.extend_from_slice(*tag);
            records.extend_from_slice(&0u32.to_be_bytes()); // checksum (unchecked)
            records.extend_from_slice(&(offset as u32).to_be_bytes());
            records.extend_from_slice(&(data.len() as u32).to_be_bytes());
            body.extend_from_slice(data);
            let pad = (4 - data.len() % 4) % 4;
            body.extend(std::iter::repeat_n(0u8, pad));
            offset += data.len() + pad;
        }

        let mut font = Vec::new();
        font.extend_from_slice(&0x0001_0000u32.to_be_bytes()); // sfnt version
        font.extend_from_slice(&num_tables.to_be_bytes());
        font.extend_from_slice(&0u16.to_be_bytes()); // searchRange
        font.extend_from_slice(&0u16.to_be_bytes()); // entrySelector
        font.extend_from_slice(&0u16.to_be_bytes()); // rangeShift
        font.extend_from_slice(&records);
        font.extend_from_slice(&body);
        font
    }

    /// A scene drawing one instance of glyph `gid` from font handle `font_id` at
    /// font size 100, baseline at device `y = 100`, into a 100x120 target.
    fn glyph_submission(gid: u32, font_id: u32) -> FrameSubmission {
        glyph_submission_synth(gid, font_id, false, false)
    }

    /// As [`glyph_submission`], but tags the run for synthetic styling: `oblique`
    /// shears the glyph, `bold` thickens it.
    fn glyph_submission_synth(gid: u32, font_id: u32, oblique: bool, bold: bool) -> FrameSubmission {
        let run = GlyphRun::new(
            vec![PositionedGlyph::new(gid, 0, 0.0, 100.0, 90.0)],
            100.0,
            "x",
        )
        .with_font(font_id)
        .with_synthesis(oblique, bold);
        let list = DisplayList::from_commands(vec![PaintCommand::DrawText {
            origin: Point::new(0.0, 0.0),
            run,
            brush: Brush::solid(Color::WHITE),
        }]);
        let snapshot = SceneSnapshot::new(list, SceneGeneration::from_raw(3));
        let frame = RenderFrame::new(snapshot, Size::new(100.0, 120.0));
        FrameSubmission::new(frame)
    }

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
    fn dashed_stroke_leaves_gaps_along_the_path() {
        use obscur_scene::paint::{Brush, Stroke};

        // A horizontal line at y=4 stroked black, width 4, dashed [8 on, 8 off]
        // starting at phase 0: x∈[0,8) paints, x∈[8,16) is a gap, and so on.
        let mut path = Path::new();
        path.move_to(Point::new(0.0, 4.0))
            .line_to(Point::new(32.0, 4.0));
        let list = DisplayList::from_commands(vec![PaintCommand::DrawPath {
            path,
            paint: Paint::stroke(
                Stroke::new(Brush::solid(Color::BLACK), 4.0).with_dash([8.0, 8.0], 0.0),
            ),
        }]);
        let snapshot = SceneSnapshot::new(list, SceneGeneration::from_raw(5));
        let frame = RenderFrame::new(snapshot, Size::new(32.0, 8.0));
        let submission = FrameSubmission::new(frame);

        let mut renderer = CpuRenderer::new(RendererId::from_raw(1));
        let _ = renderer.render(&submission, &bitmap_target(32.0, 8.0));
        let pixmap = renderer.pixmap().expect("a frame was rendered");

        // Inside the first dash (x=4) is painted; inside the first gap (x=12) is
        // transparent — proving the dash actually broke the stroke.
        let on = pixmap.pixel(4, 4).expect("on-dash pixel");
        assert!(on.alpha() > 200, "first dash painted: alpha={}", on.alpha());
        let off = pixmap.pixel(12, 4).expect("gap pixel");
        assert_eq!(off.alpha(), 0, "first gap is empty");
        // The next dash (x=20) paints again.
        let on2 = pixmap.pixel(20, 4).expect("second on-dash pixel");
        assert!(on2.alpha() > 200, "second dash painted: alpha={}", on2.alpha());
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
        // `filter: drop-shadow(...)` rides the `box-shadow` DrawShadow path.
        assert_eq!(
            renderer.supports_effect("drop-shadow"),
            SupportLevel::Emulated
        );
        // The general `filter` / `backdrop-filter` have no software approximation.
        assert_eq!(renderer.supports_effect("filter"), SupportLevel::Unavailable);
        assert_eq!(
            renderer.supports_effect("backdrop-filter"),
            SupportLevel::Unavailable
        );
    }

    /// A white background rect covering `0..16 × 0..16`, drawn first so images
    /// composite over an opaque white canvas.
    fn white_bg() -> PaintCommand {
        PaintCommand::DrawPath {
            path: Path::rect(Rect::new(Point::new(0.0, 0.0), Size::new(16.0, 16.0))),
            paint: Paint::solid_fill(Color::WHITE),
        }
    }

    /// A 2×2 image whose four pixels are all `(r, g, b, a)`.
    fn solid_image(r: u8, g: u8, b: u8, a: u8, alpha: AlphaMode) -> SceneImage {
        let px: Vec<u8> = std::iter::repeat_n([r, g, b, a], 4).flatten().collect();
        SceneImage::new(2, 2, Arc::from(px), alpha).expect("2x2 image")
    }

    #[test]
    fn draw_image_fills_dst_with_red_over_white() {
        // A 2×2 opaque-red image stretched to the dst rect (4,4)..(12,12) over a
        // white canvas: dst interior is red, outside stays white.
        let image = solid_image(255, 0, 0, 255, AlphaMode::Straight);
        let dst = Rect::new(Point::new(4.0, 4.0), Size::new(8.0, 8.0));
        let list = DisplayList::from_commands(vec![
            white_bg(),
            PaintCommand::DrawImage {
                image,
                tile: dst,
                region: dst,
                repeat_x: false,
                repeat_y: false,
            },
        ]);
        let snapshot = SceneSnapshot::new(list, SceneGeneration::from_raw(11));
        let frame = RenderFrame::new(snapshot, Size::new(16.0, 16.0));
        let submission = FrameSubmission::new(frame);

        let mut renderer = CpuRenderer::default();
        renderer
            .render(&submission, &bitmap_target(16.0, 16.0))
            .expect("renders");
        let pixmap = renderer.pixmap().expect("a frame was rendered");

        // A pixel inside the dst rect is opaque red.
        let inside = pixmap.pixel(8, 8).expect("center pixel exists");
        assert!(inside.red() > 200, "red high: {}", inside.red());
        assert!(inside.green() < 40, "green low: {}", inside.green());
        assert!(inside.blue() < 40, "blue low: {}", inside.blue());
        assert!(inside.alpha() > 200, "alpha high: {}", inside.alpha());

        // A pixel outside the dst rect stays white.
        let outside = pixmap.pixel(1, 1).expect("corner pixel exists");
        assert!(outside.red() > 200 && outside.green() > 200 && outside.blue() > 200);
    }

    #[test]
    fn no_repeat_tile_paints_only_its_rect() {
        // A 2×2 red image, no-repeat, as a 4×4 tile centered in a 16×16 white box:
        // tile at (6,6)..(10,10). Inside is red, outside (the rest of the box) is
        // white — proof position + size place a single clipped tile.
        let image = solid_image(255, 0, 0, 255, AlphaMode::Straight);
        let region = Rect::new(Point::new(0.0, 0.0), Size::new(16.0, 16.0));
        let tile = Rect::new(Point::new(6.0, 6.0), Size::new(4.0, 4.0));
        let list = DisplayList::from_commands(vec![
            white_bg(),
            PaintCommand::DrawImage {
                image,
                tile,
                region,
                repeat_x: false,
                repeat_y: false,
            },
        ]);
        let snapshot = SceneSnapshot::new(list, SceneGeneration::from_raw(21));
        let frame = RenderFrame::new(snapshot, Size::new(16.0, 16.0));
        let submission = FrameSubmission::new(frame);
        let mut renderer = CpuRenderer::default();
        renderer
            .render(&submission, &bitmap_target(16.0, 16.0))
            .expect("renders");
        let pixmap = renderer.pixmap().expect("a frame was rendered");

        // Center of the tile is red.
        let inside = pixmap.pixel(8, 8).expect("center pixel");
        assert!(inside.red() > 200 && inside.green() < 40 && inside.blue() < 40);
        // A pixel outside the tile (but inside the box) is white — no tiling.
        let outside = pixmap.pixel(2, 2).expect("box corner pixel");
        assert!(outside.red() > 200 && outside.green() > 200 && outside.blue() > 200);
    }

    #[test]
    fn repeat_x_tiles_a_horizontal_band() {
        // A 2×2 red image, repeat-x only, a 4×4 tile whose origin is the band row
        // (6,6): red tiles fill the whole row band (y 6..10) across x, but rows
        // outside the band stay white (no vertical tiling).
        let image = solid_image(255, 0, 0, 255, AlphaMode::Straight);
        let region = Rect::new(Point::new(0.0, 0.0), Size::new(16.0, 16.0));
        let tile = Rect::new(Point::new(6.0, 6.0), Size::new(4.0, 4.0));
        let list = DisplayList::from_commands(vec![
            white_bg(),
            PaintCommand::DrawImage {
                image,
                tile,
                region,
                repeat_x: true,
                repeat_y: false,
            },
        ]);
        let snapshot = SceneSnapshot::new(list, SceneGeneration::from_raw(22));
        let frame = RenderFrame::new(snapshot, Size::new(16.0, 16.0));
        let submission = FrameSubmission::new(frame);
        let mut renderer = CpuRenderer::default();
        renderer
            .render(&submission, &bitmap_target(16.0, 16.0))
            .expect("renders");
        let pixmap = renderer.pixmap().expect("a frame was rendered");

        // Far-left and far-right of the band row are red (tiled horizontally).
        let left = pixmap.pixel(1, 8).expect("left band pixel");
        assert!(left.red() > 200 && left.blue() < 40, "left band is red");
        let right = pixmap.pixel(14, 8).expect("right band pixel");
        assert!(right.red() > 200 && right.blue() < 40, "right band is red");
        // A pixel above the band (y=2) stays white — no vertical tiling.
        let above = pixmap.pixel(8, 2).expect("above-band pixel");
        assert!(above.red() > 200 && above.green() > 200 && above.blue() > 200);
    }

    #[test]
    fn cover_overflow_is_clipped_to_region() {
        // A 2×2 red image scaled to a tile larger than the region (a `cover`-style
        // overflow), no-repeat: the tile origin is above/left of the region so it
        // overflows, but the region clip keeps red only inside the region. A pixel
        // just outside the region stays white.
        let image = solid_image(255, 0, 0, 255, AlphaMode::Straight);
        let region = Rect::new(Point::new(4.0, 4.0), Size::new(8.0, 8.0));
        // Tile overflows the region on every side.
        let tile = Rect::new(Point::new(0.0, 0.0), Size::new(16.0, 16.0));
        let list = DisplayList::from_commands(vec![
            white_bg(),
            PaintCommand::DrawImage {
                image,
                tile,
                region,
                repeat_x: false,
                repeat_y: false,
            },
        ]);
        let snapshot = SceneSnapshot::new(list, SceneGeneration::from_raw(23));
        let frame = RenderFrame::new(snapshot, Size::new(16.0, 16.0));
        let submission = FrameSubmission::new(frame);
        let mut renderer = CpuRenderer::default();
        renderer
            .render(&submission, &bitmap_target(16.0, 16.0))
            .expect("renders");
        let pixmap = renderer.pixmap().expect("a frame was rendered");

        // Inside the region: red.
        let inside = pixmap.pixel(8, 8).expect("region center");
        assert!(inside.red() > 200 && inside.blue() < 40, "region interior is red");
        // Outside the region (but the tile would cover it): clipped to white.
        let outside = pixmap.pixel(1, 1).expect("outside-region pixel");
        assert!(
            outside.red() > 200 && outside.green() > 200 && outside.blue() > 200,
            "overflow clipped to region"
        );
    }

    #[test]
    fn half_alpha_image_blends_over_white() {
        // A 2×2 straight-alpha red at 50% alpha over white → pink: red stays high,
        // green/blue rise toward mid as the white shows through (proves the
        // straight→premultiplied conversion and SrcOver blend).
        let image = solid_image(255, 0, 0, 128, AlphaMode::Straight);
        let dst = Rect::new(Point::new(4.0, 4.0), Size::new(8.0, 8.0));
        let list = DisplayList::from_commands(vec![
            white_bg(),
            PaintCommand::DrawImage {
                image,
                tile: dst,
                region: dst,
                repeat_x: false,
                repeat_y: false,
            },
        ]);
        let snapshot = SceneSnapshot::new(list, SceneGeneration::from_raw(12));
        let frame = RenderFrame::new(snapshot, Size::new(16.0, 16.0));
        let submission = FrameSubmission::new(frame);

        let mut renderer = CpuRenderer::default();
        renderer
            .render(&submission, &bitmap_target(16.0, 16.0))
            .expect("renders");
        let pixmap = renderer.pixmap().expect("a frame was rendered");

        let blended = pixmap.pixel(8, 8).expect("center pixel exists");
        // Red still dominates, but green/blue are mid-range (white showing through)
        // — a pink, not pure red and not white.
        assert!(blended.red() > 200, "red high: {}", blended.red());
        assert!(
            (90..=180).contains(&blended.green()),
            "green mid (white bleed): {}",
            blended.green()
        );
        assert!(
            (90..=180).contains(&blended.blue()),
            "blue mid (white bleed): {}",
            blended.blue()
        );
    }

    #[test]
    fn image_under_opacity_layer_composites() {
        // The image is nested inside a 0.5-opacity layer over white: it reuses the
        // layer stack (offscreen + composite) and must not panic, and the dst pixel
        // is a half-strength red (the group opacity applied once).
        let image = solid_image(255, 0, 0, 255, AlphaMode::Straight);
        let dst = Rect::new(Point::new(4.0, 4.0), Size::new(8.0, 8.0));
        let layer = obscur_scene::layer::SceneLayer::new().with_opacity(0.5);
        let list = DisplayList::from_commands(vec![
            white_bg(),
            PaintCommand::PushLayer(layer),
            PaintCommand::DrawImage {
                image,
                tile: dst,
                region: dst,
                repeat_x: false,
                repeat_y: false,
            },
            PaintCommand::PopLayer,
        ]);
        let snapshot = SceneSnapshot::new(list, SceneGeneration::from_raw(13));
        let frame = RenderFrame::new(snapshot, Size::new(16.0, 16.0));
        let submission = FrameSubmission::new(frame);

        let mut renderer = CpuRenderer::default();
        renderer
            .render(&submission, &bitmap_target(16.0, 16.0))
            .expect("renders");
        let pixmap = renderer.pixmap().expect("a frame was rendered");

        // Half-opacity red over white → pink (red high, green/blue mid).
        let px = pixmap.pixel(8, 8).expect("center pixel exists");
        assert!(px.red() > 200, "red high: {}", px.red());
        assert!(px.green() > 90 && px.green() < 180, "green mid: {}", px.green());
    }

    #[test]
    fn supports_effect_reports_images_native() {
        let renderer = CpuRenderer::default();
        assert_eq!(renderer.supports_effect("image"), SupportLevel::Native);
    }

    #[test]
    fn draw_text_run_rasterizes_glyph_cells() {
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
    fn draw_shadow_fills_offset_rect_at_reduced_alpha() {
        // A drop shadow of the box (2,2)-(10,10) offset by (4,4): the software
        // backend fills the offset rounded rect at half the color's alpha (no
        // blur). A pixel inside the offset rect is painted but semi-transparent.
        let list = DisplayList::from_commands(vec![PaintCommand::DrawShadow {
            rect: obscur_geometry::Rect::new(Point::new(2.0, 2.0), Size::new(8.0, 8.0)),
            radii: CornerRadii::default(),
            color: Color::rgba(0.0, 0.0, 0.0, 1.0),
            blur: 4.0,
            spread: 0.0,
            offset: (4.0, 4.0),
            inset: false,
        }]);
        let snapshot = SceneSnapshot::new(list, SceneGeneration::from_raw(7));
        let frame = RenderFrame::new(snapshot, Size::new(24.0, 24.0));
        let submission = FrameSubmission::new(frame);

        let mut renderer = CpuRenderer::default();
        renderer
            .render(&submission, &bitmap_target(24.0, 24.0))
            .expect("renders");
        let pixmap = renderer.pixmap().expect("a frame was rendered");

        // The offset rect spans (6,6)-(14,14); a pixel inside is painted at ~half
        // alpha (0.5 of opaque black), not fully opaque.
        let inside = pixmap.pixel(9, 9).expect("pixel exists");
        assert!(inside.alpha() > 60, "shadow painted: {}", inside.alpha());
        assert!(inside.alpha() < 200, "shadow is reduced alpha: {}", inside.alpha());
        // A pixel outside the offset rect (top-left corner) is untouched.
        assert_eq!(pixmap.pixel(0, 0).expect("pixel exists").alpha(), 0);
    }

    #[test]
    fn push_transform_translates_the_rasterized_rect() {
        // A red rect at (2,2)-(10,10) drawn inside a PushTransform(translate(10, 0))
        // rasterizes shifted 10px right: the pre-transform pixel (5,5) is now empty
        // and (15,5) is painted.
        let mut path = Path::new();
        path.move_to(Point::new(2.0, 2.0))
            .line_to(Point::new(10.0, 2.0))
            .line_to(Point::new(10.0, 10.0))
            .line_to(Point::new(2.0, 10.0))
            .close();
        let list = DisplayList::from_commands(vec![
            PaintCommand::PushTransform(AffineTransform::translate(10.0, 0.0)),
            PaintCommand::DrawPath {
                path,
                paint: Paint::solid_fill(Color::rgba(1.0, 0.0, 0.0, 1.0)),
            },
            PaintCommand::PopTransform,
        ]);
        assert!(list.is_balanced());
        let snapshot = SceneSnapshot::new(list, SceneGeneration::from_raw(11));
        let frame = RenderFrame::new(snapshot, Size::new(32.0, 32.0));
        let submission = FrameSubmission::new(frame);

        let mut renderer = CpuRenderer::default();
        renderer
            .render(&submission, &bitmap_target(32.0, 32.0))
            .expect("renders");
        let pixmap = renderer.pixmap().expect("a frame was rendered");

        // The shifted rect covers (12,2)-(20,10): (15,5) is opaque red.
        let shifted = pixmap.pixel(15, 5).expect("pixel exists");
        assert!(shifted.red() > 200 && shifted.alpha() > 200, "translated rect painted");
        // The untransformed position (5,5) is now empty.
        assert_eq!(pixmap.pixel(5, 5).expect("pixel exists").alpha(), 0);
    }

    /// A filled rectangle `(x,y)..(x+w,y+h)` in `color`, as one `DrawPath`.
    fn rect_fill_cmd(x: f32, y: f32, w: f32, h: f32, color: Color) -> PaintCommand {
        let mut path = Path::new();
        path.move_to(Point::new(x, y))
            .line_to(Point::new(x + w, y))
            .line_to(Point::new(x + w, y + h))
            .line_to(Point::new(x, y + h))
            .close();
        PaintCommand::DrawPath {
            path,
            paint: Paint::solid_fill(color),
        }
    }

    /// Renders `commands` into a `size × size` bitmap and returns the pixmap.
    fn render_commands(commands: Vec<PaintCommand>, size: f32) -> tiny_skia::Pixmap {
        let list = DisplayList::from_commands(commands);
        let snapshot = SceneSnapshot::new(list, SceneGeneration::from_raw(1));
        let frame = RenderFrame::new(snapshot, Size::new(size, size));
        let submission = FrameSubmission::new(frame);
        let mut renderer = CpuRenderer::default();
        renderer
            .render(&submission, &bitmap_target(size, size))
            .expect("renders");
        renderer.pixmap().expect("a frame was rendered").clone()
    }

    #[test]
    fn half_opaque_layer_blends_solid_box_over_white() {
        use obscur_scene::layer::SceneLayer;
        // A white backdrop, then an opaque black box inside an opacity-0.5 layer.
        // The composited pixel is ~50% grey — neither fully black nor white.
        let cmds = vec![
            rect_fill_cmd(0.0, 0.0, 16.0, 16.0, Color::WHITE),
            PaintCommand::PushLayer(SceneLayer::new().with_opacity(0.5)),
            rect_fill_cmd(4.0, 4.0, 8.0, 8.0, Color::rgba(0.0, 0.0, 0.0, 1.0)),
            PaintCommand::PopLayer,
        ];
        let pixmap = render_commands(cmds, 16.0);
        let px = pixmap.pixel(8, 8).expect("center pixel");
        // Black at 0.5 over white ⇒ mid-grey (~128), not 0 and not 255.
        assert!(px.red() > 90 && px.red() < 170, "mid-grey red: {}", px.red());
        assert!(px.green() > 90 && px.green() < 170, "mid-grey green: {}", px.green());
        assert!(px.blue() > 90 && px.blue() < 170, "mid-grey blue: {}", px.blue());
    }

    #[test]
    fn group_opacity_does_not_double_darken_overlap() {
        use obscur_scene::layer::SceneLayer;
        // Two overlapping opaque black boxes inside ONE opacity-0.5 layer. The
        // overlap region must be the SAME color as a non-overlap region of either
        // box — group compositing blends the whole group once, so the overlap is
        // not darkened twice (the distinction from per-element alpha).
        let cmds = vec![
            rect_fill_cmd(0.0, 0.0, 32.0, 32.0, Color::WHITE),
            PaintCommand::PushLayer(SceneLayer::new().with_opacity(0.5)),
            // Box A: (4,4)..(20,20). Box B: (12,12)..(28,28). Overlap: (12,12)..(20,20).
            rect_fill_cmd(4.0, 4.0, 16.0, 16.0, Color::rgba(0.0, 0.0, 0.0, 1.0)),
            rect_fill_cmd(12.0, 12.0, 16.0, 16.0, Color::rgba(0.0, 0.0, 0.0, 1.0)),
            PaintCommand::PopLayer,
        ];
        let pixmap = render_commands(cmds, 32.0);
        // A non-overlap pixel of box A (e.g. (6,6)) and an overlap pixel (e.g.
        // (16,16)) must match: the overlap is not double-darkened.
        let non_overlap = pixmap.pixel(6, 6).expect("box-A-only pixel");
        let overlap = pixmap.pixel(16, 16).expect("overlap pixel");
        assert_eq!(
            non_overlap.red(),
            overlap.red(),
            "overlap ({}) equals non-overlap ({}) — group blend, not per-box alpha",
            overlap.red(),
            non_overlap.red()
        );
        // And it really is a group blend (grey, not opaque black).
        assert!(overlap.red() > 90, "group blended to grey, not black: {}", overlap.red());
    }

    #[test]
    fn nested_opacity_layers_multiply() {
        use obscur_scene::layer::SceneLayer;
        // A black box under two nested 0.5 layers composites at ~0.25 alpha over
        // white: lighter than a single 0.5 layer (~0.5), proving nested layers
        // (stack depth > 1) multiply.
        let single = vec![
            rect_fill_cmd(0.0, 0.0, 16.0, 16.0, Color::WHITE),
            PaintCommand::PushLayer(SceneLayer::new().with_opacity(0.5)),
            rect_fill_cmd(4.0, 4.0, 8.0, 8.0, Color::rgba(0.0, 0.0, 0.0, 1.0)),
            PaintCommand::PopLayer,
        ];
        let nested = vec![
            rect_fill_cmd(0.0, 0.0, 16.0, 16.0, Color::WHITE),
            PaintCommand::PushLayer(SceneLayer::new().with_opacity(0.5)),
            PaintCommand::PushLayer(SceneLayer::new().with_opacity(0.5)),
            rect_fill_cmd(4.0, 4.0, 8.0, 8.0, Color::rgba(0.0, 0.0, 0.0, 1.0)),
            PaintCommand::PopLayer,
            PaintCommand::PopLayer,
        ];
        let single_red = render_commands(single, 16.0).pixel(8, 8).expect("px").red();
        let nested_red = render_commands(nested, 16.0).pixel(8, 8).expect("px").red();
        // ~0.25 black over white ≈ 191; ~0.5 ≈ 128. Nested is LIGHTER.
        assert!(
            nested_red > single_red,
            "nested 0.5*0.5 (~0.25, red {nested_red}) is lighter than single 0.5 (~0.5, red {single_red})"
        );
        assert!(nested_red > 160, "nested ~0.25 over white ≈ 191: {nested_red}");
    }

    #[test]
    fn trivial_layer_is_byte_identical_passthrough() {
        use obscur_scene::layer::SceneLayer;
        // A trivial (opacity 1.0) layer must not change any pixel vs. drawing the
        // box directly — the passthrough regression guard.
        let plain = vec![rect_fill_cmd(4.0, 4.0, 8.0, 8.0, Color::rgba(1.0, 0.0, 0.0, 1.0))];
        let layered = vec![
            PaintCommand::PushLayer(SceneLayer::new()),
            rect_fill_cmd(4.0, 4.0, 8.0, 8.0, Color::rgba(1.0, 0.0, 0.0, 1.0)),
            PaintCommand::PopLayer,
        ];
        assert_eq!(
            render_commands(plain, 16.0).data(),
            render_commands(layered, 16.0).data(),
            "a trivial layer is a byte-identical passthrough"
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

    #[test]
    fn register_font_populates_registry() {
        let mut renderer = CpuRenderer::default();
        assert!(format!("{renderer:?}").contains("registered_fonts: 0"));
        renderer.register_font(7, outline_font());
        assert!(format!("{renderer:?}").contains("registered_fonts: 1"));
        // Re-registering the same id replaces, not grows.
        renderer.register_font(7, outline_font());
        assert!(format!("{renderer:?}").contains("registered_fonts: 1"));
    }

    #[test]
    fn registered_font_draws_real_glyph_outline() {
        // Glyph 1 is a square over font units x∈[100,900], y∈[0,700]. At font
        // size 100 (upem 1000, scale 0.1) with baseline at device y=100, it fills
        // device x∈[10,90], y∈[30,100].
        let mut renderer = CpuRenderer::default();
        renderer.register_font(4, outline_font());
        renderer
            .render(&glyph_submission(1, 4), &bitmap_target(100.0, 120.0))
            .expect("renders");
        let pixmap = renderer.pixmap().expect("a frame was rendered");

        // A pixel well inside the outline is opaque.
        let inside = pixmap.pixel(50, 65).expect("pixel exists");
        assert!(inside.alpha() > 200, "glyph interior painted: {}", inside.alpha());

        // A pixel the OLD box-coverage fallback would have filled but the real
        // outline leaves empty (left of the square's x=10 edge, on the baseline
        // band) is transparent — proving a real shape, not a full cell.
        let left_margin = pixmap.pixel(3, 60).expect("pixel exists");
        assert_eq!(left_margin.alpha(), 0, "left margin outside real outline");

        // A pixel above the square's top (device y<30) is transparent, too.
        let above = pixmap.pixel(50, 10).expect("pixel exists");
        assert_eq!(above.alpha(), 0, "above real outline");

        // A far corner stays transparent.
        assert_eq!(pixmap.pixel(97, 5).expect("pixel exists").alpha(), 0);
    }

    #[test]
    fn empty_outline_glyph_draws_nothing() {
        // Glyph 0 is the empty .notdef: with a real font registered it must
        // contribute no coverage (no box fallback for a registered font).
        let mut renderer = CpuRenderer::default();
        renderer.register_font(4, outline_font());
        renderer
            .render(&glyph_submission(0, 4), &bitmap_target(100.0, 120.0))
            .expect("renders");
        let pixmap = renderer.pixmap().expect("a frame was rendered");
        assert!(
            pixmap.pixels().iter().all(|p| p.alpha() == 0),
            "empty glyph must paint nothing"
        );
    }

    #[test]
    fn unregistered_font_falls_back_to_box() {
        // Same run, but no font registered for its handle: the box-coverage
        // fallback fills the full advance×font_size cell, so the left margin that
        // the real outline left empty is opaque here.
        let mut renderer = CpuRenderer::default();
        let result = renderer.render(&glyph_submission(1, 4), &bitmap_target(100.0, 120.0));
        assert!(matches!(result, Ok(PresentResult::Presented { .. })));
        let pixmap = renderer.pixmap().expect("a frame was rendered");
        // Cell spans x∈[0,90], y∈[0,100]; the left margin (x=3, y=60) is filled.
        let left_margin = pixmap.pixel(3, 60).expect("pixel exists");
        assert!(
            left_margin.alpha() > 200,
            "box fallback fills the full cell: {}",
            left_margin.alpha()
        );
    }

    #[test]
    fn synth_oblique_shears_the_glyph_outline() {
        // Glyph 1 is a square over font units x∈[100,900], y∈[0,700]. Upright at
        // font size 100 with baseline device y=100 it fills device x∈[10,90],
        // y∈[30,100]. Faux-oblique shears above-baseline points right by
        // skew(0.25)×height: near the top (device y≈33, ~67px above baseline) the
        // square slides ≈16px right, so a pixel past the upright right edge is now
        // covered.
        let probe_x = 95;
        let probe_y = 33;

        // Upright: the probe (x=95 > right edge 90) is NOT covered.
        let mut upright = CpuRenderer::default();
        upright.register_font(4, outline_font());
        upright
            .render(&glyph_submission(1, 4), &bitmap_target(100.0, 120.0))
            .expect("renders");
        let upright_alpha = upright
            .pixmap()
            .expect("frame")
            .pixel(probe_x, probe_y)
            .expect("pixel")
            .alpha();
        assert_eq!(upright_alpha, 0, "upright glyph leaves the probe empty");

        // Oblique: the same probe IS covered by the sheared outline.
        let mut oblique = CpuRenderer::default();
        oblique.register_font(4, outline_font());
        oblique
            .render(
                &glyph_submission_synth(1, 4, true, false),
                &bitmap_target(100.0, 120.0),
            )
            .expect("renders");
        let oblique_alpha = oblique
            .pixmap()
            .expect("frame")
            .pixel(probe_x, probe_y)
            .expect("pixel")
            .alpha();
        assert!(
            oblique_alpha > 200,
            "sheared glyph covers the probe the upright one missed: {oblique_alpha}"
        );
    }

    #[test]
    fn synth_bold_thickens_the_glyph_outline() {
        // The square's right edge is device x=90. A pixel just past it (column 91)
        // is empty upright but caught by the faux-bold stroke (width font_size ×
        // 0.03 = 3px, so ≈1.5px beyond the edge).
        let probe_x = 91;
        let probe_y = 65;

        let mut plain = CpuRenderer::default();
        plain.register_font(4, outline_font());
        plain
            .render(&glyph_submission(1, 4), &bitmap_target(100.0, 120.0))
            .expect("renders");
        let plain_alpha = plain
            .pixmap()
            .expect("frame")
            .pixel(probe_x, probe_y)
            .expect("pixel")
            .alpha();

        let mut bold = CpuRenderer::default();
        bold.register_font(4, outline_font());
        bold.render(
            &glyph_submission_synth(1, 4, false, true),
            &bitmap_target(100.0, 120.0),
        )
        .expect("renders");
        let bold_alpha = bold
            .pixmap()
            .expect("frame")
            .pixel(probe_x, probe_y)
            .expect("pixel")
            .alpha();

        assert!(
            bold_alpha > plain_alpha,
            "faux-bold thickens the mark past the upright edge: bold {bold_alpha} > plain {plain_alpha}"
        );
    }

    #[test]
    fn outline_rasterization_is_deterministic() {
        let mut a = CpuRenderer::default();
        a.register_font(4, outline_font());
        a.render(&glyph_submission(1, 4), &bitmap_target(100.0, 120.0))
            .expect("renders");
        let mut b = CpuRenderer::default();
        b.register_font(4, outline_font());
        b.render(&glyph_submission(1, 4), &bitmap_target(100.0, 120.0))
            .expect("renders");
        assert_eq!(
            a.pixmap().expect("a").data(),
            b.pixmap().expect("b").data(),
            "same input + font must yield identical pixels"
        );
    }
}
