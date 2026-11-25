//! Optional Linux gamepad input source for a window host.

use std::sync::mpsc::{self, Sender, TryRecvError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use gilrs::{Axis, Button, EventType, GilrsBuilder};
use obscur_interaction::gamepad::{
    GamepadAxis, GamepadButton, GamepadEvent, GamepadEventKind, GamepadId,
};

/// Reads normalized reports on a worker and forwards them through the host callback. Dropping the source stops the worker.
pub struct GamepadSource {
    stop: Sender<()>,
    worker: Option<JoinHandle<()>>,
}

impl GamepadSource {
    /// Starts the gamepad worker. The callback returns false to stop delivery. Hardware initialization errors end the worker.
    pub fn spawn(
        mut deliver: impl FnMut(GamepadEvent) -> bool + Send + 'static,
    ) -> std::io::Result<Self> {
        let (stop, stopped) = mpsc::channel();
        let worker = thread::Builder::new()
            .name("obscur-gamepad".into())
            .spawn(move || {
                let mut gamepads = match GilrsBuilder::new().with_force_feedback(false).build() {
                    Ok(gamepads) => gamepads,
                    Err(error) => {
                        eprintln!("Gamepad input unavailable: {error}");
                        return;
                    }
                };
                for (id, _) in gamepads.gamepads() {
                    if !deliver(GamepadEvent::new(
                        gamepad_id(id),
                        GamepadEventKind::Connected,
                    )) {
                        return;
                    }
                }
                while matches!(stopped.try_recv(), Err(TryRecvError::Empty)) {
                    let Some(event) = gamepads.next_event_blocking(Some(Duration::from_millis(50)))
                    else {
                        continue;
                    };
                    let Some(kind) = event_kind(event.event) else {
                        continue;
                    };
                    if !deliver(GamepadEvent::new(gamepad_id(event.id), kind)) {
                        return;
                    }
                }
            })?;
        Ok(Self {
            stop,
            worker: Some(worker),
        })
    }
}

impl Drop for GamepadSource {
    fn drop(&mut self) {
        let _ = self.stop.send(());
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn gamepad_id(id: gilrs::GamepadId) -> GamepadId {
    GamepadId::from_raw(usize::from(id) as u64)
}

fn event_kind(event: EventType) -> Option<GamepadEventKind> {
    match event {
        EventType::Connected => Some(GamepadEventKind::Connected),
        EventType::Disconnected => Some(GamepadEventKind::Disconnected),
        EventType::ButtonPressed(button, _) => button_of(button).map(GamepadEventKind::ButtonDown),
        EventType::ButtonReleased(button, _) => button_of(button).map(GamepadEventKind::ButtonUp),
        EventType::AxisChanged(axis, value, _) if value.is_finite() => {
            Some(GamepadEventKind::AxisMotion {
                axis: axis_of(axis)?,
                value,
            })
        }
        _ => None,
    }
}

fn button_of(button: Button) -> Option<GamepadButton> {
    Some(match button {
        Button::South => GamepadButton::South,
        Button::East => GamepadButton::East,
        Button::West => GamepadButton::West,
        Button::North => GamepadButton::North,
        Button::DPadUp => GamepadButton::DpadUp,
        Button::DPadDown => GamepadButton::DpadDown,
        Button::DPadLeft => GamepadButton::DpadLeft,
        Button::DPadRight => GamepadButton::DpadRight,
        Button::LeftTrigger => GamepadButton::LeftShoulder,
        Button::RightTrigger => GamepadButton::RightShoulder,
        Button::LeftTrigger2 => GamepadButton::LeftTrigger,
        Button::RightTrigger2 => GamepadButton::RightTrigger,
        Button::LeftThumb => GamepadButton::LeftStick,
        Button::RightThumb => GamepadButton::RightStick,
        Button::Start => GamepadButton::Start,
        Button::Select => GamepadButton::Select,
        Button::Mode => GamepadButton::Guide,
        _ => return None,
    })
}

fn axis_of(axis: Axis) -> Option<GamepadAxis> {
    Some(match axis {
        Axis::LeftStickX => GamepadAxis::LeftStickX,
        Axis::LeftStickY => GamepadAxis::LeftStickY,
        Axis::RightStickX => GamepadAxis::RightStickX,
        Axis::RightStickY => GamepadAxis::RightStickY,
        Axis::LeftZ => GamepadAxis::LeftTrigger,
        Axis::RightZ => GamepadAxis::RightTrigger,
        _ => return None,
    })
}
