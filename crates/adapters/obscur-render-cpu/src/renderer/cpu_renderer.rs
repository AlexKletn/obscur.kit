#![doc = include_str!("cpu_renderer.doc/_module.md")]

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
use obscur_scene::paint::{Brush, Paint};
use skrifa::FontRef;

use crate::error::CpuRenderError;
use crate::renderer::brush_fill::{brush_color, fill_conic, fill_linear};
use crate::renderer::clip_mask::resolve_clip_entry;
use crate::renderer::glyph_raster::draw_text_run;
use crate::renderer::group_compositing::composite_top_layer;
use crate::renderer::image_tiling::draw_image;
use crate::renderer::shadow_approximation::draw_shadow_approx;
use crate::renderer::skia_lowering::{
    sk_dimensions, solid_paint, to_sk_blend_mode, to_sk_fill_rule, to_sk_path, to_sk_transform,
};

#[doc = include_str!("cpu_renderer.doc/CpuRenderer.md")]
#[derive(Clone)]
pub struct CpuRenderer {
    #[doc = include_str!("cpu_renderer.doc/id.md")]
    id: RendererId,
    #[doc = include_str!("cpu_renderer.doc/capabilities.md")]
    capabilities: RendererCapabilities,
    #[doc = include_str!("cpu_renderer.doc/last_pixmap.md")]
    last_pixmap: Option<tiny_skia::Pixmap>,
    #[doc = include_str!("cpu_renderer.doc/frames_rendered.md")]
    frames_rendered: u64,
    #[doc = include_str!("cpu_renderer.doc/fonts.md")]
    fonts: HashMap<u32, Arc<[u8]>>,
    #[doc = include_str!("cpu_renderer.doc/diagnostics.md")]
    diagnostics: Arc<dyn DiagnosticSink>,
    #[doc = include_str!("cpu_renderer.doc/fallbacks.md")]
    fallbacks: EffectFallbackReporter,
}

impl CpuRenderer {
    #[doc = include_str!("cpu_renderer.doc/new.md")]
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

    #[doc = include_str!("cpu_renderer.doc/set_diagnostic_sink.md")]
    pub fn set_diagnostic_sink(&mut self, sink: Arc<dyn DiagnosticSink>) {
        self.diagnostics = sink;
    }

    #[doc = include_str!("cpu_renderer.doc/register_font.md")]
    pub fn register_font(&mut self, id: u32, font_bytes: Vec<u8>) {
        self.fonts
            .insert(id, Arc::from(font_bytes.into_boxed_slice()));
    }

    #[doc = include_str!("cpu_renderer.doc/pixmap.md")]
    #[must_use]
    pub fn pixmap(&self) -> Option<&tiny_skia::Pixmap> {
        self.last_pixmap.as_ref()
    }

    #[doc = include_str!("cpu_renderer.doc/frames_rendered_2.md")]
    #[must_use]
    pub const fn frames_rendered(&self) -> u64 {
        self.frames_rendered
    }

    #[doc = include_str!("cpu_renderer.doc/text_fallbacks.md")]
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

    #[doc = include_str!("cpu_renderer.doc/has_outlines_for.md")]
    fn has_outlines_for(&self, handle: u32) -> bool {
        self.fonts
            .get(&handle)
            .is_some_and(|bytes| FontRef::new(bytes).is_ok())
    }
}

impl Default for CpuRenderer {
    #[doc = include_str!("cpu_renderer.doc/default.md")]
    fn default() -> Self {
        Self::new(RendererId::from_raw(0))
    }
}

impl std::fmt::Debug for CpuRenderer {
    #[doc = include_str!("cpu_renderer.doc/fmt.md")]
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

    #[doc = include_str!("cpu_renderer.doc/render.md")]
    fn render(
        &mut self,
        submission: &FrameSubmission,
        target: &RenderTarget,
    ) -> Result<PresentResult, RendererError> {
        let (width, height) = sk_dimensions(target.size());
        if width == 0 || height == 0 {
            return Err(CpuRenderError::EmptyTarget.into_renderer_error());
        }

        // Spec §5.2: "Unsupported effects and fallbacks are explicit diagnostics."
        // Tell the host, before drawing, every effect this scene asks for that the
        // rasterization below will not reproduce — then draw the documented
        // approximation anyway, which is the CPU backend's standing policy.
        let display_list = submission.frame().scene().display_list();
        let unsupported = unsupported_effects(&*self, display_list);
        let sink = Arc::clone(&self.diagnostics);
        self.fallbacks.report(&unsupported, sink.as_ref());
        // The other half of that sentence: an effect this backend *does* draw, but
        // whose documented restriction bites on THIS scene. Unlike the list above,
        // it cannot be read off `supports_effect` — `Text` is `Limited` whether or
        // not the host registered the font a given run needs.
        let degraded = self.text_fallbacks(display_list);
        self.fallbacks.report_fallbacks(&degraded, sink.as_ref());

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
        // One entry per open `PushLayer`. A layer that pushed an offscreen records
        // how to composite it back: its opacity and its (separable) blend mode.
        // `None` marks a trivial (passthrough) layer that pushed nothing. Keeps
        // pushes and pops balanced regardless of whether a layer needed an
        // offscreen pass.
        let mut layer_stack: Vec<Option<(f32, tiny_skia::BlendMode)>> = Vec::new();

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
                        // A conic (angular) gradient is rasterized for real on the
                        // CPU backend: it is decomposed into angular wedges filled at
                        // their swept color, clipped to the element outline (§5.1's
                        // "conic gradients"). The radial brush still flattens
                        // (documented approximation, `brush_color`).
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
                    let entry = resolve_clip_entry(clip, parent, width, height, current);
                    clip_stack.push(entry);
                }
                PaintCommand::PopClip => {
                    let _ = clip_stack.pop();
                }
                // A CSS `opacity` / `mix-blend-mode` group layer. A layer with
                // reduced opacity OR a non-`Normal` blend mode pushes a fresh
                // transparent offscreen target the whole group rasterizes into;
                // `PopLayer` composites it back at that opacity and blend, so the
                // group blends as one (overlapping opaque children are not double-
                // darkened, and the group multiplies/screens/… onto the backdrop as
                // a whole — compositing-1 §5). A trivial layer (opacity `1.0`, blend
                // `Normal`) stays a passthrough: it pushes no offscreen, so the plain
                // path is byte-identical. The layer's own transform / effect fields
                // are still not honored here — a documented approximation.
                PaintCommand::PushLayer(layer) => {
                    // A layer's own clip confines its subtree exactly as a
                    // `PushClip` of the same region would, and it reaches this
                    // backend in production: the coordinator brackets every
                    // `filter: blur()` / color-matrix group with `with_clip`.
                    // Vello honors it (`push_layer`'s bounding shape), so ignoring
                    // it here painted content the author clipped away and made the
                    // two backends disagree (spec §5.2: clipping must match). One
                    // entry per push keeps the stack balanced with `PopLayer`
                    // whether or not the layer carries a clip.
                    let parent = clip_stack.last().and_then(Option::as_ref);
                    let entry = layer.clip().map_or_else(
                        || parent.cloned(),
                        |clip| resolve_clip_entry(clip, parent, width, height, current),
                    );
                    clip_stack.push(entry);
                    let blend = to_sk_blend_mode(layer.blend_mode());
                    let needs_offscreen =
                        layer.opacity() < 1.0 || blend != tiny_skia::BlendMode::SourceOver;
                    if needs_offscreen {
                        match targets
                            .last()
                            .and_then(|top| tiny_skia::Pixmap::new(top.width(), top.height()))
                        {
                            Some(offscreen) => {
                                targets.push(offscreen);
                                layer_stack.push(Some((layer.opacity().clamp(0.0, 1.0), blend)));
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
                    // The layer's clip leaves force with the layer that opened it.
                    // Popping unconditionally (rather than only when a layer is
                    // open) is what keeps a malformed list agreeing with Vello,
                    // which counts clips and layers on one shared depth and pops
                    // whatever is on top for either kind of pop.
                    let _ = clip_stack.pop();
                    // Balance a stray pop; composite the offscreen when this layer
                    // opened one.
                    if let Some(Some((opacity, blend))) = layer_stack.pop() {
                        composite_top_layer(&mut targets, opacity, blend);
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
                    blur,
                    spread,
                    offset,
                    inset,
                } => {
                    let clip = clip_stack.last().and_then(Option::as_ref);
                    let Some(pixmap) = targets.last_mut() else {
                        continue;
                    };
                    draw_shadow_approx(
                        pixmap, *rect, *radii, *color, *blur, *spread, *offset, *inset, clip,
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
                    draw_image(
                        pixmap, image, *tile, *region, *repeat_x, *repeat_y, clip, current,
                    );
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
            if let Some((opacity, blend)) = entry {
                composite_top_layer(&mut targets, opacity, blend);
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

    #[doc = include_str!("cpu_renderer.doc/supports_effect.md")]
    fn supports_effect(&self, effect: SceneEffect) -> SupportLevel {
        match effect {
            SceneEffect::Image => SupportLevel::Native,
            SceneEffect::Clip => SupportLevel::Emulated,
            SceneEffect::Shadow | SceneEffect::Text => SupportLevel::Limited,
            SceneEffect::Blur
            | SceneEffect::BackdropFilter
            | SceneEffect::ColorMatrix
            | SceneEffect::Gradient
            | SceneEffect::Mask
            | SceneEffect::BlendMode => SupportLevel::Unavailable,
        }
    }
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
    use obscur_scene::paint::SceneImage;
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
        // There is no gaussian pass anywhere in this backend: a blurred shadow
        // draws hard, a `filter: blur()` layer draws sharp, a backdrop-filter draws
        // nothing, a color-matrix layer draws unfiltered. Dropping the parameter is
        // NOT emulation.
        for effect in [
            SceneEffect::Blur,
            SceneEffect::BackdropFilter,
            SceneEffect::ColorMatrix,
        ] {
            assert_eq!(
                renderer.supports_effect(effect),
                SupportLevel::Unavailable,
                "{effect} has no pass in the software rasterizer"
            );
        }
        // A gradient brush lands as a flat solid of its first stop, and a layer's
        // blend mode and a mask clip's coverage are dropped outright.
        for effect in [
            SceneEffect::Gradient,
            SceneEffect::BlendMode,
            SceneEffect::Mask,
        ] {
            assert_eq!(
                renderer.supports_effect(effect),
                SupportLevel::Unavailable,
                "{effect} is dropped, not approximated"
            );
        }
        // What it does do: shadows without their blur, text against a registered
        // font, per-pixel clip masks, and real sampled images.
        assert_eq!(
            renderer.supports_effect(SceneEffect::Shadow),
            SupportLevel::Limited
        );
        assert_eq!(
            renderer.supports_effect(SceneEffect::Text),
            SupportLevel::Limited
        );
        assert_eq!(
            renderer.supports_effect(SceneEffect::Clip),
            SupportLevel::Emulated
        );
        assert_eq!(
            renderer.supports_effect(SceneEffect::Image),
            SupportLevel::Native
        );
    }

    #[test]
    fn blur_is_unavailable_because_a_blurred_shadow_rasterizes_hard_edged() {
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

        // 4px outside the box, well inside an 8px blur's reach: untouched.
        let outside = pixmap.pixel(8, 24).expect("pixel left of the box");
        assert_eq!(outside.alpha(), 0, "an 8px blur would have tinted this");
        // 1px inside the edge vs. the center: identical, so the edge is a step, not
        // a ramp.
        let edge = pixmap.pixel(13, 24).expect("pixel inside the left edge");
        let center = pixmap.pixel(24, 24).expect("center pixel");
        assert_eq!(
            edge.alpha(),
            center.alpha(),
            "a blurred edge would fade toward the boundary"
        );
        assert!(center.alpha() > 0, "the shadow itself is drawn");

        // Which is exactly what the table must say. Should a real gaussian ever
        // land here, the asserts above fail first and this claim gets revisited.
        assert_eq!(
            renderer.supports_effect(SceneEffect::Blur),
            SupportLevel::Unavailable
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
    fn rendering_an_unsupported_effect_tells_the_host_which_one() {
        // Spec §5.2: unsupported effects are EXPLICIT diagnostics. Rendering a
        // blurred shadow must name the blur it is about to drop.
        let sink = Arc::new(RecordingSink::default());
        let mut renderer = CpuRenderer::default();
        renderer.set_diagnostic_sink(sink.clone());

        let list = DisplayList::from_commands(vec![PaintCommand::DrawShadow {
            rect: Rect::new(Point::new(2.0, 2.0), Size::new(8.0, 8.0)),
            radii: CornerRadii::uniform(0.0),
            color: Color::BLACK,
            blur: 4.0,
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
            vec!["unsupported effect: blur".to_string()],
            "the shadow itself is drawn (Limited); only its blur is unavailable"
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
        assert!(
            inside.alpha() < 200,
            "shadow is reduced alpha: {}",
            inside.alpha()
        );
        // A pixel outside the offset rect (top-left corner) is untouched.
        assert_eq!(pixmap.pixel(0, 0).expect("pixel exists").alpha(), 0);
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
