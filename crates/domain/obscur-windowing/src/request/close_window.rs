//! Intent to close a window (spec §6.1).

use obscur_identity::WindowId;

/// An application's intent to close a window (spec §6.1).
///
/// A graceful [`CloseWindow::request`] mirrors a
/// [`WindowEvent::CloseRequested`](crate::window::WindowEvent::CloseRequested):
/// it asks the window to close and lets any application-level close
/// confirmation run. A [`CloseWindow::force`] request bypasses that
/// confirmation and closes the window unconditionally.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CloseWindow {
    /// The window to close.
    target: WindowId,
    /// Whether to bypass the application's close-requested confirmation.
    force: bool,
}

impl CloseWindow {
    /// Requests a graceful close of `target`, honoring any close confirmation.
    #[must_use]
    pub const fn request(target: WindowId) -> Self {
        Self {
            target,
            force: false,
        }
    }

    /// Requests a forced close of `target`, bypassing any close confirmation.
    #[must_use]
    pub const fn force(target: WindowId) -> Self {
        Self {
            target,
            force: true,
        }
    }

    /// Returns the window this request targets.
    #[must_use]
    pub const fn target(&self) -> WindowId {
        self.target
    }

    /// Returns whether this request bypasses the close-requested confirmation.
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
