//! The Vello/wgpu implementation of the [`Renderer`] port (spec §5.2, §2.7).

use obscur_capability::SupportLevel;
use obscur_render::frame::{FrameSubmission, PresentResult};
use obscur_render::renderer::{Renderer, RendererCapabilities, RendererError, RendererId};
use obscur_render::target::RenderTarget;

use vello::kurbo::Affine;
use vello::peniko::{Blob, FontData};
use vello::Scene;

use crate::conversion::{BlurLayer, SceneConverter};
use crate::effects::GaussianBlur;
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
    /// Cached owned wgpu gaussian-blur pipeline set for `filter: blur()` (spec
    /// §5.2). Built lazily on the first blurred frame (it needs the device) and
    /// reused across frames; a scene with no blur layer never constructs it, so
    /// the non-blur path is byte-identical to before.
    blur: Option<GaussianBlur>,
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
            blur: None,
        }
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
    fn render_to_frame(
        &mut self,
        submission: &FrameSubmission,
        width: u32,
        height: u32,
    ) -> Result<VelloFrame, RendererError> {
        // Convert first (device-free): the main scene plus any blur sub-scenes.
        let (main_scene, blurs) = self
            .scene_converter
            .convert_with_blurs(submission.frame().scene().display_list());

        let Some(device) = self.device.as_mut() else {
            return Err(
                VelloRenderError::DeviceCreation("no gpu device".into()).into_renderer_error()
            );
        };

        let frame = VelloFrame::new_offscreen(device.device(), width, height);
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

            // The `device` borrow ends here; build/reuse the owned blur pipelines and
            // composite each blurred sub-scene onto the frame in one encoder. The
            // Vello render-to-texture passes above already submitted, so this
            // trailing submit runs after them on the same queue (correct ordering).
            let gaussian = self
                .blur
                .get_or_insert_with(|| GaussianBlur::new(&gpu_device));
            let mut encoder = gpu_device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("obscur-vello-blur"),
            });
            for (s_frame, origin, blur) in &prepared {
                gaussian.render(
                    &gpu_device,
                    &mut encoder,
                    s_frame,
                    *origin,
                    blur,
                    frame.view(),
                    (width, height),
                );
            }
            gpu_queue.submit(std::iter::once(encoder.finish()));
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
                return Err(VelloRenderError::RenderFailed(error.to_string().into())
                    .into_renderer_error());
            }
            Err(error) => {
                return Err(VelloRenderError::RenderFailed(error.to_string().into())
                    .into_renderer_error());
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

    fn supports_effect(&self, effect: &str) -> SupportLevel {
        // Honest per-effect support for the Vello GPU backend (spec §5.2).
        match effect {
            // Vello rasterizes fills/strokes, clips, and images natively — a
            // `DrawImage` becomes a real `Scene::draw_image` (a sampled bitmap),
            // not an approximation.
            "clip" | "gradient" | "image" => SupportLevel::Native,
            // A `box-shadow` and a `filter: drop-shadow(...)` both render through
            // `Scene::draw_blurred_rounded_rect` — a real gaussian, but built from
            // Vello's primitives rather than a single native shadow op.
            "blur" | "shadow" | "drop-shadow" => SupportLevel::Emulated,
            // The general `filter` / `backdrop-filter` (content blur, color-matrix)
            // have no primitive on Vello 0.9 — only the `drop-shadow` subset is
            // expressible — so they are honestly unavailable rather than a silent
            // no-op (spec §5.2 explicit diagnostics).
            _ => SupportLevel::Unavailable,
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
        assert_eq!(renderer.supports_effect("clip"), SupportLevel::Native);
        assert_eq!(renderer.supports_effect("gradient"), SupportLevel::Native);
        // Images draw as a real sampled bitmap, not an approximation.
        assert_eq!(renderer.supports_effect("image"), SupportLevel::Native);
        assert_eq!(renderer.supports_effect("blur"), SupportLevel::Emulated);
        assert_eq!(renderer.supports_effect("shadow"), SupportLevel::Emulated);
        // `filter: drop-shadow(...)` renders through the real gaussian shadow path.
        assert_eq!(
            renderer.supports_effect("drop-shadow"),
            SupportLevel::Emulated
        );
        // The general `filter` / `backdrop-filter` (content blur, color-matrix)
        // have no Vello 0.9 primitive — honestly unavailable, not a silent no-op.
        assert_eq!(
            renderer.supports_effect("filter"),
            SupportLevel::Unavailable
        );
        assert_eq!(
            renderer.supports_effect("backdrop-filter"),
            SupportLevel::Unavailable
        );
        assert_eq!(
            renderer.supports_effect("time-travel"),
            SupportLevel::Unavailable
        );
    }

    #[test]
    fn renderer_is_object_safe() {
        let _: Box<dyn Renderer> = Box::new(VelloRenderer::new(RendererId::from_raw(9)));
    }
}
