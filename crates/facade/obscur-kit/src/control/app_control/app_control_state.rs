use std::cell::RefCell;
use std::rc::Rc;

use crate::css::source::SourceVersion;
use crate::geometry::Rect;
use crate::geometry::space::{PointIn, ScreenSpace};
use crate::interaction::feedback::Feedback;
use crate::interaction::gesture::SystemGestureRegions;
use crate::runtime::lifecycle::LifecycleEvent;
use crate::system::capture::CaptureProvider;
#[cfg(feature = "window-winit")]
use crate::system::capture::{CaptureRequest, CaptureResult};

#[cfg(all(feature = "render-vello", feature = "window-winit"))]
use super::super::WindowAction;
use super::super::{CommandBindings, ControlRequest, WindowPlacement, WindowState};

#[cfg(feature = "window-winit")]
pub(super) type FrameCapture = (CaptureRequest, Box<dyn FnOnce(CaptureResult)>);

/// Everything the handle owns, behind one shared cell.
#[derive(Default)]
pub(super) struct AppControlState {
    pub(super) feedback: Feedback,
    pub(super) capture_provider: Option<Rc<RefCell<dyn CaptureProvider>>>,
    pub(super) capture_protected_regions: Vec<Rect>,
    #[cfg(feature = "window-winit")]
    pub(super) window_capture_protection: Option<bool>,
    #[cfg(feature = "window-winit")]
    pub(super) frame_captures: Vec<FrameCapture>,
    pub(super) system_gesture_regions: Option<Rc<dyn SystemGestureRegions>>,
    pub(super) background_blur: Option<bool>,
    #[cfg(all(feature = "render-vello", feature = "surface"))]
    pub(super) renderer: Option<crate::RenderBackend>,
    #[cfg(all(feature = "render-vello", feature = "window-winit"))]
    pub(super) window_actions: Vec<WindowAction>,
    pub(super) window_state: Option<WindowState>,
    /// Latest client-area origin published by the window backend.
    pub(super) window_screen_origin: Option<PointIn<ScreenSpace>>,
    pub(super) window_placement: Option<WindowPlacement>,
    /// Standalone window lifecycle notifications available to the application.
    pub(super) lifecycle_events: Vec<LifecycleEvent>,
    /// Queued instructions, in the order they were asked for.
    pub(super) requests: Vec<ControlRequest>,
    /// Positional registrations and the nodes they are attached to.
    pub(super) bindings: CommandBindings,
    /// The revision the *next* stylesheet set is stamped with (spec §4.12).
    pub(super) revision: SourceVersion,
}
