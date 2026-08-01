#![doc = include_str!("cpu_render_error.doc/_module.md")]

use obscur_render::RendererError;

#[doc = include_str!("cpu_render_error.doc/CpuRenderError.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CpuRenderError {
    #[doc = include_str!("cpu_render_error.doc/PixmapAllocationFailed.md")]
    PixmapAllocationFailed {
        #[doc = include_str!("cpu_render_error.doc/width.md")]
        width: u32,
        #[doc = include_str!("cpu_render_error.doc/height.md")]
        height: u32,
    },
    #[doc = include_str!("cpu_render_error.doc/EmptyTarget.md")]
    EmptyTarget,
    #[doc = include_str!("cpu_render_error.doc/Backend.md")]
    Backend(Box<str>),
}

impl CpuRenderError {
    #[doc = include_str!("cpu_render_error.doc/into_renderer_error.md")]
    #[must_use]
    pub fn into_renderer_error(self) -> RendererError {
        match self {
            Self::PixmapAllocationFailed { width, height } => RendererError::InvalidTarget(
                format!("cpu renderer could not allocate a {width}x{height} pixel buffer").into(),
            ),
            Self::EmptyTarget => {
                RendererError::InvalidTarget("cpu renderer target has zero area".into())
            }
            Self::Backend(reason) => RendererError::Backend(reason),
        }
    }
}

impl std::fmt::Display for CpuRenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PixmapAllocationFailed { width, height } => {
                write!(f, "could not allocate a {width}x{height} pixel buffer")
            }
            Self::EmptyTarget => f.write_str("render target has zero area"),
            Self::Backend(reason) => write!(f, "cpu renderer backend failure: {reason}"),
        }
    }
}

impl std::error::Error for CpuRenderError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_renders_each_variant() {
        assert_eq!(
            CpuRenderError::PixmapAllocationFailed {
                width: 8,
                height: 4,
            }
            .to_string(),
            "could not allocate a 8x4 pixel buffer"
        );
        assert_eq!(
            CpuRenderError::EmptyTarget.to_string(),
            "render target has zero area"
        );
        assert_eq!(
            CpuRenderError::Backend("boom".into()).to_string(),
            "cpu renderer backend failure: boom"
        );
    }

    #[test]
    fn allocation_and_empty_map_to_invalid_target() {
        assert!(matches!(
            CpuRenderError::PixmapAllocationFailed {
                width: 0,
                height: 0,
            }
            .into_renderer_error(),
            RendererError::InvalidTarget(_)
        ));
        assert!(matches!(
            CpuRenderError::EmptyTarget.into_renderer_error(),
            RendererError::InvalidTarget(_)
        ));
    }

    #[test]
    fn backend_maps_to_backend_preserving_reason() {
        assert_eq!(
            CpuRenderError::Backend("vk".into()).into_renderer_error(),
            RendererError::Backend("vk".into())
        );
    }

    #[test]
    fn is_error_trait_object() {
        let err = CpuRenderError::EmptyTarget;
        let _: &dyn std::error::Error = &err;
    }
}
