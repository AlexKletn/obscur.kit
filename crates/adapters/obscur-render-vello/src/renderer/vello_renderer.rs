#![doc = include_str!("vello_renderer.doc/_module.md")]

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

use vello::peniko::{Blob, FontData};

use crate::conversion::SceneConverter;
use crate::effects::{ColorMatrix, GaussianBlur};
use crate::error::VelloRenderError;
use crate::renderer::filter_layer_pass::PreparedFilterLayers;
use crate::renderer::segmented_compositor::SegmentedCompositor;
use crate::renderer::{VelloDevice, VelloFrame, VelloSurface};

#[doc = include_str!("vello_renderer.doc/VelloRenderer.md")]
pub struct VelloRenderer {
    id: RendererId,
    capabilities: RendererCapabilities,
    device: Option<VelloDevice>,
    scene_converter: SceneConverter,
    frames_rendered: u64,
    #[doc = include_str!("vello_renderer.doc/blitter.md")]
    blitter: Option<(wgpu::TextureFormat, wgpu::util::TextureBlitter)>,
    #[doc = include_str!("vello_renderer.doc/frame_cache.md")]
    frame_cache: Option<VelloFrame>,
    #[doc = include_str!("vello_renderer.doc/blur.md")]
    blur: Option<GaussianBlur>,
    #[doc = include_str!("vello_renderer.doc/color_matrix.md")]
    color_matrix: Option<ColorMatrix>,
    #[doc = include_str!("vello_renderer.doc/diagnostics.md")]
    diagnostics: Arc<dyn DiagnosticSink>,
    #[doc = include_str!("vello_renderer.doc/fallbacks.md")]
    fallbacks: EffectFallbackReporter,
}

impl VelloRenderer {
    #[doc = include_str!("vello_renderer.doc/new.md")]
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

    #[doc = include_str!("vello_renderer.doc/set_diagnostic_sink.md")]
    pub fn set_diagnostic_sink(&mut self, sink: Arc<dyn DiagnosticSink>) {
        self.diagnostics = sink;
    }

    #[doc = include_str!("vello_renderer.doc/with_device.md")]
    #[must_use]
    pub fn with_device(mut self, device: VelloDevice) -> Self {
        self.device = Some(device);
        self
    }

    #[doc = include_str!("vello_renderer.doc/set_device.md")]
    pub fn set_device(&mut self, device: VelloDevice) {
        self.device = Some(device);
    }

    #[doc = include_str!("vello_renderer.doc/register_font.md")]
    pub fn register_font(&mut self, id: u32, font_bytes: Vec<u8>) {
        let font = FontData::new(Blob::from(font_bytes), 0);
        self.scene_converter.register_font(id, font);
    }

    #[doc = include_str!("vello_renderer.doc/set_font.md")]
    pub fn set_font(&mut self, font_bytes: Vec<u8>) {
        self.register_font(0, font_bytes);
    }

    #[doc = include_str!("vello_renderer.doc/text_fallbacks.md")]
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

    #[doc = include_str!("vello_renderer.doc/has_device.md")]
    #[must_use]
    pub const fn has_device(&self) -> bool {
        self.device.is_some()
    }

    #[doc = include_str!("vello_renderer.doc/frames_rendered.md")]
    #[must_use]
    pub const fn frames_rendered(&self) -> u64 {
        self.frames_rendered
    }

    #[doc = include_str!("vello_renderer.doc/cached_offscreen.md")]
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

    #[doc = include_str!("vello_renderer.doc/render_to_frame.md")]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
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
            // Convert first (device-free): the ordered content / backdrop segments.
            let segments = self.scene_converter.convert_segments(display_list);
            let Some(device) = self.device.as_mut() else {
                return Err(
                    VelloRenderError::DeviceCreation("no gpu device".into()).into_renderer_error()
                );
            };
            return SegmentedCompositor::new(device, &mut self.blur, &mut self.color_matrix)
                .composite(segments, width, height);
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
            let prepared = PreparedFilterLayers::render(
                &gpu_device,
                &gpu_queue,
                device.renderer_mut(),
                blurs,
            )?;

            // The `device` borrow ends here; build/reuse the owned filter pipelines
            // and post-process + composite each filter sub-scene onto the frame in
            // one encoder. The Vello render-to-texture passes above already
            // submitted, so this trailing submit runs after them on the same queue
            // (correct ordering).
            let needs_color = prepared.needs_color_matrix();
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
            let scratch = prepared.composite(
                &gpu_device,
                &mut encoder,
                gaussian,
                color_matrix.as_deref(),
                frame.view(),
                (width, height),
            );
            gpu_queue.submit(std::iter::once(encoder.finish()));
            drop(scratch);
        }

        Ok(frame)
    }

    #[doc = include_str!("vello_renderer.doc/present_to_surface.md")]
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

    #[doc = include_str!("vello_renderer.doc/render_to_image.md")]
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

    #[doc = include_str!("vello_renderer.doc/supports_effect.md")]
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
