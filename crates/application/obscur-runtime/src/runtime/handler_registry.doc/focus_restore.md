What each open scope displaced when it opened, innermost last.

It lives beside `scopes` because the two are one fact: a scope that traps focus
takes it away from whatever held it, and the thing that owns the open scopes is
the only thing that can keep the pairing one-to-one. A frame is pushed by
`save_focus_for_scope` on every trap that opens and popped by
`restore_focus_for_scope` on every trap that closes — including the traps that
opened over nothing, which is why the frames are optional rather than absent.
