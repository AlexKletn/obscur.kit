#![doc = include_str!("pending_work.doc/_module.md")]

use crate::runtime::InputDispatch;

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("pending_work.doc/has_pending_frame.md")]
    #[must_use]
    pub fn has_pending_frame(&self) -> bool {
        self.scheduler.has_pending()
    }

    #[doc = include_str!("pending_work.doc/pending_frames.md")]
    #[must_use]
    pub fn pending_frames(&self) -> usize {
        self.scheduler.pending_count()
    }

    #[doc = include_str!("pending_work.doc/pending_input.md")]
    #[must_use]
    pub fn pending_input(&self) -> usize {
        self.input.pending()
    }

    #[doc = include_str!("pending_work.doc/take_input_dispatches.md")]
    #[must_use]
    pub fn take_input_dispatches(&mut self) -> Vec<InputDispatch> {
        self.input.take_dispatches()
    }

    #[doc = include_str!("pending_work.doc/pending_background.md")]
    #[must_use]
    pub const fn pending_background(&self) -> usize {
        self.background.pending()
    }

    #[doc = include_str!("pending_work.doc/serviced_background.md")]
    #[must_use]
    pub const fn serviced_background(&self) -> u64 {
        self.background.serviced()
    }

    #[doc = include_str!("pending_work.doc/is_background_starved.md")]
    #[must_use]
    pub const fn is_background_starved(&self) -> bool {
        self.scheduler.background_starved()
    }
}
