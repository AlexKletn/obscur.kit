//! Scheduler priority bands (spec §3.6): [`SchedulerPriority`].

use crate::frame::FrameReason;

/// The priority band a piece of frame work belongs to (spec §3.6).
///
/// The variants are declared in priority order, highest urgency first, so the
/// derived [`Ord`] makes the *most urgent* band the *smallest* value. A
/// min-first queue (see [`SchedulerQueue`](super::SchedulerQueue)) therefore
/// pops the most urgent work. In short: **lower [`Ord`] == higher urgency**.
///
/// The bands map onto the numbered priority list in spec §3.6:
///
/// 1. input / focus / pointer / IME → [`SchedulerPriority::Input`]
/// 2. UI transaction commit → [`SchedulerPriority::Commit`]
/// 3. current-frame animation, layout and paint → [`SchedulerPriority::CurrentFrame`]
/// 4. resource completion → [`SchedulerPriority::ResourceCompletion`]
/// 5. background shaping / decoding / cache → [`SchedulerPriority::Background`]
/// 6. telemetry and development diagnostics → [`SchedulerPriority::Diagnostics`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SchedulerPriority {
    /// §3.6 (1): input, focus, pointer and IME work — the most urgent band.
    Input,
    /// §3.6 (2): committing a UI transaction.
    Commit,
    /// §3.6 (3): the current frame's animation, layout and paint.
    CurrentFrame,
    /// §3.6 (4): completion of an asynchronous resource load.
    ResourceCompletion,
    /// §3.6 (5): background shaping, decoding and cache work.
    ///
    /// This band has no corresponding [`FrameReason`]: it covers subsystem work
    /// that is driven by budgeting and starvation prevention rather than by a
    /// frame request, so [`SchedulerPriority::from_reason`] never returns it.
    Background,
    /// §3.6 (6): telemetry and development diagnostics — the least urgent band.
    Diagnostics,
}

impl SchedulerPriority {
    /// Returns the priority band for a given [`FrameReason`].
    ///
    /// The mapping follows spec §3.6: input dispatch is the most urgent, UI
    /// commits next, then the current frame's work (animation samples plus the
    /// layout and paint driven by window changes and external render requests),
    /// then resource completion, and finally diagnostics captures.
    ///
    /// Note that [`SchedulerPriority::Background`] is never returned: no
    /// [`FrameReason`] maps to it (see its variant documentation).
    #[must_use]
    pub const fn from_reason(reason: FrameReason) -> Self {
        match reason {
            FrameReason::Input => Self::Input,
            FrameReason::UiCommit => Self::Commit,
            // Animation, a window/surface change, and an external render request
            // all drive layout and paint for the current frame.
            FrameReason::Animation
            | FrameReason::WindowChange
            | FrameReason::ExternalRender => Self::CurrentFrame,
            FrameReason::ResourceCompletion => Self::ResourceCompletion,
            FrameReason::DiagnosticsCapture => Self::Diagnostics,
        }
    }

    /// Returns whether `self` is more urgent than `other`.
    ///
    /// Because lower [`Ord`] means higher urgency, this is simply `self < other`.
    #[must_use]
    pub const fn is_more_urgent_than(self, other: Self) -> bool {
        (self as u8) < (other as u8)
    }

    /// Returns the urgency rank of this band, where `0` is the most urgent.
    ///
    /// The rank matches the declaration order, so
    /// [`SchedulerPriority::Input`] is `0` and
    /// [`SchedulerPriority::Diagnostics`] is `5`.
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
