//! The Vello backend's typed failure, mapped into the domain's `RendererError`
//! (spec §5.2 explicit diagnostics; §2.7 the vello adapter owns its own errors).

use obscur_render::frame::{DeviceLoss, DeviceLossReason};
use obscur_render::renderer::RendererError;

/// A typed failure raised inside the Vello/wgpu backend (spec §5.2, §2.7).
///
/// Spec §5.2 requires backend failures to surface as explicit diagnostics rather
/// than silent fallbacks or panics. This enum names the distinct causes the
/// Vello adapter can hit while bringing up wgpu, configuring a surface, or
/// submitting a frame. It is local to this crate (§2.7): callers outside the
/// adapter see only the domain [`RendererError`] produced by
/// [`VelloRenderError::into_renderer_error`].
///
/// The mapping into [`RendererError`] treats an *unusable device* — creation
/// failure or a total absence of a GPU adapter — as a
/// [`RendererError::DeviceLost`], since in both cases there is no device to draw
/// with. Surface and frame failures map to the generic
/// [`RendererError::Backend`], each carrying its human-readable reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VelloRenderError {
    /// Creating the wgpu device (or queue) failed, with a human-readable reason.
    DeviceCreation(Box<str>),
    /// The render surface was lost and must be reconfigured before the next
    /// frame, with a human-readable reason.
    SurfaceLost(Box<str>),
    /// Configuring the render surface (format, size, present mode) failed, with
    /// a human-readable reason.
    SurfaceConfiguration(Box<str>),
    /// Submitting or presenting a rendered frame failed, with a human-readable
    /// reason.
    RenderFailed(Box<str>),
    /// No GPU adapter compatible with the backend's requirements was found.
    NoGpuAdapter,
}

impl VelloRenderError {
    /// Maps this backend failure into the domain [`RendererError`] (spec §5.2).
    ///
    /// A failed device creation and a missing adapter both mean there is no
    /// usable GPU device, so they become a fatal
    /// [`RendererError::DeviceLost`]. The surface and frame failures are generic
    /// backend faults and become [`RendererError::Backend`], preserving their
    /// reason text.
    #[must_use]
    pub fn into_renderer_error(self) -> RendererError {
        match self {
            Self::DeviceCreation(reason) => RendererError::DeviceLost(
                DeviceLoss::fatal(DeviceLossReason::Unknown).with_detail(reason),
            ),
            Self::NoGpuAdapter => RendererError::DeviceLost(
                DeviceLoss::fatal(DeviceLossReason::Unknown)
                    .with_detail("no compatible GPU adapter"),
            ),
            Self::SurfaceLost(reason)
            | Self::SurfaceConfiguration(reason)
            | Self::RenderFailed(reason) => RendererError::Backend(reason),
        }
    }
}

impl std::fmt::Display for VelloRenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DeviceCreation(reason) => {
                write!(f, "vello device creation failed: {reason}")
            }
            Self::SurfaceLost(reason) => write!(f, "vello surface lost: {reason}"),
            Self::SurfaceConfiguration(reason) => {
                write!(f, "vello surface configuration failed: {reason}")
            }
            Self::RenderFailed(reason) => write!(f, "vello render failed: {reason}"),
            Self::NoGpuAdapter => f.write_str("no compatible GPU adapter found"),
        }
    }
}

impl std::error::Error for VelloRenderError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_creation_maps_to_device_lost() {
        let err = VelloRenderError::DeviceCreation("no vulkan".into());
        match err.into_renderer_error() {
            RendererError::DeviceLost(loss) => {
                assert_eq!(loss.reason(), DeviceLossReason::Unknown);
                assert!(!loss.is_recoverable());
                assert_eq!(loss.detail(), Some("no vulkan"));
            }
            other => panic!("expected DeviceLost, got {other:?}"),
        }
    }

    #[test]
    fn no_gpu_adapter_maps_to_device_lost() {
        assert!(matches!(
            VelloRenderError::NoGpuAdapter.into_renderer_error(),
            RendererError::DeviceLost(_)
        ));
    }

    #[test]
    fn surface_and_render_failures_map_to_backend() {
        assert_eq!(
            VelloRenderError::SurfaceLost("gone".into()).into_renderer_error(),
            RendererError::Backend("gone".into())
        );
        assert_eq!(
            VelloRenderError::SurfaceConfiguration("bad format".into()).into_renderer_error(),
            RendererError::Backend("bad format".into())
        );
        assert_eq!(
            VelloRenderError::RenderFailed("submit".into()).into_renderer_error(),
            RendererError::Backend("submit".into())
        );
    }

    #[test]
    fn display_renders_each_variant() {
        assert_eq!(
            VelloRenderError::DeviceCreation("x".into()).to_string(),
            "vello device creation failed: x"
        );
        assert_eq!(
            VelloRenderError::NoGpuAdapter.to_string(),
            "no compatible GPU adapter found"
        );
        let _: &dyn std::error::Error = &VelloRenderError::NoGpuAdapter;
    }
}
