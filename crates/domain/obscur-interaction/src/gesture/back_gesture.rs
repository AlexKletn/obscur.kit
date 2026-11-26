//! Recognizer for a semantic back swipe.

use std::num::NonZeroUsize;

use crate::event::UiEvent;
use crate::pointer::{PointerEvent, WheelEvent};

use super::{BackGestureUpdate, SwipeGesture, SwipeUpdate};

/// Tracks a trusted left-edge touch swipe, native touchpad swipe or phased pixel scroll.
#[derive(Debug)]
pub struct BackGesture(SwipeGesture);

impl Default for BackGesture {
    fn default() -> Self {
        Self(SwipeGesture::back())
    }
}

impl BackGesture {
    /// Requires a native touchpad swipe to report this exact finger count at the start.
    /// Touchscreen swipes and phased pixel scroll keep their usual recognition rules.
    #[must_use]
    pub fn with_touchpad_finger_count(self, count: NonZeroUsize) -> Self {
        Self(self.0.with_touchpad_finger_count(count))
    }

    /// Feed a dispatched UI event. Scale converts physical wheel deltas into logical pixels. Pointer positions in dispatched UI events are already logical.
    pub fn update(&mut self, event: &UiEvent, scale: f32) -> Option<BackGestureUpdate> {
        self.0.update(event, scale).map(back_update)
    }

    /// Feed a trusted phased pixel-scroll event, with the device scale used for its physical deltas.
    pub fn update_wheel(&mut self, wheel: WheelEvent, scale: f32) -> Option<BackGestureUpdate> {
        self.0.update_wheel(wheel, scale).map(back_update)
    }

    /// Feed a trusted touch event whose position is already in logical window coordinates.
    pub fn update_touch(&mut self, pointer: PointerEvent) -> Option<BackGestureUpdate> {
        self.0.update_touch(pointer).map(back_update)
    }

    /// The recognized displacement in logical pixels while the gesture is active.
    #[must_use]
    pub const fn offset(&self) -> Option<f32> {
        self.0.offset()
    }

    /// Discard the current sequence and return its cancellation update if it was recognized.
    pub fn cancel(&mut self) -> Option<BackGestureUpdate> {
        self.0.cancel().map(back_update)
    }
}

fn back_update(update: SwipeUpdate) -> BackGestureUpdate {
    BackGestureUpdate {
        source: update.source,
        phase: update.phase,
        offset: update.offset,
    }
}

#[cfg(test)]
#[path = "back_gesture.tests.rs"]
mod tests;
