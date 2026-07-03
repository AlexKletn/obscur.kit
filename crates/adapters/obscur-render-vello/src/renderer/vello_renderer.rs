//! The Vello/wgpu implementation of the [`Renderer`] port (spec §5.2, §2.7).

use obscur_capability::SupportLevel;
use obscur_render::frame::{FrameSubmission, PresentResult};
use obscur_render::renderer::{Renderer, RendererCapabilities, RendererError, RendererId};
use obscur_render::target::RenderTarget;

use crate::conversion::SceneConverter;
use crate::error::VelloRenderError;
use crate::renderer::{VelloDevice, VelloFrame};

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
            // Blur, drop-shadow, and backdrop filters are expressible via
            // Vello's layer/blend machinery but not a single native primitive.
            "blur" | "shadow" | "backdrop-filter" => SupportLevel::Emulated,
            // Anything unrecognised is honestly unavailable rather than a
            // silent no-op (spec §5.2 explicit diagnostics).
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
        assert_eq!(
            renderer.supports_effect("backdrop-filter"),
            SupportLevel::Emulated
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
