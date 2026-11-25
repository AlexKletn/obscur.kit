//! [`AppControl`] — the handle application code holds onto a running runtime.

mod app_control_state;

use app_control_state::AppControlState;
#[cfg(feature = "window-winit")]
use app_control_state::FrameCapture;

use std::cell::RefCell;
use std::rc::Rc;

use crate::bootstrap::{CssPipeline, KitBootstrap};
use crate::css::source::{CssSource, SourceId, SourceVersion};
use crate::geometry::space::{PointIn, ScreenSpace};
use crate::geometry::{Insets, Rect};
use crate::headless::input::SelectorTarget;
use crate::identity::{InternedSymbol, NodeId};
use crate::interaction::event::ListenerPhase;
use crate::interaction::feedback::{Feedback, FeedbackEffect};
use crate::interaction::gesture::SystemGestureRegions;
use crate::runtime::frame::FrameReason;
use crate::runtime::lifecycle::LifecycleEvent;
use crate::runtime::{IntoCommandHandler, UiRuntime};
use crate::system::appearance::SystemAppearance;
use crate::system::capture::{CaptureProvider, CaptureRequest, CaptureResult, CaptureSource};

#[cfg(all(feature = "render-vello", feature = "window-winit"))]
use super::WindowAction;
use super::{ControlRequest, WindowPlacement, WindowState};

#[doc = include_str!("../../docs/api/control/app_control.doc/AppControl.md")]
#[derive(Clone, Default)]
pub struct AppControl {
    state: Rc<RefCell<AppControlState>>,
}

impl AppControl {
    #[doc = include_str!("../../docs/api/control/app_control.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Installs a capture provider for this application.
    pub fn set_capture_provider(&self, provider: Option<Rc<RefCell<dyn CaptureProvider>>>) {
        self.state.borrow_mut().capture_provider = provider;
    }

    /// Replaces the protected regions applied to every capture from this control.
    /// Coordinates use the capture provider's source space. The built-in runner
    /// interprets them as physical pixels relative to its window surface.
    pub fn set_capture_protected_regions(&self, regions: Vec<Rect>) {
        self.state.borrow_mut().capture_protected_regions = regions;
    }

    /// Reports the built-in window runner's ability to request whole-window
    /// protection from system capture. The OS controls enforcement.
    #[must_use]
    pub const fn window_capture_protection_support(&self) -> crate::capability::SupportLevel {
        use crate::capability::SupportLevel;

        if cfg!(all(
            feature = "window-winit",
            any(target_os = "windows", target_os = "macos")
        )) {
            SupportLevel::Limited
        } else {
            SupportLevel::Unavailable
        }
    }

    /// Requests system capture protection for the entire built-in window.
    /// The returned support level describes availability of the request.
    pub fn set_window_capture_protected(&self, protected: bool) -> crate::capability::SupportLevel {
        let support = self.window_capture_protection_support();
        #[cfg(feature = "window-winit")]
        if support != crate::capability::SupportLevel::Unavailable {
            self.state.borrow_mut().window_capture_protection = Some(protected);
            self.request_frame();
        }
        #[cfg(not(feature = "window-winit"))]
        let _ = protected;
        support
    }

    #[cfg(feature = "window-winit")]
    pub(crate) fn take_window_capture_protection(&self) -> Option<bool> {
        self.state.borrow_mut().window_capture_protection.take()
    }

    pub(crate) fn protect_capture_request(&self, request: CaptureRequest) -> CaptureRequest {
        self.state
            .borrow()
            .capture_protected_regions
            .iter()
            .copied()
            .fold(request, CaptureRequest::protecting)
    }

    /// Reports whether the installed provider supports this source.
    #[must_use]
    pub fn can_capture(&self, source: &CaptureSource) -> bool {
        let provider = self.state.borrow().capture_provider.clone();
        provider.is_some_and(|provider| provider.borrow().can_capture(source))
    }

    /// Captures through the installed provider. A request with protected regions
    /// is denied unless the provider can enforce that protection for its source.
    #[must_use]
    pub fn capture(&self, request: CaptureRequest) -> CaptureResult {
        let Some(provider) = self.state.borrow().capture_provider.clone() else {
            return CaptureResult::Unsupported;
        };
        let mut provider = provider.borrow_mut();
        if !provider.can_capture(request.source()) {
            return CaptureResult::Unsupported;
        }
        let request = self.protect_capture_request(request);
        if request.has_protected_regions() && !provider.protects_regions(request.source()) {
            return CaptureResult::Denied;
        }
        provider.capture(request)
    }

    /// Requests pixels from the next frame of the built-in window runner.
    /// The callback runs on the window thread. Region coordinates are physical
    /// pixels relative to the window surface.
    #[cfg(feature = "window-winit")]
    pub fn capture_next_frame(
        &self,
        request: CaptureRequest,
        on_complete: impl FnOnce(CaptureResult) + 'static,
    ) {
        let request = self.protect_capture_request(request);
        self.state
            .borrow_mut()
            .frame_captures
            .push((request, Box::new(on_complete)));
        self.request_frame();
    }

    #[cfg(feature = "window-winit")]
    pub(crate) fn take_frame_captures(&self) -> Vec<FrameCapture> {
        std::mem::take(&mut self.state.borrow_mut().frame_captures)
    }

    /// Queues updated pointing capabilities for CSS pointer and hover queries. None marks the capabilities as unknown.
    pub fn set_media_input(&self, input: Option<crate::style::matching::MediaInput>) {
        self.with_runtime(move |runtime| runtime.set_media_input(input));
    }

    /// Queues a return to the platform-detected pointing capabilities.
    pub fn clear_media_input_override(&self) {
        self.with_runtime(UiRuntime::clear_media_input_override);
    }

    /// Queues a system appearance snapshot and refreshes its CSS variables while retaining the built-in UA rules.
    pub fn set_system_appearance(&self, appearance: SystemAppearance) {
        self.with_runtime(move |runtime| {
            KitBootstrap::new().install_system_theme(runtime, &appearance);
            runtime.request_frame(FrameReason::UiCommit);
        });
    }

    /// Queues safe-area insets in logical pixels for the next runtime update.
    pub fn set_safe_area_insets(&self, insets: Insets) {
        self.with_runtime(move |runtime| {
            runtime.set_safe_area_insets(insets);
        });
    }

    /// Queues on-screen keyboard occlusion in logical pixels for the next runtime update. Use zero insets when the keyboard hides.
    pub fn set_keyboard_insets(&self, insets: Insets) {
        self.with_runtime(move |runtime| {
            runtime.set_keyboard_insets(insets);
        });
    }

    /// Enables or disables default directional focus navigation for arrows and gamepad D-pad in this app.
    pub fn set_spatial_navigation(&self, enabled: bool) {
        self.with_runtime(move |runtime| runtime.set_spatial_navigation(enabled));
    }

    /// Installs a window-local portal resolver after the next app-control turn. Passing None restores unique element id lookup.
    pub fn set_portal_host_resolver(
        &self,
        resolver: Option<Box<dyn crate::tree::portal::PortalHostResolver>>,
    ) {
        self.with_runtime(move |runtime| runtime.set_portal_host_resolver(resolver));
    }

    /// Installs the feedback adapters for this app. All clones of the app control see the replacement.
    pub fn set_feedback(&self, feedback: Feedback) {
        self.state.borrow_mut().feedback = feedback;
    }

    /// Requests a semantic tactile cue. With no haptic adapter installed, the request is silent.
    pub fn play_haptic(&self, effect: FeedbackEffect) {
        let feedback = self.state.borrow().feedback.clone();
        feedback.haptic(effect);
    }

    /// Requests a semantic audible cue. With no sound adapter installed, the request is silent.
    pub fn play_sound(&self, effect: FeedbackEffect) {
        let feedback = self.state.borrow().feedback.clone();
        feedback.sound(effect);
    }

    /// Installs the host adapter for this window. Passing `None` detaches the adapter from the app control.
    pub fn set_system_gesture_regions(&self, adapter: Option<Rc<dyn SystemGestureRegions>>) {
        self.state.borrow_mut().system_gesture_regions = adapter;
    }

    /// Returns the system gesture regions in logical window coordinates, or `None` if the host has not installed a region adapter.
    #[must_use]
    pub fn reserved_gesture_regions(&self) -> Option<Vec<Rect>> {
        let adapter = self.state.borrow().system_gesture_regions.clone()?;
        Some(adapter.reserved_regions())
    }

    /// Submits the complete set of requested exclusion rectangles in logical window coordinates. An empty slice clears the request. The result means the host accepted it for processing. The platform decides whether and where to apply the exclusions.
    pub fn request_gesture_exclusions(&self, regions: &[Rect]) -> bool {
        let adapter = self.state.borrow().system_gesture_regions.clone();
        adapter.is_some_and(|adapter| adapter.set_exclusion_regions(regions))
    }

    #[doc = include_str!("../../docs/api/control/app_control.doc/on_command_at.md")]
    pub fn on_command_at<M>(
        &self,
        target: SelectorTarget,
        command: InternedSymbol,
        handler: impl IntoCommandHandler<M>,
    ) {
        let handler = Rc::new(RefCell::new(handler.into_command_handler()));
        self.state
            .borrow_mut()
            .bindings
            .push_handler(target, command, handler);
    }

    #[doc = include_str!("../../docs/api/control/app_control.doc/add_event_listener.md")]
    pub fn add_event_listener<M>(
        &self,
        target: SelectorTarget,
        command: InternedSymbol,
        phase: ListenerPhase,
        listener: impl IntoCommandHandler<M>,
    ) {
        let listener = Rc::new(RefCell::new(listener.into_command_handler()));
        self.state
            .borrow_mut()
            .bindings
            .push_listener(target, command, phase, listener);
    }

    #[doc = include_str!("../../docs/api/control/app_control.doc/set_stylesheets.md")]
    pub fn set_stylesheets(&self, sheets: Vec<String>) {
        self.push(ControlRequest::Stylesheets(sheets));
    }

    /// [`set_stylesheets`](Self::set_stylesheets) for the common case of one sheet
    /// replacing the whole authored set.
    pub fn set_stylesheet(&self, css: impl Into<String>) {
        self.set_stylesheets(vec![css.into()]);
    }

    #[doc = include_str!("../../docs/api/control/app_control.doc/with_runtime.md")]
    pub fn with_runtime(&self, action: impl FnOnce(&mut UiRuntime) + 'static) {
        self.push(ControlRequest::Runtime(Box::new(action)));
    }

    /// Requests an update to the Wayland compositor blur state and schedules a frame.
    pub fn set_background_blur(&self, enabled: bool) {
        self.state.borrow_mut().background_blur = Some(enabled);
        self.request_frame();
    }

    #[cfg(all(feature = "render-vello", feature = "surface"))]
    pub(crate) fn take_background_blur(&self) -> Option<bool> {
        self.state.borrow_mut().background_blur.take()
    }

    #[cfg(all(feature = "render-vello", feature = "surface"))]
    /// Request a rendering backend for the next frame. Requests before the same frame coalesce. An unavailable backend leaves the current renderer active and emits a warning diagnostic.
    pub fn set_renderer(&self, backend: crate::RenderBackend) {
        self.state.borrow_mut().renderer = Some(backend);
        self.request_frame();
    }

    #[cfg(all(feature = "render-vello", feature = "surface"))]
    pub(crate) fn take_renderer(&self) -> Option<crate::RenderBackend> {
        self.state.borrow_mut().renderer.take()
    }

    /// Read the latest state of the ordinary desktop window. Returns `None` before
    /// the window opens, after it closes, and for applications that only use a layer
    /// surface. A window action requests a change, and the read reflects the state
    /// reported by the window backend.
    #[must_use]
    pub fn window_state(&self) -> Option<WindowState> {
        self.state.borrow().window_state
    }

    /// Returns the window's client-area origin in desktop physical pixels when the
    /// backend exposes it. Returns none after closing or when the origin is unavailable.
    #[must_use]
    pub fn window_screen_origin(&self) -> Option<PointIn<ScreenSpace>> {
        self.state.borrow().window_screen_origin
    }

    #[cfg(any(test, all(feature = "render-vello", feature = "window-winit")))]
    pub(crate) fn update_window_screen_origin(&self, origin: Option<PointIn<ScreenSpace>>) {
        self.state.borrow_mut().window_screen_origin = origin;
    }

    /// Returns the latest observed placement, including after the window closes.
    #[must_use]
    pub fn capture_window_placement(&self) -> Option<WindowPlacement> {
        self.state.borrow().window_placement
    }

    /// Drains standalone window lifecycle notifications from this shared handle.
    /// Keep a clone before calling `App::run` to read notifications after it returns.
    #[must_use]
    pub fn take_lifecycle_events(&self) -> Vec<LifecycleEvent> {
        std::mem::take(&mut self.state.borrow_mut().lifecycle_events)
    }

    #[cfg(any(test, all(feature = "render-vello", feature = "window-winit")))]
    /// Makes runtime lifecycle events visible to the application's control handle.
    pub(crate) fn publish_lifecycle_events(&self, events: Vec<LifecycleEvent>) {
        self.state.borrow_mut().lifecycle_events.extend(events);
    }

    #[cfg(any(test, all(feature = "render-vello", feature = "window-winit")))]
    pub(crate) fn update_window_state(&self, next: Option<WindowState>) -> bool {
        let mut state = self.state.borrow_mut();
        if next.is_none() {
            state.window_screen_origin = None;
        }
        if state.window_state == next {
            return false;
        }
        if let Some(window) = next {
            state.window_placement = WindowPlacement::observe(state.window_placement, window);
        }
        state.window_state = next;
        true
    }

    #[doc = include_str!("../../docs/api/control/app_control.doc/request_window_action.md")]
    #[cfg(all(feature = "render-vello", feature = "window-winit"))]
    pub fn request_window_action(&self, action: WindowAction) {
        self.state.borrow_mut().window_actions.push(action);
    }

    #[cfg(all(feature = "render-vello", feature = "window-winit"))]
    pub(crate) fn take_window_actions(&self) -> Vec<WindowAction> {
        std::mem::take(&mut self.state.borrow_mut().window_actions)
    }

    #[doc = include_str!("../../docs/api/control/app_control.doc/request_frame.md")]
    pub fn request_frame(&self) {
        self.push(ControlRequest::Frame);
    }

    #[doc = include_str!("../../docs/api/control/app_control.doc/has_pending.md")]
    #[must_use]
    pub fn has_pending(&self) -> bool {
        !self.state.borrow().requests.is_empty()
    }

    #[doc = include_str!("../../docs/api/control/app_control.doc/pending_bindings.md")]
    #[must_use]
    pub fn pending_bindings(&self) -> usize {
        self.state.borrow().bindings.pending()
    }

    /// How many positional registrations this handle holds, resolved or not.
    #[must_use]
    pub fn binding_count(&self) -> usize {
        self.state.borrow().bindings.len()
    }

    #[doc = include_str!("../../docs/api/control/app_control.doc/bound_node.md")]
    #[must_use]
    pub fn bound_node(&self, target: &SelectorTarget) -> Option<NodeId> {
        self.state.borrow().bindings.bound_node(target)
    }

    #[doc = include_str!("../../docs/api/control/app_control.doc/apply.md")]
    pub fn apply(&self, runtime: &mut UiRuntime, css: &mut CssPipeline) -> usize {
        // Drain first and release the borrow: an action may queue more work, and a
        // handle held by application code must not find the queue locked.
        let requests = std::mem::take(&mut self.state.borrow_mut().requests);
        let applied = requests.len();
        let mut stylesheets = None;
        for request in requests {
            match request {
                // Only one set can be current, so several replacements in one turn
                // cost one install rather than one each.
                ControlRequest::Stylesheets(sheets) => stylesheets = Some(sheets),
                ControlRequest::Runtime(action) => action(runtime),
                // A style/state change the runtime did not cause is still a UI
                // commit: the next due frame rebuilds and restyles from it.
                ControlRequest::Frame => {
                    runtime.request_frame(FrameReason::UiCommit);
                }
            }
        }
        if let Some(sheets) = stylesheets {
            let revision = {
                let mut state = self.state.borrow_mut();
                state.revision = state.revision.next();
                state.revision
            };
            css.install(runtime, &sources(&sheets, revision));
            runtime.request_frame(FrameReason::UiCommit);
        }
        applied + self.state.borrow_mut().bindings.apply(runtime)
    }

    /// Queues one request.
    fn push(&self, request: ControlRequest) {
        self.state.borrow_mut().requests.push(request);
    }
}

impl std::fmt::Debug for AppControl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = self.state.borrow();
        f.debug_struct("AppControl")
            .field("queued", &state.requests.len())
            .field("bindings", &state.bindings)
            .field("revision", &state.revision.raw())
            .finish()
    }
}

#[doc = include_str!("../../docs/api/control/app_control.doc/sources.md")]
fn sources(sheets: &[String], revision: SourceVersion) -> Vec<CssSource> {
    sheets
        .iter()
        .enumerate()
        .map(|(index, text)| {
            let position = u32::try_from(index + 1).unwrap_or(u32::MAX);
            CssSource::new(
                SourceId::from_raw(position),
                format!("app-{position}.css"),
                text.as_str(),
            )
            .at_version(revision)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{AppControl, SourceVersion, sources};
    use crate::bootstrap::CssPipeline;
    use crate::css::source::SourceId;
    use crate::geometry::Size;
    use crate::identity::RuntimeId;
    use crate::runtime::UiRuntime;

    fn runtime() -> UiRuntime {
        UiRuntime::new(RuntimeId::from_raw(1), Size::new(100.0, 100.0))
    }

    #[cfg(all(feature = "window-winit", target_os = "linux"))]
    #[test]
    fn system_capture_protection_is_unavailable_on_linux() {
        use crate::capability::SupportLevel;

        let control = AppControl::new();
        assert_eq!(
            control.set_window_capture_protected(true),
            SupportLevel::Unavailable
        );
        assert_eq!(control.take_window_capture_protection(), None);
        assert!(!control.has_pending());
    }

    #[test]
    fn capture_checks_protection_before_calling_the_provider() {
        use std::{cell::RefCell, rc::Rc};

        use crate::geometry::{Point, Rect};
        use crate::system::capture::{
            CaptureProvider, CaptureRequest, CaptureResult, CaptureSource, CapturedImage,
        };

        #[derive(Default)]
        struct Provider {
            protects: bool,
            last: Option<CaptureRequest>,
        }

        impl CaptureProvider for Provider {
            fn capture(&mut self, request: CaptureRequest) -> CaptureResult {
                self.last = Some(request);
                CaptureResult::Captured(
                    CapturedImage::new(Size::new(1.0, 1.0), "rgba8").with_bytes(vec![0, 0, 0, 0]),
                )
            }

            fn can_capture(&self, source: &CaptureSource) -> bool {
                matches!(source, CaptureSource::Surface)
            }

            fn protects_regions(&self, source: &CaptureSource) -> bool {
                self.protects && self.can_capture(source)
            }
        }

        let control = AppControl::new();
        assert_eq!(
            control.capture(CaptureRequest::surface()),
            CaptureResult::Unsupported
        );
        let provider = Rc::new(RefCell::new(Provider::default()));
        control.set_capture_provider(Some(provider.clone()));
        let region = Rect::new(Point::ORIGIN, Size::new(1.0, 1.0));
        assert!(control.can_capture(&CaptureSource::Surface));
        assert_eq!(
            control.capture(CaptureRequest::region(region)),
            CaptureResult::Unsupported
        );
        let request = CaptureRequest::surface().protecting(region);
        assert_eq!(control.capture(request.clone()), CaptureResult::Denied);
        assert!(provider.borrow().last.is_none());

        provider.borrow_mut().protects = true;
        let result = control.capture(request.clone());
        assert_eq!(result.image().unwrap().bytes(), &[0, 0, 0, 0]);
        assert_eq!(provider.borrow().last.as_ref(), Some(&request));
        control.set_capture_protected_regions(vec![region]);
        provider.borrow_mut().protects = false;
        assert_eq!(
            control.capture(CaptureRequest::surface()),
            CaptureResult::Denied
        );
    }

    #[cfg(all(feature = "render-vello", feature = "window-winit"))]
    #[test]
    fn clones_request_feedback_and_regions_through_the_current_adapters() {
        use std::cell::Cell;
        use std::rc::Rc;

        use crate::geometry::{Point, Rect};
        use crate::interaction::feedback::{
            Feedback, FeedbackEffect, HapticFeedback, SoundFeedback,
        };
        use crate::interaction::gesture::SystemGestureRegions;

        struct Effects(Rc<Cell<Option<FeedbackEffect>>>);

        impl HapticFeedback for Effects {
            fn play(&self, effect: FeedbackEffect) {
                self.0.set(Some(effect));
            }
        }

        impl SoundFeedback for Effects {
            fn play(&self, effect: FeedbackEffect) {
                self.0.set(Some(effect));
            }
        }

        struct Regions(Rect, Rc<Cell<usize>>);

        impl SystemGestureRegions for Regions {
            fn reserved_regions(&self) -> Vec<Rect> {
                vec![self.0]
            }

            fn set_exclusion_regions(&self, regions: &[Rect]) -> bool {
                self.1.set(regions.len());
                true
            }
        }

        let host = AppControl::new();
        let app = host.clone();
        assert_eq!(app.reserved_gesture_regions(), None);
        assert!(!app.request_gesture_exclusions(&[]));
        app.play_haptic(FeedbackEffect::Tick);

        let haptic = Rc::new(Cell::new(None));
        let sound = Rc::new(Cell::new(None));
        host.set_feedback(
            Feedback::default()
                .with_haptics(Effects(Rc::clone(&haptic)))
                .with_sound(Effects(Rc::clone(&sound))),
        );
        app.play_haptic(FeedbackEffect::Tick);
        app.play_sound(FeedbackEffect::Confirm);
        assert_eq!(haptic.get(), Some(FeedbackEffect::Tick));
        assert_eq!(sound.get(), Some(FeedbackEffect::Confirm));

        let edge = Rect::new(Point::new(0.0, 0.0), Size::new(20.0, 100.0));
        let count = Rc::new(Cell::new(0));
        host.set_system_gesture_regions(Some(Rc::new(Regions(edge, Rc::clone(&count)))));
        assert_eq!(app.reserved_gesture_regions(), Some(vec![edge]));
        assert!(app.request_gesture_exclusions(&[edge]));
        assert_eq!(count.get(), 1);
        assert!(app.request_gesture_exclusions(&[]));
        assert_eq!(count.get(), 0);
        host.set_system_gesture_regions(None);
        assert_eq!(app.reserved_gesture_regions(), None);
    }

    #[test]
    fn window_screen_origin_is_cleared_when_the_window_closes() {
        let control = AppControl::new();
        let origin = super::PointIn::new(crate::geometry::Point::new(-1920.0, 32.0));
        assert_eq!(control.window_screen_origin(), None);
        control.update_window_screen_origin(Some(origin));
        assert_eq!(control.clone().window_screen_origin(), Some(origin));
        control.update_window_state(None);
        assert_eq!(control.window_screen_origin(), None);
    }

    #[test]
    fn window_placement_retains_normal_size_after_maximizing_and_closing() {
        let control = AppControl::new();
        let normal = crate::WindowState {
            size: Size::new(780.0, 580.0),
            scale_factor: 1.0,
            maximized: false,
            fullscreen: false,
            minimized: Some(false),
            focused: true,
            resizable: true,
        };
        assert!(control.update_window_state(Some(normal)));
        assert!(control.update_window_state(Some(crate::WindowState {
            size: Size::new(1500.0, 900.0),
            maximized: true,
            ..normal
        })));
        assert!(control.update_window_state(None));
        let placement = control.capture_window_placement().unwrap();
        assert_eq!(placement.mode(), crate::WindowMode::Maximized);
        assert_eq!(placement.normal_size(), Some((780.0, 580.0)));
        let encoded = serde_json::to_string(&placement).unwrap();
        assert_eq!(
            serde_json::from_str::<crate::WindowPlacement>(&encoded).unwrap(),
            placement
        );
    }

    #[test]
    fn standalone_lifecycle_events_are_shared_and_drained_once() {
        let control = AppControl::new();
        let application = control.clone();
        let window = crate::identity::WindowId::from_raw(1);
        let event = crate::runtime::lifecycle::LifecycleEvent::LastWindowClosed(window);
        control.publish_lifecycle_events(vec![event]);
        assert_eq!(application.take_lifecycle_events(), [event]);
        assert!(control.take_lifecycle_events().is_empty());
    }

    #[test]
    fn the_control_reports_the_latest_state_to_all_clones_until_close() {
        let control = AppControl::new();
        let app_handle = control.clone();
        let initial = crate::WindowState {
            size: Size::new(780.0, 580.0),
            scale_factor: 2.0,
            maximized: false,
            fullscreen: false,
            minimized: None,
            focused: true,
            resizable: true,
        };

        assert_eq!(app_handle.window_state(), None);
        assert!(control.update_window_state(Some(initial)));
        assert!(!control.update_window_state(Some(initial)));
        let maximized = crate::WindowState {
            maximized: true,
            ..initial
        };
        assert!(control.update_window_state(Some(maximized)));
        assert_eq!(app_handle.window_state(), Some(maximized));
        assert!(control.update_window_state(None));
        assert_eq!(app_handle.window_state(), None);
    }

    #[test]
    fn authored_sheets_are_identified_and_versioned() {
        // The §4.12 key: an anonymous source is never cached, so identity is the
        // difference between a cache that works and one that cannot.
        let built = sources(
            &[".a {}".to_owned(), ".b {}".to_owned()],
            SourceVersion::INITIAL.next(),
        );
        assert_eq!(built[0].id(), SourceId::from_raw(1));
        assert_eq!(built[1].id(), SourceId::from_raw(2));
        assert!(
            built
                .iter()
                .all(|source| source.id() != SourceId::ANONYMOUS)
        );
        assert!(
            built
                .iter()
                .all(|source| source.version() == SourceVersion::INITIAL.next())
        );
    }

    #[test]
    fn a_replacement_bumps_the_revision_rather_than_growing_the_cache() {
        let mut runtime = runtime();
        let mut css = CssPipeline::new();
        let control = AppControl::new();

        control.set_stylesheet(".box { width: 10px }");
        control.apply(&mut runtime, &mut css);
        assert_eq!(css.cached_sources(), 1, "the sheet is cached at all");

        control.set_stylesheet(".box { width: 20px }");
        control.apply(&mut runtime, &mut css);
        assert_eq!(
            css.cached_sources(),
            1,
            "the same position replaced its entry instead of adding one"
        );
    }

    #[test]
    fn several_replacements_in_one_turn_collapse_to_the_last() {
        let mut runtime = runtime();
        let mut css = CssPipeline::new();
        let control = AppControl::new();
        control.set_stylesheets(vec![".a {}".to_owned(), ".b {}".to_owned()]);
        control.set_stylesheet(".only {}");
        control.apply(&mut runtime, &mut css);
        assert_eq!(css.sheets().len(), 1, "the last set is the current one");
    }

    #[test]
    fn a_queued_action_runs_once_and_only_on_apply() {
        use std::cell::Cell;
        use std::rc::Rc;

        let mut runtime = runtime();
        let mut css = CssPipeline::new();
        let control = AppControl::new();
        let runs = Rc::new(Cell::new(0_u32));
        let counter = Rc::clone(&runs);
        control.with_runtime(move |_runtime| counter.set(counter.get() + 1));

        assert_eq!(runs.get(), 0, "queued, not run");
        assert!(control.has_pending());
        control.apply(&mut runtime, &mut css);
        assert_eq!(runs.get(), 1);
        control.apply(&mut runtime, &mut css);
        assert_eq!(runs.get(), 1, "a request is applied once, not every frame");
        assert!(!control.has_pending());
    }

    #[test]
    fn an_action_may_queue_more_work_without_locking_the_queue() {
        // The re-entrancy property the type promises: the queue borrow is released
        // before any action runs, so an action that queues is not a panic.
        let mut runtime = runtime();
        let mut css = CssPipeline::new();
        let control = AppControl::new();
        let inner = control.clone();
        control.with_runtime(move |_runtime| inner.request_frame());
        control.apply(&mut runtime, &mut css);
        assert!(
            control.has_pending(),
            "the nested request landed on the next turn's queue"
        );
    }

    #[test]
    fn every_clone_names_one_queue() {
        let control = AppControl::new();
        control.clone().request_frame();
        assert!(control.has_pending());
    }
}
