//! Dispatch instructions from the application to the native window.

use obscur_diagnostics::{Diagnostic, DiagnosticCode, DiagnosticSeverity, DiagnosticSubsystem};
use winit::event_loop::ActiveEventLoop;
use winit::window::{ResizeDirection, WindowId};

use super::AppRunner;
use crate::interaction::pointer::{PointerButton, PointerEvent, PointerEventKind};
use crate::runtime::lifecycle::ShutdownRequest;
use crate::{WindowAction, WindowResizeDirection};

impl AppRunner {
    /// Records the close request before applying the standalone host's exit policy.
    pub(super) fn request_window_close(&mut self, event_loop: &ActiveEventLoop, window: WindowId) {
        let window = crate::identity::WindowId::from_raw(window.into());
        self.runtime
            .lifecycle_mut()
            .request_shutdown(ShutdownRequest::WindowClose(window));
        event_loop.exit();
    }

    /// Drain queued native window commands. Returns true when a close command exits the loop.
    pub(super) fn apply_window_actions(
        &mut self,
        event_loop: &ActiveEventLoop,
        mut primary_press: Option<PointerEvent>,
    ) -> bool {
        let Some(window) = &self.window else {
            return false;
        };
        for action in self.control.take_window_actions() {
            match action {
                WindowAction::Minimize => window.set_minimized(true),
                WindowAction::ToggleMaximized => window.set_maximized(!window.is_maximized()),
                WindowAction::Close => {
                    self.request_window_close(event_loop, window.id());
                    return true;
                }
                WindowAction::BeginResize(_) if !self.resize => {}
                WindowAction::BeginMove | WindowAction::BeginResize(_) => {
                    let Some(press) = primary_press else {
                        self.runtime.report_diagnostic(&Diagnostic::new(
                            DiagnosticCode::new("window.interactive-action.no-press"),
                            DiagnosticSeverity::Warning,
                            DiagnosticSubsystem::Platform,
                            "Interactive window movement and resizing require a primary mouse press.",
                        ));
                        continue;
                    };
                    let result = if let WindowAction::BeginResize(direction) = action {
                        window.drag_resize_window(resize_direction(direction))
                    } else {
                        window.drag_window()
                    };
                    if let Err(error) = result {
                        self.runtime.report_diagnostic(&Diagnostic::new(
                            DiagnosticCode::new("window.interactive-action.failed"),
                            DiagnosticSeverity::Warning,
                            DiagnosticSubsystem::Platform,
                            format!("{action:?}: {error}"),
                        ));
                        continue;
                    }

                    // The window system may consume the release during its grab.
                    primary_press = None;
                    self.held_buttons.release(PointerButton::Primary);
                    self.runtime.handle_pointer(PointerEvent {
                        kind: PointerEventKind::Cancel,
                        buttons: self.held_buttons,
                        button: None,
                        pressure: 0.0,
                        ..press
                    });
                }
            }
        }
        false
    }
}

/// Convert an Obscur resize direction to the equivalent winit direction.
const fn resize_direction(direction: WindowResizeDirection) -> ResizeDirection {
    match direction {
        WindowResizeDirection::North => ResizeDirection::North,
        WindowResizeDirection::NorthEast => ResizeDirection::NorthEast,
        WindowResizeDirection::East => ResizeDirection::East,
        WindowResizeDirection::SouthEast => ResizeDirection::SouthEast,
        WindowResizeDirection::South => ResizeDirection::South,
        WindowResizeDirection::SouthWest => ResizeDirection::SouthWest,
        WindowResizeDirection::West => ResizeDirection::West,
        WindowResizeDirection::NorthWest => ResizeDirection::NorthWest,
    }
}
