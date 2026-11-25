use winit::dpi::{PhysicalPosition, PhysicalSize};

#[doc = include_str!("../../docs/api/conversion/display_converter.doc/MonitorSnapshot.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct MonitorSnapshot {
    /// The human-readable monitor name, or `None` if winit could not read one.
    pub name: Option<String>,
    /// The monitor's resolution in physical pixels.
    pub size: PhysicalSize<u32>,
    /// The monitor's top-left corner in the global physical coordinate space.
    pub position: PhysicalPosition<i32>,
    /// The monitor's scale factor mapping logical to physical pixels.
    pub scale_factor: f64,
    /// The refresh rate in millihertz, or `None` if winit could not read one.
    pub refresh_rate_millihertz: Option<u32>,
}

impl MonitorSnapshot {
    /// Reads `monitor`'s current state.
    #[must_use]
    pub fn read(monitor: &winit::monitor::MonitorHandle) -> Self {
        Self {
            name: monitor.name(),
            size: monitor.size(),
            position: monitor.position(),
            scale_factor: monitor.scale_factor(),
            refresh_rate_millihertz: monitor.refresh_rate_millihertz(),
        }
    }
}
