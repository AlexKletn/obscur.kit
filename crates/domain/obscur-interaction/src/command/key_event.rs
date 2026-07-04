//! The [`KeyEvent`] payload carried by a keyboard key transition (spec §6.7).

use crate::command::{LogicalKey, PhysicalKey};

/// A single keyboard key transition delivered to the focused node (spec §6.7).
///
/// A `KeyEvent` is the domain counterpart of a platform key event: it pairs the
/// layout-independent [`physical`](Self::physical) key position with the
/// layout-dependent [`logical`](Self::logical) key the active layout produced
/// (absent for a key the layout does not resolve to a modeled logical key, such
/// as a bare modifier or an unmapped named key) and a [`pressed`](Self::pressed)
/// flag distinguishing a press from a release.
///
/// It is the keyboard analogue of
/// [`PointerEvent`](crate::pointer::PointerEvent): a pure payload that the
/// application layer routes to the focused node and resolves against a
/// [`CommandRouter`](crate::command::CommandRouter). The two halves come from a
/// backend converter (e.g. the winit `KeyboardConverter`), which splits a native
/// key event into its physical and logical parts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyEvent {
    /// The layout-independent physical key position.
    pub physical: PhysicalKey,
    /// The layout-dependent produced key, if the layout resolves one.
    pub logical: Option<LogicalKey>,
    /// Whether this is a press (`true`) or a release (`false`).
    pub pressed: bool,
}

impl KeyEvent {
    /// Creates a key event for `physical` with an optional produced `logical`
    /// key and a `pressed` flag.
    #[must_use]
    pub const fn new(physical: PhysicalKey, logical: Option<LogicalKey>, pressed: bool) -> Self {
        Self {
            physical,
            logical,
            pressed,
        }
    }

    /// Creates a key **press** event.
    #[must_use]
    pub const fn press(physical: PhysicalKey, logical: Option<LogicalKey>) -> Self {
        Self::new(physical, logical, true)
    }

    /// Creates a key **release** event.
    #[must_use]
    pub const fn release(physical: PhysicalKey, logical: Option<LogicalKey>) -> Self {
        Self::new(physical, logical, false)
    }

    /// Returns the layout-independent physical key position.
    #[must_use]
    pub const fn physical(&self) -> PhysicalKey {
        self.physical
    }

    /// Returns the layout-dependent produced key, if any.
    #[must_use]
    pub const fn logical(&self) -> Option<LogicalKey> {
        self.logical
    }

    /// Returns whether this is a press (`true`) or a release (`false`).
    #[must_use]
    pub const fn pressed(&self) -> bool {
        self.pressed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::NamedKey;

    #[test]
    fn new_records_all_three_parts() {
        let event = KeyEvent::new(
            PhysicalKey::KeyA,
            Some(LogicalKey::character('a')),
            true,
        );
        assert_eq!(event.physical(), PhysicalKey::KeyA);
        assert_eq!(event.logical(), Some(LogicalKey::character('a')));
        assert!(event.pressed());
    }

    #[test]
    fn press_and_release_constructors() {
        let down = KeyEvent::press(PhysicalKey::Enter, Some(LogicalKey::named(NamedKey::Enter)));
        assert!(down.pressed());
        let up = KeyEvent::release(PhysicalKey::Enter, Some(LogicalKey::named(NamedKey::Enter)));
        assert!(!up.pressed());
    }

    #[test]
    fn logical_is_optional() {
        // A bare modifier has no modeled logical key.
        let shift = KeyEvent::press(PhysicalKey::ShiftLeft, None);
        assert_eq!(shift.logical(), None);
        assert_eq!(shift.physical(), PhysicalKey::ShiftLeft);
    }
}
