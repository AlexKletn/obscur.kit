//! The [`WinitWindow`] pairing of a domain [`WindowId`] with a live winit window
//! (spec §6.1 windowing; §6.2 dynamic properties).

use std::sync::Arc;

use obscur_identity::WindowId;
use obscur_windowing::window::{WindowProperties, WindowState};
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::window::{Fullscreen, Window as NativeWindow, WindowLevel};

/// A live platform window paired with the domain [`WindowId`] it materializes
/// (spec §6.1, §6.2).
///
/// The winit backend is callback-driven: real [`winit::window::Window`]s can
/// only be created inside an
/// [`ActiveEventLoop`](winit::event_loop::ActiveEventLoop) callback, well after
/// `create_window` has already returned a [`WindowId`]. A `WinitWindow` is the
/// record the event-loop handler stores once the OS window finally exists, keyed
/// by the id the backend minted up front. The native window is held behind an
/// [`Arc`] so a frame scheduler or renderer can share it cheaply across threads
/// without reopening the OS handle.
#[derive(Debug, Clone)]
pub struct WinitWindow {
    /// The domain identity this window materializes.
    id: WindowId,
    /// The live platform window, shareable across threads.
    window: Arc<NativeWindow>,
}

impl WinitWindow {
    /// Pairs a domain `id` with its freshly created native `window`.
    #[must_use]
    pub fn new(id: WindowId, window: Arc<NativeWindow>) -> Self {
        Self { id, window }
    }

    /// Returns the domain identity this window materializes.
    #[must_use]
    pub const fn id(&self) -> WindowId {
        self.id
    }

    /// Returns the underlying live winit window.
    #[must_use]
    pub fn winit(&self) -> &NativeWindow {
        &self.window
    }

    /// Reconciles the live winit window against the desired domain `props`
    /// (spec §6.2).
    ///
    /// Because the domain
    /// [`UpdateWindow`](obscur_windowing::request::UpdateWindow) delta exposes no
    /// field-level accessors, the backend instead resolves the delta against the
    /// [`Window`](obscur_windowing::window::Window) record it owns and hands the
    /// *resulting* full desired properties here. Pushing the full property set is
    /// idempotent, so applying it repeatedly is harmless.
    ///
    /// Every property winit can set is pushed: title, inner size, outer position
    /// (only when an explicit one is requested — a platform-chosen `None`
    /// position is left to the platform), resizability, server-side decorations,
    /// always-on-top window level, and presentation state. Transparency is fixed
    /// at creation time in winit and cannot be toggled afterwards, which is why
    /// [`WinitWindowBackend`]'s `update_window` never lets a changed
    /// `transparent` flag reach this far: it keeps the record on the live value
    /// and reports the update `conditional()`, rather than accepting a flip that
    /// would be silently dropped here.
    ///
    /// [`WinitWindowBackend`]: super::WinitWindowBackend
    pub fn apply_properties(&self, props: &WindowProperties) {
        self.window.set_title(&props.title);
        let _ = self.window.request_inner_size(LogicalSize::new(
            f64::from(props.size.width),
            f64::from(props.size.height),
        ));
        if let Some(position) = props.position {
            self.window.set_outer_position(LogicalPosition::new(
                f64::from(position.x),
                f64::from(position.y),
            ));
        }
        self.window.set_resizable(props.resizable);
        self.window.set_decorations(props.decorated);
        self.window.set_window_level(if props.always_on_top {
            WindowLevel::AlwaysOnTop
        } else {
            WindowLevel::Normal
        });
        self.apply_state(props.state);
    }

    /// Maps a domain [`WindowState`] onto winit's separate minimize / maximize /
    /// fullscreen toggles.
    ///
    /// The domain models presentation as one mutually-exclusive state, whereas
    /// winit exposes three independent setters; [`WindowState::Normal`] therefore
    /// clears all three, and each other variant sets exactly its own.
    /// [`WindowState::Fullscreen`] uses borderless fullscreen on the current
    /// monitor, the portable form that needs no monitor/video-mode handle.
    fn apply_state(&self, state: WindowState) {
        match state {
            WindowState::Normal => {
                self.window.set_minimized(false);
                self.window.set_maximized(false);
                self.window.set_fullscreen(None);
            }
            WindowState::Minimized => self.window.set_minimized(true),
            WindowState::Maximized => self.window.set_maximized(true),
            WindowState::Fullscreen => {
                self.window
                    .set_fullscreen(Some(Fullscreen::Borderless(None)));
            }
        }
    }
}
