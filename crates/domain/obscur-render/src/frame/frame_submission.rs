//! How a rendered frame is submitted for presentation (spec §5.2).

use obscur_scene::scene::SceneGeneration;

use crate::frame::RenderFrame;

/// How a submitted frame is scheduled onto the display (spec §5.2).
///
/// This mirrors the presentation modes a swapchain exposes: whether presentation
/// blocks on the vertical blank (vsync) and how intermediate frames are queued.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PresentMode {
    /// Present on the vertical blank, blocking until the previous frame is
    /// consumed (vsync). The default.
    #[default]
    Fifo,
    /// Present immediately without waiting for vsync, allowing tearing.
    Immediate,
    /// Triple-buffered: present on the vertical blank but replace any queued
    /// frame rather than block.
    Mailbox,
}

impl PresentMode {
    /// Whether this mode waits for the vertical blank before presenting.
    ///
    /// True for [`PresentMode::Fifo`] and [`PresentMode::Mailbox`]; false for
    /// [`PresentMode::Immediate`], which tears.
    #[must_use]
    pub const fn waits_for_vsync(&self) -> bool {
        matches!(self, Self::Fifo | Self::Mailbox)
    }
}

/// A [`RenderFrame`] together with the [`PresentMode`] it should be presented
/// with (spec §5.2).
///
/// This is the unit handed to the renderer for presentation: the frame to draw
/// and the scheduling policy for putting it on screen.
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
