#![doc = include_str!("scheduler_priority.doc/_module.md")]

use crate::frame::FrameReason;

#[doc = include_str!("scheduler_priority.doc/SchedulerPriority.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SchedulerPriority {
    #[doc = include_str!("scheduler_priority.doc/Input.md")]
    Input,
    #[doc = include_str!("scheduler_priority.doc/Commit.md")]
    Commit,
    #[doc = include_str!("scheduler_priority.doc/CurrentFrame.md")]
    CurrentFrame,
    #[doc = include_str!("scheduler_priority.doc/ResourceCompletion.md")]
    ResourceCompletion,
    #[doc = include_str!("scheduler_priority.doc/Background.md")]
    Background,
    #[doc = include_str!("scheduler_priority.doc/Diagnostics.md")]
    Diagnostics,
}

impl SchedulerPriority {
    #[doc = include_str!("scheduler_priority.doc/from_reason.md")]
    #[must_use]
    pub const fn from_reason(reason: FrameReason) -> Self {
        match reason {
            // A text edit is user input: same most-urgent band as `Input`.
            FrameReason::Input | FrameReason::TextEdit => Self::Input,
            FrameReason::UiCommit => Self::Commit,
            // Animation, a window/surface change, and an external render request
            // all drive layout and paint for the current frame.
            FrameReason::Animation | FrameReason::WindowChange | FrameReason::ExternalRender => {
                Self::CurrentFrame
            }
            FrameReason::ResourceCompletion => Self::ResourceCompletion,
            FrameReason::DiagnosticsCapture => Self::Diagnostics,
        }
    }

    #[doc = include_str!("scheduler_priority.doc/is_more_urgent_than.md")]
    #[must_use]
    pub const fn is_more_urgent_than(self, other: Self) -> bool {
        (self as u8) < (other as u8)
    }

    #[doc = include_str!("scheduler_priority.doc/rank.md")]
    #[must_use]
    pub const fn rank(self) -> u8 {
        self as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_is_most_urgent_and_diagnostics_least() {
        assert_eq!(SchedulerPriority::Input.rank(), 0);
        assert_eq!(SchedulerPriority::Diagnostics.rank(), 5);
        assert!(SchedulerPriority::Input < SchedulerPriority::Diagnostics);
    }

    #[test]
    fn ordering_is_highest_urgency_first() {
        let mut bands = [
            SchedulerPriority::Diagnostics,
            SchedulerPriority::Background,
            SchedulerPriority::CurrentFrame,
            SchedulerPriority::Input,
            SchedulerPriority::ResourceCompletion,
            SchedulerPriority::Commit,
        ];
        bands.sort();
        assert_eq!(
            bands,
            [
                SchedulerPriority::Input,
                SchedulerPriority::Commit,
                SchedulerPriority::CurrentFrame,
                SchedulerPriority::ResourceCompletion,
                SchedulerPriority::Background,
                SchedulerPriority::Diagnostics,
            ]
        );
    }

    #[test]
    fn from_reason_maps_every_reason() {
        assert_eq!(
            SchedulerPriority::from_reason(FrameReason::Input),
            SchedulerPriority::Input
        );
        assert_eq!(
            SchedulerPriority::from_reason(FrameReason::TextEdit),
            SchedulerPriority::Input
        );
        assert_eq!(
            SchedulerPriority::from_reason(FrameReason::UiCommit),
            SchedulerPriority::Commit
        );
        assert_eq!(
            SchedulerPriority::from_reason(FrameReason::Animation),
            SchedulerPriority::CurrentFrame
        );
        assert_eq!(
            SchedulerPriority::from_reason(FrameReason::WindowChange),
            SchedulerPriority::CurrentFrame
        );
        assert_eq!(
            SchedulerPriority::from_reason(FrameReason::ExternalRender),
            SchedulerPriority::CurrentFrame
        );
        assert_eq!(
            SchedulerPriority::from_reason(FrameReason::ResourceCompletion),
            SchedulerPriority::ResourceCompletion
        );
        assert_eq!(
            SchedulerPriority::from_reason(FrameReason::DiagnosticsCapture),
            SchedulerPriority::Diagnostics
        );
    }

    #[test]
    fn from_reason_never_returns_background() {
        for reason in [
            FrameReason::Input,
            FrameReason::TextEdit,
            FrameReason::UiCommit,
            FrameReason::Animation,
            FrameReason::ResourceCompletion,
            FrameReason::WindowChange,
            FrameReason::ExternalRender,
            FrameReason::DiagnosticsCapture,
        ] {
            assert_ne!(
                SchedulerPriority::from_reason(reason),
                SchedulerPriority::Background
            );
        }
    }

    #[test]
    fn is_more_urgent_than() {
        assert!(SchedulerPriority::Input.is_more_urgent_than(SchedulerPriority::Commit));
        assert!(SchedulerPriority::Commit.is_more_urgent_than(SchedulerPriority::Diagnostics));
        assert!(!SchedulerPriority::Diagnostics.is_more_urgent_than(SchedulerPriority::Input));
        assert!(!SchedulerPriority::Input.is_more_urgent_than(SchedulerPriority::Input));
    }
}
