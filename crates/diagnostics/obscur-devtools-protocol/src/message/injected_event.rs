#[doc = include_str!("../../docs/api/message/command_message.doc/InjectedEvent.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InjectedEvent {
    /// A synthetic pointer press at logical coordinates `(x, y)`.
    PointerDown {
        /// Logical x coordinate.
        x: i32,
        /// Logical y coordinate.
        y: i32,
    },
    /// A synthetic pointer release at logical coordinates `(x, y)`.
    PointerUp {
        /// Logical x coordinate.
        x: i32,
        /// Logical y coordinate.
        y: i32,
    },
    /// A synthetic key press identified by a stable key code.
    KeyDown {
        /// Stable key code.
        key: u32,
    },
    /// A synthetic key release identified by a stable key code.
    KeyUp {
        /// Stable key code.
        key: u32,
    },
}
