//! Native keyboard and input-method events for the standalone window.

use winit::event::{ElementState, Ime, KeyEvent as NativeKeyEvent};
use winit::keyboard::PhysicalKey as NativePhysicalKey;

use super::AppRunner;
use crate::interaction::command::KeyEvent;
use crate::text::ime::{CompositionEvent, Preedit};
use crate::window_winit::conversion::KeyboardConverter;

impl AppRunner {
    pub(super) fn handle_native_key(&mut self, event: NativeKeyEvent) {
        let NativePhysicalKey::Code(code) = event.physical_key else {
            return;
        };
        let Some(physical) = KeyboardConverter::physical_key(code) else {
            return;
        };
        let logical = KeyboardConverter::logical_key(&event.logical_key);
        // The OS callback is the trust boundary for keyboard input.
        let key = KeyEvent::new(physical, logical, event.state == ElementState::Pressed)
            .with_timestamp(self.now_micros())
            .as_trusted();
        let key = if event.repeat { key.as_repeat() } else { key };
        self.runtime.handle_key(key);
        self.request_redraw();
    }

    pub(super) fn handle_native_ime(&mut self, ime: Ime) {
        let event = match ime {
            Ime::Enabled => CompositionEvent::Start,
            Ime::Preedit(text, cursor) => {
                let preedit = Preedit::new(text);
                CompositionEvent::Update(match cursor {
                    Some((start, _)) => preedit.with_caret(start),
                    None => preedit,
                })
            }
            Ime::Commit(text) => CompositionEvent::Commit(text.into()),
            Ime::Disabled => CompositionEvent::Cancel,
        };
        self.runtime.handle_ime(event);
        self.request_redraw();
    }
}
