A key went down on the focused node — `on:keydown="…"`.

Dispatched **before** the runtime does anything with the key, so a listener that
calls `prevent_default` suppresses the text edit, the shortcut and the
activation that would otherwise have followed. That ordering is the whole point:
a `keydown` delivered after the field had already taken the character could
report the key but never refuse it.

Fires for auto-repeat too; `KeyEvent::is_repeat` is how a handler tells the two
apart.
