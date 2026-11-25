//! The [`KeyboardConverter`] pure winit→domain key converter (spec §6.7
//! keyboard; §6.1 windowing; §2.7 "преобразует native events в доменные
//! события").

use obscur_interaction::command::{LogicalKey, Modifier, Modifiers, NamedKey, PhysicalKey};

#[doc = include_str!("../../docs/api/conversion/keyboard_converter.doc/KeyboardConverter.md")]
#[derive(Debug, Clone, Copy, Default)]
pub struct KeyboardConverter;

impl KeyboardConverter {
    /// Creates a converter.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[doc = include_str!("../../docs/api/conversion/keyboard_converter.doc/physical_key.md")]
    #[must_use]
    pub fn physical_key(code: winit::keyboard::KeyCode) -> Option<PhysicalKey> {
        use winit::keyboard::KeyCode as Kc;
        let mapped = match code {
            Kc::KeyA => PhysicalKey::KeyA,
            Kc::KeyB => PhysicalKey::KeyB,
            Kc::KeyC => PhysicalKey::KeyC,
            Kc::KeyD => PhysicalKey::KeyD,
            Kc::KeyE => PhysicalKey::KeyE,
            Kc::KeyF => PhysicalKey::KeyF,
            Kc::KeyG => PhysicalKey::KeyG,
            Kc::KeyH => PhysicalKey::KeyH,
            Kc::KeyI => PhysicalKey::KeyI,
            Kc::KeyJ => PhysicalKey::KeyJ,
            Kc::KeyK => PhysicalKey::KeyK,
            Kc::KeyL => PhysicalKey::KeyL,
            Kc::KeyM => PhysicalKey::KeyM,
            Kc::KeyN => PhysicalKey::KeyN,
            Kc::KeyO => PhysicalKey::KeyO,
            Kc::KeyP => PhysicalKey::KeyP,
            Kc::KeyQ => PhysicalKey::KeyQ,
            Kc::KeyR => PhysicalKey::KeyR,
            Kc::KeyS => PhysicalKey::KeyS,
            Kc::KeyT => PhysicalKey::KeyT,
            Kc::KeyU => PhysicalKey::KeyU,
            Kc::KeyV => PhysicalKey::KeyV,
            Kc::KeyW => PhysicalKey::KeyW,
            Kc::KeyX => PhysicalKey::KeyX,
            Kc::KeyY => PhysicalKey::KeyY,
            Kc::KeyZ => PhysicalKey::KeyZ,
            Kc::Digit0 => PhysicalKey::Digit0,
            Kc::Digit1 => PhysicalKey::Digit1,
            Kc::Digit2 => PhysicalKey::Digit2,
            Kc::Digit3 => PhysicalKey::Digit3,
            Kc::Digit4 => PhysicalKey::Digit4,
            Kc::Digit5 => PhysicalKey::Digit5,
            Kc::Digit6 => PhysicalKey::Digit6,
            Kc::Digit7 => PhysicalKey::Digit7,
            Kc::Digit8 => PhysicalKey::Digit8,
            Kc::Digit9 => PhysicalKey::Digit9,
            Kc::F1 => PhysicalKey::F1,
            Kc::F2 => PhysicalKey::F2,
            Kc::F3 => PhysicalKey::F3,
            Kc::F4 => PhysicalKey::F4,
            Kc::F5 => PhysicalKey::F5,
            Kc::F6 => PhysicalKey::F6,
            Kc::F7 => PhysicalKey::F7,
            Kc::F8 => PhysicalKey::F8,
            Kc::F9 => PhysicalKey::F9,
            Kc::F10 => PhysicalKey::F10,
            Kc::F11 => PhysicalKey::F11,
            Kc::F12 => PhysicalKey::F12,
            Kc::Escape => PhysicalKey::Escape,
            Kc::Enter => PhysicalKey::Enter,
            Kc::Space => PhysicalKey::Space,
            Kc::Tab => PhysicalKey::Tab,
            Kc::Backspace => PhysicalKey::Backspace,
            Kc::Delete => PhysicalKey::Delete,
            Kc::ArrowUp => PhysicalKey::ArrowUp,
            Kc::ArrowDown => PhysicalKey::ArrowDown,
            Kc::ArrowLeft => PhysicalKey::ArrowLeft,
            Kc::ArrowRight => PhysicalKey::ArrowRight,
            Kc::Home => PhysicalKey::Home,
            Kc::End => PhysicalKey::End,
            Kc::PageUp => PhysicalKey::PageUp,
            Kc::PageDown => PhysicalKey::PageDown,
            Kc::ShiftLeft => PhysicalKey::ShiftLeft,
            Kc::ShiftRight => PhysicalKey::ShiftRight,
            Kc::ControlLeft => PhysicalKey::ControlLeft,
            Kc::ControlRight => PhysicalKey::ControlRight,
            Kc::AltLeft => PhysicalKey::AltLeft,
            Kc::AltRight => PhysicalKey::AltRight,
            Kc::SuperLeft => PhysicalKey::MetaLeft,
            Kc::SuperRight => PhysicalKey::MetaRight,
            _ => return None,
        };
        Some(mapped)
    }

    #[doc = include_str!("../../docs/api/conversion/keyboard_converter.doc/modifiers.md")]
    #[must_use]
    pub fn modifiers(state: winit::keyboard::ModifiersState) -> Modifiers {
        let mut modifiers = Modifiers::new();
        for (held, modifier) in [
            (state.shift_key(), Modifier::Shift),
            (state.control_key(), Modifier::Ctrl),
            (state.alt_key(), Modifier::Alt),
            (state.super_key(), Modifier::Meta),
        ] {
            if held {
                modifiers = modifiers.with(modifier);
            }
        }
        modifiers
    }

    #[doc = include_str!("../../docs/api/conversion/keyboard_converter.doc/logical_key.md")]
    #[must_use]
    pub fn logical_key(key: &winit::keyboard::Key) -> Option<LogicalKey> {
        use winit::keyboard::{Key, NamedKey as Winit};
        match key {
            Key::Character(text) => text.chars().next().map(LogicalKey::character),
            Key::Named(named) => {
                let mapped = match named {
                    Winit::Enter => NamedKey::Enter,
                    Winit::Escape => NamedKey::Escape,
                    Winit::Tab => NamedKey::Tab,
                    Winit::Space => NamedKey::Space,
                    Winit::Backspace => NamedKey::Backspace,
                    Winit::Delete => NamedKey::Delete,
                    Winit::ArrowUp => NamedKey::ArrowUp,
                    Winit::ArrowDown => NamedKey::ArrowDown,
                    Winit::ArrowLeft => NamedKey::ArrowLeft,
                    Winit::ArrowRight => NamedKey::ArrowRight,
                    Winit::Home => NamedKey::Home,
                    Winit::End => NamedKey::End,
                    Winit::PageUp => NamedKey::PageUp,
                    Winit::PageDown => NamedKey::PageDown,
                    Winit::F1 => NamedKey::F1,
                    Winit::F2 => NamedKey::F2,
                    Winit::F3 => NamedKey::F3,
                    Winit::F4 => NamedKey::F4,
                    Winit::F5 => NamedKey::F5,
                    Winit::F6 => NamedKey::F6,
                    Winit::F7 => NamedKey::F7,
                    Winit::F8 => NamedKey::F8,
                    Winit::F9 => NamedKey::F9,
                    Winit::F10 => NamedKey::F10,
                    Winit::F11 => NamedKey::F11,
                    Winit::F12 => NamedKey::F12,
                    _ => return None,
                };
                Some(LogicalKey::named(mapped))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::keyboard::{Key, KeyCode, NamedKey as WinitNamed};

    #[test]
    fn letters_and_named_positions_map_across() {
        assert_eq!(
            KeyboardConverter::physical_key(KeyCode::KeyA),
            Some(PhysicalKey::KeyA)
        );
        assert_eq!(
            KeyboardConverter::physical_key(KeyCode::Enter),
            Some(PhysicalKey::Enter)
        );
        assert_eq!(
            KeyboardConverter::physical_key(KeyCode::ArrowUp),
            Some(PhysicalKey::ArrowUp)
        );
    }

    #[test]
    fn super_positions_map_to_meta() {
        assert_eq!(
            KeyboardConverter::physical_key(KeyCode::SuperLeft),
            Some(PhysicalKey::MetaLeft)
        );
        assert_eq!(
            KeyboardConverter::physical_key(KeyCode::SuperRight),
            Some(PhysicalKey::MetaRight)
        );
    }

    #[test]
    fn unmodeled_physical_key_is_none() {
        // A media/browser key outside the modeled subset.
        assert_eq!(
            KeyboardConverter::physical_key(KeyCode::MediaPlayPause),
            None
        );
    }

    #[test]
    fn character_key_maps_to_first_char() {
        assert_eq!(
            KeyboardConverter::logical_key(&Key::Character("a".into())),
            Some(LogicalKey::character('a'))
        );
    }

    #[test]
    fn named_key_maps_across() {
        assert_eq!(
            KeyboardConverter::logical_key(&Key::Named(WinitNamed::Enter)),
            Some(LogicalKey::named(NamedKey::Enter))
        );
    }

    #[test]
    fn unmodeled_named_key_is_none() {
        assert_eq!(
            KeyboardConverter::logical_key(&Key::Named(WinitNamed::MediaPlay)),
            None
        );
    }

    #[test]
    fn an_empty_modifier_state_converts_to_an_empty_chord() {
        assert!(KeyboardConverter::modifiers(winit::keyboard::ModifiersState::empty()).is_empty());
    }

    #[test]
    fn each_modifier_crosses_the_boundary_on_its_own() {
        use winit::keyboard::ModifiersState as State;
        for (state, modifier) in [
            (State::SHIFT, Modifier::Shift),
            (State::CONTROL, Modifier::Ctrl),
            (State::ALT, Modifier::Alt),
            (State::SUPER, Modifier::Meta),
        ] {
            let chord = KeyboardConverter::modifiers(state);
            assert!(chord.contains(modifier), "{modifier:?} did not cross");
            assert_eq!(
                chord,
                Modifiers::new().with(modifier),
                "{modifier:?} brought something else with it"
            );
        }
    }

    #[test]
    fn a_chord_converts_whole() {
        use winit::keyboard::ModifiersState as State;
        // The case the fold gets wrong after a focus loss, and the reason this
        // conversion exists: the platform states the whole chord at once.
        let chord = KeyboardConverter::modifiers(State::CONTROL | State::SHIFT);
        assert_eq!(
            chord,
            Modifiers::new().with(Modifier::Ctrl).with(Modifier::Shift)
        );
    }
}
