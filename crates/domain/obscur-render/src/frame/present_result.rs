//! The typed outcome of presenting a frame (spec §5.2 explicit diagnostics).

use obscur_scene::scene::SceneGeneration;

#[doc = include_str!("../../docs/api/frame/present_result.doc/PresentResult.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PresentResult {
    /// The frame reached the screen, echoing which scene generation was
    /// presented.
    Presented {
        /// The generation of the scene that reached the screen.
        generation: SceneGeneration,
    },
    /// A stale intermediate frame was dropped before presentation (spec §3.5).
    Skipped,
    /// The GPU device was lost during presentation; see
    /// [`DeviceLoss`](crate::frame::DeviceLoss) for the reason and
    /// recoverability.
    DeviceLost,
    /// Presentation failed for the given human-readable reason.
    Failed(Box<str>),
}

impl PresentResult {
    /// Whether the frame was successfully presented.
    #[must_use]
    pub const fn is_presented(&self) -> bool {
        matches!(self, Self::Presented { .. })
    }

    #[doc = include_str!("../../docs/api/frame/present_result.doc/is_recoverable.md")]
    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::Presented { .. } | Self::Skipped | Self::DeviceLost
        )
    }

    /// The failure reason when this outcome is [`PresentResult::Failed`], else
    /// `None`.
    #[must_use]
    pub fn failure_reason(&self) -> Option<&str> {
        match self {
            Self::Failed(reason) => Some(reason),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_presented_only_for_presented() {
        let presented = PresentResult::Presented {
            generation: SceneGeneration::from_raw(4),
        };
        assert!(presented.is_presented());
        assert!(!PresentResult::Skipped.is_presented());
        assert!(!PresentResult::DeviceLost.is_presented());
        assert!(!PresentResult::Failed("boom".into()).is_presented());
    }

    #[test]
    fn is_recoverable_except_failed() {
        assert!(
            PresentResult::Presented {
                generation: SceneGeneration::FIRST,
            }
            .is_recoverable()
        );
        assert!(PresentResult::Skipped.is_recoverable());
        assert!(PresentResult::DeviceLost.is_recoverable());
        // Failed is terminal.
        assert!(!PresentResult::Failed("boom".into()).is_recoverable());
    }

    #[test]
    fn failure_reason_only_for_failed() {
        assert_eq!(
            PresentResult::Failed("swapchain lost".into()).failure_reason(),
            Some("swapchain lost")
        );
        assert_eq!(PresentResult::Skipped.failure_reason(), None);
        assert_eq!(
            PresentResult::Presented {
                generation: SceneGeneration::FIRST,
            }
            .failure_reason(),
            None
        );
    }
}
