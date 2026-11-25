A layout-independent physical key position (spec §6.7).

A physical key names a scan position on the keyboard, independent of the
active layout: `KeyA` is the key where QWERTY prints `A`, whatever character
that key produces under the user's layout. Physical keys are the right
binding surface for spatial controls such as WASD movement, where the shape
of the layout matters more than the produced character. For text entry and
shortcuts, prefer [`LogicalKey`](crate::command::LogicalKey).

This is a closed enum covering a practical subset of keys (letters, digits,
function keys, editing and navigation keys, and the left/right modifier
pairs). It is non-exhaustive in spirit — real keyboards have more keys — but
deliberately closed so callers can match exhaustively.
