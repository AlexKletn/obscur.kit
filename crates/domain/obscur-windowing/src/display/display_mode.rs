//! A display's video mode: resolution and refresh rate (spec §6.6).

use obscur_geometry::Size;

/// A single video mode a display can present: a physical resolution paired with
/// an optional refresh rate (spec §6.6).
///
/// The refresh rate is stored in millihertz as an integer so the value stays
/// exact and deterministic across platforms: fractional rates such as 59.94Hz
/// are represented precisely as `59_940` mHz rather than as a lossy `f32`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DisplayMode {
    /// The mode's resolution in physical pixels.
    pub resolution: Size,
    /// The refresh rate in millihertz, or `None` if unknown.
    pub refresh_millihertz: Option<u32>,
}

impl DisplayMode {
    /// Creates a mode with the given physical resolution and an unknown refresh
    /// rate.
    #[must_use]
    pub const fn new(resolution: Size) -> Self {
        Self {
            resolution,
            refresh_millihertz: None,
        }
    }

    /// Sets the refresh rate in millihertz, returning the updated mode.
    #[must_use]
    pub const fn with_refresh_millihertz(mut self, refresh_millihertz: u32) -> Self {
        self.refresh_millihertz = Some(refresh_millihertz);
        self
    }

    /// Returns the mode's resolution in physical pixels.
    #[must_use]
    pub const fn resolution(&self) -> Size {
        self.resolution
    }

    /// Returns the refresh rate in millihertz, or `None` if unknown.
    #[must_use]
    pub const fn refresh_millihertz(&self) -> Option<u32> {
        self.refresh_millihertz
    }

    /// Returns the refresh rate in hertz, or `None` if unknown.
    ///
    /// This is a convenience conversion of [`Self::refresh_millihertz`] and may
    /// lose precision; the millihertz value remains the exact source of truth.
    #[must_use]
    pub fn refresh_hz(&self) -> Option<f32> {
        self.refresh_millihertz.map(|mhz| mhz as f32 / 1000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_has_unknown_refresh() {
        let mode = DisplayMode::new(Size::new(1920.0, 1080.0));
        assert_eq!(mode.resolution(), Size::new(1920.0, 1080.0));
        assert_eq!(mode.refresh_millihertz(), None);
        assert_eq!(mode.refresh_hz(), None);
    }

    #[test]
    fn millihertz_to_hz_conversion_is_exact() {
        let mode = DisplayMode::new(Size::new(1920.0, 1080.0)).with_refresh_millihertz(59_940);
        assert_eq!(mode.refresh_millihertz(), Some(59_940));
        assert_eq!(mode.refresh_hz(), Some(59.94));
    }

    #[test]
    fn whole_hz_conversion() {
        let mode = DisplayMode::new(Size::new(2560.0, 1440.0)).with_refresh_millihertz(144_000);
        assert_eq!(mode.refresh_hz(), Some(144.0));
    }
}
