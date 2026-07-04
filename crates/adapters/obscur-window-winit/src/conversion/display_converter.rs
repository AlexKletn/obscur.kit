//! The [`DisplayConverter`] pure winit→domain monitor converter (spec §6.6
//! display enumeration; §2.7 "преобразует native events в доменные события").

use obscur_geometry::{Point, Rect, ScaleFactor, Size};
use obscur_windowing::display::{Display, DisplayId, DisplayMode};

/// A stateless converter from a winit [`MonitorHandle`](winit::monitor::MonitorHandle)
/// to the domain's [`Display`] (spec §6.6, §2.7).
///
/// This is a **pure** projection of a monitor snapshot: the caller supplies the
/// stable [`DisplayId`] (the adapter mints and retains it), and the converter
/// reads the monitor's current size, refresh rate, scale factor, name, and
/// position into the domain model.
///
/// Global-coordinate honesty (spec §6.6): platforms such as Wayland do not
/// expose where a monitor sits in a global logical space, and winit then reports
/// a `(0, 0)` position for every monitor. This converter still records that
/// position as the display's bounds — it cannot distinguish an honest origin
/// from an unavailable one at this layer — so callers that need the Wayland
/// distinction consult the backend's
/// [`WindowFeature::GlobalDisplayCoordinates`](obscur_windowing::port::WindowFeature)
/// capability rather than trusting a `(0, 0)` origin.
#[derive(Debug, Clone, Copy, Default)]
pub struct DisplayConverter;

impl DisplayConverter {
    /// Creates a converter.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Converts a winit `monitor` snapshot to a domain [`Display`] with the
    /// caller-supplied `id` (spec §6.6).
    ///
    /// The size and position casts to `f32` are lossless enough at UI scale:
    /// monitor pixel dimensions and coordinates stay within the precise integer
    /// range of `f32`. The refresh rate is preserved exactly in millihertz.
    #[must_use]
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    pub fn convert(&self, id: DisplayId, monitor: &winit::monitor::MonitorHandle) -> Display {
        let size = monitor.size();
        let resolution = Size::new(size.width as f32, size.height as f32);

        let mut mode = DisplayMode::new(resolution);
        if let Some(refresh) = monitor.refresh_rate_millihertz() {
            mode = mode.with_refresh_millihertz(refresh);
        }

        let position = monitor.position();
        let bounds = Rect::new(Point::new(position.x as f32, position.y as f32), resolution);

        let mut display = Display::new(id, mode)
            .with_scale_factor(ScaleFactor::new(monitor.scale_factor() as f32))
            .with_logical_bounds(bounds)
            .with_physical_bounds(bounds);

        if let Some(name) = monitor.name() {
            display = display.with_name(name);
        }

        display
    }
}

// No `#[cfg(test)]` unit tests: `winit::monitor::MonitorHandle` cannot be
// constructed without a live event loop and a real platform connection, so it is
// not available in a headless `cargo test` run. The conversion logic is pure and
// exercised end-to-end by the backend integration path (a separate sub-step).
