//! The [`PhysicalKey`] layout-independent key position (spec §6.7).

use crate::command::Modifier;

/// A layout-independent physical key position (spec §6.7).
///
/// A physical key names a scan position on the keyboard, independent of the
/// active layout: `KeyA` is the key where QWERTY prints `A`, whatever character
/// that key produces under the user's layout. Physical keys are the right
/// binding surface for spatial controls such as WASD movement, where the shape
/// of the layout matters more than the produced character. For text entry and
/// shortcuts, prefer [`LogicalKey`](crate::command::LogicalKey).
///
/// This is a closed enum covering a practical subset of keys (letters, digits,
/// function keys, editing and navigation keys, and the left/right modifier
/// pairs). It is non-exhaustive in spirit — real keyboards have more keys — but
/// deliberately closed so callers can match exhaustively.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhysicalKey {
    /// The `A` scan position.
    KeyA,
    /// The `B` scan position.
    KeyB,
    /// The `C` scan position.
    KeyC,
    /// The `D` scan position.
    KeyD,
    /// The `E` scan position.
    KeyE,
    /// The `F` scan position.
    KeyF,
    /// The `G` scan position.
    KeyG,
    /// The `H` scan position.
    KeyH,
    /// The `I` scan position.
    KeyI,
    /// The `J` scan position.
    KeyJ,
    /// The `K` scan position.
    KeyK,
    /// The `L` scan position.
    KeyL,
    /// The `M` scan position.
    KeyM,
    /// The `N` scan position.
    KeyN,
    /// The `O` scan position.
    KeyO,
    /// The `P` scan position.
    KeyP,
    /// The `Q` scan position.
    KeyQ,
    /// The `R` scan position.
    KeyR,
    /// The `S` scan position.
    KeyS,
    /// The `T` scan position.
    KeyT,
    /// The `U` scan position.
    KeyU,
    /// The `V` scan position.
    KeyV,
    /// The `W` scan position.
    KeyW,
    /// The `X` scan position.
    KeyX,
    /// The `Y` scan position.
    KeyY,
    /// The `Z` scan position.
    KeyZ,
    /// The `0` scan position on the main number row.
    Digit0,
    /// The `1` scan position on the main number row.
    Digit1,
    /// The `2` scan position on the main number row.
    Digit2,
    /// The `3` scan position on the main number row.
    Digit3,
    /// The `4` scan position on the main number row.
    Digit4,
    /// The `5` scan position on the main number row.
    Digit5,
    /// The `6` scan position on the main number row.
    Digit6,
    /// The `7` scan position on the main number row.
    Digit7,
    /// The `8` scan position on the main number row.
    Digit8,
    /// The `9` scan position on the main number row.
    Digit9,
    /// The `F1` function key.
    F1,
    /// The `F2` function key.
    F2,
    /// The `F3` function key.
    F3,
    /// The `F4` function key.
    F4,
    /// The `F5` function key.
    F5,
    /// The `F6` function key.
    F6,
    /// The `F7` function key.
    F7,
    /// The `F8` function key.
    F8,
    /// The `F9` function key.
    F9,
    /// The `F10` function key.
    F10,
    /// The `F11` function key.
    F11,
    /// The `F12` function key.
    F12,
    /// The `Escape` key.
    Escape,
    /// The `Enter` / `Return` key.
    Enter,
    /// The space bar.
    Space,
    /// The `Tab` key.
    Tab,
    /// The `Backspace` key.
    Backspace,
    /// The forward `Delete` key.
    Delete,
    /// The up arrow key.
    ArrowUp,
    /// The down arrow key.
    ArrowDown,
    /// The left arrow key.
    ArrowLeft,
    /// The right arrow key.
    ArrowRight,
    /// The `Home` key.
    Home,
    /// The `End` key.
    End,
    /// The `Page Up` key.
    PageUp,
    /// The `Page Down` key.
    PageDown,
    /// The left `Shift` key.
    ShiftLeft,
    /// The right `Shift` key.
    ShiftRight,
    /// The left `Control` key.
    ControlLeft,
    /// The right `Control` key.
    ControlRight,
    /// The left `Alt` / `Option` key.
    AltLeft,
    /// The right `Alt` / `Option` key.
    AltRight,
    /// The left `Meta` / `Super` / `Command` key.
    MetaLeft,
    /// The right `Meta` / `Super` / `Command` key.
    MetaRight,
}

impl PhysicalKey {
    /// Returns whether this is one of the left/right modifier keys
    /// (`Shift`, `Control`, `Alt`, or `Meta`).
    #[must_use]
    pub const fn is_modifier(&self) -> bool {
        matches!(
            self,
            Self::ShiftLeft
                | Self::ShiftRight
                | Self::ControlLeft
                | Self::ControlRight
                | Self::AltLeft
                | Self::AltRight
                | Self::MetaLeft
                | Self::MetaRight
        )
    }

    /// Returns the [`Modifier`] this key contributes when it is a left/right
    /// modifier key, or `None` for a non-modifier key.
    ///
    /// The left and right members of each pair collapse to the same logical
    /// [`Modifier`]: `ShiftLeft`/`ShiftRight` → [`Modifier::Shift`],
    /// `ControlLeft`/`ControlRight` → [`Modifier::Ctrl`],
    /// `AltLeft`/`AltRight` → [`Modifier::Alt`], and
    /// `MetaLeft`/`MetaRight` → [`Modifier::Meta`]. This lets a keyboard input
    /// step fold physical modifier presses into a [`Modifiers`](crate::command::Modifiers)
    /// set for shortcut matching.
    #[must_use]
    pub const fn modifier(&self) -> Option<Modifier> {
        match self {
            Self::ShiftLeft | Self::ShiftRight => Some(Modifier::Shift),
            Self::ControlLeft | Self::ControlRight => Some(Modifier::Ctrl),
            Self::AltLeft | Self::AltRight => Some(Modifier::Alt),
            Self::MetaLeft | Self::MetaRight => Some(Modifier::Meta),
            _ => None,
        }
    }

    /// Returns whether this is one of the four arrow keys.
    #[must_use]
    pub const fn is_arrow(&self) -> bool {
        matches!(
            self,
            Self::ArrowUp | Self::ArrowDown | Self::ArrowLeft | Self::ArrowRight
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_modifier_covers_left_and_right_pairs() {
        for key in [
            PhysicalKey::ShiftLeft,
            PhysicalKey::ShiftRight,
            PhysicalKey::ControlLeft,
            PhysicalKey::ControlRight,
            PhysicalKey::AltLeft,
            PhysicalKey::AltRight,
            PhysicalKey::MetaLeft,
            PhysicalKey::MetaRight,
        ] {
            assert!(key.is_modifier(), "{key:?} should be a modifier");
        }
        assert!(!PhysicalKey::KeyW.is_modifier());
        assert!(!PhysicalKey::Space.is_modifier());
        assert!(!PhysicalKey::ArrowUp.is_modifier());
    }

    #[test]
    fn modifier_folds_left_and_right_pairs() {
        assert_eq!(PhysicalKey::ShiftLeft.modifier(), Some(Modifier::Shift));
        assert_eq!(PhysicalKey::ShiftRight.modifier(), Some(Modifier::Shift));
        assert_eq!(PhysicalKey::ControlLeft.modifier(), Some(Modifier::Ctrl));
        assert_eq!(PhysicalKey::ControlRight.modifier(), Some(Modifier::Ctrl));
        assert_eq!(PhysicalKey::AltLeft.modifier(), Some(Modifier::Alt));
        assert_eq!(PhysicalKey::AltRight.modifier(), Some(Modifier::Alt));
        assert_eq!(PhysicalKey::MetaLeft.modifier(), Some(Modifier::Meta));
        assert_eq!(PhysicalKey::MetaRight.modifier(), Some(Modifier::Meta));
        // A non-modifier key contributes nothing.
        assert_eq!(PhysicalKey::KeyA.modifier(), None);
        assert_eq!(PhysicalKey::Enter.modifier(), None);
    }

    #[test]
    fn is_arrow_only_for_arrows() {
        for key in [
            PhysicalKey::ArrowUp,
            PhysicalKey::ArrowDown,
            PhysicalKey::ArrowLeft,
            PhysicalKey::ArrowRight,
        ] {
            assert!(key.is_arrow(), "{key:?} should be an arrow");
        }
        assert!(!PhysicalKey::KeyA.is_arrow());
        assert!(!PhysicalKey::Home.is_arrow());
        assert!(!PhysicalKey::ShiftLeft.is_arrow());
    }
}
