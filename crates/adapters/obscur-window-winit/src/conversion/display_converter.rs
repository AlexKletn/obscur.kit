//! The [`DisplayConverter`] pure winit→domain monitor converter (spec §6.6
//! display enumeration; §2.7 "преобразует native events в доменные события").

use obscur_geometry::{Point, Rect, ScaleFactor, Size};
use obscur_windowing::display::{Display, DisplayId, DisplayMode};
use winit::dpi::{PhysicalPosition, PhysicalSize};

/// One winit monitor's readings, taken at a single instant (spec §6.6).
///
/// A [`MonitorHandle`](winit::monitor::MonitorHandle) cannot be constructed
/// without a live event loop and a real platform connection, so a projection
/// written directly against it is reachable only from a running winit
/// application. This snapshot is the seam that splits that problem in two: it
/// carries exactly the readings the projection consumes, in winit's own `dpi`
/// types, so [`DisplayConverter::project`] — the half with the decisions in it —
/// is an ordinary pure function, and [`MonitorSnapshot::read`] is left holding
/// nothing but five accessor calls.
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

/// A stateless converter from a winit monitor to the domain's [`Display`]
/// (spec §6.6, §2.7).
///
/// This is a **pure** projection of a monitor snapshot: the caller supplies the
/// stable [`DisplayId`] (the adapter mints and retains it), and the converter
/// reads the monitor's current size, refresh rate, scale factor, name, and
/// position into the domain model.
///
/// Global-coordinate honesty (spec §6.6: "On Wayland, *unavailable* global
/// coordinates are represented honestly"): the condition is unavailability, not a
/// platform. Wayland compositors do publish each output's logical position, over
/// **xdg-output**, and winit reports it — so a Wayland session is not by itself a
/// reason to blank anything. What cannot be read is a compositor that withholds by
/// answering `(0, 0)` for every output: that is an absence, not an origin, and
/// publishing it would describe a topology in which every monitor overlaps. The
/// converter cannot detect that itself, so
/// the caller chooses the constructor: [`DisplayConverter::new`] projects the
/// position into bounds, and [`DisplayConverter::without_global_coordinates`]
/// leaves both bounds `None`, which is what
/// [`Display::has_global_coordinates`] is there to report.
#[derive(Debug, Clone, Copy)]
pub struct DisplayConverter {
    /// Whether the platform underneath exposes where each monitor sits in a
    /// global coordinate space.
    global_coordinates: bool,
}

impl DisplayConverter {
    /// Creates a converter for a platform that exposes global monitor
    /// coordinates (spec §6.6).
    #[must_use]
    pub const fn new() -> Self {
        Self {
            global_coordinates: true,
        }
    }

    /// Creates a converter for a platform that does not expose global monitor
    /// coordinates — Wayland (spec §6.6).
    ///
    /// The projected displays carry no `logical_bounds` and no
    /// `physical_bounds`: winit's `(0, 0)` position is unavailability, and the
    /// domain models unavailable geometry as `None`.
    #[must_use]
    pub const fn without_global_coordinates() -> Self {
        Self {
            global_coordinates: false,
        }
    }

    /// Converts a live winit `monitor` to a domain [`Display`] with the
    /// caller-supplied `id` (spec §6.6).
    #[must_use]
    pub fn convert(&self, id: DisplayId, monitor: &winit::monitor::MonitorHandle) -> Display {
        self.project(id, &MonitorSnapshot::read(monitor))
    }

    /// Projects a monitor `snapshot` onto a domain [`Display`] with the
    /// caller-supplied `id` (spec §6.6).
    ///
    /// The size and position casts to `f32` are lossless enough at UI scale:
    /// monitor pixel dimensions and coordinates stay within the precise integer
    /// range of `f32`. The refresh rate is preserved exactly in millihertz.
    ///
    /// [`Display::available_modes`], [`Display::work_area`], and
    /// [`Display::color`] are left at their empty defaults: winit 0.30 exposes
    /// no reserved-region and no color/HDR query at all, and its
    /// `video_modes()` enumerates *exclusive-fullscreen* modes, which this
    /// backend refuses to present (`supports(ExclusiveFullscreen)` is
    /// `Limited`, degraded to borderless), so nothing downstream could act on
    /// the list.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn project(&self, id: DisplayId, snapshot: &MonitorSnapshot) -> Display {
        let resolution = Size::new(snapshot.size.width as f32, snapshot.size.height as f32);

        let mut mode = DisplayMode::new(resolution);
        if let Some(refresh) = snapshot.refresh_rate_millihertz {
            mode = mode.with_refresh_millihertz(refresh);
        }

        let scale_factor = ScaleFactor::new(snapshot.scale_factor as f32);
        let mut display = Display::new(id, mode).with_scale_factor(scale_factor);

        if self.global_coordinates {
            let physical = Rect::new(
                Point::new(snapshot.position.x as f32, snapshot.position.y as f32),
                resolution,
            );
            display = display
                .with_physical_bounds(physical)
                .with_logical_bounds(to_logical(physical, scale_factor));
        }

        if let Some(name) = &snapshot.name {
            display = display.with_name(name.as_str());
        }

        display
    }
}

impl Default for DisplayConverter {
    fn default() -> Self {
        Self::new()
    }
}

/// Divides physical `bounds` by `scale_factor` into the logical space
/// [`Display::logical_bounds`] is documented in.
///
/// winit reports both a monitor's size and its position in physical pixels, so
/// the two bounds differ by exactly the scale factor. Publishing the physical
/// rectangle as the logical one would place a 2× monitor's logical origin at
/// twice its true logical coordinate.
fn to_logical(bounds: Rect, scale_factor: ScaleFactor) -> Rect {
    let scale = scale_factor.get();
    Rect::new(
        Point::new(bounds.origin.x / scale, bounds.origin.y / scale),
        Size::new(bounds.size.width / scale, bounds.size.height / scale),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 4K panel at 2×, offset to the right of a 1080p primary, at 59.94Hz —
    /// every reading distinct so no field can be satisfied by another's value.
    fn snapshot() -> MonitorSnapshot {
        MonitorSnapshot {
            name: Some("DELL U2720Q".to_owned()),
            size: PhysicalSize::new(3840, 2160),
            position: PhysicalPosition::new(1920, 120),
            scale_factor: 2.0,
            refresh_rate_millihertz: Some(59_940),
        }
    }

    #[test]
    fn every_reading_reaches_the_domain_display() {
        let display = DisplayConverter::new().project(DisplayId::from_raw(7), &snapshot());

        assert_eq!(display.id(), DisplayId::from_raw(7));
        assert_eq!(display.name(), Some("DELL U2720Q"));
        assert_eq!(display.scale_factor(), ScaleFactor::new(2.0));
        // The mode's resolution is physical, and the refresh rate stays exact.
        assert_eq!(
            display.current_mode(),
            DisplayMode::new(Size::new(3840.0, 2160.0)).with_refresh_millihertz(59_940)
        );
        assert_eq!(display.current_mode().refresh_hz(), Some(59.94));
        // A converted monitor is not primary until the enumerator says so.
        assert!(!display.is_primary());
    }

    /// §6.6: physical bounds are winit's own readings; logical bounds are those
    /// divided by the scale factor. Reporting the physical rectangle as the
    /// logical one puts a 2× monitor at twice its true logical origin.
    #[test]
    fn logical_bounds_are_the_physical_bounds_divided_by_the_scale_factor() {
        let display = DisplayConverter::new().project(DisplayId::from_raw(0), &snapshot());

        assert_eq!(
            display.physical_bounds(),
            Some(Rect::new(
                Point::new(1920.0, 120.0),
                Size::new(3840.0, 2160.0)
            ))
        );
        assert_eq!(
            display.logical_bounds(),
            Some(Rect::new(
                Point::new(960.0, 60.0),
                Size::new(1920.0, 1080.0)
            ))
        );
        assert!(display.has_global_coordinates());
    }

    /// At 1× the two spaces coincide — the division must not perturb them.
    #[test]
    fn an_unscaled_monitor_has_identical_logical_and_physical_bounds() {
        let unscaled = MonitorSnapshot {
            scale_factor: 1.0,
            ..snapshot()
        };
        let display = DisplayConverter::new().project(DisplayId::from_raw(0), &unscaled);
        assert_eq!(display.logical_bounds(), display.physical_bounds());
    }

    /// §6.6: "On Wayland, *unavailable* global coordinates are represented
    /// honestly." When a compositor withholds its topology — every output
    /// answering the origin — projecting that would claim every monitor sits at
    /// the same place, so both bounds must be absent instead, and everything winit
    /// *does* know must survive. Whether that has happened is the caller's to
    /// decide (`positions_carry_a_topology`); this only pins what the projection
    /// does once told.
    #[test]
    fn without_global_coordinates_the_bounds_are_absent_not_fabricated() {
        let wayland = MonitorSnapshot {
            name: Some("eDP-1".to_owned()),
            position: PhysicalPosition::new(0, 0),
            ..snapshot()
        };
        let display = DisplayConverter::without_global_coordinates()
            .project(DisplayId::from_raw(3), &wayland);

        assert_eq!(display.logical_bounds(), None);
        assert_eq!(display.physical_bounds(), None);
        assert!(!display.has_global_coordinates());

        // Absence is confined to the coordinates: the panel is still described.
        assert_eq!(display.name(), Some("eDP-1"));
        assert_eq!(display.scale_factor(), ScaleFactor::new(2.0));
        assert_eq!(
            display.current_mode().resolution(),
            Size::new(3840.0, 2160.0)
        );
    }

    /// The same monitor must not gain bounds merely because it sits at the
    /// origin on a platform that *does* publish coordinates: a real `(0, 0)`
    /// origin and an unavailable one differ only by the caller's constructor.
    #[test]
    fn a_real_origin_is_kept_where_coordinates_are_available() {
        let at_origin = MonitorSnapshot {
            position: PhysicalPosition::new(0, 0),
            scale_factor: 1.0,
            ..snapshot()
        };
        let display = DisplayConverter::new().project(DisplayId::from_raw(0), &at_origin);
        assert_eq!(
            display.logical_bounds(),
            Some(Rect::new(Point::ORIGIN, Size::new(3840.0, 2160.0)))
        );
        assert!(display.has_global_coordinates());
    }

    /// Unknowable readings stay unknown rather than defaulting to a plausible
    /// number.
    #[test]
    fn unknown_readings_are_not_invented() {
        let unknown = MonitorSnapshot {
            name: None,
            refresh_rate_millihertz: None,
            ..snapshot()
        };
        let display = DisplayConverter::new().project(DisplayId::from_raw(0), &unknown);
        assert_eq!(display.name(), None);
        assert_eq!(display.current_mode().refresh_millihertz(), None);
        // The resolution is still known, so the mode is still populated.
        assert_eq!(
            display.current_mode().resolution(),
            Size::new(3840.0, 2160.0)
        );
    }

    /// winit's `video_modes()` lists exclusive-fullscreen modes this backend
    /// refuses to present, and winit 0.30 has no reserved-region or color query
    /// at all, so these stay empty — asserted so the day one is wired the test
    /// says so rather than silently agreeing.
    #[test]
    fn modes_work_area_and_color_are_left_at_their_documented_defaults() {
        let display = DisplayConverter::new().project(DisplayId::from_raw(0), &snapshot());
        assert!(display.available_modes().is_empty());
        assert_eq!(display.work_area(), None);
        assert!(!display.color().is_hdr());
    }

    /// A negative or zero scale factor cannot come from a working platform, but
    /// it must not divide the bounds into infinity if one ever does.
    #[test]
    fn a_nonsense_scale_factor_falls_back_to_identity() {
        let broken = MonitorSnapshot {
            scale_factor: 0.0,
            ..snapshot()
        };
        let display = DisplayConverter::new().project(DisplayId::from_raw(0), &broken);
        assert_eq!(display.scale_factor(), ScaleFactor::IDENTITY);
        assert_eq!(display.logical_bounds(), display.physical_bounds());
    }
}
