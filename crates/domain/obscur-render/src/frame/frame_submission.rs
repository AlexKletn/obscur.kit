#![doc = include_str!("frame_submission.doc/_module.md")]

use obscur_scene::scene::SceneGeneration;

use crate::frame::RenderFrame;

#[doc = include_str!("frame_submission.doc/PresentMode.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PresentMode {
    #[doc = include_str!("frame_submission.doc/Fifo.md")]
    #[default]
    Fifo,
    #[doc = include_str!("frame_submission.doc/Immediate.md")]
    Immediate,
    #[doc = include_str!("frame_submission.doc/Mailbox.md")]
    Mailbox,
}

impl PresentMode {
    #[doc = include_str!("frame_submission.doc/waits_for_vsync.md")]
    #[must_use]
    pub const fn waits_for_vsync(&self) -> bool {
        matches!(self, Self::Fifo | Self::Mailbox)
    }
}

#[doc = include_str!("frame_submission.doc/FrameSubmission.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct FrameSubmission {
    frame: RenderFrame,
    present_mode: PresentMode,
}

impl FrameSubmission {
    #[doc = include_str!("frame_submission.doc/new.md")]
    #[must_use]
    pub fn new(frame: RenderFrame) -> Self {
        Self {
            frame,
            present_mode: PresentMode::Fifo,
        }
    }

    #[doc = include_str!("frame_submission.doc/with_present_mode.md")]
    #[must_use]
    pub fn with_present_mode(mut self, present_mode: PresentMode) -> Self {
        self.present_mode = present_mode;
        self
    }

    #[doc = include_str!("frame_submission.doc/frame.md")]
    #[must_use]
    pub const fn frame(&self) -> &RenderFrame {
        &self.frame
    }

    #[doc = include_str!("frame_submission.doc/present_mode.md")]
    #[must_use]
    pub const fn present_mode(&self) -> PresentMode {
        self.present_mode
    }

    #[doc = include_str!("frame_submission.doc/generation.md")]
    #[must_use]
    pub fn generation(&self) -> SceneGeneration {
        self.frame.generation()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::Size;
    use obscur_scene::command::DisplayList;
    use obscur_scene::scene::SceneSnapshot;

    fn a_frame() -> RenderFrame {
        RenderFrame::new(
            SceneSnapshot::new(DisplayList::new(), SceneGeneration::from_raw(3)),
            Size::new(320.0, 240.0),
        )
    }

    #[test]
    fn present_mode_waits_for_vsync() {
        assert!(PresentMode::Fifo.waits_for_vsync());
        assert!(PresentMode::Mailbox.waits_for_vsync());
        assert!(!PresentMode::Immediate.waits_for_vsync());
    }

    #[test]
    fn present_mode_default_is_fifo() {
        assert_eq!(PresentMode::default(), PresentMode::Fifo);
    }

    #[test]
    fn new_defaults_to_fifo() {
        let submission = FrameSubmission::new(a_frame());
        assert_eq!(submission.present_mode(), PresentMode::Fifo);
        assert_eq!(submission.generation(), SceneGeneration::from_raw(3));
    }

    #[test]
    fn with_present_mode_overrides() {
        let submission = FrameSubmission::new(a_frame()).with_present_mode(PresentMode::Immediate);
        assert_eq!(submission.present_mode(), PresentMode::Immediate);
        assert!(!submission.present_mode().waits_for_vsync());
        assert_eq!(submission.frame().viewport(), Size::new(320.0, 240.0));
    }
}
