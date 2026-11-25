#![doc = include_str!("../../docs/api/port/window_backend.doc/_module.md")]

use obscur_capability::SupportLevel;
use obscur_identity::WindowId;

use crate::chrome::{ChromeMode, DragRegion, ResizeRegion, SystemMenuRequest};
use crate::display::{Display, DisplayEvent, DisplayId};
use crate::presentation::{PresentationMode, SleepInhibition};
use crate::request::{
    ActivateWindow, ActivationResult, CloseWindow, CreateWindow, UpdateWindow,
    WindowOperationResult,
};
use crate::window::{Window, WindowEvent};

use super::{WindowError, WindowFeature};

#[doc = include_str!("../../docs/api/port/window_backend.doc/WindowBackend.md")]
pub trait WindowBackend {
    #[doc = include_str!("../../docs/api/port/window_backend.doc/create_window.md")]
    fn create_window(&mut self, request: CreateWindow) -> Result<WindowId, WindowError>;

    #[doc = include_str!("../../docs/api/port/window_backend.doc/close_window.md")]
    fn close_window(&mut self, request: CloseWindow) -> WindowOperationResult;

    /// Returns the retained [`Window`] record for `id`, or `None` if this
    /// backend holds no such window (spec §6.1).
    #[must_use]
    fn window(&self, id: WindowId) -> Option<&Window>;

    #[doc = include_str!("../../docs/api/port/window_backend.doc/update_window.md")]
    fn update_window(&mut self, changes: &UpdateWindow) -> WindowOperationResult;

    #[doc = include_str!("../../docs/api/port/window_backend.doc/activate.md")]
    fn activate(&mut self, request: ActivateWindow) -> ActivationResult;

    #[doc = include_str!("../../docs/api/port/window_backend.doc/set_chrome_mode.md")]
    fn set_chrome_mode(&mut self, id: WindowId, mode: ChromeMode) -> WindowOperationResult;

    #[doc = include_str!("../../docs/api/port/window_backend.doc/set_drag_regions.md")]
    fn set_drag_regions(&mut self, id: WindowId, regions: &[DragRegion]) -> WindowOperationResult;

    #[doc = include_str!("../../docs/api/port/window_backend.doc/set_resize_regions.md")]
    fn set_resize_regions(
        &mut self,
        id: WindowId,
        regions: &[ResizeRegion],
    ) -> WindowOperationResult;

    #[doc = include_str!("../../docs/api/port/window_backend.doc/show_system_menu.md")]
    fn show_system_menu(
        &mut self,
        id: WindowId,
        request: SystemMenuRequest,
    ) -> WindowOperationResult;

    #[doc = include_str!("../../docs/api/port/window_backend.doc/set_presentation.md")]
    fn set_presentation(&mut self, id: WindowId, mode: &PresentationMode) -> WindowOperationResult;

    #[doc = include_str!("../../docs/api/port/window_backend.doc/inhibit_sleep.md")]
    fn inhibit_sleep(&mut self, request: SleepInhibition) -> WindowOperationResult;

    #[doc = include_str!("../../docs/api/port/window_backend.doc/displays.md")]
    #[must_use]
    fn displays(&self) -> Vec<Display>;

    /// Returns the [`DisplayId`] of the primary display, or `None` if the
    /// backend cannot identify one (spec §6.6).
    #[must_use]
    fn primary_display(&self) -> Option<DisplayId>;

    #[doc = include_str!("../../docs/api/port/window_backend.doc/supports.md")]
    #[must_use]
    fn supports(&self, feature: WindowFeature) -> SupportLevel;

    #[doc = include_str!("../../docs/api/port/window_backend.doc/poll_window_events.md")]
    fn poll_window_events(&mut self) -> Vec<(WindowId, WindowEvent)>;

    #[doc = include_str!("../../docs/api/port/window_backend.doc/poll_display_events.md")]
    fn poll_display_events(&mut self) -> Vec<DisplayEvent>;
}
