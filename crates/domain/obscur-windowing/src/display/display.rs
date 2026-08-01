#![doc = include_str!("display.doc/_module.md")]

use obscur_geometry::{Rect, ScaleFactor};

use crate::display::{DisplayColorCapability, DisplayMode};

#[doc = include_str!("display.doc/DisplayId.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DisplayId(u64);

impl DisplayId {
    #[doc = include_str!("display.doc/from_raw.md")]
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    #[doc = include_str!("display.doc/raw.md")]
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[doc = include_str!("display.doc/Display.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct Display {
    #[doc = include_str!("display.doc/id.md")]
    pub id: DisplayId,
    #[doc = include_str!("display.doc/name.md")]
    pub name: Option<Box<str>>,
    #[doc = include_str!("display.doc/logical_bounds.md")]
    pub logical_bounds: Option<Rect>,
    #[doc = include_str!("display.doc/physical_bounds.md")]
    pub physical_bounds: Option<Rect>,
    #[doc = include_str!("display.doc/work_area.md")]
    pub work_area: Option<Rect>,
    #[doc = include_str!("display.doc/scale_factor.md")]
    pub scale_factor: ScaleFactor,
    #[doc = include_str!("display.doc/current_mode.md")]
    pub current_mode: DisplayMode,
    #[doc = include_str!("display.doc/available_modes.md")]
    pub available_modes: Vec<DisplayMode>,
    #[doc = include_str!("display.doc/color.md")]
    pub color: DisplayColorCapability,
    #[doc = include_str!("display.doc/is_primary.md")]
    pub is_primary: bool,
}

impl Display {
    #[doc = include_str!("display.doc/new.md")]
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

    #[doc = include_str!("display.doc/with_name.md")]
    #[must_use]
    pub fn with_name(mut self, name: impl Into<Box<str>>) -> Self {
        self.name = Some(name.into());
        self
    }

    #[doc = include_str!("display.doc/with_logical_bounds.md")]
    #[must_use]
    pub fn with_logical_bounds(mut self, bounds: Rect) -> Self {
        self.logical_bounds = Some(bounds);
        self
    }

    #[doc = include_str!("display.doc/with_physical_bounds.md")]
    #[must_use]
    pub fn with_physical_bounds(mut self, bounds: Rect) -> Self {
        self.physical_bounds = Some(bounds);
        self
    }

    #[doc = include_str!("display.doc/with_work_area.md")]
    #[must_use]
    pub fn with_work_area(mut self, work_area: Rect) -> Self {
        self.work_area = Some(work_area);
        self
    }

    #[doc = include_str!("display.doc/with_scale_factor.md")]
    #[must_use]
    pub fn with_scale_factor(mut self, scale_factor: ScaleFactor) -> Self {
        self.scale_factor = scale_factor;
        self
    }

    #[doc = include_str!("display.doc/with_available_modes.md")]
    #[must_use]
    pub fn with_available_modes(mut self, available_modes: Vec<DisplayMode>) -> Self {
        self.available_modes = available_modes;
        self
    }

    #[doc = include_str!("display.doc/with_color.md")]
    #[must_use]
    pub fn with_color(mut self, color: DisplayColorCapability) -> Self {
        self.color = color;
        self
    }

    #[doc = include_str!("display.doc/primary.md")]
    #[must_use]
    pub fn primary(mut self, is_primary: bool) -> Self {
        self.is_primary = is_primary;
        self
    }

    #[doc = include_str!("display.doc/id_2.md")]
    #[must_use]
    pub const fn id(&self) -> DisplayId {
        self.id
    }

    #[doc = include_str!("display.doc/name_2.md")]
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    #[doc = include_str!("display.doc/logical_bounds_2.md")]
    #[must_use]
    pub const fn logical_bounds(&self) -> Option<Rect> {
        self.logical_bounds
    }

    #[doc = include_str!("display.doc/physical_bounds_2.md")]
    #[must_use]
    pub const fn physical_bounds(&self) -> Option<Rect> {
        self.physical_bounds
    }

    #[doc = include_str!("display.doc/work_area_2.md")]
    #[must_use]
    pub const fn work_area(&self) -> Option<Rect> {
        self.work_area
    }

    #[doc = include_str!("display.doc/scale_factor_2.md")]
    #[must_use]
    pub const fn scale_factor(&self) -> ScaleFactor {
        self.scale_factor
    }

    #[doc = include_str!("display.doc/current_mode_2.md")]
    #[must_use]
    pub const fn current_mode(&self) -> DisplayMode {
        self.current_mode
    }

    #[doc = include_str!("display.doc/available_modes_2.md")]
    #[must_use]
    pub fn available_modes(&self) -> &[DisplayMode] {
        &self.available_modes
    }

    #[doc = include_str!("display.doc/color_2.md")]
    #[must_use]
    pub const fn color(&self) -> DisplayColorCapability {
        self.color
    }

    #[doc = include_str!("display.doc/is_primary_2.md")]
    #[must_use]
    pub const fn is_primary(&self) -> bool {
        self.is_primary
    }

    #[doc = include_str!("display.doc/has_global_coordinates.md")]
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
