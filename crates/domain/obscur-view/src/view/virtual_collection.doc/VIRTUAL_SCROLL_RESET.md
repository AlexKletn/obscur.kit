Reserved symbol for a virtual list's **scroll-reset token** — an opaque value
whose *change* means "this list now shows different content, start it at the top"
(spec §4.9).

A scroll offset belongs to the content it was scrolled through. When an app
replaces a list's items — a file manager entering another directory — the offset
is meaningless against the new content, but the runtime cannot see that the model
changed: the marker looks the same and the container node is reconciled in place,
so its `scroll_top` survives and the new list opens scrolled into nowhere. The app
is the only party that knows its content changed, and the view is its only channel
to the runtime, so it states a token here (typically a generation counter); the
runtime resets the container's scroll whenever the token it last saw differs.

It sits in the structural `0x0000_03xx` namespace with the other realization
inputs, not the CSS-relevant reserved attributes, so no selector can match on it.
