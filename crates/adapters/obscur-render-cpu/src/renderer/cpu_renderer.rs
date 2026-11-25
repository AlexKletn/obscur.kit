#![doc = include_str!("../../docs/api/renderer/cpu_renderer.doc/_module.md")]

use std::collections::HashMap;
use std::sync::Arc;

use obscur_capability::SupportLevel;
use obscur_diagnostics::{DiagnosticSink, NullDiagnosticSink};
use obscur_render::frame::{FrameSubmission, PresentResult};
use obscur_render::target::RenderTarget;
use obscur_render::{
    EffectFallback, EffectFallbackReporter, Renderer, RendererCapabilities, RendererError,
    RendererId, SceneEffect, text_run_font_handles, unsupported_effects,
};
use obscur_scene::command::PaintCommand;
use obscur_scene::layer::{CompositionMode, LayerEffect};
use obscur_scene::paint::{Brush, Paint, SceneImage};
use obscur_scene::path::Path;
use skrifa::FontRef;

use crate::error::CpuRenderError;
use crate::renderer::brush_fill::{brush_color, fill_conic, fill_linear, radial_paint};
use crate::renderer::clip_mask::{build_clip_mask, image_mask, resolve_clip_entry};
use crate::renderer::gaussian_blur::{MAX_SIGMA, blur_backdrop};
use crate::renderer::glyph_raster::draw_text_run;
use crate::renderer::image_tiling::draw_image;
use crate::renderer::layer_composite::LayerComposite;
use crate::renderer::render_scope::RenderScope;
use crate::renderer::shadow_approximation::draw_shadow_approx;
use crate::renderer::skia_lowering::{
    sk_dimensions, solid_paint, to_sk_blend_mode, to_sk_fill_rule, to_sk_path, to_sk_transform,
};

fn device_scale(transform: tiny_skia::Transform) -> f32 {
    (transform.sx * transform.sy - transform.kx * transform.ky)
        .abs()
        .sqrt()
}

#[doc = include_str!("../../docs/api/renderer/cpu_renderer.doc/CpuRenderer.md")]
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
    /// Where the §5.2 explicit fallback diagnostics go. Defaults to
    /// [`NullDiagnosticSink`], so an effect this backend will not reproduce follows
    /// exactly the same code path whether or not a host is listening. Shared
    /// (`Arc`) because a `CpuRenderer` is [`Clone`] and a clone reports to the same
    /// host sink.
    diagnostics: Arc<dyn DiagnosticSink>,
    /// Which unsupported effects have already been reported to `diagnostics`, so a
    /// standing gradient does not re-raise its diagnostic every frame.
    fallbacks: EffectFallbackReporter,
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
            diagnostics: Arc::new(NullDiagnosticSink),
            fallbacks: EffectFallbackReporter::new(),
        }
    }

    #[doc = include_str!("../../docs/api/renderer/cpu_renderer.doc/set_diagnostic_sink.md")]
    pub fn set_diagnostic_sink(&mut self, sink: Arc<dyn DiagnosticSink>) {
        self.diagnostics = sink;
    }

    #[doc = include_str!("../../docs/api/renderer/cpu_renderer.doc/register_font.md")]
    pub fn register_font(&mut self, id: u32, font_bytes: Vec<u8>) {
        self.fonts
            .insert(id, Arc::from(font_bytes.into_boxed_slice()));
    }

    #[doc = include_str!("../../docs/api/renderer/cpu_renderer.doc/pixmap.md")]
    #[must_use]
    pub fn pixmap(&self) -> Option<&tiny_skia::Pixmap> {
        self.last_pixmap.as_ref()
    }

    /// The number of frames this renderer has rasterized so far.
    #[must_use]
    pub const fn frames_rendered(&self) -> u64 {
        self.frames_rendered
    }

    #[doc = include_str!("../../docs/api/renderer/cpu_renderer.doc/text_fallbacks.md")]
    fn text_fallbacks(&self, list: &obscur_scene::command::DisplayList) -> Vec<EffectFallback> {
        text_run_font_handles(list)
            .into_iter()
            .filter(|handle| !self.has_outlines_for(*handle))
            .map(|handle| {
                EffectFallback::new(
                    SceneEffect::Text,
                    format!(
                        "no usable font is registered for handle {handle}, so its runs \
                         rasterize as advance-width coverage cells, not glyph outlines; \
                         register one with CpuRenderer::register_font"
                    ),
                )
            })
            .collect()
    }

    /// Whether a run tagged with `handle` will draw real outlines here.
    fn has_outlines_for(&self, handle: u32) -> bool {
        self.fonts
            .get(&handle)
            .is_some_and(|bytes| FontRef::new(bytes).is_ok())
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
    /// buffer is summarized by its presence rather than dumped, and the host's
    /// diagnostic sink is elided (it is a trait object).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CpuRenderer")
            .field("id", &self.id)
            .field("capabilities", &self.capabilities)
            .field("has_pixmap", &self.last_pixmap.is_some())
            .field("frames_rendered", &self.frames_rendered)
            .field("registered_fonts", &self.fonts.len())
            .field("reported_fallbacks", &self.fallbacks.reported())
            .finish_non_exhaustive()
    }
}

impl Renderer for CpuRenderer {
    fn id(&self) -> RendererId {
        self.id
    }

    fn capabilities(&self) -> RendererCapabilities {
        self.capabilities
    }

    #[doc = include_str!("../../docs/api/renderer/cpu_renderer.doc/render.md")]
    fn render(
        &mut self,
        submission: &FrameSubmission,
        target: &RenderTarget,
    ) -> Result<PresentResult, RendererError> {
        self.render_with_pixmap_allocator(submission, target, allocate_pixmap)
    }

    #[doc = include_str!("../../docs/api/renderer/cpu_renderer.doc/supports_effect.md")]
    fn supports_effect(&self, effect: SceneEffect) -> SupportLevel {
        match effect {
            SceneEffect::Image => SupportLevel::Native,
            SceneEffect::Clip => SupportLevel::Emulated,
            SceneEffect::Shadow
            | SceneEffect::Text
            | SceneEffect::Gradient
            | SceneEffect::BlendMode
            | SceneEffect::ColorMatrix
            | SceneEffect::Blur
            | SceneEffect::BackdropFilter => SupportLevel::Limited,
            SceneEffect::ImageMask => SupportLevel::Emulated,
            SceneEffect::Mask => SupportLevel::Unavailable,
        }
    }
}

impl CpuRenderer {
    fn render_with_pixmap_allocator(
        &mut self,
        submission: &FrameSubmission,
        target: &RenderTarget,
        mut allocate_pixmap: impl FnMut(u32, u32) -> Result<tiny_skia::Pixmap, RendererError>,
    ) -> Result<PresentResult, RendererError> {
        let size = target.size();
        let (width, height) = sk_dimensions(size).ok_or_else(|| {
            RendererError::InvalidTarget(
                format!(
                    "cpu renderer target has invalid dimensions {}x{}",
                    size.width, size.height
                )
                .into(),
            )
        })?;
        if width == 0 || height == 0 {
            return Err(CpuRenderError::EmptyTarget.into_renderer_error());
        }
        if SceneImage::rgba8_byte_size(width, height).is_none() {
            return Err(RendererError::InvalidTarget(
                format!("RGBA8 buffer for {width}x{height} pixels exceeds addressable size").into(),
            ));
        }
        let root_pixmap = allocate_pixmap(width, height)?;

        let display_list = submission.frame().scene().display_list();
        // Repeated draw commands can hit the same fallback in one frame.
        let mut frame_fallbacks = Vec::new();
        let mut record_fallback = |effect, reason| {
            if !frame_fallbacks.contains(&(effect, reason)) {
                frame_fallbacks.push((effect, reason));
            }
        };

        // The layer/target stack: every fill / text / shadow op draws into the
        // *top* target. The root frame buffer is the bottom entry; a non-trivial
        // `PushLayer` (a CSS `opacity` group) pushes a fresh transparent target of
        // the same size, so the whole group rasterizes offscreen, and the matching
        // `PopLayer` composites it back onto the parent at the layer's opacity —
        // one group blend, so overlapping opaque children are NOT double-darkened
        // (real group compositing, not per-element alpha). Trivial layers stay a
        // cheap passthrough (no offscreen), keeping the plain path byte-identical.
        let mut targets: Vec<tiny_skia::Pixmap> = vec![root_pixmap];
        // A single entry keeps clipping and compositing in sync for each push.
        let mut scopes: Vec<RenderScope> = Vec::new();
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
                    let clip = scopes.last().and_then(RenderScope::clip);
                    let Some(pixmap) = targets.last_mut() else {
                        continue;
                    };
                    match paint {
                        // A conic (angular) gradient is rasterized for real on the
                        // CPU backend: it is decomposed into angular wedges filled at
                        // their swept color, clipped to the element outline (§5.1's
                        // "conic gradients").
                        Paint::Fill(Brush::Conic(conic)) => {
                            fill_conic(
                                pixmap,
                                conic,
                                &sk_path,
                                to_sk_fill_rule(path.fill_rule()),
                                width,
                                height,
                                clip,
                                current,
                            );
                        }
                        // A linear (axial) gradient is rasterized for real via
                        // tiny-skia's linear shader. tiny-skia interpolates its stop
                        // colors in straight-alpha space, but css-color-4 §12.3 (and
                        // §5.7's "premultiplied alpha") require interpolation in
                        // PREMULTIPLIED space, so the authored stops are pre-expanded
                        // into a dense stop list whose samples already follow the
                        // premultiplied curve (`fill_linear`).
                        Paint::Fill(Brush::Linear(linear)) => {
                            fill_linear(
                                pixmap,
                                linear,
                                &sk_path,
                                to_sk_fill_rule(path.fill_rule()),
                                clip,
                                current,
                            );
                        }
                        Paint::Fill(Brush::Radial(gradient)) => {
                            if let Some(paint) = radial_paint(gradient) {
                                pixmap.fill_path(
                                    &sk_path,
                                    &paint,
                                    to_sk_fill_rule(path.fill_rule()),
                                    current,
                                    clip,
                                );
                            }
                        }
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
                            let paint = match stroke.brush() {
                                Brush::Radial(gradient) => radial_paint(gradient),
                                brush => brush_color(brush).and_then(solid_paint),
                            };
                            let Some(sk_paint) = paint else {
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
                    let layer_depth = scopes
                        .iter()
                        .rev()
                        .find_map(|scope| scope.layer_transform_depth);
                    if layer_depth.is_none_or(|depth| transform_stack.len() > depth + 1) {
                        transform_stack.pop();
                    }
                }
                PaintCommand::PushClip(clip) => {
                    let parent = scopes.last().and_then(RenderScope::clip);
                    if let Some(mask) = image_mask(clip, parent, width, height, current) {
                        targets.push(allocate_pixmap(width, height)?);
                        scopes.push(RenderScope {
                            clip: None,
                            layer_transform_depth: None,
                            composite: Some(LayerComposite {
                                opacity: 1.0,
                                blend: tiny_skia::BlendMode::SourceOver,
                                color_matrix: None,
                                blur: 0.0,
                                mask: Some(mask),
                            }),
                        });
                    } else {
                        scopes.push(RenderScope {
                            clip: resolve_clip_entry(clip, parent, width, height, current),
                            composite: None,
                            layer_transform_depth: None,
                        });
                    }
                }
                // A CSS `opacity` / `mix-blend-mode` group layer. A layer with
                // reduced opacity OR a non-`Normal` blend mode pushes a fresh
                // transparent offscreen target the whole group rasterizes into;
                // `PopLayer` composites it back at that opacity and blend, so the
                // group blends as one (overlapping opaque children are not double-
                // darkened, and the group multiplies/screens/… onto the backdrop as
                // a whole — compositing-1 §5). A trivial layer (opacity `1.0`, blend
                // `Normal`) stays a passthrough: it pushes no offscreen, so the plain
                // path is byte-identical. A color matrix also opens an offscreen
                // and filters the group before blending it back.
                PaintCommand::PushLayer(layer) => {
                    let layer_transform_depth = transform_stack.len();
                    let current = current.pre_concat(to_sk_transform(&layer.transform()));
                    transform_stack.push(current);
                    let composition = layer.composition();
                    let blend = match composition {
                        CompositionMode::SrcOver => to_sk_blend_mode(layer.blend_mode()),
                        CompositionMode::Src => tiny_skia::BlendMode::Source,
                        CompositionMode::DstOver => tiny_skia::BlendMode::DestinationOver,
                        CompositionMode::Clear => tiny_skia::BlendMode::Clear,
                    };
                    let replaces_destination =
                        matches!(composition, CompositionMode::Src | CompositionMode::Clear);
                    if !composition.is_src_over() && !layer.blend_mode().is_normal() {
                        record_fallback(
                            SceneEffect::BlendMode,
                            "CPU applies the composition operator without the non-normal blend mode",
                        );
                    }
                    let color_matrix = layer
                        .effects()
                        .iter()
                        .find_map(|effect| effect.as_color_matrix());
                    let blur = layer
                        .effects()
                        .iter()
                        .find_map(|effect| match effect {
                            LayerEffect::Blur { radius } if *radius > 0.0 => Some(*radius),
                            _ => None,
                        })
                        .unwrap_or(0.0);
                    let scale = device_scale(current);
                    let blur = blur * scale;
                    if blur > MAX_SIGMA {
                        record_fallback(
                            SceneEffect::Blur,
                            "CPU layer blur is capped at 16 device pixels",
                        );
                    }
                    let inherited_clip = scopes.last().and_then(RenderScope::clip);
                    let mask = layer.clip().and_then(|clip| {
                        image_mask(clip, inherited_clip, width, height, current).or_else(|| {
                            replaces_destination
                                .then(|| {
                                    resolve_clip_entry(clip, inherited_clip, width, height, current)
                                })
                                .flatten()
                        })
                    });
                    let needs_offscreen = mask.is_some()
                        || blur > 0.0
                        || color_matrix.is_some()
                        || layer.is_isolated()
                        || layer.opacity() < 1.0
                        || blend != tiny_skia::BlendMode::SourceOver;
                    let offscreen = needs_offscreen
                        .then(|| allocate_pixmap(width, height))
                        .transpose()?;
                    // Apply the enclosing clip after convolution so pixels beyond
                    // its edge can still contribute to the filtered result.
                    let parent = if (blur > 0.0 || mask.is_some() || replaces_destination)
                        && offscreen.is_some()
                    {
                        None
                    } else {
                        scopes.last().and_then(RenderScope::clip)
                    };
                    let entry = if replaces_destination && offscreen.is_some() {
                        None
                    } else {
                        layer.clip().map_or_else(
                            || parent.cloned(),
                            |clip| resolve_clip_entry(clip, parent, width, height, current),
                        )
                    };
                    let composite = offscreen.map(|offscreen| {
                        targets.push(offscreen);
                        LayerComposite {
                            opacity: layer.opacity(),
                            blend,
                            color_matrix,
                            blur,
                            mask,
                        }
                    });
                    scopes.push(RenderScope {
                        clip: entry,
                        composite,
                        layer_transform_depth: Some(layer_transform_depth),
                    });
                }
                PaintCommand::PopLayer | PaintCommand::PopClip => {
                    let closes_layer = matches!(command, PaintCommand::PopLayer);
                    if !scopes
                        .last()
                        .is_some_and(|scope| scope.layer_transform_depth.is_some() == closes_layer)
                    {
                        continue;
                    }
                    if let Some(scope) = scopes.pop() {
                        scope.finish(
                            &mut transform_stack,
                            &mut targets,
                            scopes.last().and_then(RenderScope::clip),
                        );
                    }
                }
                PaintCommand::DrawText { origin, run, brush } => {
                    let clip = scopes.last().and_then(RenderScope::clip);
                    let Some(pixmap) = targets.last_mut() else {
                        continue;
                    };
                    draw_text_run(pixmap, *origin, run, brush, clip, current, &self.fonts);
                }
                PaintCommand::DrawShadow {
                    rect,
                    radii,
                    color,
                    blur,
                    spread,
                    offset,
                    inset,
                } => {
                    let clip = scopes.last().and_then(RenderScope::clip);
                    let Some(pixmap) = targets.last_mut() else {
                        continue;
                    };
                    let sigma = (blur.max(0.0) / 2.0) * device_scale(current);
                    if sigma > MAX_SIGMA {
                        record_fallback(
                            SceneEffect::Blur,
                            "CPU shadow blur is capped at 16 device pixels",
                        );
                    }
                    draw_shadow_approx(
                        pixmap, *rect, *radii, *color, sigma, *spread, *offset, *inset, clip,
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
                    let clip = scopes.last().and_then(RenderScope::clip);
                    let Some(pixmap) = targets.last_mut() else {
                        continue;
                    };
                    draw_image(
                        pixmap, image, *tile, *region, *repeat_x, *repeat_y, clip, current,
                    );
                }
                PaintCommand::BackdropFilter {
                    bounds,
                    radii,
                    blur,
                } => {
                    let scale = device_scale(current);
                    let sigma = blur * scale;
                    if sigma.is_nan()
                        || sigma <= 0.0
                        || bounds.size.width <= 0.0
                        || bounds.size.height <= 0.0
                    {
                        continue;
                    }
                    if sigma > MAX_SIGMA {
                        record_fallback(
                            SceneEffect::BackdropFilter,
                            "CPU backdrop blur is capped at 16 device pixels",
                        );
                    }
                    let parent = scopes.last().and_then(RenderScope::clip);
                    let Some(panel) = to_sk_path(&Path::rounded_rect(*bounds, *radii)) else {
                        continue;
                    };
                    let Some(coverage) = build_clip_mask(
                        width,
                        height,
                        &panel,
                        tiny_skia::FillRule::Winding,
                        parent,
                        current,
                    ) else {
                        continue;
                    };
                    if let Some(target) = targets.last_mut() {
                        blur_backdrop(target, &coverage, sigma);
                    }
                }
            }
        }

        // Unwind any layer left open by an unbalanced list: composite each dangling
        // offscreen back at its opacity so its content is not lost, leaving only the
        // root frame buffer.
        while let Some(scope) = scopes.pop() {
            scope.finish(
                &mut transform_stack,
                &mut targets,
                scopes.last().and_then(RenderScope::clip),
            );
        }
        // The root frame buffer is the only remaining target (a well-formed list
        // leaves exactly one; the unwind above guarantees it for a malformed one).
        let pixmap = targets.pop().ok_or_else(|| {
            RendererError::Backend("cpu renderer lost the root pixel buffer".into())
        })?;

        let unsupported = unsupported_effects(&*self, display_list);
        let mut degraded = self.text_fallbacks(display_list);
        degraded.extend(
            frame_fallbacks
                .into_iter()
                .map(|(effect, reason)| EffectFallback::new(effect, reason)),
        );
        self.fallbacks
            .report(&unsupported, self.diagnostics.as_ref());
        self.fallbacks
            .report_fallbacks(&degraded, self.diagnostics.as_ref());
        self.last_pixmap = Some(pixmap);
        self.frames_rendered += 1;
        Ok(PresentResult::Presented {
            generation: submission.generation(),
        })
    }
}

fn allocate_pixmap(width_px: u32, height_px: u32) -> Result<tiny_skia::Pixmap, RendererError> {
    tiny_skia::Pixmap::new(width_px, height_px).ok_or_else(|| {
        CpuRenderError::PixmapAllocationFailed {
            width: width_px,
            height: height_px,
        }
        .into_renderer_error()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::{AffineTransform, CornerRadii, Point, Rect, Size};
    use obscur_render::frame::RenderFrame;
    use obscur_render::target::BitmapTarget;
    use obscur_scene::clip::Clip;
    use obscur_scene::color::{AlphaMode, Color};
    use obscur_scene::command::DisplayList;
    use obscur_scene::path::Path;
    use obscur_scene::scene::{SceneGeneration, SceneSnapshot};
    use obscur_scene::text::{GlyphRun, PositionedGlyph};

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
    fn glyph_submission_synth(
        gid: u32,
        font_id: u32,
        oblique: bool,
        bold: bool,
    ) -> FrameSubmission {
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

    /// A [`DiagnosticSink`] that keeps every message it is handed, so a test can
    /// assert what a host would have been told.
    #[derive(Default)]
    struct RecordingSink {
        messages: std::sync::Mutex<Vec<String>>,
    }

    impl RecordingSink {
        fn messages(&self) -> Vec<String> {
            self.messages.lock().expect("sink not poisoned").clone()
        }
    }

    impl DiagnosticSink for RecordingSink {
        fn emit(&self, diagnostic: &obscur_diagnostics::Diagnostic) {
            self.messages
                .lock()
                .expect("sink not poisoned")
                .push(diagnostic.message.clone());
        }
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
        assert!(
            on2.alpha() > 200,
            "second dash painted: alpha={}",
            on2.alpha()
        );
    }

    #[test]
    fn failed_layer_buffer_preserves_the_last_rendered_frame() {
        use obscur_scene::layer::SceneLayer;
        use obscur_scene::paint::BlendMode;

        let mut renderer = CpuRenderer::new(RendererId::from_raw(1));
        let sink = Arc::new(RecordingSink::default());
        renderer.set_diagnostic_sink(sink.clone());
        let target = bitmap_target(16.0, 16.0);
        renderer.render(&red_rect_submission(), &target).unwrap();
        let previous = renderer.pixmap().unwrap().data().to_vec();
        let commands = DisplayList::from_commands(vec![
            PaintCommand::PushLayer(
                SceneLayer::new()
                    .with_composition(CompositionMode::Src)
                    .with_blend_mode(BlendMode::Multiply),
            ),
            PaintCommand::PopLayer,
        ]);
        let frame = FrameSubmission::new(RenderFrame::new(
            SceneSnapshot::new(commands, SceneGeneration::from_raw(2)),
            Size::new(16.0, 16.0),
        ));
        let failure = RendererError::Backend("test offscreen allocation failure".into());
        let mut allocations = 0;
        let result = renderer.render_with_pixmap_allocator(&frame, &target, |width, height| {
            allocations += 1;
            if allocations == 1 {
                allocate_pixmap(width, height)
            } else {
                Err(failure.clone())
            }
        });

        assert_eq!(allocations, 2);
        assert_eq!(result, Err(failure));
        assert_eq!(renderer.frames_rendered(), 1);
        assert_eq!(renderer.pixmap().unwrap().data(), previous);
        assert!(sink.messages().is_empty());

        renderer.render(&frame, &target).unwrap();
        assert_eq!(sink.messages().len(), 1);
        assert!(sink.messages()[0].contains("without the non-normal blend mode"));
    }

    #[test]
    fn invalid_target_preserves_the_last_rendered_frame() {
        let mut renderer = CpuRenderer::new(RendererId::from_raw(1));
        let submission = red_rect_submission();
        renderer
            .render(&submission, &bitmap_target(16.0, 16.0))
            .unwrap();
        let previous = renderer.pixmap().unwrap().data().to_vec();
        for width in [f32::INFINITY, 4_294_967_296.0] {
            assert!(matches!(
                renderer.render(&submission, &bitmap_target(width, 16.0)),
                Err(RendererError::InvalidTarget(_))
            ));
            assert_eq!(renderer.frames_rendered(), 1);
            assert_eq!(renderer.pixmap().unwrap().data(), previous);
        }
        let huge = bitmap_target(3_000_000_000.0, 3_000_000_000.0);
        assert!(matches!(
            renderer.render(&submission, &huge),
            Err(RendererError::InvalidTarget(reason)) if reason.contains("exceeds addressable size")
        ));
        assert_eq!(renderer.frames_rendered(), 1);
        assert_eq!(renderer.pixmap().unwrap().data(), previous);
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
    fn supports_effect_reports_the_gpu_class_effects_unavailable() {
        let renderer = CpuRenderer::default();
        assert_eq!(
            renderer.supports_effect(SceneEffect::BackdropFilter),
            SupportLevel::Limited
        );
        // Fills and radial strokes use gradients. Other strokes and text
        // still take the representative first-stop color.
        assert_eq!(
            renderer.supports_effect(SceneEffect::Gradient),
            SupportLevel::Limited
        );
        assert_eq!(
            renderer.supports_effect(SceneEffect::BlendMode),
            SupportLevel::Limited
        );
        assert_eq!(
            renderer.supports_effect(SceneEffect::Mask),
            SupportLevel::Unavailable
        );
    }

    #[test]
    fn shadow_blur_spreads_coverage_beyond_the_silhouette() {
        // The pixels are the authority on the table. A 24x24 box shadow with an
        // 8px blur radius, no offset and no spread: if this backend blurred, the
        // shadow would spill well outside the box and fade across its edge.
        let rect = Rect::new(Point::new(12.0, 12.0), Size::new(24.0, 24.0));
        let list = DisplayList::from_commands(vec![PaintCommand::DrawShadow {
            rect,
            radii: CornerRadii::uniform(0.0),
            color: Color::BLACK,
            blur: 8.0,
            spread: 0.0,
            offset: (0.0, 0.0),
            inset: false,
        }]);
        let submission = FrameSubmission::new(RenderFrame::new(
            SceneSnapshot::new(list, SceneGeneration::from_raw(1)),
            Size::new(48.0, 48.0),
        ));
        let mut renderer = CpuRenderer::default();
        let _ = renderer.render(&submission, &bitmap_target(48.0, 48.0));
        let pixmap = renderer.pixmap().expect("a frame was rendered");

        let outside = pixmap.pixel(8, 24).expect("pixel left of the box");
        let edge = pixmap.pixel(13, 24).expect("pixel inside the left edge");
        let center = pixmap.pixel(24, 24).expect("center pixel");
        assert!(outside.alpha() > 0);
        assert!(outside.alpha() < edge.alpha());
        assert!(edge.alpha() < center.alpha());

        assert_eq!(
            renderer.supports_effect(SceneEffect::Blur),
            SupportLevel::Limited
        );
        // The shadow is still drawn — offset, spread, radii and color are honored —
        // so the shadow itself is Limited, not Unavailable.
        assert_eq!(
            renderer.supports_effect(SceneEffect::Shadow),
            SupportLevel::Limited
        );
    }

    #[test]
    fn a_spread_shadows_corner_is_eased_not_grown_raw() {
        // Spec §6.1.1, in pixels: the rule is only real if the rasterizer obeys it.
        // A 100x100 box with a 50px radius and a 100px spread (r = .5, the spec's
        // own ratio). The shadow rect is the box inflated by the spread — origin
        // (0,0), 300x300 — and its corner radius is
        //   50 + 100 x (1 + (.5 - 1)^3) = 137.5,   NOT the raw 50 + 100 = 150.
        // Neither radius is clamped by §4.5 on a 300x300 rect (137.5*2 and 150*2
        // both fit), so nothing downstream can mask the difference.
        //
        // The two arcs differ where it counts: on the top-left diagonal a corner of
        // radius R starts painting at t = R * (1 - 1/sqrt(2)), i.e. t = 40.27 for
        // 137.5 but t = 43.93 for 150. Pixel (42,42) sits between them — inside the
        // eased corner, cut away by the raw one. It is the whole bug, one pixel.
        let rect = Rect::new(Point::new(100.0, 100.0), Size::new(100.0, 100.0));
        let list = DisplayList::from_commands(vec![PaintCommand::DrawShadow {
            rect,
            radii: CornerRadii::uniform(50.0),
            color: Color::BLACK,
            blur: 0.0,
            spread: 100.0,
            offset: (0.0, 0.0),
            inset: false,
        }]);
        let submission = FrameSubmission::new(RenderFrame::new(
            SceneSnapshot::new(list, SceneGeneration::from_raw(1)),
            Size::new(300.0, 300.0),
        ));
        let mut renderer = CpuRenderer::default();
        let _ = renderer.render(&submission, &bitmap_target(300.0, 300.0));
        let pixmap = renderer.pixmap().expect("a frame was rendered");

        // Control: deep inside the shadow, painted under either radius. If this is
        // blank the probe below proves nothing (a shadow that never drew would
        // "pass" a test asserting absence).
        assert!(
            pixmap.pixel(150, 150).expect("center").alpha() > 0,
            "the spread shadow must actually be drawn"
        );
        // Control: (20,20) is outside BOTH arcs (t = 20 < 40.27) — this is the
        // corner being cut at all, and it anchors which corner is under test.
        assert_eq!(
            pixmap.pixel(20, 20).expect("outer corner").alpha(),
            0,
            "the shadow's top-left corner must be rounded away here"
        );
        // Control: (60,60) is inside both (t = 60 > 43.93).
        assert!(
            pixmap.pixel(60, 60).expect("inner diagonal").alpha() > 0,
            "the corner must be filled past both arcs"
        );
        // The discriminating pixel. Raw growth (radius 150) rounds this away.
        assert!(
            pixmap.pixel(42, 42).expect("the eased corner").alpha() > 0,
            "§6.1.1: the spread shadow's corner radius must be the eased 137.5, \
             not the raw 150 — a raw +spread cuts this pixel away"
        );
    }

    #[test]
    fn an_inset_shadows_negative_spread_eases_the_hole_it_grows() {
        // §6.1.1's *second* trigger, which only an inset shadow can reach: "or in
        // the case of an inner shadow, less than the absolute value of a negative
        // spread distance". An inset shadow CONTRACTS by `spread`, so a negative
        // spread GROWS its hole — and growth is the eased direction.
        //
        // 600x600 box, 50px radius, spread -100, offset (300,300). The lit hole is
        // the box grown by 100 and shifted by 300: origin (200,200), 800x800, with
        // radii 50 + 100 x (1 + (.5 - 1)^3) = 137.5 eased, 150 raw. The offset is
        // what makes this observable at all: with no offset a negative-spread hole
        // swallows the whole box and nothing paints.
        //
        // On the hole's top-left diagonal the arc starts at t = (200 + R) - R/sqrt(2),
        // i.e. 240.27 eased vs 243.93 raw. Pixel (242,242) is inside the eased hole
        // (unpainted) but outside the raw one (painted) — the sign of the bug is
        // INVERTED here versus the drop-shadow probe, which is exactly why this
        // case needs its own test.
        let rect = Rect::new(Point::new(0.0, 0.0), Size::new(600.0, 600.0));
        let list = DisplayList::from_commands(vec![PaintCommand::DrawShadow {
            rect,
            radii: CornerRadii::uniform(50.0),
            color: Color::BLACK,
            blur: 0.0,
            spread: -100.0,
            offset: (300.0, 300.0),
            inset: true,
        }]);
        let submission = FrameSubmission::new(RenderFrame::new(
            SceneSnapshot::new(list, SceneGeneration::from_raw(1)),
            Size::new(600.0, 600.0),
        ));
        let mut renderer = CpuRenderer::default();
        let _ = renderer.render(&submission, &bitmap_target(600.0, 600.0));
        let pixmap = renderer.pixmap().expect("a frame was rendered");

        // Control: well outside the hole — the inset shadow is drawn at all.
        assert!(
            pixmap.pixel(100, 100).expect("lit band").alpha() > 0,
            "the inset shadow must actually be drawn"
        );
        // Control: deep inside the hole under either radius — the hole exists.
        assert_eq!(
            pixmap.pixel(400, 400).expect("hole centre").alpha(),
            0,
            "the lit hole must be punched out"
        );
        // Control: inside the hole's corner bite under either radius, so the corner
        // is genuinely rounded and this diagonal is the one under test.
        assert!(
            pixmap.pixel(230, 230).expect("corner bite").alpha() > 0,
            "the hole's corner must be rounded away here"
        );
        // The discriminating pixel: a raw -spread grows the hole's radius to 150
        // and uncovers this pixel.
        assert_eq!(
            pixmap.pixel(242, 242).expect("the eased hole edge").alpha(),
            0,
            "§6.1.1: an inset shadow's negative spread must ease the hole to 137.5, \
             not grow it raw to 150 — a raw radius leaves this pixel lit"
        );
    }

    #[test]
    fn shadow_blur_radius_cap_is_reported_once() {
        let sink = Arc::new(RecordingSink::default());
        let mut renderer = CpuRenderer::default();
        renderer.set_diagnostic_sink(sink.clone());

        let list = DisplayList::from_commands(vec![PaintCommand::DrawShadow {
            rect: Rect::new(Point::new(2.0, 2.0), Size::new(8.0, 8.0)),
            radii: CornerRadii::uniform(0.0),
            color: Color::BLACK,
            blur: 40.0,
            spread: 0.0,
            offset: (0.0, 0.0),
            inset: false,
        }]);
        let submission = FrameSubmission::new(RenderFrame::new(
            SceneSnapshot::new(list, SceneGeneration::from_raw(1)),
            Size::new(16.0, 16.0),
        ));
        let result = renderer.render(&submission, &bitmap_target(16.0, 16.0));

        // The frame still presents: the CPU backend approximates by design, and the
        // diagnostic is how the host learns what the approximation cost.
        assert!(result.expect("presented").is_presented());
        assert_eq!(
            sink.messages(),
            vec![
                "degraded effect: blur (CPU shadow blur is capped at 16 device pixels)".to_string()
            ],
            "the shadow is drawn at the reported radius cap"
        );

        // A second frame demanding the same effect does not re-raise it: this is a
        // property of the backend, not of the frame.
        let _ = renderer.render(&submission, &bitmap_target(16.0, 16.0));
        assert_eq!(sink.messages().len(), 1);
    }

    #[test]
    fn a_scene_within_the_backends_means_raises_nothing() {
        let sink = Arc::new(RecordingSink::default());
        let mut renderer = CpuRenderer::default();
        renderer.set_diagnostic_sink(sink.clone());
        // A solid fill demands no effect this backend lacks.
        let _ = renderer.render(&red_rect_submission(), &bitmap_target(16.0, 16.0));
        assert!(sink.messages().is_empty());
    }

    /// A one-glyph run tagged with font handle `handle`, in a 16×16 frame.
    fn text_submission(handle: u32) -> FrameSubmission {
        let run = GlyphRun::new(vec![PositionedGlyph::new(1, 0, 0.0, 12.0, 8.0)], 12.0, "a")
            .with_font(handle);
        let list = DisplayList::from_commands(vec![PaintCommand::DrawText {
            origin: Point::new(1.0, 1.0),
            run,
            brush: Brush::solid(Color::BLACK),
        }]);
        FrameSubmission::new(RenderFrame::new(
            SceneSnapshot::new(list, SceneGeneration::from_raw(1)),
            Size::new(16.0, 16.0),
        ))
    }

    #[test]
    fn a_text_run_without_its_font_tells_the_host_it_degraded() {
        // Spec §5.2: unsupported effects AND FALLBACKS are explicit diagnostics.
        // `Text` is `Limited`, so it raises no `unsupported effect` — yet a run whose
        // font was never registered does not draw its glyphs, it draws boxes. The
        // table declares that restriction; only this says it actually bit.
        let sink = Arc::new(RecordingSink::default());
        let mut renderer = CpuRenderer::default();
        renderer.set_diagnostic_sink(sink.clone());

        let result = renderer.render(&text_submission(3), &bitmap_target(16.0, 16.0));

        // The frame still presents: the fallback IS the documented approximation.
        assert!(result.expect("presented").is_presented());
        let messages = sink.messages();
        assert_eq!(messages.len(), 1, "one fallback, once: {messages:?}");
        assert!(
            messages[0].starts_with(
                "degraded effect: text (no usable font is registered \
                                     for handle 3"
            ),
            "the diagnostic must name the effect and the handle: {messages:?}"
        );

        // A second frame with the same gap does not re-raise it.
        let _ = renderer.render(&text_submission(3), &bitmap_target(16.0, 16.0));
        assert_eq!(sink.messages().len(), 1);
    }

    #[test]
    fn a_text_run_with_its_font_registered_raises_nothing() {
        // The other direction, and the one that keeps the diagnostic worth reading: a
        // run that draws real outlines has not fallen back, so it must stay silent.
        // Reporting `Limited` unconditionally would fire on every well-fonted frame.
        let sink = Arc::new(RecordingSink::default());
        let mut renderer = CpuRenderer::default();
        renderer.set_diagnostic_sink(sink.clone());
        renderer.register_font(3, outline_font());

        let _ = renderer.render(&text_submission(3), &bitmap_target(16.0, 16.0));

        assert!(
            sink.messages().is_empty(),
            "a run drawing real outlines has not degraded: {:?}",
            sink.messages()
        );
    }

    #[test]
    fn a_font_registered_under_another_handle_does_not_cover_this_run() {
        // The predicate must be `draw_text_run`'s: an EXACT registry hit. This
        // backend has no id-0 primary fallback (Vello does), so a run tagged 3 with
        // only handle 9 registered draws boxes — and must say so.
        let sink = Arc::new(RecordingSink::default());
        let mut renderer = CpuRenderer::default();
        renderer.set_diagnostic_sink(sink.clone());
        renderer.register_font(9, outline_font());
        renderer.register_font(0, outline_font());

        let _ = renderer.render(&text_submission(3), &bitmap_target(16.0, 16.0));

        assert_eq!(sink.messages().len(), 1, "handle 3 is still uncovered");
    }

    #[test]
    fn registered_but_unparseable_bytes_are_still_a_fallback() {
        // `draw_text_run` needs bytes `skrifa` accepts, not merely a registry entry,
        // so a registry-only scan would call this run covered and stay silent while
        // the paint path drew boxes.
        let sink = Arc::new(RecordingSink::default());
        let mut renderer = CpuRenderer::default();
        renderer.set_diagnostic_sink(sink.clone());
        renderer.register_font(3, vec![0xAA; 64]);

        let _ = renderer.render(&text_submission(3), &bitmap_target(16.0, 16.0));

        assert_eq!(
            sink.messages().len(),
            1,
            "unparseable bytes draw boxes, exactly like no bytes"
        );
    }

    #[test]
    fn an_empty_text_run_is_not_a_fallback() {
        // A run with no glyphs draws nothing whether or not its font resolves, so a
        // missing font costs it nothing and there is nothing to report.
        let sink = Arc::new(RecordingSink::default());
        let mut renderer = CpuRenderer::default();
        renderer.set_diagnostic_sink(sink.clone());
        let run = GlyphRun::new(Vec::new(), 12.0, "").with_font(3);
        let list = DisplayList::from_commands(vec![PaintCommand::DrawText {
            origin: Point::new(1.0, 1.0),
            run,
            brush: Brush::solid(Color::BLACK),
        }]);
        let submission = FrameSubmission::new(RenderFrame::new(
            SceneSnapshot::new(list, SceneGeneration::from_raw(1)),
            Size::new(16.0, 16.0),
        ));

        let _ = renderer.render(&submission, &bitmap_target(16.0, 16.0));

        assert!(sink.messages().is_empty());
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

    /// Renders a 2×2 image stretched over (4,4)..(12,12) on white and answers the
    /// pixel at the centre.
    fn image_over_white(image: SceneImage, generation: u64) -> tiny_skia::PremultipliedColorU8 {
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
        let snapshot = SceneSnapshot::new(list, SceneGeneration::from_raw(generation));
        let submission = FrameSubmission::new(RenderFrame::new(snapshot, Size::new(16.0, 16.0)));
        let mut renderer = CpuRenderer::default();
        renderer
            .render(&submission, &bitmap_target(16.0, 16.0))
            .expect("renders");
        renderer
            .pixmap()
            .expect("a frame was rendered")
            .pixel(8, 8)
            .expect("centre pixel exists")
    }

    #[test]
    fn a_premultiplied_image_is_passed_through_and_agrees_with_the_straight_one() {
        // The premultiplied branch had NO test at all, and it is the one that
        // borrows the image's bytes instead of duplicating them — so this is
        // where a passthrough that quietly stopped being a passthrough would
        // show up.
        //
        // The same colour said twice: half-alpha red as straight (255,0,0,128)
        // and as its own premultiplied form (128,0,0,128). Over white they must
        // land on the same pixel, because they ARE the same picture.
        let straight = image_over_white(solid_image(255, 0, 0, 128, AlphaMode::Straight), 31);
        let premultiplied =
            image_over_white(solid_image(128, 0, 0, 128, AlphaMode::Premultiplied), 32);

        assert_eq!(
            (
                premultiplied.red(),
                premultiplied.green(),
                premultiplied.blue()
            ),
            (straight.red(), straight.green(), straight.blue()),
            "the borrowed path and the converted one paint the same colour"
        );
        // And it really is a half-alpha red over white, not an opaque red or an
        // untouched background: both channels sit near the midpoint.
        assert!(
            premultiplied.red() > 200,
            "red stays high over white: {premultiplied:?}"
        );
        assert!(
            (100..200).contains(&premultiplied.green()),
            "and green is halfway, not 0 (opaque red) or 255 (no image): {premultiplied:?}"
        );
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
        assert!(
            inside.red() > 200 && inside.blue() < 40,
            "region interior is red"
        );
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
        assert!(
            px.green() > 90 && px.green() < 180,
            "green mid: {}",
            px.green()
        );
    }

    #[test]
    fn supports_effect_reports_images_native() {
        let renderer = CpuRenderer::default();
        assert_eq!(
            renderer.supports_effect(SceneEffect::Image),
            SupportLevel::Native
        );
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
        assert!(
            inside.alpha() > 200,
            "glyph cell painted: {}",
            inside.alpha()
        );
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
    fn blurred_shadow_follows_its_offset() {
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

        let inside = pixmap.pixel(9, 9).expect("pixel exists");
        assert!(inside.alpha() > 60, "shadow painted: {}", inside.alpha());
        assert!(
            inside.alpha() < 255,
            "shadow is reduced alpha: {}",
            inside.alpha()
        );
        // A pixel outside the offset rect (top-left corner) is untouched.
        assert_eq!(pixmap.pixel(23, 23).expect("pixel exists").alpha(), 0);
    }

    #[test]
    fn blurred_inset_shadow_softens_the_hole_and_stays_inside_the_box() {
        let pixels = render_commands(
            vec![PaintCommand::DrawShadow {
                rect: Rect::new(Point::new(4.0, 4.0), Size::new(24.0, 24.0)),
                radii: CornerRadii::uniform(2.0),
                color: Color::BLACK,
                blur: 8.0,
                spread: 0.0,
                offset: (0.0, 0.0),
                inset: true,
            }],
            32.0,
        );
        let alpha = |x| pixels.pixel(x, 16).unwrap().alpha();
        assert_eq!(alpha(3), 0);
        assert!(alpha(4) > alpha(7));
        assert!(alpha(7) > alpha(16));
        assert!(alpha(16) <= 1);
    }

    #[test]
    fn inset_shadow_paints_inner_ring_confined_to_the_box() {
        // `box-shadow: 0 0 0 3px black inset` on the box (4,4)-(20,20): the software
        // backend paints an inner frame (a 3px ring) INSIDE the box — the box edge
        // is shadowed, the box center is clear, and nothing outside the box is
        // touched (an inset shadow never bleeds out).
        let list = DisplayList::from_commands(vec![PaintCommand::DrawShadow {
            rect: obscur_geometry::Rect::new(Point::new(4.0, 4.0), Size::new(16.0, 16.0)),
            radii: CornerRadii::default(),
            color: Color::rgba(0.0, 0.0, 0.0, 1.0),
            blur: 0.0,
            spread: 3.0,
            offset: (0.0, 0.0),
            inset: true,
        }]);
        let snapshot = SceneSnapshot::new(list, SceneGeneration::from_raw(9));
        let frame = RenderFrame::new(snapshot, Size::new(24.0, 24.0));
        let submission = FrameSubmission::new(frame);

        let mut renderer = CpuRenderer::default();
        renderer
            .render(&submission, &bitmap_target(24.0, 24.0))
            .expect("renders");
        let pixmap = renderer.pixmap().expect("a frame was rendered");

        // A pixel just inside the box edge (within the 3px frame) is shadowed.
        let ring = pixmap.pixel(5, 12).expect("pixel exists");
        assert!(ring.alpha() > 200, "inner ring painted: {}", ring.alpha());
        // The box center (past the 3px frame) is clear — a ring, not a filled blob.
        assert_eq!(
            pixmap.pixel(12, 12).expect("pixel exists").alpha(),
            0,
            "box interior stays clear"
        );
        // A pixel OUTSIDE the box is untouched: the inset shadow is confined.
        assert_eq!(
            pixmap.pixel(1, 12).expect("pixel exists").alpha(),
            0,
            "inset shadow does not bleed outside the box"
        );
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
        assert!(
            shifted.red() > 200 && shifted.alpha() > 200,
            "translated rect painted"
        );
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
    fn alpha_image_mask_composites_overlapping_children_once() {
        use obscur_scene::clip::Mask;
        use obscur_scene::paint::SceneImage;

        let image = SceneImage::new(1, 1, vec![255, 255, 255, 128], AlphaMode::Straight).unwrap();
        let bounds = Rect::new(Point::new(0.0, 0.0), Size::new(8.0, 8.0));
        let pixels = render_commands(
            vec![
                PaintCommand::PushTransform(AffineTransform::translate(4.0, 4.0)),
                PaintCommand::PushClip(Clip::image_mask(Mask::alpha(), image, bounds)),
                PaintCommand::PopTransform,
                rect_fill_cmd(0.0, 0.0, 16.0, 16.0, Color::WHITE),
                rect_fill_cmd(6.0, 6.0, 4.0, 4.0, Color::WHITE),
                PaintCommand::PopClip,
            ],
            16.0,
        );
        assert_eq!(pixels.pixel(5, 5).unwrap().alpha(), 128);
        assert_eq!(pixels.pixel(8, 8).unwrap().alpha(), 128);
        assert_eq!(pixels.pixel(3, 8).unwrap().alpha(), 0);
    }

    #[test]
    fn luminance_image_mask_multiplies_linear_luminance_and_alpha() {
        use obscur_scene::clip::Mask;
        use obscur_scene::layer::SceneLayer;
        use obscur_scene::paint::SceneImage;

        let image = SceneImage::new(1, 1, vec![128, 0, 0, 128], AlphaMode::Premultiplied).unwrap();
        let bounds = Rect::new(Point::new(2.0, 2.0), Size::new(8.0, 8.0));
        let layer = SceneLayer::new().with_clip(Clip::image_mask(Mask::luminance(), image, bounds));
        let pixels = render_commands(
            vec![
                PaintCommand::PushLayer(layer),
                rect_fill_cmd(0.0, 0.0, 16.0, 16.0, Color::WHITE),
                PaintCommand::PopLayer,
            ],
            16.0,
        );
        assert_eq!(pixels.pixel(5, 5).unwrap().alpha(), 27);
        assert_eq!(pixels.pixel(1, 5).unwrap().alpha(), 0);
    }

    #[test]
    fn backdrop_blur_replaces_pixels_inside_the_rounded_panel_and_parent_clip() {
        let white = Color::rgba(1.0, 1.0, 1.0, 0.5);
        let panel = Rect::new(Point::new(8.0, 4.0), Size::new(16.0, 24.0));
        let commands = vec![
            rect_fill_cmd(0.0, 0.0, 16.0, 32.0, white),
            PaintCommand::PushClip(Clip::rect(Rect::new(
                Point::new(0.0, 0.0),
                Size::new(32.0, 20.0),
            ))),
            PaintCommand::BackdropFilter {
                bounds: panel,
                radii: CornerRadii::uniform(6.0),
                blur: 2.0,
            },
            PaintCommand::PopClip,
        ];
        let pixels = render_commands(commands, 32.0);
        let alpha = |x, y| pixels.pixel(x, y).unwrap().alpha();
        assert!((50..100).contains(&alpha(15, 16)));
        assert!((20..80).contains(&alpha(16, 16)));
        assert_eq!(alpha(8, 4), 128);
        assert_eq!(alpha(16, 21), 0);
        assert_eq!(alpha(15, 2), 128);
    }

    #[test]
    fn color_matrix_filters_the_layer_in_straight_alpha() {
        use obscur_scene::layer::{LayerEffect, SceneLayer};

        let mut layer = SceneLayer::new();
        layer.add_effect(LayerEffect::color_matrix([
            0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0,
            0.0, 0.0, 0.0,
        ]));
        let pixmap = render_commands(
            vec![
                PaintCommand::PushLayer(layer),
                rect_fill_cmd(4.0, 4.0, 8.0, 8.0, Color::rgba(1.0, 0.0, 0.0, 0.5)),
                PaintCommand::PopLayer,
            ],
            16.0,
        );
        let pixel = pixmap.pixel(8, 8).unwrap();
        assert_eq!(
            [pixel.red(), pixel.green(), pixel.blue(), pixel.alpha()],
            [0, 64, 0, 64]
        );
        assert_eq!(pixmap.pixel(0, 0).unwrap().alpha(), 0);
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
        assert!(
            px.red() > 90 && px.red() < 170,
            "mid-grey red: {}",
            px.red()
        );
        assert!(
            px.green() > 90 && px.green() < 170,
            "mid-grey green: {}",
            px.green()
        );
        assert!(
            px.blue() > 90 && px.blue() < 170,
            "mid-grey blue: {}",
            px.blue()
        );
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
        assert!(
            overlap.red() > 90,
            "group blended to grey, not black: {}",
            overlap.red()
        );
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
        assert!(
            nested_red > 160,
            "nested ~0.25 over white ≈ 191: {nested_red}"
        );
    }

    #[test]
    fn isolated_group_does_not_blend_its_children_with_the_outer_backdrop() {
        use obscur_scene::layer::SceneLayer;
        use obscur_scene::paint::BlendMode;

        let sample = |isolated| {
            render_commands(
                vec![
                    rect_fill_cmd(0.0, 0.0, 16.0, 16.0, Color::rgba(1.0, 0.0, 0.0, 1.0)),
                    PaintCommand::PushLayer(SceneLayer::new().with_isolation(isolated)),
                    PaintCommand::PushLayer(SceneLayer::new().with_blend_mode(BlendMode::Multiply)),
                    rect_fill_cmd(0.0, 0.0, 16.0, 16.0, Color::rgba(0.0, 0.0, 1.0, 1.0)),
                    PaintCommand::PopLayer,
                    PaintCommand::PopLayer,
                ],
                16.0,
            )
            .pixel(8, 8)
            .expect("center pixel")
        };
        assert_eq!(sample(false).blue(), 0);
        assert_eq!(sample(true).blue(), 255);
    }

    #[test]
    fn blend_effect_overrides_the_layer_mode() {
        use obscur_scene::layer::SceneLayer;
        use obscur_scene::paint::BlendMode;

        let mut layer = SceneLayer::new().with_blend_mode(BlendMode::Screen);
        layer.add_effect(LayerEffect::blend(BlendMode::Multiply));
        let image = render_commands(
            vec![
                rect_fill_cmd(0.0, 0.0, 16.0, 16.0, Color::rgba(1.0, 0.0, 0.0, 1.0)),
                PaintCommand::PushLayer(layer),
                rect_fill_cmd(0.0, 0.0, 16.0, 16.0, Color::rgba(0.0, 0.0, 1.0, 1.0)),
                PaintCommand::PopLayer,
            ],
            16.0,
        );
        let pixel = image.pixel(8, 8).unwrap();
        assert_eq!(
            [pixel.red(), pixel.green(), pixel.blue(), pixel.alpha()],
            [0, 0, 0, 255]
        );
    }

    #[test]
    fn layer_composition_respects_the_destination_and_clip() {
        use obscur_geometry::{Point, Rect, Size};
        use obscur_scene::{clip::Clip, layer::SceneLayer};

        for (mode, overlap, empty_source) in [
            (CompositionMode::Src, [0, 0, 255, 255], [0; 4]),
            (CompositionMode::Clear, [0; 4], [0; 4]),
            (CompositionMode::DstOver, [255, 0, 0, 255], [255, 0, 0, 255]),
        ] {
            let bounds =
                |x, width| Clip::rect(Rect::new(Point::new(x, 0.0), Size::new(width, 16.0)));
            let image = render_commands(
                vec![
                    rect_fill_cmd(0.0, 0.0, 16.0, 16.0, Color::rgba(1.0, 0.0, 0.0, 1.0)),
                    PaintCommand::PushClip(bounds(2.0, 10.0)),
                    PaintCommand::PushLayer(
                        SceneLayer::new()
                            .with_composition(mode)
                            .with_clip(bounds(4.0, 10.0)),
                    ),
                    rect_fill_cmd(0.0, 0.0, 8.0, 16.0, Color::rgba(0.0, 0.0, 1.0, 1.0)),
                    PaintCommand::PopLayer,
                    PaintCommand::PopClip,
                ],
                16.0,
            );
            let pixel = |x| {
                let pixel = image.pixel(x, 8).unwrap();
                [pixel.red(), pixel.green(), pixel.blue(), pixel.alpha()]
            };
            assert_eq!(pixel(6), overlap, "{mode:?}");
            assert_eq!(pixel(10), empty_source, "{mode:?}");
            for outside in [3, 13] {
                assert_eq!(pixel(outside), [255, 0, 0, 255], "{mode:?}");
            }
        }
    }

    #[test]
    fn trivial_layer_is_byte_identical_passthrough() {
        use obscur_scene::layer::SceneLayer;
        // A trivial (opacity 1.0) layer must not change any pixel vs. drawing the
        // box directly — the passthrough regression guard.
        let plain = vec![rect_fill_cmd(
            4.0,
            4.0,
            8.0,
            8.0,
            Color::rgba(1.0, 0.0, 0.0, 1.0),
        )];
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
    fn mismatched_pop_does_not_remove_a_clip_or_layer() {
        use obscur_scene::layer::SceneLayer;

        let clip = PaintCommand::PushClip(Clip::rect(obscur_geometry::Rect::new(
            Point::new(0.0, 0.0),
            Size::new(4.0, 4.0),
        )));
        let layer = PaintCommand::PushLayer(SceneLayer::new().with_opacity(0.5));
        for (open, stray, close) in [
            (clip, PaintCommand::PopLayer, PaintCommand::PopClip),
            (layer, PaintCommand::PopClip, PaintCommand::PopLayer),
        ] {
            let draw = rect_fill_cmd(0.0, 0.0, 8.0, 8.0, Color::WHITE);
            let expected = render_commands(vec![open.clone(), draw.clone(), close.clone()], 8.0);
            let actual = render_commands(vec![open, stray, draw, close], 8.0);
            assert_eq!(actual.data(), expected.data());
        }
    }

    #[test]
    fn transform_pops_cannot_cross_layer_boundaries() {
        use obscur_geometry::AffineTransform;
        use obscur_scene::layer::SceneLayer;

        let open = PaintCommand::PushLayer(
            SceneLayer::new().with_transform(AffineTransform::translate(3.0, 0.0)),
        );
        let inside = rect_fill_cmd(0.0, 0.0, 2.0, 2.0, Color::WHITE);
        let outside = rect_fill_cmd(0.0, 4.0, 2.0, 2.0, Color::WHITE);
        let expected = render_commands(
            vec![
                open.clone(),
                inside.clone(),
                PaintCommand::PopLayer,
                outside.clone(),
            ],
            8.0,
        );
        let actual = render_commands(
            vec![
                open,
                PaintCommand::PopTransform,
                inside,
                PaintCommand::PushTransform(AffineTransform::translate(2.0, 0.0)),
                PaintCommand::PopLayer,
                outside,
            ],
            8.0,
        );
        assert_eq!(actual.data(), expected.data());
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
        assert!(
            inside.alpha() > 200,
            "glyph interior painted: {}",
            inside.alpha()
        );

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
