#[doc = include_str!("../../docs/api/frame/present_mode.doc/PresentMode.md")]
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
    #[doc = include_str!("../../docs/api/frame/frame_submission.doc/waits_for_vsync.md")]
    #[must_use]
    pub const fn waits_for_vsync(&self) -> bool {
        matches!(self, Self::Fifo | Self::Mailbox)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
