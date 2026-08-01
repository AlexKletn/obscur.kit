#![doc = include_str!("vello_render_error.doc/_module.md")]

use obscur_render::frame::{DeviceLoss, DeviceLossReason};
use obscur_render::renderer::RendererError;

#[doc = include_str!("vello_render_error.doc/VelloRenderError.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VelloRenderError {
    #[doc = include_str!("vello_render_error.doc/DeviceCreation.md")]
    DeviceCreation(Box<str>),
    #[doc = include_str!("vello_render_error.doc/SurfaceLost.md")]
    SurfaceLost(Box<str>),
    #[doc = include_str!("vello_render_error.doc/SurfaceConfiguration.md")]
    SurfaceConfiguration(Box<str>),
    #[doc = include_str!("vello_render_error.doc/RenderFailed.md")]
    RenderFailed(Box<str>),
    #[doc = include_str!("vello_render_error.doc/NoGpuAdapter.md")]
    NoGpuAdapter,
}

impl VelloRenderError {
    #[doc = include_str!("vello_render_error.doc/into_renderer_error.md")]
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
