//! The hexagonal port for platform window management (spec §6.1-6.6).
//!
//! This module defines [`WindowBackend`], the single trait through which the
//! pure windowing domain drives the outside platform: lifecycle (§6.1), dynamic
//! property updates (§6.2), activation and element focus (§6.3), custom chrome
//! regions (§6.4), fullscreen and presentation (§6.5), and display enumeration
//! (§6.6), plus capability introspection and event draining. Interface adapters
//! (a `winit` backend, a WASM backend, a headless test backend) implement it;
//! the domain never talks to a platform directly.
//!
//! The guiding contract, spec-wide: every operation returns a **typed result**
//! and reports **honest capability levels**. A backend never panics on an
//! unsupported operation — it returns [`WindowOperationResult::Unsupported`],
//! [`SupportLevel::Unavailable`], or [`ActivationResult::Unsupported`] as
//! appropriate. The colocated [`NullWindowBackend`] is a reference no-op
//! implementation that documents this honest-capability contract concretely and
//! makes the port testable.

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

/// A queryable platform capability key for [`WindowBackend::supports`].
///
/// Each variant names one capability the spec (§6.2-6.6) flags as
/// platform-conditional, so an application can introspect the honest
/// [`SupportLevel`] a backend offers before relying on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WindowFeature {
    /// Window/framebuffer transparency (spec §6.2 dynamic properties).
    Transparency,
    /// Keeping a window above others (§6.2 `always_on_top`).
    AlwaysOnTop,
    /// Client-side / custom chrome with app-defined regions (§6.4).
    CustomChrome,
    /// Blur-behind / acrylic backdrop effects (§6.2, §6.4 presentation).
    BlurBehind,
    /// Exclusive (mode-setting) fullscreen (§6.5).
    ExclusiveFullscreen,
    /// Kiosk / locked presentation that suppresses system UI (§6.5).
    Kiosk,
    /// Inhibiting system sleep / screensaver (§6.5).
    SleepInhibition,
    /// Global logical display coordinates — the Wayland-honesty capability:
    /// platforms that hide global screen coordinates report this unavailable
    /// rather than fabricating an origin (§6.6).
    GlobalDisplayCoordinates,
    /// Popping the platform system/window menu (§6.4).
    SystemMenu,
    /// Per-monitor DPI / scale-factor awareness (§6.6).
    PerMonitorDpi,
}

/// The failure type for the fallible window-creation path
/// ([`WindowBackend::create_window`], spec §6.1).
///
/// Creation is the one operation modeled as `Result` rather than a
/// [`WindowOperationResult`], because a failure yields no [`WindowId`] to
/// operate on. Non-creation operations report their honest outcome through
/// their own typed result types instead.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WindowError {
    /// The backend cannot create windows at all on this platform/configuration.
    Unsupported,
    /// The request was well-formed for the type system but semantically invalid
    /// for this backend, carrying a human-readable reason.
    InvalidRequest(Box<str>),
    /// The backend attempted creation but the platform failed, carrying a
    /// human-readable reason.
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

/// The hexagonal port for platform window management (spec §6.1-6.6).
///
/// Every windowing operation the domain performs funnels through this trait: an
/// adapter implements it once per platform, and the domain stores it as
/// `Box<dyn WindowBackend>`. The trait is therefore deliberately **object-safe**
/// — no generic methods and no `Self`-returning methods — so adapters can be
/// held polymorphically.
///
/// Contract: methods report honest, typed outcomes and **never panic** on an
/// unsupported operation. Query methods return a best-effort snapshot; mutating
/// methods return a typed result recording whether — and *how* — the operation
/// was satisfied.
pub trait WindowBackend {
    /// Creates a new window from `request`, minting and returning its
    /// [`WindowId`] (spec §6.1).
    ///
    /// # Errors
    ///
    /// Returns [`WindowError`] when creation cannot be satisfied; a backend that
    /// cannot create windows at all returns [`WindowError::Unsupported`] rather
    /// than panicking.
    fn create_window(&mut self, request: CreateWindow) -> Result<WindowId, WindowError>;

    /// Closes the window named by `request`, honoring or bypassing the
    /// close-confirmation per [`CloseWindow::is_forced`] (spec §6.1).
    ///
    /// Reports the honest outcome via [`WindowOperationResult`]; an unknown
    /// target yields [`WindowOperationResult::Failed`], never a panic.
    fn close_window(&mut self, request: CloseWindow) -> WindowOperationResult;

    /// Returns the retained [`Window`] record for `id`, or `None` if this
    /// backend holds no such window (spec §6.1).
    #[must_use]
    fn window(&self, id: WindowId) -> Option<&Window>;

    /// Applies the dynamic property delta `changes` to its target window
    /// (spec §6.2).
    ///
    /// The target window is taken from [`UpdateWindow::target`]; there is no
    /// separate `id` parameter, so the request is the single source of truth and
    /// an id/target mismatch is impossible by construction. Reports the honest
    /// [`WindowOperationResult`] — including the [`SupportLevel`] at which the
    /// property changes were satisfied — and returns
    /// [`WindowOperationResult::Failed`] for an unknown target rather than
    /// panicking.
    fn update_window(&mut self, changes: &UpdateWindow) -> WindowOperationResult;

    /// Activates a window and optionally moves element focus per `request`
    /// (spec §6.3).
    ///
    /// Returns [`ActivationResult`]: focus-stealing prevention is a normal
    /// [`ActivationResult::Denied`] outcome, and a backend without programmatic
    /// activation returns [`ActivationResult::Unsupported`] — neither is an
    /// error.
    fn activate(&mut self, request: ActivateWindow) -> ActivationResult;

    /// Sets the chrome mode (server- vs client-side decorations) for window `id`
    /// (spec §6.4).
    ///
    /// Reports the honest [`WindowOperationResult`]; a platform that cannot
    /// honor the requested mode returns [`WindowOperationResult::Unsupported`].
    fn set_chrome_mode(&mut self, id: WindowId, mode: ChromeMode) -> WindowOperationResult;

    /// Declares the draggable (title-bar-like) regions of window `id` for
    /// custom chrome (spec §6.4).
    ///
    /// `regions` is borrowed because the caller retains ownership. Reports the
    /// honest [`WindowOperationResult`].
    fn set_drag_regions(&mut self, id: WindowId, regions: &[DragRegion]) -> WindowOperationResult;

    /// Declares the resize-handle regions of window `id` for custom chrome
    /// (spec §6.4).
    ///
    /// `regions` is borrowed because the caller retains ownership. Reports the
    /// honest [`WindowOperationResult`].
    fn set_resize_regions(
        &mut self,
        id: WindowId,
        regions: &[ResizeRegion],
    ) -> WindowOperationResult;

    /// Pops the platform system/window menu for window `id` per `request`
    /// (spec §6.4).
    ///
    /// Reports the honest [`WindowOperationResult`]; a platform without a system
    /// menu returns [`WindowOperationResult::Unsupported`].
    fn show_system_menu(
        &mut self,
        id: WindowId,
        request: SystemMenuRequest,
    ) -> WindowOperationResult;

    /// Applies the presentation `mode` (fullscreen, kiosk, and related policy)
    /// to window `id` (spec §6.5).
    ///
    /// `mode` is borrowed because the caller may reuse it across windows.
    /// Reports the honest [`WindowOperationResult`], including the
    /// [`SupportLevel`] for an emulated or conditionally-supported mode.
    fn set_presentation(&mut self, id: WindowId, mode: &PresentationMode) -> WindowOperationResult;

    /// Requests inhibition of system sleep / screensaver per `request`
    /// (spec §6.5).
    ///
    /// Reports the honest [`WindowOperationResult`]; a platform that cannot
    /// inhibit sleep returns [`WindowOperationResult::Unsupported`].
    fn inhibit_sleep(&mut self, request: SleepInhibition) -> WindowOperationResult;

    /// Enumerates the displays currently known to the backend (spec §6.6).
    ///
    /// Returns an empty [`Vec`] when no displays are available; geometry the
    /// platform does not expose is reported as `None` on each [`Display`] rather
    /// than fabricated.
    #[must_use]
    fn displays(&self) -> Vec<Display>;

    /// Returns the [`DisplayId`] of the primary display, or `None` if the
    /// backend cannot identify one (spec §6.6).
    #[must_use]
    fn primary_display(&self) -> Option<DisplayId>;

    /// Reports the honest [`SupportLevel`] at which this backend offers
    /// `feature` (spec §6.2-6.6).
    ///
    /// Returns [`SupportLevel::Unavailable`] for a feature the platform does not
    /// provide; this is the introspection counterpart of the per-operation
    /// typed results.
    #[must_use]
    fn supports(&self, feature: WindowFeature) -> SupportLevel;

    /// Drains and returns the window events queued since the last poll, each
    /// paired with the window it targets (spec §6.1-6.5).
    ///
    /// [`WindowEvent`] carries no [`WindowId`] of its own, so the pairing is
    /// supplied here. Returns an empty [`Vec`] when nothing is queued.
    fn poll_window_events(&mut self) -> Vec<(WindowId, WindowEvent)>;

    /// Drains and returns the display events queued since the last poll
    /// (spec §6.6).
    ///
    /// Returns an empty [`Vec`] when nothing is queued.
    fn poll_display_events(&mut self) -> Vec<DisplayEvent>;
}

/// A reference, no-op [`WindowBackend`] that owns nothing platform-specific
/// (spec §6.1-6.6).
///
/// `NullWindowBackend` retains created [`Window`] records in memory, keyed by a
/// monotonically increasing [`WindowId`], and honestly reports that it supports
/// no real platform capability: activation is
/// [`ActivationResult::Unsupported`], every chrome/presentation/sleep operation
/// is [`WindowOperationResult::Unsupported`], [`Self::supports`] is
/// [`SupportLevel::Unavailable`] for every feature, and it enumerates no
/// displays and emits no events. It exists to exercise the port and to document
/// the honest-capability contract concretely — its "unsupported" answers are
/// correct behavior, not stubs waiting to be filled in.
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
