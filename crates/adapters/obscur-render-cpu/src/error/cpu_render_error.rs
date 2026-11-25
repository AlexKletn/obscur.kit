//! The CPU renderer adapter's internal error, mapped onto the domain
//! [`RendererError`] (spec §5.2 explicit diagnostics).

use obscur_render::RendererError;

#[doc = include_str!("../../docs/api/error/cpu_render_error.doc/CpuRenderError.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CpuRenderError {
    /// `tiny_skia::Pixmap::new` returned `None` for the requested dimensions —
    /// the pixel buffer could not be allocated (e.g. a zero or too-large edge).
    PixmapAllocationFailed {
        /// The requested buffer width, in physical pixels.
        width: u32,
        /// The requested buffer height, in physical pixels.
        height: u32,
    },
    /// The render target has zero area, so there is nothing to rasterize into.
    EmptyTarget,
    /// A generic rasterizer-backend failure, carrying a human-readable reason.
    Backend(Box<str>),
}

impl CpuRenderError {
    #[doc = include_str!("../../docs/api/error/cpu_render_error.doc/into_renderer_error.md")]
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
