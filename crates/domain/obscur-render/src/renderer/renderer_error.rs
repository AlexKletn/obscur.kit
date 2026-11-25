//! The typed failure of a render operation (spec §5.2 explicit diagnostics).

use crate::frame::DeviceLoss;

#[doc = include_str!("../../docs/api/renderer/renderer_error.doc/RendererError.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RendererError {
    /// The GPU device was lost; see [`DeviceLoss`] for the reason and whether
    /// the runtime may rebuild and retry.
    DeviceLost(DeviceLoss),
    #[doc = include_str!("../../docs/api/renderer/renderer_error.doc/UnsupportedEffect.md")]
    UnsupportedEffect(Box<str>),
    /// The render target was invalid for this backend, with a human-readable
    /// reason.
    InvalidTarget(Box<str>),
    /// The backend ran out of memory servicing the operation.
    OutOfMemory,
    /// A generic backend failure, carrying a human-readable reason.
    Backend(Box<str>),
}

impl RendererError {
    /// Whether this error is a device-loss event.
    #[must_use]
    pub const fn is_device_lost(&self) -> bool {
        matches!(self, Self::DeviceLost(_))
    }

    #[doc = include_str!("../../docs/api/renderer/renderer_error.doc/is_recoverable.md")]
    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        match self {
            Self::DeviceLost(loss) => loss.is_recoverable(),
            Self::UnsupportedEffect(_) => true,
            Self::InvalidTarget(_) | Self::OutOfMemory | Self::Backend(_) => false,
        }
    }
}

impl std::fmt::Display for RendererError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DeviceLost(loss) => write!(f, "renderer device lost: {:?}", loss.reason()),
            Self::UnsupportedEffect(effect) => write!(f, "unsupported effect: {effect}"),
            Self::InvalidTarget(reason) => write!(f, "invalid render target: {reason}"),
            Self::OutOfMemory => f.write_str("renderer out of memory"),
            Self::Backend(reason) => write!(f, "renderer backend failure: {reason}"),
        }
    }
}

impl std::error::Error for RendererError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::DeviceLossReason;

    #[test]
    fn display_renders_each_variant() {
        assert_eq!(
            RendererError::UnsupportedEffect("blur".into()).to_string(),
            "unsupported effect: blur"
        );
        assert_eq!(
            RendererError::InvalidTarget("no swapchain".into()).to_string(),
            "invalid render target: no swapchain"
        );
        assert_eq!(
            RendererError::OutOfMemory.to_string(),
            "renderer out of memory"
        );
        assert_eq!(
            RendererError::Backend("vk error".into()).to_string(),
            "renderer backend failure: vk error"
        );
        assert_eq!(
            RendererError::DeviceLost(DeviceLoss::fatal(DeviceLossReason::Removed)).to_string(),
            "renderer device lost: Removed"
        );
    }

    #[test]
    fn is_error_trait_object() {
        let err = RendererError::OutOfMemory;
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn is_device_lost_only_for_device_lost() {
        let loss = RendererError::DeviceLost(DeviceLoss::recoverable(DeviceLossReason::Reset));
        assert!(loss.is_device_lost());
        assert!(!RendererError::OutOfMemory.is_device_lost());
    }

    #[test]
    fn recoverability_follows_variant() {
        // Device loss follows the loss flag.
        assert!(
            RendererError::DeviceLost(DeviceLoss::recoverable(DeviceLossReason::Reset))
                .is_recoverable()
        );
        assert!(
            !RendererError::DeviceLost(DeviceLoss::fatal(DeviceLossReason::Removed))
                .is_recoverable()
        );
        // Unsupported effect is a fallback signal: always recoverable.
        assert!(RendererError::UnsupportedEffect("shadow".into()).is_recoverable());
        // The rest are terminal.
        assert!(!RendererError::InvalidTarget("x".into()).is_recoverable());
        assert!(!RendererError::OutOfMemory.is_recoverable());
        assert!(!RendererError::Backend("x".into()).is_recoverable());
    }
}
