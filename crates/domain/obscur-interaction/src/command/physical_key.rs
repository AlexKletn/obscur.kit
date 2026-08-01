#![doc = include_str!("physical_key.doc/_module.md")]

use crate::command::Modifier;

#[doc = include_str!("physical_key.doc/PhysicalKey.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhysicalKey {
    #[doc = include_str!("physical_key.doc/KeyA.md")]
    KeyA,
    #[doc = include_str!("physical_key.doc/KeyB.md")]
    KeyB,
    #[doc = include_str!("physical_key.doc/KeyC.md")]
    KeyC,
    #[doc = include_str!("physical_key.doc/KeyD.md")]
    KeyD,
    #[doc = include_str!("physical_key.doc/KeyE.md")]
    KeyE,
    #[doc = include_str!("physical_key.doc/KeyF.md")]
    KeyF,
    #[doc = include_str!("physical_key.doc/KeyG.md")]
    KeyG,
    #[doc = include_str!("physical_key.doc/KeyH.md")]
    KeyH,
    #[doc = include_str!("physical_key.doc/KeyI.md")]
    KeyI,
    #[doc = include_str!("physical_key.doc/KeyJ.md")]
    KeyJ,
    #[doc = include_str!("physical_key.doc/KeyK.md")]
    KeyK,
    #[doc = include_str!("physical_key.doc/KeyL.md")]
    KeyL,
    #[doc = include_str!("physical_key.doc/KeyM.md")]
    KeyM,
    #[doc = include_str!("physical_key.doc/KeyN.md")]
    KeyN,
    #[doc = include_str!("physical_key.doc/KeyO.md")]
    KeyO,
    #[doc = include_str!("physical_key.doc/KeyP.md")]
    KeyP,
    #[doc = include_str!("physical_key.doc/KeyQ.md")]
    KeyQ,
    #[doc = include_str!("physical_key.doc/KeyR.md")]
    KeyR,
    #[doc = include_str!("physical_key.doc/KeyS.md")]
    KeyS,
    #[doc = include_str!("physical_key.doc/KeyT.md")]
    KeyT,
    #[doc = include_str!("physical_key.doc/KeyU.md")]
    KeyU,
    #[doc = include_str!("physical_key.doc/KeyV.md")]
    KeyV,
    #[doc = include_str!("physical_key.doc/KeyW.md")]
    KeyW,
    #[doc = include_str!("physical_key.doc/KeyX.md")]
    KeyX,
    #[doc = include_str!("physical_key.doc/KeyY.md")]
    KeyY,
    #[doc = include_str!("physical_key.doc/KeyZ.md")]
    KeyZ,
    #[doc = include_str!("physical_key.doc/Digit0.md")]
    Digit0,
    #[doc = include_str!("physical_key.doc/Digit1.md")]
    Digit1,
    #[doc = include_str!("physical_key.doc/Digit2.md")]
    Digit2,
    #[doc = include_str!("physical_key.doc/Digit3.md")]
    Digit3,
    #[doc = include_str!("physical_key.doc/Digit4.md")]
    Digit4,
    #[doc = include_str!("physical_key.doc/Digit5.md")]
    Digit5,
    #[doc = include_str!("physical_key.doc/Digit6.md")]
    Digit6,
    #[doc = include_str!("physical_key.doc/Digit7.md")]
    Digit7,
    #[doc = include_str!("physical_key.doc/Digit8.md")]
    Digit8,
    #[doc = include_str!("physical_key.doc/Digit9.md")]
    Digit9,
    #[doc = include_str!("physical_key.doc/F1.md")]
    F1,
    #[doc = include_str!("physical_key.doc/F2.md")]
    F2,
    #[doc = include_str!("physical_key.doc/F3.md")]
    F3,
    #[doc = include_str!("physical_key.doc/F4.md")]
    F4,
    #[doc = include_str!("physical_key.doc/F5.md")]
    F5,
    #[doc = include_str!("physical_key.doc/F6.md")]
    F6,
    #[doc = include_str!("physical_key.doc/F7.md")]
    F7,
    #[doc = include_str!("physical_key.doc/F8.md")]
    F8,
    #[doc = include_str!("physical_key.doc/F9.md")]
    F9,
    #[doc = include_str!("physical_key.doc/F10.md")]
    F10,
    #[doc = include_str!("physical_key.doc/F11.md")]
    F11,
    #[doc = include_str!("physical_key.doc/F12.md")]
    F12,
    #[doc = include_str!("physical_key.doc/Escape.md")]
    Escape,
    #[doc = include_str!("physical_key.doc/Enter.md")]
    Enter,
    #[doc = include_str!("physical_key.doc/Space.md")]
    Space,
    #[doc = include_str!("physical_key.doc/Tab.md")]
    Tab,
    #[doc = include_str!("physical_key.doc/Backspace.md")]
    Backspace,
    #[doc = include_str!("physical_key.doc/Delete.md")]
    Delete,
    #[doc = include_str!("physical_key.doc/ArrowUp.md")]
    ArrowUp,
    #[doc = include_str!("physical_key.doc/ArrowDown.md")]
    ArrowDown,
    #[doc = include_str!("physical_key.doc/ArrowLeft.md")]
    ArrowLeft,
    #[doc = include_str!("physical_key.doc/ArrowRight.md")]
    ArrowRight,
    #[doc = include_str!("physical_key.doc/Home.md")]
    Home,
    #[doc = include_str!("physical_key.doc/End.md")]
    End,
    #[doc = include_str!("physical_key.doc/PageUp.md")]
    PageUp,
    #[doc = include_str!("physical_key.doc/PageDown.md")]
    PageDown,
    #[doc = include_str!("physical_key.doc/ShiftLeft.md")]
    ShiftLeft,
    #[doc = include_str!("physical_key.doc/ShiftRight.md")]
    ShiftRight,
    #[doc = include_str!("physical_key.doc/ControlLeft.md")]
    ControlLeft,
    #[doc = include_str!("physical_key.doc/ControlRight.md")]
    ControlRight,
    #[doc = include_str!("physical_key.doc/AltLeft.md")]
    AltLeft,
    #[doc = include_str!("physical_key.doc/AltRight.md")]
    AltRight,
    #[doc = include_str!("physical_key.doc/MetaLeft.md")]
    MetaLeft,
    #[doc = include_str!("physical_key.doc/MetaRight.md")]
    MetaRight,
}

impl PhysicalKey {
    #[doc = include_str!("physical_key.doc/is_modifier.md")]
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

    #[doc = include_str!("physical_key.doc/modifier.md")]
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

    #[doc = include_str!("physical_key.doc/is_arrow.md")]
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
