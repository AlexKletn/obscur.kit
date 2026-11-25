//! The Vello/wgpu implementation of the [`Renderer`] port (spec §5.2, §2.7).

mod frame_output;
mod shared_face;

use frame_output::FrameOutput;
use shared_face::SharedFace;

use std::sync::Arc;

use obscur_capability::SupportLevel;
use obscur_diagnostics::DiagnosticSink;
use obscur_render::frame::{FrameSubmission, PresentResult};
use obscur_render::renderer::{
    EffectFallback, EffectFallbackReporter, Renderer, RendererCapabilities, RendererError,
    RendererId, SceneEffect, text_run_font_handles, unsupported_effects,
};
use obscur_render::target::RenderTarget;

use obscur_scene::command::{DisplayList, PaintCommand};

use vello::peniko::{Blob, FontData};

use crate::conversion::{SceneConverter, SceneSegment};
use crate::effects::{ColorMatrix, GaussianBlur};
use crate::error::VelloRenderError;
use crate::renderer::damage_stats::DamageStats;
use crate::renderer::premultiply_blit::PremultiplyBlitter;
use crate::renderer::segmented_compositor::SegmentedCompositor;
use crate::renderer::{AcquiredImage, VelloDevice, VelloFrame, VelloSurface};

#[doc = include_str!("../../docs/api/renderer/vello_renderer.doc/VelloRenderer.md")]
pub struct VelloRenderer {
    id: RendererId,
    capabilities: RendererCapabilities,
    device: Option<VelloDevice>,
    scene_converter: SceneConverter,
    #[doc = include_str!("../../docs/api/renderer/vello_renderer.doc/scene.md")]
    scene: vello::Scene,
    frames_rendered: u64,
    damage_stats: DamageStats,
    /// Caches the surface blitters for each alpha conversion. Each entry is keyed by
    /// the target format, so consecutive Lyon and fallback Vello frames can reuse
    /// their respective pipelines.
    blitters: [Option<(wgpu::TextureFormat, PremultiplyBlitter)>; 2],
    #[doc = include_str!("../../docs/api/renderer/vello_renderer.doc/frame_cache.md")]
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
    #[doc = include_str!("../../docs/api/renderer/vello_renderer.doc/diagnostics.md")]
    diagnostics: Option<Arc<dyn DiagnosticSink>>,
    /// Which unsupported effects have already been reported, so a standing mask
    /// clip does not re-raise its diagnostic every frame.
    fallbacks: EffectFallbackReporter,
}

impl VelloRenderer {
    #[doc = include_str!("../../docs/api/renderer/vello_renderer.doc/new.md")]
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
            scene: vello::Scene::new(),
            frames_rendered: 0,
            damage_stats: DamageStats::default(),
            blitters: [None, None],
            frame_cache: None,
            blur: None,
            color_matrix: None,
            diagnostics: None,
            fallbacks: EffectFallbackReporter::new(),
        }
    }

    #[doc = include_str!("../../docs/api/renderer/vello_renderer.doc/set_diagnostic_sink.md")]
    pub fn set_diagnostic_sink(&mut self, sink: Arc<dyn DiagnosticSink>) {
        self.diagnostics = Some(sink);
    }

    /// Attaches a host-provided [`VelloDevice`], returning the updated renderer.
    #[must_use]
    pub fn with_device(mut self, device: VelloDevice) -> Self {
        self.set_device(device);
        self
    }

    #[cfg(feature = "hybrid")]
    #[doc = include_str!("../../docs/api/renderer/vello_renderer.doc/with_hybrid_device.md")]
    #[must_use]
    pub fn with_hybrid_device(mut self, device: wgpu::Device, queue: wgpu::Queue) -> Self {
        self.set_device(VelloDevice::hybrid(device, queue));
        self
    }

    #[cfg(feature = "lyon")]
    /// Attaches Lyon to the supplied wgpu device. Frames that require unsupported commands use the classic Vello renderer on the same device.
    #[must_use]
    pub fn with_lyon_device(mut self, device: wgpu::Device, queue: wgpu::Queue) -> Self {
        self.set_device(VelloDevice::lyon(device, queue));
        self
    }

    fn is_hybrid(&self) -> bool {
        #[cfg(feature = "hybrid")]
        if self
            .device
            .as_ref()
            .is_some_and(|device| device.hybrid.is_some())
        {
            return true;
        }
        false
    }

    /// The GPU bindings and backend currently installed in the renderer.
    #[must_use]
    pub const fn gpu_device(&self) -> Option<&VelloDevice> {
        self.device.as_ref()
    }

    /// Attaches (or replaces) the host-provided [`VelloDevice`].
    pub fn set_device(&mut self, device: VelloDevice) {
        self.damage_stats = DamageStats::default();
        self.device = Some(device);
        let classic = !self.is_hybrid();
        self.capabilities = self.capabilities.msaa(classic).custom_shaders(classic);
        self.frame_cache = None;
        self.blitters = [None, None];
        self.blur = None;
        self.color_matrix = None;
        self.fallbacks = EffectFallbackReporter::new();
    }

    #[doc = include_str!("../../docs/api/renderer/vello_renderer.doc/register_font.md")]
    pub fn register_font(&mut self, id: u32, font_bytes: impl Into<Arc<[u8]>>) {
        let font = FontData::new(Blob::new(Arc::new(SharedFace(font_bytes.into()))), 0);
        self.scene_converter.register_font(id, font);
        #[cfg(feature = "lyon")]
        self.damage_stats.invalidate_scene();
        #[cfg(feature = "lyon")]
        if let Some(lyon) = self.device.as_mut().and_then(|device| device.lyon.as_mut()) {
            lyon.invalidate_fonts();
        }
    }

    #[doc = include_str!("../../docs/api/renderer/vello_renderer.doc/set_font.md")]
    pub fn set_font(&mut self, font_bytes: impl Into<Arc<[u8]>>) {
        self.register_font(0, font_bytes);
    }

    #[doc = include_str!("../../docs/api/renderer/vello_renderer.doc/text_fallbacks.md")]
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

    fn report_classic_filter_fallbacks(list: &DisplayList, fallbacks: &mut Vec<EffectFallback>) {
        let mut depth = 0usize;
        let mut backdrop_layers = Vec::new();
        let mut backdrop_omitted = false;
        let mut blur_omitted = false;
        let mut matrix_omitted = false;
        for command in list.commands() {
            match command {
                PaintCommand::PushClip(_) => depth += 1,
                PaintCommand::PushLayer(layer) => {
                    if depth != 0 || !SceneConverter::can_extract_filter(layer) {
                        blur_omitted |= crate::conversion::layer_blur_radius(layer).is_some();
                        matrix_omitted |= layer
                            .effects()
                            .iter()
                            .any(|effect| effect.as_color_matrix().is_some());
                    }
                    depth += 1;
                    backdrop_layers.push(SceneConverter::can_split_backdrop_layer(layer));
                }
                PaintCommand::PopLayer => {
                    backdrop_layers.pop();
                    depth = depth.saturating_sub(1);
                }
                PaintCommand::PopClip => {
                    depth = depth.saturating_sub(1);
                }
                PaintCommand::BackdropFilter { .. } => {
                    backdrop_omitted |= backdrop_layers.contains(&false);
                }
                _ => {}
            }
        }
        if backdrop_omitted {
            fallbacks.push(EffectFallback::new(
                SceneEffect::BackdropFilter,
                "classic Vello omits backdrop filters inside compositing layers",
            ));
        }
        for (omitted, effect) in [
            (blur_omitted, SceneEffect::Blur),
            (matrix_omitted, SceneEffect::ColorMatrix),
        ] {
            if omitted {
                fallbacks.push(EffectFallback::new(
                    effect,
                    "classic Vello omits filter layers that require clipping or compositing the filtered output inside the scene",
                ));
            }
        }
    }

    /// Reports restrictions on shadow and blur effects already classified as limited by the experimental renderer.
    fn report_hybrid_fallbacks(&self, list: &DisplayList, fallbacks: &mut Vec<EffectFallback>) {
        if list
            .commands()
            .iter()
            .any(|command| matches!(command, PaintCommand::DrawShadow { inset: true, .. }))
        {
            fallbacks.push(EffectFallback::new(
                SceneEffect::Shadow,
                "inset shadows are omitted by Vello Hybrid",
            ));
        }

        if list.commands().iter().any(|command| {
            matches!(command, PaintCommand::PushLayer(layer) if crate::conversion::layer_blur_radius(layer).is_some())
        }) {
            fallbacks.push(EffectFallback::new(SceneEffect::Blur, "layer blur is omitted by Vello Hybrid"));
        }
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

    /// Renders one submission and returns the frame with its output format.
    /// Classic Vello's direct texture output is straight
    /// RGBA. The segmented compositor, including filter layers and backdrop effects,
    /// returns premultiplied RGBA. Lyon and hybrid also return premultiplied RGBA.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn render_to_frame(
        &mut self,
        submission: &FrameSubmission,
        width: u32,
        height: u32,
        #[cfg(feature = "lyon")] target: Option<&VelloFrame>,
    ) -> Result<(VelloFrame, FrameOutput), RendererError> {
        let display_list = submission.frame().scene().display_list();

        let hybrid = self.is_hybrid();
        #[cfg(feature = "hybrid")]
        let classic_mask_fallback = hybrid
            && display_list.commands().iter().any(|command| {
                use obscur_scene::clip::Clip;
                match command {
                    PaintCommand::PushClip(clip) => matches!(clip, Clip::ImageMask { .. }),
                    PaintCommand::PushLayer(layer) => {
                        matches!(layer.clip(), Some(Clip::ImageMask { .. }))
                    }
                    _ => false,
                }
            });
        #[cfg(not(feature = "hybrid"))]
        let classic_mask_fallback = false;

        // Spec §5.2: "Unsupported effects and fallbacks are explicit diagnostics."
        // Every entry point to this backend funnels through here, so this is where a
        // host is told which of the effects its scene asked for will not survive the
        // conversion below.
        //
        // Both scans are two more full walks of the display list, per frame, for the
        // life of the process — and until a host calls `set_diagnostic_sink` there is
        // nowhere for their product to go. Nothing else observes them:
        // `EffectFallbackReporter`'s state is read only by `CpuRenderer`'s `Debug`
        // (this crate exposes no reader at all). So when no one is listening they are
        // skipped outright rather than computed and discarded. With a sink installed
        // this path is byte-identical to what it was; without one the frame is the
        // same frame, two walks cheaper.
        if let Some(sink) = self.diagnostics.clone() {
            let unsupported = if classic_mask_fallback {
                obscur_render::renderer::scene_effects(display_list)
                    .into_iter()
                    .filter(|&effect| classic_effect_support(effect) == SupportLevel::Unavailable)
                    .collect()
            } else {
                unsupported_effects(&*self, display_list)
            };
            self.fallbacks.report(&unsupported, sink.as_ref());
            // The other half of that sentence: an effect this backend *does* draw,
            // but whose documented restriction bites on THIS scene. It cannot be read
            // off `supports_effect` — `Text` is `Limited` whether or not the host
            // registered the font a given run needs — and this is the fallback that
            // costs a host the most to discover by looking: an unregistered run is
            // dropped in silence, so the text is simply absent with nothing to see.
            let mut degraded = self.text_fallbacks(display_list);
            if classic_mask_fallback {
                degraded.push(EffectFallback::new(
                    SceneEffect::ImageMask,
                    "Vello Hybrid renders image-mask frames with classic Vello",
                ));
            }
            if hybrid && !classic_mask_fallback {
                self.report_hybrid_fallbacks(display_list, &mut degraded);
            }
            self.fallbacks.report_fallbacks(&degraded, sink.as_ref());
        }

        #[cfg(feature = "hybrid")]
        if hybrid && !classic_mask_fallback {
            let gpu = self
                .device
                .as_ref()
                .expect("attached hybrid device")
                .device()
                .clone();
            let frame = self.cached_offscreen(&gpu, width, height);
            let device = self.device.as_mut().expect("attached hybrid device");
            let queue = device.queue().clone();
            device.hybrid.as_mut().expect("hybrid renderer").render(
                &gpu,
                &queue,
                &frame,
                display_list,
                &self.scene_converter,
            )?;
            return Ok((frame, FrameOutput::Premultiplied));
        }

        #[cfg(feature = "hybrid")]
        if classic_mask_fallback {
            self.device
                .as_mut()
                .expect("attached hybrid device")
                .ensure_classic()
                .map_err(VelloRenderError::into_renderer_error)?;
        }

        #[cfg(feature = "lyon")]
        if let Some(result) = self.render_lyon(submission, width, height, target)? {
            return Ok((result, FrameOutput::Lyon));
        }

        if let Some(sink) = self.diagnostics.clone() {
            let mut degraded = Vec::new();
            Self::report_classic_filter_fallbacks(display_list, &mut degraded);
            self.fallbacks.report_fallbacks(&degraded, sink.as_ref());
        }

        // `backdrop-filter` frosts the frame accumulated so far, so it cannot be
        // rendered in a single Vello pass: fall to the segmented compositing path.
        // Lists with only ordinary draws keep the single-render fast path below.
        if display_list
            .commands()
            .iter()
            .any(|command| matches!(command, PaintCommand::BackdropFilter { .. }))
        {
            // Convert first (device-free): the ordered content / backdrop segments.
            let segments = self.scene_converter.convert_segments(display_list);
            let Some(device) = self.device.as_mut() else {
                return Err(
                    VelloRenderError::DeviceCreation("no gpu device".into()).into_renderer_error()
                );
            };
            return SegmentedCompositor::new(device, &mut self.blur, &mut self.color_matrix)
                .composite(segments, width, height)
                .map(|frame| (frame, FrameOutput::Premultiplied));
        }

        // Reuse the renderer's scene buffers when no filter splits the draw order.
        let mut main_scene = std::mem::take(&mut self.scene);
        let mut segments = self
            .scene_converter
            .convert_ordered_into(&mut main_scene, display_list);

        if !segments.is_empty() {
            let Some(device) = self.device.as_mut() else {
                self.scene = main_scene;
                return Err(
                    VelloRenderError::DeviceCreation("no gpu device".into()).into_renderer_error()
                );
            };
            segments.push(SceneSegment::Content {
                scene: main_scene,
                blurs: Vec::new(),
            });
            return SegmentedCompositor::new(device, &mut self.blur, &mut self.color_matrix)
                .composite(segments, width, height)
                .map(|frame| (frame, FrameOutput::Premultiplied));
        }

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
        let rendered = device.renderer_mut().render_to_texture(
            &gpu_device,
            &gpu_queue,
            &main_scene,
            frame.view(),
            &params,
        );
        // Back before the `?`, so a failed render does not cost the buffers as
        // well. The scene's contents are meaningless from here on; the point of
        // putting it back is the six allocations underneath it.
        self.scene = main_scene;
        rendered.map_err(|error| {
            VelloRenderError::RenderFailed(error.to_string().into()).into_renderer_error()
        })?;

        Ok((frame, FrameOutput::Straight))
    }

    #[cfg(feature = "lyon")]
    /// Tries the Lyon backend and initializes classic Vello lazily when a display command requires it.
    fn render_lyon(
        &mut self,
        submission: &FrameSubmission,
        width: u32,
        height: u32,
        target: Option<&VelloFrame>,
    ) -> Result<Option<VelloFrame>, RendererError> {
        use obscur_diagnostics::{
            Diagnostic, DiagnosticCode, DiagnosticSeverity, DiagnosticSubsystem,
        };

        let Some(device) = self.device.as_ref().filter(|device| device.lyon.is_some()) else {
            return Ok(None);
        };
        let gpu = device.device().clone();
        let queue = device.queue().clone();
        let frame = target
            .cloned()
            .unwrap_or_else(|| self.cached_offscreen(&gpu, width, height));
        let Some(device) = self.device.as_mut() else {
            return Ok(None);
        };
        let Some(lyon) = device.lyon.as_mut() else {
            return Ok(None);
        };
        let list = submission.frame().scene().shared_display_list();
        match lyon.render(&gpu, &queue, &frame, list, &self.scene_converter) {
            Ok(()) => Ok(Some(frame)),
            Err(reason) => {
                if !device.lyon_fallback {
                    return Err(RendererError::Backend(format!("Lyon: {reason}").into()));
                }
                if let Some(sink) = self.diagnostics.as_ref()
                    && lyon.note_fallback(reason)
                {
                    sink.emit(&Diagnostic::new(
                        DiagnosticCode::new("render.lyon.fallback"),
                        DiagnosticSeverity::Warning,
                        DiagnosticSubsystem::Render,
                        format!("Lyon uses Vello for this frame: {reason}"),
                    ));
                }
                device
                    .ensure_classic()
                    .map_err(VelloRenderError::into_renderer_error)?;
                Ok(None)
            }
        }
    }

    #[doc = include_str!("../../docs/api/renderer/vello_renderer.doc/present_to_surface.md")]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn present_to_surface(
        &mut self,
        submission: &FrameSubmission,
        surface: &VelloSurface,
    ) -> Result<PresentResult, RendererError> {
        // Acquires here, on the caller's thread, and therefore blocks it. Kept
        // for hosts that have no thread to spare; a host that reads input on
        // this thread must use `present_to_texture` with an image fetched by a
        // `SurfaceAcquirer` instead, or its input path waits on the compositor
        // (audit 1-D — a spec violation, core principle 5).
        let surface_texture = surface
            .current_texture()
            .map_err(VelloRenderError::into_renderer_error)?;
        let image = AcquiredImage {
            texture: surface_texture,
            width: surface.config().width,
            height: surface.config().height,
            format: surface.blit_format(),
        };
        self.present_to_texture(submission, image)
    }

    #[doc = include_str!("../../docs/api/renderer/vello_renderer.doc/present_to_texture.md")]
    pub fn present_to_texture(
        &mut self,
        submission: &FrameSubmission,
        image: AcquiredImage,
    ) -> Result<PresentResult, RendererError> {
        self.present_to_texture_with(submission, image, || {})
    }

    #[doc = include_str!("../../docs/api/renderer/vello_renderer.doc/present_to_texture_with.md")]
    pub fn present_to_texture_with<F: FnOnce()>(
        &mut self,
        submission: &FrameSubmission,
        image: AcquiredImage,
        before_commit: F,
    ) -> Result<PresentResult, RendererError> {
        self.present_to_texture_using(submission, image, before_commit, |texture| {
            texture.present();
            true
        })?;
        Ok(PresentResult::Presented {
            generation: submission.generation(),
        })
    }

    /// Draw the frame and pass the rendered surface texture to the supplied presenter.
    /// The pre-commit callback runs after rendering succeeds, immediately before the
    /// texture is handed off.
    /// The presenter reports whether it completed the operation inline so diagnostic
    /// timing can distinguish completion from a handoff.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn present_to_texture_using(
        &mut self,
        submission: &FrameSubmission,
        image: AcquiredImage,
        before_commit: impl FnOnce(),
        present: impl FnOnce(wgpu::SurfaceTexture) -> bool,
    ) -> Result<(), RendererError> {
        static TRACE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        let detailed = *TRACE.get_or_init(|| {
            std::env::var_os("OBSCUR_FRAME_STATS").is_some_and(|mode| mode == "each")
        });
        let started = detailed.then(std::time::Instant::now);
        let AcquiredImage {
            texture: surface_texture,
            width,
            height,
            format,
        } = image;
        if width == 0 || height == 0 {
            return Err(
                VelloRenderError::RenderFailed("empty surface extent".into()).into_renderer_error(),
            );
        }
        // Use the configured unorm view so encoded premultiplied colors stay encoded.
        let surface_frame = VelloFrame::from_texture(surface_texture.texture.clone(), format);
        let measure_damage = DamageStats::enabled();
        #[cfg(feature = "lyon")]
        let direct = matches!(
            format,
            wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Bgra8Unorm
        ) && !force_surface_blit()
            && !measure_damage;
        let (frame, output) = self.render_to_frame(
            submission,
            width,
            height,
            #[cfg(feature = "lyon")]
            direct.then_some(&surface_frame),
        )?;
        let premultiply = matches!(output, FrameOutput::Straight);
        let raster_done = detailed.then(std::time::Instant::now);
        #[cfg(feature = "lyon")]
        if DamageStats::scene_enabled() {
            let region_id = self.frames_rendered + 1;
            if matches!(output, FrameOutput::Lyon)
                && let Some(lyon) = self.device.as_ref().and_then(|device| device.lyon.as_ref())
            {
                self.damage_stats.record_scene(
                    submission.frame().scene(),
                    (width, height),
                    format,
                    region_id,
                    lyon.text_bounds((width, height)),
                );
            } else {
                self.damage_stats.invalidate_scene();
            }
        }
        if measure_damage && let Some(device) = &self.device {
            self.damage_stats
                .capture(device, &frame, self.frames_rendered + 1);
        }
        if frame.texture() != surface_frame.texture() {
            self.blit_to_surface(&frame, &surface_frame, format, premultiply);
        }
        // Everything that could have failed has succeeded, and the very next
        // thing this function does is commit the surface. Anything that has to
        // be bound to *that* commit — a presentation feedback, a frame callback —
        // belongs here and nowhere else. Asked earlier, it would bind to this
        // commit only when the render happened to succeed, and to some later,
        // unrelated commit when it did not, silently shifting every subsequent
        // reading by one.
        let blit_done = detailed.then(std::time::Instant::now);
        before_commit();
        let presented_inline = present(surface_texture);

        if let (Some(started), Some(raster_done), Some(blit_done)) =
            (started, raster_done, blit_done)
        {
            let step = if presented_inline {
                "commit"
            } else {
                "handoff"
            };
            println!(
                "[present-steps] raster_submit_ms={:.3} blit_submit_ms={:.3} {step}_ms={:.3}",
                (raster_done - started).as_secs_f64() * 1000.0,
                (blit_done - raster_done).as_secs_f64() * 1000.0,
                blit_done.elapsed().as_secs_f64() * 1000.0,
            );
        }
        self.frames_rendered += 1;
        Ok(())
    }

    fn blit_to_surface(
        &mut self,
        frame: &VelloFrame,
        target: &VelloFrame,
        format: wgpu::TextureFormat,
        premultiply: bool,
    ) {
        let device = self.device.as_ref().expect("a rendered frame has a GPU");
        let gpu = device.device();
        let slot = &mut self.blitters[usize::from(premultiply)];
        if slot.as_ref().is_some_and(|(cached, _)| *cached != format) {
            *slot = None;
        }
        let (_, blitter) = slot.get_or_insert_with(|| {
            (
                format,
                PremultiplyBlitter::with_alpha_conversion(gpu, format, premultiply),
            )
        });
        let mut encoder = gpu.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("obscur-vello-present"),
        });
        blitter.copy(gpu, &mut encoder, frame.view(), target.view());
        device.queue().submit([encoder.finish()]);
    }

    #[doc = include_str!("../../docs/api/renderer/vello_renderer.doc/render_to_image.md")]
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
        let (frame, output) = self.render_to_frame(
            submission,
            width,
            height,
            #[cfg(feature = "lyon")]
            None,
        )?;
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
        let Some((unpadded_bytes_per_row, padded_bytes_per_row)) =
            super::readback_layout::readback_row_bytes(width)
        else {
            return Err(
                VelloRenderError::RenderFailed("readback row size overflow".into())
                    .into_renderer_error(),
            );
        };
        let buffer_size = u64::from(padded_bytes_per_row) * u64::from(height);
        if buffer_size > gpu_device.limits().max_buffer_size {
            return Err(VelloRenderError::RenderFailed(
                "readback exceeds the GPU buffer limit".into(),
            )
            .into_renderer_error());
        }
        if usize::try_from(buffer_size).is_err() {
            return Err(VelloRenderError::RenderFailed(
                "readback exceeds the host address space".into(),
            )
            .into_renderer_error());
        }
        // Each row is no wider than the corresponding row of the checked buffer.
        let row = unpadded_bytes_per_row as usize;
        let stride = padded_bytes_per_row as usize;
        let mut pixels = Vec::new();
        pixels
            .try_reserve_exact(row * height as usize)
            .map_err(|error| {
                VelloRenderError::RenderFailed(format!("allocate readback pixels: {error}").into())
                    .into_renderer_error()
            })?;

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
        {
            let view = readback.get_mapped_range(..);
            for chunk in view.chunks_exact(stride) {
                pixels.extend_from_slice(&chunk[..row]);
            }
        }
        readback.unmap();

        // Vello stores straight alpha. The readback API exposes premultiplied
        // RGBA8, matching the CPU pixmap and its PNG encoder.
        if matches!(output, FrameOutput::Straight) {
            for pixel in pixels.chunks_exact_mut(4) {
                let alpha = u16::from(pixel[3]);
                for channel in &mut pixel[..3] {
                    *channel = ((u16::from(*channel) * alpha + 127) / 255) as u8;
                }
            }
        }

        self.frames_rendered += 1;
        Ok((width, height, pixels))
    }
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
        let (frame, _) = self.render_to_frame(
            submission,
            width,
            height,
            #[cfg(feature = "lyon")]
            None,
        )?;

        if DamageStats::enabled()
            && let Some(device) = &self.device
        {
            self.damage_stats
                .capture(device, &frame, self.frames_rendered + 1);
        }
        self.frames_rendered += 1;
        Ok(PresentResult::Presented {
            generation: submission.generation(),
        })
    }

    #[doc = include_str!("../../docs/api/renderer/vello_renderer.doc/supports_effect.md")]
    fn supports_effect(&self, effect: SceneEffect) -> SupportLevel {
        if self.is_hybrid() {
            return match effect {
                SceneEffect::Gradient | SceneEffect::Clip => SupportLevel::Native,
                SceneEffect::Text
                | SceneEffect::Shadow
                | SceneEffect::Blur
                | SceneEffect::ImageMask
                | SceneEffect::BlendMode => SupportLevel::Limited,
                SceneEffect::Mask
                | SceneEffect::Image
                | SceneEffect::BackdropFilter
                | SceneEffect::ColorMatrix => SupportLevel::Unavailable,
            };
        }
        classic_effect_support(effect)
    }
}

fn classic_effect_support(effect: SceneEffect) -> SupportLevel {
    match effect {
        SceneEffect::Gradient | SceneEffect::Clip | SceneEffect::Image | SceneEffect::BlendMode => {
            SupportLevel::Native
        }
        SceneEffect::Shadow | SceneEffect::BackdropFilter => SupportLevel::Emulated,
        SceneEffect::Blur | SceneEffect::ColorMatrix | SceneEffect::Text => SupportLevel::Limited,
        SceneEffect::ImageMask => SupportLevel::Limited,
        SceneEffect::Mask => SupportLevel::Unavailable,
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
    fn rendering_a_clipped_filter_reports_the_omitted_effect() {
        use obscur_geometry::{Point, Rect};
        use obscur_scene::clip::Clip;
        use obscur_scene::layer::{LayerEffect, SceneLayer};

        let size = Size::new(64.0, 48.0);
        let clip = Clip::rect(Rect::new(Point::ORIGIN, size));
        let mut layer = SceneLayer::new().with_clip(clip.clone());
        layer.add_effect(LayerEffect::blur(2.0));
        let list = DisplayList::from_commands(vec![
            PaintCommand::PushClip(clip),
            PaintCommand::PushLayer(layer),
            PaintCommand::PopLayer,
            PaintCommand::PopClip,
        ]);
        let submission = FrameSubmission::new(RenderFrame::new(
            SceneSnapshot::new(list, SceneGeneration::from_raw(1)),
            size,
        ));
        let sink = Arc::new(RecordingSink::default());
        let mut renderer = VelloRenderer::new(RendererId::from_raw(11));
        renderer.set_diagnostic_sink(sink.clone());
        let _ = renderer.render(&submission, &a_target());
        assert_eq!(sink.messages().len(), 1);
        assert!(sink.messages()[0].contains("classic Vello omits filter layers"));
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

    /// The scans that raise these diagnostics are skipped while no sink is
    /// installed — two full display-list walks a frame that nothing could observe.
    /// The risk that buys is *silence*: a scan that recorded into
    /// `EffectFallbackReporter` while no one was listening would mark the effect
    /// "already raised", and the host that installs a sink one frame later would
    /// never hear about it. Skipping means not recording either, so the first
    /// frame after a sink arrives raises everything the scene still earns.
    #[test]
    fn a_sink_installed_after_the_frames_began_still_hears_the_scene() {
        use obscur_scene::clip::{Clip, Mask};
        let list = DisplayList::from_commands(vec![
            PaintCommand::PushClip(Clip::mask(Mask::alpha())),
            PaintCommand::PopClip,
        ]);
        let a_frame = || {
            FrameSubmission::new(RenderFrame::new(
                SceneSnapshot::new(list.clone(), SceneGeneration::from_raw(1)),
                Size::new(64.0, 48.0),
            ))
        };
        let mut renderer = VelloRenderer::new(RendererId::from_raw(15));
        // Three frames with nobody listening.
        for _ in 0..3 {
            let _ = renderer.render(&a_frame(), &a_target());
        }
        let sink = Arc::new(RecordingSink::default());
        renderer.set_diagnostic_sink(sink.clone());
        let _ = renderer.render(&a_frame(), &a_target());
        assert_eq!(
            sink.messages(),
            vec!["unsupported effect: mask".to_string()],
            "the mask was never raised while no sink was installed, so it is still owed"
        );
        // And still raised once, not once per frame, after that.
        let _ = renderer.render(&a_frame(), &a_target());
        assert_eq!(sink.messages().len(), 1);
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
        assert_eq!(
            renderer.supports_effect(SceneEffect::BlendMode),
            SupportLevel::Native
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

    #[doc = include_str!("../../docs/api/renderer/vello_renderer.doc/a_registered_face_is_the_hosts_allocation_not_a_copy_of_it.md")]
    #[test]
    fn a_registered_face_is_the_hosts_allocation_not_a_copy_of_it() {
        // What the kit's font registry holds, and what it hands over.
        let host: Arc<[u8]> = Arc::from(vec![0x00, 0x01, 0x00, 0x00, 0x42, 0x43]);
        let host_addr = host.as_ptr();

        let mut renderer = VelloRenderer::new(RendererId::from_raw(21));
        renderer.register_font(7, Arc::clone(&host));

        let registered = renderer
            .scene_converter
            .face_bytes(7)
            .expect("the face registered under handle 7");
        assert!(
            std::ptr::eq(registered.as_ptr(), host_addr),
            "the registered face must BE the host's allocation, not a copy of \
             it: the registry keeps every face for the life of the process, so \
             a copy here is a second permanent copy — 26 MB apiece for the CJK \
             faces on this desktop"
        );
        assert_eq!(registered.len(), host.len());

        // And the sharing is the only thing that changed: a caller with a
        // `Vec` still works, and still gets its own bytes.
        renderer.register_font(8, vec![0x00, 0x01, 0x00, 0x00, 0x44]);
        assert_eq!(
            renderer.scene_converter.face_bytes(8),
            Some([0x00, 0x01, 0x00, 0x00, 0x44].as_slice())
        );
    }

    #[test]
    fn re_registering_the_very_same_allocation_is_a_no_op() {
        // The guard in `SceneConverter::register_font` exists so that replacing
        // a face with itself does not orphan every glyph outline Vello cached
        // under the old `Blob` id. Sharing makes the common case an address
        // comparison; this checks that it still ANSWERS the same.
        let host: Arc<[u8]> = Arc::from(vec![1u8, 2, 3, 4]);
        let mut renderer = VelloRenderer::new(RendererId::from_raw(22));
        renderer.register_font(0, Arc::clone(&host));
        let first = renderer
            .scene_converter
            .face_bytes(0)
            .expect("registered")
            .as_ptr();
        renderer.register_font(0, Arc::clone(&host));
        let second = renderer
            .scene_converter
            .face_bytes(0)
            .expect("still registered")
            .as_ptr();
        assert!(std::ptr::eq(first, second), "the face was not replaced");
    }
}

#[cfg(feature = "lyon")]
fn force_surface_blit() -> bool {
    static FORCE_BLIT: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *FORCE_BLIT.get_or_init(|| {
        std::env::var_os("OBSCUR_LYON_SURFACE_BLIT").is_some_and(|value| value == "1")
    })
}
