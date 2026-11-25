//! Wayland keyboard delivery for a layer surface.

use obscur_interaction::command::{
    KeyEvent, LogicalKey, Modifier, Modifiers, NamedKey, PhysicalKey,
};
use obscur_windowing::layer::LayerPopupEvent;
use smithay_client_toolkit::seat::keyboard::{
    KeyEvent as NativeKey, KeyboardHandler, Keysym, Modifiers as NativeModifiers,
};
use wayland_client::protocol::{wl_keyboard::WlKeyboard, wl_seat::WlSeat, wl_surface::WlSurface};
use wayland_client::{Connection, Proxy, QueueHandle};

use super::LayerState;
use crate::event::LayerInput;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum KeyboardTarget {
    Panel,
    Popup,
}

impl LayerState {
    fn owns_keyboard(&self, keyboard: &WlKeyboard) -> bool {
        self.keyboard
            .as_ref()
            .is_some_and(|(_, active)| active == keyboard)
    }

    fn keyboard_target_for(&self, surface: &WlSurface) -> Option<KeyboardTarget> {
        if self.is_panel_surface(surface) {
            Some(KeyboardTarget::Panel)
        } else if self
            .popup
            .as_ref()
            .is_some_and(|popup| popup.surface() == surface)
        {
            Some(KeyboardTarget::Popup)
        } else {
            None
        }
    }

    pub(super) fn keyboard_route(&self) -> Option<KeyboardTarget> {
        // A layer-shell compositor can leave native keyboard focus on the panel
        // during the grab. The active menu still owns that input within the kit.
        match self.keyboard_target {
            Some(KeyboardTarget::Panel)
                if self
                    .popup
                    .as_ref()
                    .is_some_and(|popup| popup.grabbed && popup.mapped) =>
            {
                Some(KeyboardTarget::Popup)
            }
            target => target,
        }
    }

    fn keyboard_focus_changed(&mut self, target: KeyboardTarget, focused: bool) {
        let modifiers = if focused {
            self.seat_modifiers
        } else {
            Modifiers::new()
        };
        match target {
            KeyboardTarget::Panel => {
                self.push_input(LayerInput::Modifiers(modifiers));
                self.push_input(LayerInput::KeyboardFocus(focused));
            }
            KeyboardTarget::Popup => {
                self.popup_events
                    .push(LayerPopupEvent::Modifiers(modifiers));
                self.popup_events
                    .push(LayerPopupEvent::KeyboardFocus(focused));
            }
        }
    }

    pub(super) fn keyboard_route_changed(&mut self, previous: Option<KeyboardTarget>) {
        let next = self.keyboard_route();
        if previous != next {
            if let Some(previous) = previous {
                self.keyboard_focus_changed(previous, false);
            }
            if let Some(next) = next {
                self.keyboard_focus_changed(next, true);
            }
        }
    }

    fn set_keyboard_target(&mut self, next: Option<KeyboardTarget>) {
        if self.keyboard_target != next {
            let previous = self.keyboard_route();
            self.keyboard_target = next;
            self.seat_modifiers = Modifiers::new();
            self.keyboard_route_changed(previous);
        }
    }

    pub(super) fn popup_keyboard_closed(&mut self, previous: Option<KeyboardTarget>) {
        if self.keyboard_target == Some(KeyboardTarget::Popup) {
            self.keyboard_target = None;
            self.seat_modifiers = Modifiers::new();
        }
        self.keyboard_route_changed(previous);
    }

    pub(super) fn bind_keyboard(&mut self, qh: &QueueHandle<Self>, seat: WlSeat) {
        if self.keyboard.is_some() {
            return;
        }
        let Some(loop_handle) = self.keyboard_loop.clone() else {
            return;
        };
        if let Ok(keyboard) = self.seat_state.get_keyboard_with_repeat(
            qh,
            &seat,
            None,
            loop_handle,
            Box::new(|state: &mut Self, keyboard, event| {
                state.emit_key(keyboard, event, true, true, None);
            }),
        ) {
            self.ime.bind_seat(&seat, qh);
            self.keyboard = Some((seat, keyboard));
        }
    }

    pub(super) fn remove_keyboard(&mut self, qh: &QueueHandle<Self>, seat: &WlSeat) {
        if !self
            .keyboard
            .as_ref()
            .is_some_and(|(owner, _)| owner == seat)
        {
            return;
        }
        self.set_keyboard_target(None);
        self.consuming_menu_escape = false;
        self.ime.remove_seat();
        if let Some((_, keyboard)) = self.keyboard.take()
            && keyboard.version() >= 3
        {
            keyboard.release();
        }
        let replacement = self.seat_state.seats().find(|other| {
            other != seat
                && self
                    .seat_state
                    .info(other)
                    .is_some_and(|info| info.has_keyboard)
        });
        if let Some(replacement) = replacement {
            self.bind_keyboard(qh, replacement);
        }
    }

    fn emit_key(
        &mut self,
        keyboard: &WlKeyboard,
        key: NativeKey,
        pressed: bool,
        repeat: bool,
        serial: Option<u32>,
    ) {
        if !self.owns_keyboard(keyboard) {
            return;
        }
        let Some(event) = convert_key(&key, pressed, repeat) else {
            return;
        };
        if event.logical() == Some(LogicalKey::named(NamedKey::Escape)) {
            if !pressed && core::mem::take(&mut self.consuming_menu_escape) {
                return;
            }
            if repeat && self.consuming_menu_escape {
                return;
            }
            if pressed && !repeat {
                self.consuming_menu_escape = self.keyboard_target.is_some()
                    && self.popup.as_ref().is_some_and(|popup| popup.grabbed);
                if self.consuming_menu_escape {
                    self.dismiss_popup();
                    return;
                }
            }
        }
        let Some(target) = self.keyboard_route() else {
            return;
        };
        match target {
            KeyboardTarget::Panel => {
                self.push_input(LayerInput::Key(event));
                if let (Some(serial), Some((seat, _))) = (serial, &self.keyboard) {
                    let seat = seat.clone();
                    self.mark_input_activation(&seat, serial);
                }
            }
            KeyboardTarget::Popup => self.popup_events.push(LayerPopupEvent::Key(event)),
        }
    }
}

impl KeyboardHandler for LayerState {
    fn enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        keyboard: &WlKeyboard,
        surface: &WlSurface,
        _: u32,
        _: &[u32],
        _: &[Keysym],
    ) {
        if self.owns_keyboard(keyboard) {
            self.set_keyboard_target(self.keyboard_target_for(surface));
        }
    }

    fn leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        keyboard: &WlKeyboard,
        surface: &WlSurface,
        _: u32,
    ) {
        if self.owns_keyboard(keyboard) && self.keyboard_target == self.keyboard_target_for(surface)
        {
            self.set_keyboard_target(None);
        }
    }

    fn press_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        keyboard: &WlKeyboard,
        serial: u32,
        key: NativeKey,
    ) {
        self.emit_key(keyboard, key, true, false, Some(serial));
    }

    fn release_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        keyboard: &WlKeyboard,
        _: u32,
        key: NativeKey,
    ) {
        self.emit_key(keyboard, key, false, false, None);
    }

    fn update_modifiers(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        keyboard: &WlKeyboard,
        _: u32,
        modifiers: NativeModifiers,
        _: u32,
    ) {
        if self.owns_keyboard(keyboard)
            && let Some(target) = self.keyboard_route()
        {
            let mut value = Modifiers::new();
            for (active, modifier) in [
                (modifiers.ctrl, Modifier::Ctrl),
                (modifiers.alt, Modifier::Alt),
                (modifiers.shift, Modifier::Shift),
                (modifiers.logo, Modifier::Meta),
            ] {
                if active {
                    value = value.with(modifier);
                }
            }
            self.seat_modifiers = value;
            match target {
                KeyboardTarget::Panel => self.push_input(LayerInput::Modifiers(value)),
                KeyboardTarget::Popup => self.popup_events.push(LayerPopupEvent::Modifiers(value)),
            }
        }
    }
}

fn convert_key(key: &NativeKey, pressed: bool, repeat: bool) -> Option<KeyEvent> {
    let logical = logical_key(key);
    let physical =
        physical_key(key.raw_code).or_else(|| logical.map(|_| PhysicalKey::Unidentified))?;
    let event = KeyEvent::new(physical, logical, pressed)
        .with_timestamp(u64::from(key.time) * 1_000)
        .as_trusted();
    Some(if repeat { event.as_repeat() } else { event })
}

fn logical_key(key: &NativeKey) -> Option<LogicalKey> {
    let named = match key.keysym {
        Keysym::Return => NamedKey::Enter,
        Keysym::KP_Enter => NamedKey::Enter,
        Keysym::Escape => NamedKey::Escape,
        Keysym::Tab => NamedKey::Tab,
        Keysym::ISO_Left_Tab => NamedKey::Tab,
        Keysym::space => NamedKey::Space,
        Keysym::BackSpace => NamedKey::Backspace,
        Keysym::Delete => NamedKey::Delete,
        Keysym::Up => NamedKey::ArrowUp,
        Keysym::Down => NamedKey::ArrowDown,
        Keysym::Left => NamedKey::ArrowLeft,
        Keysym::Right => NamedKey::ArrowRight,
        Keysym::Home => NamedKey::Home,
        Keysym::End => NamedKey::End,
        Keysym::Page_Up => NamedKey::PageUp,
        Keysym::Page_Down => NamedKey::PageDown,
        Keysym::F1 => NamedKey::F1,
        Keysym::F2 => NamedKey::F2,
        Keysym::F3 => NamedKey::F3,
        Keysym::F4 => NamedKey::F4,
        Keysym::F5 => NamedKey::F5,
        Keysym::F6 => NamedKey::F6,
        Keysym::F7 => NamedKey::F7,
        Keysym::F8 => NamedKey::F8,
        Keysym::F9 => NamedKey::F9,
        Keysym::F10 => NamedKey::F10,
        Keysym::F11 => NamedKey::F11,
        Keysym::F12 => NamedKey::F12,
        _ => {
            let text = key.utf8.as_deref().and_then(|value| {
                let mut chars = value.chars();
                let first = chars.next()?;
                (chars.next().is_none() && !first.is_control()).then_some(first)
            });
            return text
                .or_else(|| key.keysym.key_char().filter(|c| !c.is_control()))
                .map(LogicalKey::character);
        }
    };
    Some(LogicalKey::named(named))
}

fn physical_key(code: u32) -> Option<PhysicalKey> {
    use PhysicalKey::*;
    Some(match code {
        1 => Escape,
        2 => Digit1,
        3 => Digit2,
        4 => Digit3,
        5 => Digit4,
        6 => Digit5,
        7 => Digit6,
        8 => Digit7,
        9 => Digit8,
        10 => Digit9,
        11 => Digit0,
        14 => Backspace,
        15 => Tab,
        16 => KeyQ,
        17 => KeyW,
        18 => KeyE,
        19 => KeyR,
        20 => KeyT,
        21 => KeyY,
        22 => KeyU,
        23 => KeyI,
        24 => KeyO,
        25 => KeyP,
        28 => Enter,
        29 => ControlLeft,
        30 => KeyA,
        31 => KeyS,
        32 => KeyD,
        33 => KeyF,
        34 => KeyG,
        35 => KeyH,
        36 => KeyJ,
        37 => KeyK,
        38 => KeyL,
        42 => ShiftLeft,
        44 => KeyZ,
        45 => KeyX,
        46 => KeyC,
        47 => KeyV,
        48 => KeyB,
        49 => KeyN,
        50 => KeyM,
        54 => ShiftRight,
        56 => AltLeft,
        57 => Space,
        59 => F1,
        60 => F2,
        61 => F3,
        62 => F4,
        63 => F5,
        64 => F6,
        65 => F7,
        66 => F8,
        67 => F9,
        68 => F10,
        87 => F11,
        88 => F12,
        96 => Enter,
        97 => ControlRight,
        100 => AltRight,
        102 => Home,
        103 => ArrowUp,
        104 => PageUp,
        105 => ArrowLeft,
        106 => ArrowRight,
        107 => End,
        108 => ArrowDown,
        109 => PageDown,
        111 => Delete,
        125 => MetaLeft,
        126 => MetaRight,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physical_position_and_logical_layout_are_independent() {
        let key = NativeKey {
            raw_code: 30,
            keysym: Keysym::from_char('ф'),
            utf8: Some("ф".into()),
            time: 19,
        };
        let event = convert_key(&key, true, true).unwrap();
        assert_eq!(event.physical(), PhysicalKey::KeyA);
        assert_eq!(event.logical(), Some(LogicalKey::character('ф')));
        assert!(event.is_trusted());
        assert!(event.is_repeat());
        assert_eq!(event.timestamp(), 19_000);
    }

    #[test]
    fn named_key_releases_and_typable_unidentified_keys_are_delivered() {
        let release = NativeKey {
            raw_code: 15,
            keysym: Keysym::ISO_Left_Tab,
            utf8: None,
            time: 0,
        };
        let event = convert_key(&release, false, false).unwrap();
        assert_eq!(event.logical(), Some(LogicalKey::named(NamedKey::Tab)));
        assert!(!event.pressed());
        let punctuation = NativeKey {
            raw_code: 39,
            keysym: Keysym::semicolon,
            ..release
        };
        let event = convert_key(&punctuation, true, false).unwrap();
        assert_eq!(event.physical(), PhysicalKey::Unidentified);
        assert_eq!(event.logical(), Some(LogicalKey::character(';')));
    }
}
