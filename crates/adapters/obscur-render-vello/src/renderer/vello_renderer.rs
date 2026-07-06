//! The Vello/wgpu implementation of the [`Renderer`] port (spec §5.2, §2.7).

use obscur_capability::SupportLevel;
use obscur_render::frame::{FrameSubmission, PresentResult};
use obscur_render::renderer::{Renderer, RendererCapabilities, RendererError, RendererId};
use obscur_render::target::RenderTarget;

use vello::peniko::{Blob, FontData};

use crate::conversion::SceneConverter;
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

        // Convert first (device-free) so the display list drives Vello's encoding
        // regardless of the device's presence, mirroring the port `render`.
        let scene = self
            .scene_converter
            .convert(submission.frame().scene().display_list());

        let Some(device) = self.device.as_mut() else {
            return Err(
                VelloRenderError::DeviceCreation("no gpu device".into()).into_renderer_error()
            );
        };

        // Acquire the swapchain image to present into.
        let surface_texture = surface
            .current_texture()
            .map_err(VelloRenderError::into_renderer_error)?;

        // Rasterize into the offscreen intermediate exactly like the port path.
        let frame = VelloFrame::new_offscreen(device.device(), width, height);
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
            .render_to_texture(&gpu_device, &gpu_queue, &scene, frame.view(), &params)
            .map_err(|error| {
                VelloRenderError::RenderFailed(error.to_string().into()).into_renderer_error()
            })?;

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

        // Convert first (device-free) so a missing device still validates the
        // scene path; the display list drives Vello's encoding.
        let scene = self
            .scene_converter
            .convert(submission.frame().scene().display_list());

        let Some(device) = self.device.as_mut() else {
            // Honest: without a host-provided device the Vello backend cannot
            // rasterize. A host must `set_device` first.
            return Err(
                VelloRenderError::DeviceCreation("no gpu device".into()).into_renderer_error()
            );
        };

        let frame = VelloFrame::new_offscreen(device.device(), width, height);
        let params = vello::RenderParams {
            base_color: vello::peniko::Color::TRANSPARENT,
            width,
            height,
            antialiasing_method: vello::AaConfig::Area,
        };
        // `render_to_texture` needs the device/queue immutably while the renderer
        // is borrowed mutably. wgpu's `Device`/`Queue` are cheap ref-counted
        // handles, so clone them to sidestep the overlapping borrow.
        let gpu_device = device.device().clone();
        let gpu_queue = device.queue().clone();
        device
            .renderer_mut()
            .render_to_texture(&gpu_device, &gpu_queue, &scene, frame.view(), &params)
            .map_err(|error| {
                VelloRenderError::RenderFailed(error.to_string().into()).into_renderer_error()
            })?;

        self.frames_rendered += 1;
        Ok(PresentResult::Presented {
            generation: submission.generation(),
        })
    }

    fn supports_effect(&self, effect: &str) -> SupportLevel {
        // Honest per-effect support for the Vello GPU backend (spec §5.2).
        match effect {
            // Vello rasterizes fills/strokes and clips natively.
            "clip" | "gradient" => SupportLevel::Native,
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
    fn supports_effect_is_honest_per_effect() {
        let renderer = VelloRenderer::new(RendererId::from_raw(4));
        assert_eq!(renderer.supports_effect("clip"), SupportLevel::Native);
        assert_eq!(renderer.supports_effect("gradient"), SupportLevel::Native);
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
