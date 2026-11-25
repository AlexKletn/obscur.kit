//! Window placement state for application-owned persistence.

use serde::{Deserialize, Serialize};

use super::WindowMode;

/// A versioned snapshot of a window’s normal logical size and presentation mode.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WindowPlacement {
    version: u32,
    normal_size: Option<(f32, f32)>,
    mode: WindowMode,
}

impl WindowPlacement {
    const VERSION: u32 = 1;

    #[cfg(any(test, all(feature = "render-vello", feature = "window-winit")))]
    pub(super) fn observe(previous: Option<Self>, window: super::WindowState) -> Option<Self> {
        if window.minimized == Some(true) {
            return previous;
        }
        let mode = if window.fullscreen {
            WindowMode::Fullscreen
        } else if window.maximized {
            WindowMode::Maximized
        } else {
            WindowMode::Normal
        };
        let mut normal_size = previous.and_then(|placement| placement.normal_size);
        let (width, height) = (window.size.width, window.size.height);
        if mode == WindowMode::Normal && valid_size(width, height) {
            normal_size = Some((width, height));
        }
        Some(Self {
            version: Self::VERSION,
            normal_size,
            mode,
        })
    }

    /// The saved presentation mode.
    #[must_use]
    pub const fn mode(self) -> WindowMode {
        self.mode
    }

    /// The last observed normal size in logical pixels, when available.
    #[must_use]
    pub const fn normal_size(self) -> Option<(f32, f32)> {
        self.normal_size
    }

    /// Checks the format version and validates the saved size.
    #[must_use]
    pub fn is_supported(self) -> bool {
        self.version == Self::VERSION && self.normal_size.is_none_or(|(w, h)| valid_size(w, h))
    }
}

fn valid_size(width: f32, height: f32) -> bool {
    width.is_finite() && width > 0.0 && height.is_finite() && height > 0.0
}
