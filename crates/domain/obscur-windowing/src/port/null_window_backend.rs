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

use super::{WindowBackend, WindowError, WindowFeature};

#[doc = include_str!("../../docs/api/port/window_backend.doc/NullWindowBackend.md")]
#[derive(Debug, Default)]
pub struct NullWindowBackend {
    /// The retained window records, keyed by their minted identity.
    windows: HashMap<WindowId, Window>,
    /// The source of fresh window identities: a plain incrementing counter.
    next_id: u64,
}

impl NullWindowBackend {
    /// Creates an empty backend holding no windows.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Mints the next fresh [`WindowId`] from the monotonic counter.
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
}
