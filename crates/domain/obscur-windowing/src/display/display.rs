//! A single monitor and its geometry, modes, and color capability (spec §6.6).

use obscur_geometry::{Rect, ScaleFactor};

use crate::display::{DisplayColorCapability, DisplayMode};

/// Stable, opaque identifier for a display.
///
/// `DisplayId` is a tightly-coupled part of [`Display`]: it identifies one
/// monitor and has no meaning apart from the display model, so it is colocated
/// here rather than given its own module. Backed by an integer handle; the
/// numeric value carries no meaning outside the subsystem that mints it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DisplayId(u64);

impl DisplayId {
    /// Wraps a raw handle value into a `DisplayId`.
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the raw handle value backing this identifier.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// A single monitor: its identity, geometry, video modes, and color capability
/// (spec §6.6).
///
/// Geometry fields are `Option` so absence can be modeled honestly. In
/// particular, [`Self::logical_bounds`] is `None` on platforms that do not
/// expose global screen coordinates — a Wayland compositor, for example, never
/// reveals where a monitor sits in a global logical space. `None` therefore
/// means "honestly unavailable", and must never be conflated with a fabricated
/// `(0, 0)` origin.
#[derive(Debug, Clone, PartialEq)]
pub struct Display {
    /// The stable identifier of this display.
    pub id: DisplayId,
    /// The human-readable monitor name, or `None` if unknown.
    pub name: Option<Box<str>>,
    /// The display's position and size in the global logical coordinate space,
    /// or `None` when global coordinates are unavailable.
    ///
    /// `None` is the honest representation on platforms such as Wayland that do
    /// not expose global screen coordinates; it does not mean the display sits
    /// at the origin.
    pub logical_bounds: Option<Rect>,
    /// The display's bounds in physical pixels, or `None` where unavailable.
    pub physical_bounds: Option<Rect>,
    /// The safe/usable area excluding panels and other reserved regions, or
    /// `None` if unknown.
    pub work_area: Option<Rect>,
    /// The display's scale factor mapping logical to physical pixels.
    pub scale_factor: ScaleFactor,
    /// The currently active video mode.
    pub current_mode: DisplayMode,
    /// The video modes this display can present.
    pub available_modes: Vec<DisplayMode>,
    /// The display's color and HDR capability.
    pub color: DisplayColorCapability,
    /// Whether this is the primary display.
    pub is_primary: bool,
}

impl Display {
    /// Creates a display with the given identity and current mode, defaulting
    /// everything else: no name, all bounds `None` (honest absence), an identity
    /// scale factor, no additional modes, the baseline color capability, and not
    /// primary.
    #[must_use]
    pub fn new(id: DisplayId, current_mode: DisplayMode) -> Self {
        Self {
            id,
            name: None,
            logical_bounds: None,
            physical_bounds: None,
            work_area: None,
            scale_factor: ScaleFactor::IDENTITY,
            current_mode,
            available_modes: Vec::new(),
            color: DisplayColorCapability::new(),
            is_primary: false,
        }
    }

    /// Sets the human-readable monitor name, returning the updated display.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<Box<str>>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the global logical bounds, returning the updated display.
    #[must_use]
    pub fn with_logical_bounds(mut self, bounds: Rect) -> Self {
        self.logical_bounds = Some(bounds);
        self
    }

    /// Sets the physical-pixel bounds, returning the updated display.
    #[must_use]
    pub fn with_physical_bounds(mut self, bounds: Rect) -> Self {
        self.physical_bounds = Some(bounds);
        self
    }

    /// Sets the safe/usable work area, returning the updated display.
    #[must_use]
    pub fn with_work_area(mut self, work_area: Rect) -> Self {
        self.work_area = Some(work_area);
        self
    }

    /// Sets the scale factor, returning the updated display.
    #[must_use]
    pub fn with_scale_factor(mut self, scale_factor: ScaleFactor) -> Self {
        self.scale_factor = scale_factor;
        self
    }

    /// Sets the available video modes, returning the updated display.
    #[must_use]
    pub fn with_available_modes(mut self, available_modes: Vec<DisplayMode>) -> Self {
        self.available_modes = available_modes;
        self
    }

    /// Sets the color capability, returning the updated display.
    #[must_use]
    pub fn with_color(mut self, color: DisplayColorCapability) -> Self {
        self.color = color;
        self
    }

    /// Sets whether this display is primary, returning the updated display.
    #[must_use]
    pub fn primary(mut self, is_primary: bool) -> Self {
        self.is_primary = is_primary;
        self
    }

    /// Returns the display's stable identifier.
    #[must_use]
    pub const fn id(&self) -> DisplayId {
        self.id
    }

    /// Returns the human-readable monitor name, or `None` if unknown.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Returns the global logical bounds, or `None` when unavailable.
    #[must_use]
    pub const fn logical_bounds(&self) -> Option<Rect> {
        self.logical_bounds
    }

    /// Returns the physical-pixel bounds, or `None` where unavailable.
    #[must_use]
    pub const fn physical_bounds(&self) -> Option<Rect> {
        self.physical_bounds
    }

    /// Returns the safe/usable work area, or `None` if unknown.
    #[must_use]
    pub const fn work_area(&self) -> Option<Rect> {
        self.work_area
    }

    /// Returns the display's scale factor.
    #[must_use]
    pub const fn scale_factor(&self) -> ScaleFactor {
        self.scale_factor
    }

    /// Returns the currently active video mode.
    #[must_use]
    pub const fn current_mode(&self) -> DisplayMode {
        self.current_mode
    }

    /// Returns the video modes this display can present.
    #[must_use]
    pub fn available_modes(&self) -> &[DisplayMode] {
        &self.available_modes
    }

    /// Returns the display's color and HDR capability.
    #[must_use]
    pub const fn color(&self) -> DisplayColorCapability {
        self.color
    }

    /// Returns whether this is the primary display.
    #[must_use]
    pub const fn is_primary(&self) -> bool {
        self.is_primary
    }

    /// Returns whether this display exposes global logical coordinates.
    ///
    /// This is `true` exactly when [`Self::logical_bounds`] is present. It is
    /// `false` on platforms such as Wayland that do not reveal a global screen
    /// coordinate space, letting callers respect that absence rather than assume
    /// a fabricated origin.
    #[must_use]
    pub const fn has_global_coordinates(&self) -> bool {
        self.logical_bounds.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::{Point, Size};

    fn a_mode() -> DisplayMode {
        DisplayMode::new(Size::new(1920.0, 1080.0)).with_refresh_millihertz(60_000)
    }

    #[test]
    fn defaults_have_no_global_coordinates() {
        let d = Display::new(DisplayId::from_raw(1), a_mode());
        assert_eq!(d.id(), DisplayId::from_raw(1));
        assert_eq!(d.name(), None);
        // Wayland-honesty: absent bounds mean "unavailable", never a (0,0) origin.
        assert_eq!(d.logical_bounds(), None);
        assert_eq!(d.physical_bounds(), None);
        assert_eq!(d.work_area(), None);
        assert!(!d.has_global_coordinates());
        assert_eq!(d.scale_factor(), ScaleFactor::IDENTITY);
        assert_eq!(d.current_mode(), a_mode());
        assert!(d.available_modes().is_empty());
        assert_eq!(d.color(), DisplayColorCapability::new());
        assert!(!d.is_primary());
    }

    #[test]
    fn fluent_setters_populate_fields() {
        let bounds = Rect::new(Point::new(0.0, 0.0), Size::new(1920.0, 1080.0));
        let physical = Rect::new(Point::new(0.0, 0.0), Size::new(3840.0, 2160.0));
        let work = Rect::new(Point::new(0.0, 24.0), Size::new(1920.0, 1056.0));
        let d = Display::new(DisplayId::from_raw(2), a_mode())
            .with_name("DELL U2720Q")
            .with_logical_bounds(bounds)
            .with_physical_bounds(physical)
            .with_work_area(work)
            .with_scale_factor(ScaleFactor::new(2.0))
            .with_available_modes(vec![a_mode()])
            .with_color(DisplayColorCapability::new().with_hdr(true))
            .primary(true);

        assert_eq!(d.name(), Some("DELL U2720Q"));
        assert_eq!(d.logical_bounds(), Some(bounds));
        assert!(d.has_global_coordinates());
        assert_eq!(d.physical_bounds(), Some(physical));
        assert_eq!(d.work_area(), Some(work));
        assert_eq!(d.scale_factor(), ScaleFactor::new(2.0));
        assert_eq!(d.available_modes(), &[a_mode()]);
        assert!(d.color().is_hdr());
        assert!(d.is_primary());
    }

    #[test]
    fn display_id_round_trips() {
        assert_eq!(DisplayId::from_raw(42).raw(), 42);
    }
}
