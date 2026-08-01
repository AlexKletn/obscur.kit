#![doc = include_str!("close_window.doc/_module.md")]

use obscur_identity::WindowId;

#[doc = include_str!("close_window.doc/CloseWindow.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CloseWindow {
    #[doc = include_str!("close_window.doc/target.md")]
    target: WindowId,
    #[doc = include_str!("close_window.doc/force.md")]
    force: bool,
}

impl CloseWindow {
    #[doc = include_str!("close_window.doc/request.md")]
    #[must_use]
    pub const fn request(target: WindowId) -> Self {
        Self {
            target,
            force: false,
        }
    }

    #[doc = include_str!("close_window.doc/force_2.md")]
    #[must_use]
    pub const fn force(target: WindowId) -> Self {
        Self {
            target,
            force: true,
        }
    }

    #[doc = include_str!("close_window.doc/target_2.md")]
    #[must_use]
    pub const fn target(&self) -> WindowId {
        self.target
    }

    #[doc = include_str!("close_window.doc/is_forced.md")]
    #[must_use]
    pub const fn is_forced(&self) -> bool {
        self.force
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn win() -> WindowId {
        WindowId::from_raw(1)
    }

    #[test]
    fn request_is_graceful() {
        let req = CloseWindow::request(win());
        assert_eq!(req.target(), win());
        assert!(!req.is_forced());
    }

    #[test]
    fn force_bypasses_confirmation() {
        let req = CloseWindow::force(win());
        assert_eq!(req.target(), win());
        assert!(req.is_forced());
    }

    #[test]
    fn graceful_and_forced_differ() {
        assert_ne!(CloseWindow::request(win()), CloseWindow::force(win()));
    }
}
