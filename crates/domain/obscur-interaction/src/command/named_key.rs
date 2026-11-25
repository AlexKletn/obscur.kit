#[doc = include_str!("../../docs/api/command/logical_key.doc/NamedKey.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NamedKey {
    /// The `Enter` / `Return` key.
    Enter,
    /// The `Escape` key.
    Escape,
    /// The `Tab` key.
    Tab,
    /// The space key (as a named key rather than a `' '` character).
    Space,
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
}
