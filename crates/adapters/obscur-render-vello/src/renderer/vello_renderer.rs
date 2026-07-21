//! The Vello/wgpu implementation of the [`Renderer`] port (spec §5.2, §2.7).

use std::sync::Arc;

use obscur_capability::SupportLevel;
use obscur_diagnostics::{DiagnosticSink, NullDiagnosticSink};
use obscur_render::frame::{FrameSubmission, PresentResult};
use obscur_render::renderer::{
    EffectFallback, EffectFallbackReporter, Renderer, RendererCapabilities, RendererError,
    RendererId, SceneEffect, text_run_font_handles, unsupported_effects,
};
use obscur_render::target::RenderTarget;

use obscur_scene::command::{DisplayList, PaintCommand};

use vello::Scene;
use vello::kurbo::Affine;
use vello::peniko::{Blob, FontData};

use crate::conversion::{BlurLayer, SceneConverter, SceneSegment};
use crate::effects::{ColorMatrix, GaussianBlur};
use crate::error::VelloRenderError;
use crate::renderer::{VelloDevice, VelloFrame, VelloSurface};

/// A GPU-accelerated [`Renderer`] backed by Vello and wgpu (spec §5.2).
///
/// # Optional device
///
/// The GPU device is host-provided and therefore **optional** (spec §2.7): a
/// freshly constructed `VelloRenderer` holds no [`VelloDevice`], and
/// [`Renderer::render`] then returns an honest
/// [`RendererError::DeviceLost`] rather than fabricating a device (which would
/// require the async `request_adapter` path the adapter refuses to own). A host
/// attaches a device with [`Self::with_device`] / [`Self::set_device`] once it
/// has created the wgpu context. Everything except the actual GPU submission —
/// identity, capabilities, effect support, scene conversion — works without a
/// device and is unit-tested that way.
///
/// # Effect support
///
/// The advertised capability profile is a GPU backend's: offscreen output, MSAA,
/// custom shaders (gating the trusted extensions of spec §5.3), no HDR. See
/// [`Renderer::supports_effect`] for the honest per-effect [`SupportLevel`].
pub struct VelloRenderer {
    id: RendererId,
    capabilities: RendererCapabilities,
    device: Option<VelloDevice>,
    scene_converter: SceneConverter,
    frames_rendered: u64,
    /// Cached surface blitter for the on-screen present path, keyed by the
    /// surface format it was built for. Rebuilding a `TextureBlitter` recompiles
    /// its render pipeline, so we keep one alive across frames and only rebuild
    /// when absent or when the surface format changes.
    blitter: Option<(wgpu::TextureFormat, wgpu::util::TextureBlitter)>,
    /// Cached offscreen render target, reused while the extent holds.
    ///
    /// Vello renders into an offscreen and the result is blitted to the surface
    /// (the surface cannot carry the STORAGE_BINDING usage Vello needs). That
    /// target used to be allocated **per frame** — a full-window RGBA texture
    /// created and dropped 60-120 times a second, which dominated present. Vello
    /// clears it to `base_color` on every render, so reuse shows nothing stale.
    frame_cache: Option<VelloFrame>,
    /// Cached owned wgpu gaussian-blur pipeline set for `filter: blur()` (spec
    /// §5.2). Built lazily on the first blurred frame (it needs the device) and
    /// reused across frames; a scene with no blur layer never constructs it, so
    /// the non-blur path is byte-identical to before.
    blur: Option<GaussianBlur>,
    /// Cached owned wgpu color-matrix pipeline for the CSS color-matrix `filter`
    /// functions (grayscale/brightness/contrast/saturate/sepia/invert/hue-rotate/
    /// opacity). Built lazily on the first color-filtered frame and reused; a scene
    /// with no color-matrix layer never constructs it (byte-identical to before).
    color_matrix: Option<ColorMatrix>,
    /// Where the §5.2 explicit fallback diagnostics go; [`NullDiagnosticSink`]
    /// until a host installs one, so an unreproducible effect follows the same code
    /// path whether or not anyone is listening.
    diagnostics: Arc<dyn DiagnosticSink>,
    /// Which unsupported effects have already been reported, so a standing mask
    /// clip does not re-raise its diagnostic every frame.
    fallbacks: EffectFallbackReporter,
}

impl VelloRenderer {
    /// Creates a device-less Vello renderer with the given `id`.
    ///
    /// The capability profile is the Vello GPU backend's: offscreen and MSAA
    /// capable, custom shaders permitted (spec §5.3), HDR not offered. A host
    /// must call [`Self::set_device`] before [`Renderer::render`] can submit GPU
    /// work.
    #[must_use]
    pub fn new(id: RendererId) -> Self {
        Self {
            id,
            capabilities: RendererCapabilities::new()
                .offscreen(true)
                .msaa(true)
                .hdr(false)
                .custom_shaders(true),
            device: None,
            scene_converter: SceneConverter::new(),
            frames_rendered: 0,
            blitter: None,
            frame_cache: None,
            blur: None,
            color_matrix: None,
            diagnostics: Arc::new(NullDiagnosticSink),
            fallbacks: EffectFallbackReporter::new(),
        }
    }

    /// Installs the [`DiagnosticSink`] the §5.2 explicit diagnostics are emitted to.
    /// Two kinds arrive here, each raised the first time it applies:
    ///
    /// - a [`RendererError::UnsupportedEffect`] per effect a rendered scene demands
    ///   that this backend reports [`Unavailable`](SupportLevel::Unavailable);
    /// - an [`EffectFallback`] per effect it *did* draw, approximated, because the
    ///   restriction its [`Limited`](SupportLevel::Limited) level names actually bit
    ///   on that scene — today, a text run this renderer has no font for, which is
    ///   dropped silently and so is invisible in the output.
    ///
    /// Not every approximation is announced — the text fallback above is the only one
    /// today. [`Renderer::supports_effect`] remains the complete statement of what
    /// this backend does with an effect.
    ///
    /// Installing a sink is the **host's** job and nothing in this workspace does it:
    /// the default is a [`NullDiagnosticSink`], and the `Renderer` port carries no
    /// sink, so a generic runtime cannot install one — only whoever constructs this
    /// concrete backend can. Until it is called, both diagnostics above are built and
    /// dropped. That is deliberate (the render path does not branch on whether anyone
    /// is listening), but it does mean §5.2's "explicit diagnostics" are *available*
    /// here, not *delivered*.
    pub fn set_diagnostic_sink(&mut self, sink: Arc<dyn DiagnosticSink>) {
        self.diagnostics = sink;
    }

    /// Attaches a host-provided [`VelloDevice`], returning the updated renderer.
    #[must_use]
    pub fn with_device(mut self, device: VelloDevice) -> Self {
        self.device = Some(device);
        self
    }

    /// Attaches (or replaces) the host-provided [`VelloDevice`].
    pub fn set_device(&mut self, device: VelloDevice) {
        self.device = Some(device);
    }

    /// Registers (or replaces) a host-provided font under the run font handle
    /// `id` (`0` = primary; higher ids are fallback faces).
    ///
    /// Fonts are a host resource, mirroring the host-provides-device pattern:
    /// the adapter never bundles one, so the host reads a font file and hands
    /// the raw bytes here. The bytes are wrapped in a peniko [`FontData`] (sfnt
    /// face index 0) and forwarded to the scene converter, which draws each
    /// [`DrawText`](obscur_scene::command::PaintCommand::DrawText) run against
    /// the font whose id matches the run's
    /// [`font`](obscur_scene::text::GlyphRun::font) handle.
    ///
    /// The `id` **must** match the face id the runtime's font resolver (e.g.
    /// `FontRegistry`) assigned to the same bytes, so scene run handles line up
    /// with these fonts. Until a covering font is registered, its runs are
    /// skipped.
    pub fn register_font(&mut self, id: u32, font_bytes: Vec<u8>) {
        let font = FontData::new(Blob::from(font_bytes), 0);
        self.scene_converter.register_font(id, font);
    }

    /// Attaches (or replaces) the primary host font (run font handle `0`).
    ///
    /// Back-compat shim over [`Self::register_font`]: single-font hosts keep
    /// working, and every untagged run renders against this face. Until this (or
    /// [`Self::register_font`]) is called, text runs are skipped.
    pub fn set_font(&mut self, font_bytes: Vec<u8>) {
        self.register_font(0, font_bytes);
    }

    /// The [`SceneEffect::Text`] fallbacks this backend will take drawing `list`:
    /// one per font handle whose runs will be skipped for want of a font
    /// (spec §5.2).
    ///
    /// This backend's restriction on `Text` is harsher than the CPU rasterizer's —
    /// there is no box-coverage stand-in here, the run is simply not drawn — which
    /// is why naming it matters most: nothing appears, so there is no artifact for a
    /// host to notice.
    ///
    /// The predicate is [`SceneConverter::can_draw_text`], i.e. `font_for`'s registry
    /// lookup, because that is the only gate the draw path applies. It is therefore
    /// **narrower than the CPU backend's**, which also requires the bytes to parse:
    /// registering unparseable bytes here satisfies this scan and raises nothing,
    /// and Vello's own glyph pipeline then **panics** rather than drawing nothing
    /// (`Scene::draw_glyphs` unwraps its `FontRef`). Reporting it would mean parsing
    /// fonts this backend never parses, so it is left un-diagnosed rather than
    /// guessed at — but the failure is a crash, not a blank run, and a host that
    /// registers untrusted bytes owns that.
    fn text_fallbacks(&self, list: &DisplayList) -> Vec<EffectFallback> {
        text_run_font_handles(list)
            .into_iter()
            .filter(|handle| !self.scene_converter.can_draw_text(*handle))
            .map(|handle| {
                EffectFallback::new(
                    SceneEffect::Text,
                    format!(
                        "no font is registered for handle {handle} and none for the \
                         primary handle 0, so its runs are skipped and draw nothing; \
                         register one with VelloRenderer::register_font"
                    ),
                )
            })
            .collect()
    }

    /// Whether a GPU device is attached and rendering can submit work.
    #[must_use]
    pub const fn has_device(&self) -> bool {
        self.device.is_some()
    }

    /// The number of frames this renderer has successfully rendered.
    #[must_use]
    pub const fn frames_rendered(&self) -> u64 {
        self.frames_rendered
    }

    /// Renders `submission` into an offscreen [`VelloFrame`]: the main Vello scene,
    /// then our **own** wgpu gaussian pass for every `filter: blur()` layer.
    ///
    /// This is the shared core the on-screen present ([`Self::present_to_surface`]),
    /// the headless read-back ([`Self::render_to_image`]), and the port
    /// [`Renderer::render`] all funnel through, so every path produces the exact
    /// same pixels. The display list is converted into a main scene plus a list of
    /// [`BlurLayer`] sub-scenes ([`SceneConverter::convert_with_blurs`]); the main
    /// scene is rasterized into the frame, then each blur layer is rendered into a
    /// margin-padded offscreen, run through a separable gaussian, and composited
    /// back onto the frame. With no blur layer this is exactly the previous single
    /// [`render_to_texture`](vello::Renderer::render_to_texture) — byte-identical.
    ///
    /// # Errors
    ///
    /// [`RendererError::DeviceLost`] when no device is attached, or
    /// [`RendererError::Backend`] on a Vello rasterization failure.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    /// The offscreen target for `width` x `height`, allocated once and reused until
    /// the extent changes (a resize).
    fn cached_offscreen(&mut self, device: &wgpu::Device, width: u32, height: u32) -> VelloFrame {
        if let Some(frame) = self.frame_cache.as_ref() {
            if frame.width() == width && frame.height() == height {
                return frame.clone();
            }
        }
        let frame = VelloFrame::new_offscreen(device, width, height);
        self.frame_cache = Some(frame.clone());
        frame
    }

    fn render_to_frame(
        &mut self,
        submission: &FrameSubmission,
        width: u32,
        height: u32,
    ) -> Result<VelloFrame, RendererError> {
        let display_list = submission.frame().scene().display_list();

        // Spec §5.2: "Unsupported effects and fallbacks are explicit diagnostics."
        // Every entry point to this backend funnels through here, so this is where a
        // host is told which of the effects its scene asked for will not survive the
        // conversion below.
        let unsupported = unsupported_effects(&*self, display_list);
        let sink = Arc::clone(&self.diagnostics);
        self.fallbacks.report(&unsupported, sink.as_ref());
        // The other half of that sentence: an effect this backend *does* draw, but
        // whose documented restriction bites on THIS scene. It cannot be read off
        // `supports_effect` — `Text` is `Limited` whether or not the host registered
        // the font a given run needs — and this is the fallback that costs a host
        // the most to discover by looking: an unregistered run is dropped in
        // silence, so the text is simply absent with nothing to see.
        let degraded = self.text_fallbacks(display_list);
        self.fallbacks.report_fallbacks(&degraded, sink.as_ref());

        // `backdrop-filter` frosts the frame accumulated so far, so it cannot be
        // rendered in a single Vello pass: fall to the segmented compositing path.
        // A list with no backdrop-filter keeps the byte-identical single-render fast
        // path below (main scene + any `filter: blur` sub-scenes composited on top).
        if display_list
            .commands()
            .iter()
            .any(|command| matches!(command, PaintCommand::BackdropFilter { .. }))
        {
            return self.render_segmented(display_list, width, height);
        }

        // Convert first (device-free): the main scene plus any blur sub-scenes.
        let (main_scene, blurs) = self.scene_converter.convert_with_blurs(display_list);

        let Some(device) = self.device.as_ref() else {
            return Err(
                VelloRenderError::DeviceCreation("no gpu device".into()).into_renderer_error()
            );
        };
        // A cheap ref-counted clone, so the cache lookup below does not hold a
        // borrow of `self.device` while it takes `&mut self`.
        let cache_device = device.device().clone();
        let frame = self.cached_offscreen(&cache_device, width, height);
        let Some(device) = self.device.as_mut() else {
            return Err(
                VelloRenderError::DeviceCreation("no gpu device".into()).into_renderer_error()
            );
        };
        // `new_offscreen` clamps to >= 1 per axis; use the effective extent below.
        let (width, height) = frame.size();
        let params = vello::RenderParams {
            base_color: vello::peniko::Color::TRANSPARENT,
            width,
            height,
            antialiasing_method: vello::AaConfig::Area,
        };
        // wgpu `Device`/`Queue` are cheap ref-counted handles; clone them so the
        // renderer can be borrowed mutably alongside the immutable device/queue.
        let gpu_device = device.device().clone();
        let gpu_queue = device.queue().clone();
        device
            .renderer_mut()
            .render_to_texture(&gpu_device, &gpu_queue, &main_scene, frame.view(), &params)
            .map_err(|error| {
                VelloRenderError::RenderFailed(error.to_string().into()).into_renderer_error()
            })?;

        if !blurs.is_empty() {
            // Render each blur sub-scene into its own margin-padded offscreen: the
            // margin (~3σ) keeps the gaussian tail from being clipped at the edge.
            let mut prepared: Vec<(VelloFrame, (f32, f32), BlurLayer)> =
                Vec::with_capacity(blurs.len());
            for blur in blurs {
                let margin = (blur.radius * 3.0).ceil().max(1.0);
                let s_width = (blur.bounds.size.width + 2.0 * margin).ceil().max(1.0) as u32;
                let s_height = (blur.bounds.size.height + 2.0 * margin).ceil().max(1.0) as u32;
                let s_frame = VelloFrame::new_offscreen(&gpu_device, s_width, s_height);
                let (s_width, s_height) = s_frame.size();
                // Offset the sub-scene so the bounds' top-left maps to (margin, margin).
                let offset = Affine::translate((
                    f64::from(margin) - f64::from(blur.bounds.origin.x),
                    f64::from(margin) - f64::from(blur.bounds.origin.y),
                ));
                let mut placed = Scene::new();
                placed.append(&blur.scene, Some(offset));
                let sub_params = vello::RenderParams {
                    base_color: vello::peniko::Color::TRANSPARENT,
                    width: s_width,
                    height: s_height,
                    antialiasing_method: vello::AaConfig::Area,
                };
                device
                    .renderer_mut()
                    .render_to_texture(
                        &gpu_device,
                        &gpu_queue,
                        &placed,
                        s_frame.view(),
                        &sub_params,
                    )
                    .map_err(|error| {
                        VelloRenderError::RenderFailed(error.to_string().into())
                            .into_renderer_error()
                    })?;
                // Absolute pixel top-left the padded offscreen composites back at.
                let origin = (blur.bounds.origin.x - margin, blur.bounds.origin.y - margin);
                prepared.push((s_frame, origin, blur));
            }

            // The `device` borrow ends here; build/reuse the owned filter pipelines
            // and post-process + composite each filter sub-scene onto the frame in
            // one encoder. The Vello render-to-texture passes above already
            // submitted, so this trailing submit runs after them on the same queue
            // (correct ordering).
            let needs_color = prepared.iter().any(|(_, _, b)| b.color_matrix.is_some());
            let gaussian = self
                .blur
                .get_or_insert_with(|| GaussianBlur::new(&gpu_device));
            // Disjoint field borrow: the color-matrix pipeline is built only when a
            // color op is present (byte-identical no-color path otherwise).
            let color_matrix = needs_color.then(|| {
                self.color_matrix
                    .get_or_insert_with(|| ColorMatrix::new(&gpu_device))
            });
            let mut encoder = gpu_device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("obscur-vello-filter"),
            });
            // Keep any color-matrix scratch textures alive until the encoder submits.
            let mut scratch: Vec<wgpu::Texture> = Vec::new();
            for (s_frame, origin, blur) in &prepared {
                if let Some(tex) = apply_filter_layer(
                    &gpu_device,
                    &mut encoder,
                    gaussian,
                    color_matrix.as_deref(),
                    s_frame,
                    *origin,
                    blur,
                    frame.view(),
                    (width, height),
                ) {
                    scratch.push(tex);
                }
            }
            gpu_queue.submit(std::iter::once(encoder.finish()));
            drop(scratch);
        }

        Ok(frame)
    }

    /// Renders a display list containing `backdrop-filter` via **segmented
    /// compositing** (spec §5.2, CSS `backdrop-filter` — the acrylic effect).
    ///
    /// `backdrop-filter` frosts the frame accumulated *so far*, so a single Vello
    /// render will not do. The list is split at each
    /// [`BackdropFilter`](PaintCommand::BackdropFilter) into
    /// [`SceneSegment`]s ([`SceneConverter::convert_segments`]): each
    /// [`Content`](SceneSegment::Content) run is rendered to a transparent offscreen
    /// and composited onto an **accumulating frame** `T` (premultiplied-over, so
    /// segments layer in paint order — no Vello load/no-clear mode needed), and each
    /// [`Backdrop`](SceneSegment::Backdrop) boundary frosts `T` in place under the
    /// panel's rounded border box via the owned
    /// [`GaussianBlur::render_backdrop`]. A content run's `filter: blur` layers are
    /// composited on top just as the non-segmented path does. The same owned
    /// [`GaussianBlur`] pipelines are reused; this path is shared by every caller of
    /// [`Self::render_to_frame`] (present + read-back + the port `render`).
    ///
    /// # Errors
    ///
    /// [`RendererError::DeviceLost`] when no device is attached, or
    /// [`RendererError::Backend`] on a Vello rasterization failure.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    fn render_segmented(
        &mut self,
        display_list: &DisplayList,
        width: u32,
        height: u32,
    ) -> Result<VelloFrame, RendererError> {
        // Convert first (device-free): the ordered content / backdrop segments.
        let segments = self.scene_converter.convert_segments(display_list);

        let Some(device) = self.device.as_ref() else {
            return Err(
                VelloRenderError::DeviceCreation("no gpu device".into()).into_renderer_error()
            );
        };
        let gpu_device = device.device().clone();
        let gpu_queue = device.queue().clone();

        // The accumulating frame `T`: segments composite onto it in order, and each
        // backdrop boundary frosts it in place. Clear it to transparent first (it is
        // never a Vello render target, so nothing else initialises it).
        let frame = VelloFrame::new_offscreen(&gpu_device, width, height);
        let (width, height) = frame.size();
        {
            let mut encoder = gpu_device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("obscur-vello-backdrop-clear"),
            });
            drop(encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("obscur-vello-backdrop-clear-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: frame.view(),
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            }));
            gpu_queue.submit(std::iter::once(encoder.finish()));
        }

        // Build (once) / reuse the owned blur + composite pipelines.
        if self.blur.is_none() {
            self.blur = Some(GaussianBlur::new(&gpu_device));
        }

        for segment in segments {
            match segment {
                SceneSegment::Content { scene, blurs } => {
                    // Render this segment's content into a fresh transparent
                    // offscreen (Vello clears it to `base_color`).
                    let seg_frame = VelloFrame::new_offscreen(&gpu_device, width, height);
                    let params = vello::RenderParams {
                        base_color: vello::peniko::Color::TRANSPARENT,
                        width,
                        height,
                        antialiasing_method: vello::AaConfig::Area,
                    };
                    {
                        let device = self
                            .device
                            .as_mut()
                            .expect("device present (checked above)");
                        device
                            .renderer_mut()
                            .render_to_texture(
                                &gpu_device,
                                &gpu_queue,
                                &scene,
                                seg_frame.view(),
                                &params,
                            )
                            .map_err(|error| {
                                VelloRenderError::RenderFailed(error.to_string().into())
                                    .into_renderer_error()
                            })?;
                    }

                    // Render this segment's `filter: blur` sub-scenes into their own
                    // margin-padded offscreens (same prep as the fast path).
                    let mut prepared: Vec<(VelloFrame, (f32, f32), BlurLayer)> =
                        Vec::with_capacity(blurs.len());
                    for blur in blurs {
                        let margin = (blur.radius * 3.0).ceil().max(1.0);
                        let s_width =
                            (blur.bounds.size.width + 2.0 * margin).ceil().max(1.0) as u32;
                        let s_height =
                            (blur.bounds.size.height + 2.0 * margin).ceil().max(1.0) as u32;
                        let s_frame = VelloFrame::new_offscreen(&gpu_device, s_width, s_height);
                        let (s_width, s_height) = s_frame.size();
                        let offset = Affine::translate((
                            f64::from(margin) - f64::from(blur.bounds.origin.x),
                            f64::from(margin) - f64::from(blur.bounds.origin.y),
                        ));
                        let mut placed = Scene::new();
                        placed.append(&blur.scene, Some(offset));
                        let sub_params = vello::RenderParams {
                            base_color: vello::peniko::Color::TRANSPARENT,
                            width: s_width,
                            height: s_height,
                            antialiasing_method: vello::AaConfig::Area,
                        };
                        {
                            let device = self
                                .device
                                .as_mut()
                                .expect("device present (checked above)");
                            device
                                .renderer_mut()
                                .render_to_texture(
                                    &gpu_device,
                                    &gpu_queue,
                                    &placed,
                                    s_frame.view(),
                                    &sub_params,
                                )
                                .map_err(|error| {
                                    VelloRenderError::RenderFailed(error.to_string().into())
                                        .into_renderer_error()
                                })?;
                        }
                        let origin = (blur.bounds.origin.x - margin, blur.bounds.origin.y - margin);
                        prepared.push((s_frame, origin, blur));
                    }

                    // Composite the segment (whole-frame) onto `T`, then its filter
                    // layers on top, in one encoder submitted after the Vello renders.
                    let needs_color = prepared.iter().any(|(_, _, b)| b.color_matrix.is_some());
                    let gaussian = self.blur.as_ref().expect("blur pipelines built above");
                    // Build/reuse the color-matrix pipeline only when a color op is
                    // present (byte-identical no-color path otherwise).
                    let color_matrix = needs_color.then(|| {
                        self.color_matrix
                            .get_or_insert_with(|| ColorMatrix::new(&gpu_device))
                    });
                    let mut encoder =
                        gpu_device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("obscur-vello-segment-composite"),
                        });
                    gaussian.composite_over(
                        &gpu_device,
                        &mut encoder,
                        seg_frame.view(),
                        frame.view(),
                    );
                    let mut scratch: Vec<wgpu::Texture> = Vec::new();
                    for (s_frame, origin, blur) in &prepared {
                        if let Some(tex) = apply_filter_layer(
                            &gpu_device,
                            &mut encoder,
                            gaussian,
                            color_matrix.as_deref(),
                            s_frame,
                            *origin,
                            blur,
                            frame.view(),
                            (width, height),
                        ) {
                            scratch.push(tex);
                        }
                    }
                    gpu_queue.submit(std::iter::once(encoder.finish()));
                    drop(scratch);
                }
                SceneSegment::Backdrop {
                    bounds,
                    radii,
                    blur,
                } => {
                    let gaussian = self.blur.as_ref().expect("blur pipelines built above");
                    let mut encoder =
                        gpu_device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("obscur-vello-backdrop"),
                        });
                    gaussian.render_backdrop(
                        &gpu_device,
                        &mut encoder,
                        frame.texture(),
                        frame.view(),
                        (width, height),
                        [
                            bounds.origin.x,
                            bounds.origin.y,
                            bounds.origin.x + bounds.size.width,
                            bounds.origin.y + bounds.size.height,
                        ],
                        // The backdrop mask's WGSL SDF (`rounded_rect_coverage` in
                        // `gaussian_blur.wgsl`) is a per-corner *circular* rounded
                        // box: it selects one corner radius by quadrant and does
                        // `length(q) - r`, which has no elliptical form. So a panel
                        // with elliptical corners (`border-radius: 50%` on a
                        // non-square box) frosts to the closest circle per corner —
                        // the mean of its two axes, exact whenever the corner is
                        // already circular. Confined to the backdrop-filter effect;
                        // the panel's own clip and fill keep the true ellipse via
                        // `Path`. See `docs/conformance-audit.md` finding 4-A.
                        [
                            radii.top_left.circular_approximation(),
                            radii.top_right.circular_approximation(),
                            radii.bottom_right.circular_approximation(),
                            radii.bottom_left.circular_approximation(),
                        ],
                        blur,
                    );
                    gpu_queue.submit(std::iter::once(encoder.finish()));
                }
            }
        }

        Ok(frame)
    }

    /// Presents a submission **on-screen** by rendering into the adapter's
    /// offscreen [`VelloFrame`] and blitting it to a windowed [`VelloSurface`].
    ///
    /// This is the counterpart to the port [`Renderer::render`], which targets an
    /// offscreen texture only. Vello 0.9 has no render-to-surface helper, so the
    /// documented pattern is followed here: rasterize into an
    /// `Rgba8Unorm + STORAGE_BINDING` intermediate (`VelloFrame::new_offscreen`)
    /// with [`vello::Renderer::render_to_texture`], then copy that texture onto
    /// the acquired swapchain image with a [`wgpu::util::TextureBlitter`]. The
    /// blitter is cached on the renderer (keyed by surface format) so its render
    /// pipeline is only compiled once, not per frame.
    ///
    /// This is an inherent method, not part of the [`Renderer`] trait: only a host
    /// that owns a window and a [`VelloSurface`] can drive it, whereas the trait's
    /// offscreen path stays device- and window-agnostic.
    ///
    /// # Errors
    ///
    /// Returns [`RendererError::DeviceLost`] when no GPU device is attached,
    /// [`RendererError::Backend`] when the surface has a zero extent or Vello's
    /// rasterization fails, and surfaces any swapchain-acquisition failure
    /// (e.g. [`RendererError::Backend`] for a lost/outdated surface).
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn present_to_surface(
        &mut self,
        submission: &FrameSubmission,
        surface: &VelloSurface,
    ) -> Result<PresentResult, RendererError> {
        // Physical swapchain extent; wgpu rejects a zero-sized surface, so treat
        // it as a backend error rather than submitting empty work.
        let width = surface.config().width;
        let height = surface.config().height;
        if width == 0 || height == 0 {
            return Err(
                VelloRenderError::RenderFailed("empty surface extent".into()).into_renderer_error(),
            );
        }
        let format = surface.config().format;

        // Rasterize the display list (main scene + owned blur passes) into the
        // offscreen intermediate — the same shared path the headless read-back uses.
        let frame = self.render_to_frame(submission, width, height)?;

        // Re-borrow the (attached) device for the blit. Cheap ref-counted clones let
        // the renderer be borrowed mutably alongside the immutable device/queue.
        let Some(device) = self.device.as_mut() else {
            return Err(
                VelloRenderError::DeviceCreation("no gpu device".into()).into_renderer_error()
            );
        };
        let gpu_device = device.device().clone();
        let gpu_queue = device.queue().clone();

        // Acquire the swapchain image to present into.
        let surface_texture = surface
            .current_texture()
            .map_err(VelloRenderError::into_renderer_error)?;

        // Obtain a blitter matching the surface format, rebuilding only on a
        // format change (or first use) to avoid recompiling the pipeline. Drop a
        // stale (differently formatted) cache entry, then lazily build one.
        if self
            .blitter
            .as_ref()
            .is_some_and(|(cached, _)| *cached != format)
        {
            self.blitter = None;
        }
        let (_, blitter) = self
            .blitter
            .get_or_insert_with(|| (format, wgpu::util::TextureBlitter::new(&gpu_device, format)));

        // Copy offscreen -> swapchain and present.
        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = gpu_device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("obscur-vello-present"),
        });
        blitter.copy(&gpu_device, &mut encoder, frame.view(), &surface_view);
        gpu_queue.submit(std::iter::once(encoder.finish()));
        surface_texture.present();

        self.frames_rendered += 1;
        Ok(PresentResult::Presented {
            generation: submission.generation(),
        })
    }

    /// Renders `submission` fully **headless** into an offscreen texture and reads
    /// the pixels back to CPU memory as tightly-packed RGBA8 (spec §5.6 offscreen
    /// output).
    ///
    /// This is the read-back counterpart to the port [`Renderer::render`] (which
    /// leaves the result on the GPU) and to [`Self::present_to_surface`] (which
    /// blits it to a window): it rasterizes into the same
    /// [`VelloFrame::new_offscreen`] intermediate with
    /// [`vello::Renderer::render_to_texture`], then copies that texture into a
    /// mappable [`wgpu::Buffer`] — honoring wgpu's
    /// [`COPY_BYTES_PER_ROW_ALIGNMENT`](wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) 256-byte
    /// row stride — waits for the copy with a blocking [`Device::poll`], un-pads the
    /// rows, and returns `(width, height, rgba8)`. It **does not** disturb the
    /// on-screen path or the trait `render`; a host uses it to produce PNGs from a
    /// display list without a window.
    ///
    /// The returned bytes are **premultiplied** RGBA8 (Vello's texture output), so
    /// they can be handed straight to e.g. `tiny_skia::Pixmap::from_vec`.
    ///
    /// # Errors
    ///
    /// Returns [`RendererError::DeviceLost`] when no GPU device is attached, and
    /// [`RendererError::Backend`] on a zero extent, a Vello rasterization failure,
    /// or a buffer-map / device-poll failure.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn render_to_image(
        &mut self,
        submission: &FrameSubmission,
        width: u32,
        height: u32,
    ) -> Result<(u32, u32, Vec<u8>), RendererError> {
        if width == 0 || height == 0 {
            return Err(VelloRenderError::RenderFailed("empty target".into()).into_renderer_error());
        }

        // Rasterize the display list (main scene + owned blur passes) into the
        // offscreen frame — the same shared path the on-screen present uses.
        let frame = self.render_to_frame(submission, width, height)?;
        // `new_offscreen` clamps to >= 1 per axis; read the effective extent back so
        // the buffer stride and un-padding below match the texture exactly.
        let (width, height) = frame.size();

        // Re-borrow the (attached) device for the read-back copy.
        let Some(device) = self.device.as_mut() else {
            return Err(
                VelloRenderError::DeviceCreation("no gpu device".into()).into_renderer_error()
            );
        };
        let gpu_device = device.device().clone();
        let gpu_queue = device.queue().clone();

        // Copy the offscreen texture into a mappable buffer. `copy_texture_to_buffer`
        // requires each row to begin on a 256-byte boundary, so the destination is
        // padded to `padded_bytes_per_row` and the padding is dropped after mapping.
        let unpadded_bytes_per_row = width * 4;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(align) * align;
        let buffer_size = u64::from(padded_bytes_per_row) * u64::from(height);
        let readback = gpu_device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("obscur-vello-readback"),
            size: buffer_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = gpu_device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("obscur-vello-readback"),
        });
        encoder.copy_texture_to_buffer(
            frame.texture().as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        gpu_queue.submit(std::iter::once(encoder.finish()));

        // Map the buffer and block until the GPU copy has completed (the map
        // callback fires during the `Wait` poll).
        let (sender, receiver) = std::sync::mpsc::channel();
        readback.map_async(wgpu::MapMode::Read, .., move |result| {
            let _ = sender.send(result);
        });
        gpu_device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|error| {
                VelloRenderError::RenderFailed(error.to_string().into()).into_renderer_error()
            })?;
        match receiver.recv() {
            Ok(Ok(())) => {}
            Ok(Err(error)) => {
                return Err(
                    VelloRenderError::RenderFailed(error.to_string().into()).into_renderer_error()
                );
            }
            Err(error) => {
                return Err(
                    VelloRenderError::RenderFailed(error.to_string().into()).into_renderer_error()
                );
            }
        }

        // Copy out, dropping the per-row padding so the result is tight RGBA8.
        let row = unpadded_bytes_per_row as usize;
        let stride = padded_bytes_per_row as usize;
        let mut pixels = Vec::with_capacity(row * height as usize);
        {
            let view = readback.get_mapped_range(..);
            for chunk in view.chunks_exact(stride) {
                pixels.extend_from_slice(&chunk[..row]);
            }
        }
        readback.unmap();

        self.frames_rendered += 1;
        Ok((width, height, pixels))
    }
}

/// Post-processes and composites one prepared `filter` sub-scene onto the target
/// frame: the owned color-matrix pass (if the layer carries one) then the gaussian
/// blur (if `radius > 0`), sharing the offscreen/composite plumbing.
///
/// `s_frame` is the layer's subtree rendered into its margin-padded offscreen (in
/// absolute coordinates offset so its top-left maps to `origin`). The order is:
/// color-matrix (into a scratch texture) → blur → composite, matching a
/// `filter: <color-op> blur()` chain (a color op applied before the spatial
/// gaussian). A color-only layer (`radius == 0`) composites the color result
/// directly; a blur-only layer (`color_matrix == None`) is byte-identical to the
/// shipped blur path.
///
/// Returns the color-matrix scratch texture (when one was allocated) so the caller
/// keeps it alive until the shared encoder is submitted.
#[allow(clippy::too_many_arguments)]
fn apply_filter_layer(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    gaussian: &GaussianBlur,
    color_matrix: Option<&ColorMatrix>,
    s_frame: &VelloFrame,
    origin: (f32, f32),
    layer: &BlurLayer,
    target_view: &wgpu::TextureView,
    target_size: (u32, u32),
) -> Option<wgpu::Texture> {
    let (sw, sh) = s_frame.size();

    // Optional color-matrix pre-pass: transform the sub-scene into a scratch texture.
    let scratch = layer.color_matrix.and_then(|matrix| {
        let color_matrix = color_matrix?;
        let scratch = color_matrix.scratch_texture(device, sw, sh);
        let src_view = s_frame
            .texture()
            .create_view(&wgpu::TextureViewDescriptor::default());
        let scratch_view = scratch.create_view(&wgpu::TextureViewDescriptor::default());
        color_matrix.apply(device, encoder, &src_view, &scratch_view, &matrix);
        Some(scratch)
    });

    // The source for the following blur/composite is the color-matrix scratch (if a
    // matrix was applied) or the raw sub-scene frame otherwise. A color-transformed
    // scratch may be translucent (e.g. `opacity()`), so it composites premultiplied;
    // the raw blur path stays `false` (byte-identical to the shipped opaque path).
    let (source_view, premultiply) = match &scratch {
        Some(tex) => (
            tex.create_view(&wgpu::TextureViewDescriptor::default()),
            true,
        ),
        None => (
            s_frame
                .texture()
                .create_view(&wgpu::TextureViewDescriptor::default()),
            false,
        ),
    };

    if layer.radius > 0.0 {
        gaussian.render_view(
            device,
            encoder,
            &source_view,
            (sw, sh),
            origin,
            layer.radius,
            target_view,
            target_size,
        );
    } else {
        gaussian.composite_texture(
            device,
            encoder,
            &source_view,
            (sw, sh),
            origin,
            target_view,
            target_size,
            premultiply,
        );
    }

    scratch
}

impl Default for VelloRenderer {
    fn default() -> Self {
        Self::new(RendererId::from_raw(0))
    }
}

impl Renderer for VelloRenderer {
    fn id(&self) -> RendererId {
        self.id
    }

    fn capabilities(&self) -> RendererCapabilities {
        self.capabilities
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn render(
        &mut self,
        submission: &FrameSubmission,
        target: &RenderTarget,
    ) -> Result<PresentResult, RendererError> {
        let size = target.size();
        // Physical-pixel target dimensions: rounding a finite, non-negative
        // extent to an integer is intentional (clamped to >= 1 in VelloFrame).
        let width = size.width.round() as u32;
        let height = size.height.round() as u32;
        if width == 0 || height == 0 {
            return Err(VelloRenderError::RenderFailed("empty target".into()).into_renderer_error());
        }

        // Rasterize into an offscreen frame via the shared path (main scene + owned
        // blur passes); the port `render` leaves the result on the GPU. A missing
        // device yields an honest error rather than a fabricated one.
        let _frame = self.render_to_frame(submission, width, height)?;

        self.frames_rendered += 1;
        Ok(PresentResult::Presented {
            generation: submission.generation(),
        })
    }

    /// The honest [`SupportLevel`] at which this GPU backend reproduces each
    /// [`SceneEffect`] (spec §5.2).
    ///
    /// - [`Gradient`](SceneEffect::Gradient), [`Clip`](SceneEffect::Clip), and
    ///   [`Image`](SceneEffect::Image) are [`Native`](SupportLevel::Native): a
    ///   gradient brush becomes a peniko gradient over the same geometry and
    ///   stops, a clip becomes a bounded `push_layer`, and a `DrawImage` a real
    ///   sampled `draw_image`.
    /// - [`BlendMode`](SceneEffect::BlendMode) is [`Limited`](SupportLevel::Limited),
    ///   because the variant names two inputs and this backend honours one. A
    ///   layer's `blend_mode()` **field** becomes a peniko `Mix` natively. A
    ///   [`LayerEffect::Blend`](obscur_scene::layer::LayerEffect::Blend) in the
    ///   layer's *effects list* is **dropped**: `push_layer` reads the field, and
    ///   the effects list is scanned only for `Blur` and `ColorMatrix`. Reporting
    ///   `Native` claimed both. Nothing produces a `LayerEffect::Blend` today — it
    ///   duplicates the field — so this is latent, not live.
    /// - [`Shadow`](SceneEffect::Shadow) and
    ///   [`BackdropFilter`](SceneEffect::BackdropFilter) are
    ///   [`Emulated`](SupportLevel::Emulated): both are real gaussians — a shadow
    ///   through `draw_blurred_rounded_rect`, a backdrop through the owned segmented
    ///   wgpu compositing path — but assembled from primitives rather than a single
    ///   native op.
    /// - [`Blur`](SceneEffect::Blur) and [`ColorMatrix`](SceneEffect::ColorMatrix)
    ///   are [`Limited`](SupportLevel::Limited): a `filter: blur()` / color-op layer
    ///   runs through our owned wgpu passes (a real gaussian, a real per-pixel
    ///   affine), but only when the layer is peeled off the main scene — which needs
    ///   the layer to carry a bounded clip, and does not recurse. **A filter layer
    ///   nested inside another filter layer is drawn unfiltered**, as is one with no
    ///   clip bounds. A `DrawShadow`'s own blur radius has no such restriction.
    /// - [`Text`](SceneEffect::Text) is `Limited`: glyphs draw against the font
    ///   registered for the run's handle; a run whose font was never registered is
    ///   **skipped entirely**.
    /// - [`Mask`](SceneEffect::Mask) is [`Unavailable`](SupportLevel::Unavailable):
    ///   a mask clip converts to the unbounded outline, so its coverage is never
    ///   applied — the same as on the CPU backend (spec §5.2: clipping must match
    ///   across backends).
    fn supports_effect(&self, effect: SceneEffect) -> SupportLevel {
        match effect {
            SceneEffect::Gradient | SceneEffect::Clip | SceneEffect::Image => SupportLevel::Native,
            SceneEffect::Shadow | SceneEffect::BackdropFilter => SupportLevel::Emulated,
            SceneEffect::Blur
            | SceneEffect::ColorMatrix
            | SceneEffect::Text
            | SceneEffect::BlendMode => SupportLevel::Limited,
            SceneEffect::Mask => SupportLevel::Unavailable,
        }
    }
}

impl std::fmt::Debug for VelloRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VelloRenderer")
            .field("id", &self.id)
            .field("capabilities", &self.capabilities)
            .field("has_device", &self.has_device())
            .field("frames_rendered", &self.frames_rendered)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::Size;
    use obscur_render::frame::RenderFrame;
    use obscur_render::target::{BitmapTarget, RenderTarget};
    use obscur_scene::command::DisplayList;
    use obscur_scene::scene::{SceneGeneration, SceneSnapshot};

    fn submission(generation: u64) -> FrameSubmission {
        FrameSubmission::new(RenderFrame::new(
            SceneSnapshot::new(DisplayList::new(), SceneGeneration::from_raw(generation)),
            Size::new(64.0, 48.0),
        ))
    }

    fn a_target() -> RenderTarget {
        RenderTarget::bitmap(BitmapTarget::new(Size::new(64.0, 48.0)))
    }

    /// A [`DiagnosticSink`] that keeps every message it is handed.
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

    #[test]
    fn rendering_an_unsupported_effect_tells_the_host_which_one() {
        use obscur_scene::clip::{Clip, Mask};
        // A mask clip's coverage is applied by neither backend. Spec §5.2 wants the
        // host told, not left to compare screenshots. The report runs before the GPU
        // submission, so it is observable without a device.
        let sink = Arc::new(RecordingSink::default());
        let mut renderer = VelloRenderer::new(RendererId::from_raw(11));
        renderer.set_diagnostic_sink(sink.clone());
        let list = DisplayList::from_commands(vec![
            PaintCommand::PushClip(Clip::mask(Mask::alpha())),
            PaintCommand::PopClip,
        ]);
        let submission = FrameSubmission::new(RenderFrame::new(
            SceneSnapshot::new(list, SceneGeneration::from_raw(1)),
            Size::new(64.0, 48.0),
        ));
        let _ = renderer.render(&submission, &a_target());
        assert_eq!(
            sink.messages(),
            vec!["unsupported effect: mask".to_string()]
        );
    }

    #[test]
    fn a_scene_within_the_backends_means_raises_nothing() {
        let sink = Arc::new(RecordingSink::default());
        let mut renderer = VelloRenderer::new(RendererId::from_raw(12));
        renderer.set_diagnostic_sink(sink.clone());
        let _ = renderer.render(&submission(1), &a_target());
        assert!(sink.messages().is_empty());
    }

    /// A one-glyph run tagged with font handle `handle`.
    fn text_submission(handle: u32) -> FrameSubmission {
        use obscur_scene::text::{GlyphRun, PositionedGlyph};
        let run = GlyphRun::new(vec![PositionedGlyph::new(1, 0, 0.0, 12.0, 8.0)], 12.0, "a")
            .with_font(handle);
        let list = DisplayList::from_commands(vec![PaintCommand::DrawText {
            origin: obscur_geometry::Point::new(1.0, 1.0),
            run,
            brush: obscur_scene::paint::Brush::solid(obscur_scene::color::Color::BLACK),
        }]);
        FrameSubmission::new(RenderFrame::new(
            SceneSnapshot::new(list, SceneGeneration::from_raw(1)),
            Size::new(64.0, 48.0),
        ))
    }

    #[test]
    fn a_text_run_without_its_font_tells_the_host_it_was_skipped() {
        // Spec §5.2: unsupported effects AND FALLBACKS are explicit diagnostics.
        // `Text` is `Limited`, so it raises no `unsupported effect` — and this
        // backend's restriction is that an uncovered run is dropped in silence.
        // Nothing is drawn, so nothing on screen tells the host why. The report runs
        // before the GPU submission, so it is observable without a device.
        let sink = Arc::new(RecordingSink::default());
        let mut renderer = VelloRenderer::new(RendererId::from_raw(13));
        renderer.set_diagnostic_sink(sink.clone());

        let _ = renderer.render(&text_submission(3), &a_target());

        let messages = sink.messages();
        assert_eq!(messages.len(), 1, "one fallback, once: {messages:?}");
        assert!(
            messages[0].starts_with("degraded effect: text (no font is registered for handle 3"),
            "the diagnostic must name the effect and the handle: {messages:?}"
        );

        // A property of this renderer's registry, not of the frame: not re-raised.
        let _ = renderer.render(&text_submission(3), &a_target());
        assert_eq!(sink.messages().len(), 1);
    }

    #[test]
    fn an_empty_scene_raises_no_text_fallback() {
        // The anti-noise direction at this level: a scene with no text must not
        // acquire a text diagnostic just because `Text` is declared `Limited`. The
        // positive case (a run its registry DOES cover) is pinned on
        // `SceneConverter::can_draw_text`, the predicate this scan calls, because
        // driving a covered run through `render` here would need a parseable font —
        // `vello::DrawGlyphs::draw` unwraps `FontRef::from_index` — and this crate's
        // tests have no font file to hand.
        let sink = Arc::new(RecordingSink::default());
        let mut renderer = VelloRenderer::new(RendererId::from_raw(14));
        renderer.set_diagnostic_sink(sink.clone());
        let _ = renderer.render(&submission(1), &a_target());
        assert!(sink.messages().is_empty());
    }

    #[test]
    fn new_renderer_has_no_device() {
        let renderer = VelloRenderer::new(RendererId::from_raw(1));
        assert!(!renderer.has_device());
        assert_eq!(renderer.frames_rendered(), 0);
    }

    #[test]
    fn id_and_capabilities_reflect_construction() {
        let renderer = VelloRenderer::new(RendererId::from_raw(7));
        assert_eq!(renderer.id(), RendererId::from_raw(7));
        let caps = renderer.capabilities();
        assert!(caps.has_offscreen());
        assert!(caps.has_custom_shaders());
        assert!(caps.has_msaa());
        assert!(!caps.has_hdr());
    }

    #[test]
    fn default_uses_zero_id_and_no_device() {
        let renderer = VelloRenderer::default();
        assert_eq!(renderer.id(), RendererId::from_raw(0));
        assert!(!renderer.has_device());
    }

    #[test]
    fn render_without_device_reports_device_loss() {
        let mut renderer = VelloRenderer::new(RendererId::from_raw(2));
        let result = renderer.render(&submission(5), &a_target());
        assert!(matches!(result, Err(RendererError::DeviceLost(_))));
        // A failed render never advances the frame counter.
        assert_eq!(renderer.frames_rendered(), 0);
    }

    #[test]
    fn render_into_empty_target_is_a_backend_error() {
        let mut renderer = VelloRenderer::new(RendererId::from_raw(3));
        let empty = RenderTarget::bitmap(BitmapTarget::new(Size::new(0.0, 0.0)));
        assert!(matches!(
            renderer.render(&submission(1), &empty),
            Err(RendererError::Backend(_))
        ));
    }

    #[test]
    fn render_to_image_without_device_reports_device_loss() {
        // The headless read-back path is device-bound like the port `render`: with
        // no host device it returns an honest DeviceLost rather than fabricating one.
        let mut renderer = VelloRenderer::new(RendererId::from_raw(6));
        let result = renderer.render_to_image(&submission(1), 64, 48);
        assert!(matches!(result, Err(RendererError::DeviceLost(_))));
        assert_eq!(renderer.frames_rendered(), 0);
    }

    #[test]
    fn render_to_image_with_zero_extent_is_a_backend_error() {
        let mut renderer = VelloRenderer::new(RendererId::from_raw(8));
        assert!(matches!(
            renderer.render_to_image(&submission(1), 0, 0),
            Err(RendererError::Backend(_))
        ));
    }

    #[test]
    fn supports_effect_is_honest_per_effect() {
        let renderer = VelloRenderer::new(RendererId::from_raw(4));
        // Real Vello primitives: a bounded push_layer, a peniko gradient, a sampled
        // draw_image.
        for effect in [SceneEffect::Clip, SceneEffect::Gradient, SceneEffect::Image] {
            assert_eq!(renderer.supports_effect(effect), SupportLevel::Native);
        }
        // Not Native: the variant names two inputs and `push_layer` honours one.
        // The layer's `blend_mode()` field becomes a peniko `Mix`; a
        // `LayerEffect::Blend` in the effects list is dropped, because that list is
        // scanned only for `Blur` and `ColorMatrix`.
        assert_eq!(
            renderer.supports_effect(SceneEffect::BlendMode),
            SupportLevel::Limited
        );
        // Real gaussians, assembled from primitives rather than a native op: a
        // `box-shadow` / `filter: drop-shadow(...)` through
        // `draw_blurred_rounded_rect`, a `backdrop-filter: blur()` through the owned
        // segmented wgpu compositing path.
        assert_eq!(
            renderer.supports_effect(SceneEffect::Shadow),
            SupportLevel::Emulated
        );
        assert_eq!(
            renderer.supports_effect(SceneEffect::BackdropFilter),
            SupportLevel::Emulated
        );
        // The owned wgpu blur / color-matrix passes are real, but only reach a
        // filter layer that the converter peels off the main scene: a nested filter
        // layer is drawn unfiltered, which is a restriction, not a rasterization
        // difference.
        assert_eq!(
            renderer.supports_effect(SceneEffect::Blur),
            SupportLevel::Limited
        );
        assert_eq!(
            renderer.supports_effect(SceneEffect::ColorMatrix),
            SupportLevel::Limited
        );
        // A run whose font was never registered is skipped entirely.
        assert_eq!(
            renderer.supports_effect(SceneEffect::Text),
            SupportLevel::Limited
        );
        // A mask clip's coverage is never applied on either backend.
        assert_eq!(
            renderer.supports_effect(SceneEffect::Mask),
            SupportLevel::Unavailable
        );
    }

    #[test]
    fn renderer_is_object_safe() {
        let _: Box<dyn Renderer> = Box::new(VelloRenderer::new(RendererId::from_raw(9)));
    }
}
