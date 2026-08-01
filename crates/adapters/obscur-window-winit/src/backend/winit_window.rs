#![doc = include_str!("winit_window.doc/_module.md")]

use std::sync::Arc;

use obscur_identity::WindowId;
use obscur_windowing::window::{WindowProperties, WindowState};
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::window::{Fullscreen, Window as NativeWindow, WindowLevel};

#[doc = include_str!("winit_window.doc/WinitWindow.md")]
#[derive(Debug, Clone)]
pub struct WinitWindow {
    #[doc = include_str!("winit_window.doc/id.md")]
    id: WindowId,
    #[doc = include_str!("winit_window.doc/window.md")]
    window: Arc<NativeWindow>,
}

impl WinitWindow {
    #[doc = include_str!("winit_window.doc/new.md")]
    #[must_use]
    pub fn new(id: WindowId, window: Arc<NativeWindow>) -> Self {
        Self { id, window }
    }

    #[doc = include_str!("winit_window.doc/id_2.md")]
    #[must_use]
    pub const fn id(&self) -> WindowId {
        self.id
    }

    #[doc = include_str!("winit_window.doc/winit.md")]
    #[must_use]
    pub fn winit(&self) -> &NativeWindow {
        &self.window
    }

    #[doc = include_str!("winit_window.doc/shared.md")]
    #[must_use]
    pub fn shared(&self) -> Arc<NativeWindow> {
        Arc::clone(&self.window)
    }

    #[doc = include_str!("winit_window.doc/apply_properties.md")]
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

    #[doc = include_str!("winit_window.doc/apply_state.md")]
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
