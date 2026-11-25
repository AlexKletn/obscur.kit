//! How a rendered frame is submitted for presentation (spec §5.2).

use obscur_scene::scene::SceneGeneration;

use crate::frame::RenderFrame;

use super::PresentMode;

#[doc = include_str!("../../docs/api/frame/frame_submission.doc/FrameSubmission.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct FrameSubmission {
    frame: RenderFrame,
    present_mode: PresentMode,
}

impl FrameSubmission {
    /// Submits `frame` for presentation using the default
    /// [`PresentMode::Fifo`] (vsync).
    #[must_use]
    pub fn new(frame: RenderFrame) -> Self {
        Self {
            frame,
            present_mode: PresentMode::Fifo,
        }
    }

    /// Sets the presentation mode, returning the updated submission.
    #[must_use]
    pub fn with_present_mode(mut self, present_mode: PresentMode) -> Self {
        self.present_mode = present_mode;
        self
    }

    /// The frame to be presented.
    #[must_use]
    pub const fn frame(&self) -> &RenderFrame {
        &self.frame
    }

    /// The presentation mode this frame is submitted with.
    #[must_use]
    pub const fn present_mode(&self) -> PresentMode {
        self.present_mode
    }

    /// The generation of the scene carried by this submission's frame.
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
