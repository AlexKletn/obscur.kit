#![doc = include_str!("window_backend.doc/_module.md")]

use std::collections::HashMap;

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

#[doc = include_str!("window_backend.doc/WindowFeature.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WindowFeature {
    #[doc = include_str!("window_backend.doc/Transparency.md")]
    Transparency,
    #[doc = include_str!("window_backend.doc/AlwaysOnTop.md")]
    AlwaysOnTop,
    #[doc = include_str!("window_backend.doc/CustomChrome.md")]
    CustomChrome,
    #[doc = include_str!("window_backend.doc/BlurBehind.md")]
    BlurBehind,
    #[doc = include_str!("window_backend.doc/ExclusiveFullscreen.md")]
    ExclusiveFullscreen,
    #[doc = include_str!("window_backend.doc/Kiosk.md")]
    Kiosk,
    #[doc = include_str!("window_backend.doc/SleepInhibition.md")]
    SleepInhibition,
    #[doc = include_str!("window_backend.doc/GlobalDisplayCoordinates.md")]
    GlobalDisplayCoordinates,
    #[doc = include_str!("window_backend.doc/SystemMenu.md")]
    SystemMenu,
    #[doc = include_str!("window_backend.doc/PerMonitorDpi.md")]
    PerMonitorDpi,
}

#[doc = include_str!("window_backend.doc/WindowError.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WindowError {
    #[doc = include_str!("window_backend.doc/Unsupported.md")]
    Unsupported,
    #[doc = include_str!("window_backend.doc/InvalidRequest.md")]
    InvalidRequest(Box<str>),
    #[doc = include_str!("window_backend.doc/BackendFailure.md")]
    BackendFailure(Box<str>),
}

impl std::fmt::Display for WindowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported => f.write_str("window creation is unsupported"),
            Self::InvalidRequest(reason) => write!(f, "invalid window request: {reason}"),
            Self::BackendFailure(reason) => write!(f, "window backend failure: {reason}"),
        }
    }
}

impl std::error::Error for WindowError {}

#[doc = include_str!("window_backend.doc/WindowBackend.md")]
pub trait WindowBackend {
    #[doc = include_str!("window_backend.doc/create_window.md")]
    fn create_window(&mut self, request: CreateWindow) -> Result<WindowId, WindowError>;

    #[doc = include_str!("window_backend.doc/close_window.md")]
    fn close_window(&mut self, request: CloseWindow) -> WindowOperationResult;

    #[doc = include_str!("window_backend.doc/window.md")]
    #[must_use]
    fn window(&self, id: WindowId) -> Option<&Window>;

    #[doc = include_str!("window_backend.doc/update_window.md")]
    fn update_window(&mut self, changes: &UpdateWindow) -> WindowOperationResult;

    #[doc = include_str!("window_backend.doc/activate.md")]
    fn activate(&mut self, request: ActivateWindow) -> ActivationResult;

    #[doc = include_str!("window_backend.doc/set_chrome_mode.md")]
    fn set_chrome_mode(&mut self, id: WindowId, mode: ChromeMode) -> WindowOperationResult;

    #[doc = include_str!("window_backend.doc/set_drag_regions.md")]
    fn set_drag_regions(&mut self, id: WindowId, regions: &[DragRegion]) -> WindowOperationResult;

    #[doc = include_str!("window_backend.doc/set_resize_regions.md")]
    fn set_resize_regions(
        &mut self,
        id: WindowId,
        regions: &[ResizeRegion],
    ) -> WindowOperationResult;

    #[doc = include_str!("window_backend.doc/show_system_menu.md")]
    fn show_system_menu(
        &mut self,
        id: WindowId,
        request: SystemMenuRequest,
    ) -> WindowOperationResult;

    #[doc = include_str!("window_backend.doc/set_presentation.md")]
    fn set_presentation(&mut self, id: WindowId, mode: &PresentationMode) -> WindowOperationResult;

    #[doc = include_str!("window_backend.doc/inhibit_sleep.md")]
    fn inhibit_sleep(&mut self, request: SleepInhibition) -> WindowOperationResult;

    #[doc = include_str!("window_backend.doc/displays.md")]
    #[must_use]
    fn displays(&self) -> Vec<Display>;

    #[doc = include_str!("window_backend.doc/primary_display.md")]
    #[must_use]
    fn primary_display(&self) -> Option<DisplayId>;

    #[doc = include_str!("window_backend.doc/supports.md")]
    #[must_use]
    fn supports(&self, feature: WindowFeature) -> SupportLevel;

    #[doc = include_str!("window_backend.doc/poll_window_events.md")]
    fn poll_window_events(&mut self) -> Vec<(WindowId, WindowEvent)>;

    #[doc = include_str!("window_backend.doc/poll_display_events.md")]
    fn poll_display_events(&mut self) -> Vec<DisplayEvent>;
}

#[doc = include_str!("window_backend.doc/NullWindowBackend.md")]
#[derive(Debug, Default)]
pub struct NullWindowBackend {
    #[doc = include_str!("window_backend.doc/windows.md")]
    windows: HashMap<WindowId, Window>,
    #[doc = include_str!("window_backend.doc/next_id.md")]
    next_id: u64,
}

impl NullWindowBackend {
    #[doc = include_str!("window_backend.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("window_backend.doc/allocate_id.md")]
    fn allocate_id(&mut self) -> WindowId {
        let id = WindowId::from_raw(self.next_id);
        self.next_id += 1;
        id
    }
}

impl WindowBackend for NullWindowBackend {
    fn create_window(&mut self, request: CreateWindow) -> Result<WindowId, WindowError> {
        let id = self.allocate_id();
        self.windows.insert(id, request.into_window(id));
        Ok(id)
    }

    fn close_window(&mut self, request: CloseWindow) -> WindowOperationResult {
        let target = request.target();
        if self.windows.remove(&target).is_none() {
            return WindowOperationResult::failed("unknown window");
        }
        // §6.1 owner/parent relations: destroying an owner destroys the windows
        // it owns, transitively. Repeatedly remove any window whose owner is no
        // longer present in the retained set (i.e. was itself just destroyed).
        loop {
            let orphaned: Vec<WindowId> = self
                .windows
                .iter()
                .filter(|(_, window)| {
                    window
                        .relation()
                        .owner
                        .is_some_and(|owner| !self.windows.contains_key(&owner))
                })
                .map(|(id, _)| *id)
                .collect();
            if orphaned.is_empty() {
                break;
            }
            for id in orphaned {
                self.windows.remove(&id);
            }
        }
        WindowOperationResult::native()
    }

    fn window(&self, id: WindowId) -> Option<&Window> {
        self.windows.get(&id)
    }

    fn update_window(&mut self, changes: &UpdateWindow) -> WindowOperationResult {
        match self.windows.get_mut(&changes.target()) {
            Some(window) => {
                // §6.2: keeping a window above others requires a compositor to
                // restack it; a headless backend has none, so always-on-top is
                // `unsupported` and the property must not be silently applied.
                if changes.always_on_top().is_some() {
                    return WindowOperationResult::Unsupported;
                }
                changes.apply(window.properties_mut());
                WindowOperationResult::native()
            }
            None => WindowOperationResult::failed("unknown window"),
        }
    }

    fn activate(&mut self, _request: ActivateWindow) -> ActivationResult {
        // Honest: a headless null backend has no real focus to grant.
        ActivationResult::Unsupported
    }

    fn set_chrome_mode(&mut self, _id: WindowId, _mode: ChromeMode) -> WindowOperationResult {
        WindowOperationResult::Unsupported
    }

    fn set_drag_regions(
        &mut self,
        _id: WindowId,
        _regions: &[DragRegion],
    ) -> WindowOperationResult {
        WindowOperationResult::Unsupported
    }

    fn set_resize_regions(
        &mut self,
        _id: WindowId,
        _regions: &[ResizeRegion],
    ) -> WindowOperationResult {
        WindowOperationResult::Unsupported
    }

    fn show_system_menu(
        &mut self,
        _id: WindowId,
        _request: SystemMenuRequest,
    ) -> WindowOperationResult {
        WindowOperationResult::Unsupported
    }

    fn set_presentation(
        &mut self,
        _id: WindowId,
        _mode: &PresentationMode,
    ) -> WindowOperationResult {
        WindowOperationResult::Unsupported
    }

    fn inhibit_sleep(&mut self, _request: SleepInhibition) -> WindowOperationResult {
        WindowOperationResult::Unsupported
    }

    fn displays(&self) -> Vec<Display> {
        Vec::new()
    }

    fn primary_display(&self) -> Option<DisplayId> {
        None
    }

    fn supports(&self, _feature: WindowFeature) -> SupportLevel {
        // Honest: a null backend supports no real platform capability.
        SupportLevel::Unavailable
    }

    fn poll_window_events(&mut self) -> Vec<(WindowId, WindowEvent)> {
        Vec::new()
    }

    fn poll_display_events(&mut self) -> Vec<DisplayEvent> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::window::{WindowProperties, WindowState};

    /// Compile-time proof the port is object-safe: it must be usable as
    /// `Box<dyn WindowBackend>` for adapters to be stored polymorphically.
    #[test]
    fn window_backend_is_object_safe() {
        let backend: Box<dyn WindowBackend> = Box::new(NullWindowBackend::new());
        // Exercise one method through the trait object to be sure.
        assert!(backend.displays().is_empty());
    }

    fn assert_obj_safe(_: &dyn WindowBackend) {}

    #[test]
    fn create_lookup_update_close_roundtrip() {
        let mut backend = NullWindowBackend::new();
        assert_obj_safe(&backend);

        let request = CreateWindow::new(WindowProperties::default().with_title("Main"));
        let id = backend
            .create_window(request)
            .expect("null backend always creates");

        // Lookup finds the freshly materialized record.
        let window = backend.window(id).expect("window is retained");
        assert_eq!(window.id(), id);
        assert_eq!(&*window.properties().title, "Main");

        // update_window applies the delta via changes.apply, keyed by target.
        let delta = UpdateWindow::new(id)
            .set_title("Renamed")
            .set_state(WindowState::Maximized);
        assert_eq!(
            backend.update_window(&delta),
            WindowOperationResult::native()
        );
        let window = backend.window(id).expect("still retained");
        assert_eq!(&*window.properties().title, "Renamed");
        assert_eq!(window.properties().state, WindowState::Maximized);

        // Closing a known window succeeds; the record is gone afterwards.
        assert_eq!(
            backend.close_window(CloseWindow::request(id)),
            WindowOperationResult::native()
        );
        assert!(backend.window(id).is_none());
    }

    #[test]
    fn ids_are_monotonic() {
        let mut backend = NullWindowBackend::new();
        let a = backend
            .create_window(CreateWindow::new(WindowProperties::default()))
            .unwrap();
        let b = backend
            .create_window(CreateWindow::new(WindowProperties::default()))
            .unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn unknown_window_operations_fail_honestly() {
        let mut backend = NullWindowBackend::new();
        let ghost = WindowId::from_raw(999);

        assert_eq!(
            backend.close_window(CloseWindow::request(ghost)),
            WindowOperationResult::failed("unknown window")
        );
        assert_eq!(
            backend.update_window(&UpdateWindow::new(ghost).set_resizable(false)),
            WindowOperationResult::failed("unknown window")
        );
    }

    #[test]
    fn activation_is_unsupported_not_a_bug() {
        let mut backend = NullWindowBackend::new();
        assert_eq!(
            backend.activate(ActivateWindow::window(WindowId::from_raw(0))),
            ActivationResult::Unsupported
        );
    }

    #[test]
    fn chrome_presentation_and_sleep_are_unsupported() {
        use crate::presentation::{PresentationMode, SleepInhibition, SleepInhibitionScope};
        use obscur_geometry::Point;

        let mut backend = NullWindowBackend::new();
        let id = WindowId::from_raw(0);

        assert_eq!(
            backend.set_chrome_mode(id, ChromeMode::default()),
            WindowOperationResult::Unsupported
        );
        assert_eq!(
            backend.set_drag_regions(id, &[]),
            WindowOperationResult::Unsupported
        );
        assert_eq!(
            backend.set_resize_regions(id, &[]),
            WindowOperationResult::Unsupported
        );
        assert_eq!(
            backend.show_system_menu(id, SystemMenuRequest::at(Point::new(0.0, 0.0))),
            WindowOperationResult::Unsupported
        );
        assert_eq!(
            backend.set_presentation(id, &PresentationMode::default()),
            WindowOperationResult::Unsupported
        );
        assert_eq!(
            backend.inhibit_sleep(SleepInhibition::reasonless(SleepInhibitionScope::System)),
            WindowOperationResult::Unsupported
        );
    }

    #[test]
    fn supports_is_unavailable_for_every_feature() {
        let backend = NullWindowBackend::new();
        for feature in [
            WindowFeature::Transparency,
            WindowFeature::AlwaysOnTop,
            WindowFeature::CustomChrome,
            WindowFeature::BlurBehind,
            WindowFeature::ExclusiveFullscreen,
            WindowFeature::Kiosk,
            WindowFeature::SleepInhibition,
            WindowFeature::GlobalDisplayCoordinates,
            WindowFeature::SystemMenu,
            WindowFeature::PerMonitorDpi,
        ] {
            assert_eq!(backend.supports(feature), SupportLevel::Unavailable);
        }
    }

    #[test]
    fn displays_and_events_are_empty() {
        let mut backend = NullWindowBackend::new();
        assert!(backend.displays().is_empty());
        assert_eq!(backend.primary_display(), None);
        assert!(backend.poll_window_events().is_empty());
        assert!(backend.poll_display_events().is_empty());
    }

    #[test]
    fn window_error_displays_and_is_error() {
        let err = WindowError::InvalidRequest(Box::from("bad size"));
        assert_eq!(err.to_string(), "invalid window request: bad size");
        let _: &dyn std::error::Error = &err;
        assert_eq!(
            WindowError::Unsupported.to_string(),
            "window creation is unsupported"
        );
        assert_eq!(
            WindowError::BackendFailure(Box::from("x11 down")).to_string(),
            "window backend failure: x11 down"
        );
    }
}
