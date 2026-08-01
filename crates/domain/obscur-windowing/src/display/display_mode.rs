#![doc = include_str!("display_mode.doc/_module.md")]

use obscur_geometry::Size;

#[doc = include_str!("display_mode.doc/DisplayMode.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DisplayMode {
    #[doc = include_str!("display_mode.doc/resolution.md")]
    pub resolution: Size,
    #[doc = include_str!("display_mode.doc/refresh_millihertz.md")]
    pub refresh_millihertz: Option<u32>,
}

impl DisplayMode {
    #[doc = include_str!("display_mode.doc/new.md")]
    #[must_use]
    pub const fn new(resolution: Size) -> Self {
        Self {
            resolution,
            refresh_millihertz: None,
        }
    }

    #[doc = include_str!("display_mode.doc/with_refresh_millihertz.md")]
    #[must_use]
    pub const fn with_refresh_millihertz(mut self, refresh_millihertz: u32) -> Self {
        self.refresh_millihertz = Some(refresh_millihertz);
        self
    }

    #[doc = include_str!("display_mode.doc/resolution_2.md")]
    #[must_use]
    pub const fn resolution(&self) -> Size {
        self.resolution
    }

    #[doc = include_str!("display_mode.doc/refresh_millihertz_2.md")]
    #[must_use]
    pub const fn refresh_millihertz(&self) -> Option<u32> {
        self.refresh_millihertz
    }

    #[doc = include_str!("display_mode.doc/refresh_hz.md")]
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
