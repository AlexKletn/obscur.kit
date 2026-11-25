//! [`InputEvent`]: one platform event queued for the frame's dispatch phase.

use obscur_geometry::Point;
use obscur_interaction::command::KeyEvent;
use obscur_interaction::pointer::PointerEvent;
use obscur_text::ime::CompositionEvent;

#[doc = include_str!("../../docs/api/runtime/input_event.doc/InputEvent.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    /// A pointer moved, pressed, released, entered, left or was cancelled.
    Pointer(PointerEvent),
    /// A key was pressed or released.
    Key(KeyEvent),
    /// An IME composition was updated or committed.
    Composition(CompositionEvent),
    /// A scroll wheel/gesture moved by `delta` at `position`.
    Scroll {
        /// The vertical scroll delta, in logical pixels.
        delta: f32,
        /// Where the scroll happened, in viewport coordinates.
        position: Point,
    },
}
