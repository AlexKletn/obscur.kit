#![doc = include_str!("present_result.doc/_module.md")]

use obscur_scene::scene::SceneGeneration;

#[doc = include_str!("present_result.doc/PresentResult.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PresentResult {
    #[doc = include_str!("present_result.doc/Presented.md")]
    Presented {
        #[doc = include_str!("present_result.doc/generation.md")]
        generation: SceneGeneration,
    },
    #[doc = include_str!("present_result.doc/Skipped.md")]
    Skipped,
    #[doc = include_str!("present_result.doc/DeviceLost.md")]
    DeviceLost,
    #[doc = include_str!("present_result.doc/Failed.md")]
    Failed(Box<str>),
}

impl PresentResult {
    #[doc = include_str!("present_result.doc/is_presented.md")]
    #[must_use]
    pub const fn is_presented(&self) -> bool {
        matches!(self, Self::Presented { .. })
    }

    #[doc = include_str!("present_result.doc/is_recoverable.md")]
    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::Presented { .. } | Self::Skipped | Self::DeviceLost
        )
    }

    #[doc = include_str!("present_result.doc/failure_reason.md")]
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
