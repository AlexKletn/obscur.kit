Returns the [`Modifier`] this key contributes when it is a left/right
modifier key, or `None` for a non-modifier key.

The left and right members of each pair collapse to the same logical
[`Modifier`]: `ShiftLeft`/`ShiftRight` → [`Modifier::Shift`],
`ControlLeft`/`ControlRight` → [`Modifier::Ctrl`],
`AltLeft`/`AltRight` → [`Modifier::Alt`], and
`MetaLeft`/`MetaRight` → [`Modifier::Meta`]. This lets a keyboard input
step fold physical modifier presses into a [`Modifiers`](crate::command::Modifiers)
set for shortcut matching.
