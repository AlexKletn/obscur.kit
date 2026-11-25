//! Renderer selection shared by the ordinary window and layer-shell entry points.

use super::AppError;

/// GPU backend requested by a standalone application.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RenderBackend {
    #[default]
    /// Classic Vello, the default backend.
    Vello,
    /// Vello Sparse Strips, available with the `render-vello-hybrid` feature.
    VelloHybrid,
    /// Lyon tessellation and wgpu rasterization, available with the `render-lyon` feature.
    Lyon,
}

impl RenderBackend {
    /// The backend name used in diagnostics and environment configuration.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Vello => "vello",
            Self::VelloHybrid => "vello-hybrid",
            Self::Lyon => "lyon",
        }
    }

    pub(super) fn resolve(explicit: Option<Self>) -> Result<Self, AppError> {
        let backend = match explicit {
            Some(backend) => backend,
            None => match std::env::var("OBSCUR_RENDERER") {
                Ok(value) => Self::from_name(&value)?,
                Err(std::env::VarError::NotPresent) => Self::default(),
                Err(std::env::VarError::NotUnicode(_)) => {
                    return Err(AppError::Gpu("OBSCUR_RENDERER must be UTF-8".into()));
                }
            },
        };
        match backend {
            Self::VelloHybrid if !cfg!(feature = "render-vello-hybrid") => Err(AppError::Gpu(
                "vello-hybrid requires the render-vello-hybrid feature".into(),
            )),
            Self::Lyon if !cfg!(feature = "render-lyon") => Err(AppError::Gpu(
                "lyon requires the render-lyon feature".into(),
            )),
            _ => Ok(backend),
        }
    }

    fn from_name(name: &str) -> Result<Self, AppError> {
        match name {
            "vello" | "classic" => Ok(Self::Vello),
            "vello-hybrid" => Ok(Self::VelloHybrid),
            "lyon" => Ok(Self::Lyon),
            _ => Err(AppError::Gpu(format!(
                "unknown OBSCUR_RENDERER value {name:?}, expected vello, vello-hybrid or lyon"
            ))),
        }
    }
}
