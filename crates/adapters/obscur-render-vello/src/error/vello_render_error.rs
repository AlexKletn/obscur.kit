//! The Vello backend's typed failure, mapped into the domain's `RendererError`
//! (spec §5.2 explicit diagnostics; §2.7 the vello adapter owns its own errors).

use obscur_render::frame::{DeviceLoss, DeviceLossReason};
use obscur_render::renderer::RendererError;

#[doc = include_str!("../../docs/api/error/vello_render_error.doc/VelloRenderError.md")]
#[derive(Debug)]
pub enum VelloRenderError {
    /// Creating the wgpu device (or queue) failed, with a human-readable reason.
    DeviceCreation(Box<str>),
    /// Vello failed to initialize its renderer for an existing device.
    RendererInitialization(vello::Error),
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
    #[doc = include_str!("../../docs/api/error/vello_render_error.doc/into_renderer_error.md")]
    #[must_use]
    pub fn into_renderer_error(self) -> RendererError {
        match self {
            Self::DeviceCreation(reason) => RendererError::DeviceLost(
                DeviceLoss::fatal(DeviceLossReason::Unknown).with_detail(reason),
            ),
            Self::RendererInitialization(error) => RendererError::DeviceLost(
                DeviceLoss::fatal(DeviceLossReason::Unknown)
                    .with_detail(error.to_string().into_boxed_str()),
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
            Self::RendererInitialization(error) => {
                write!(f, "vello renderer initialization failed: {error}")
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

impl std::error::Error for VelloRenderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::RendererInitialization(error) => Some(error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;

    #[test]
    fn renderer_initialization_exposes_the_vello_error() {
        let error = VelloRenderError::RendererInitialization(vello::Error::NoCompatibleDevice);
        let source = error
            .source()
            .and_then(|source| source.downcast_ref::<vello::Error>());

        assert!(matches!(source, Some(vello::Error::NoCompatibleDevice)));
    }

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
