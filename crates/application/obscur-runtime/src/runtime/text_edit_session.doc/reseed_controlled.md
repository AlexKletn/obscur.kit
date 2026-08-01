Re-seeds the buffer from an **external** rewrite of the field's retained
value while it stays focused — the controlled-input reseed (WHATWG HTML
§4.10.5.5: a text control reflects its `value`, so when the app rewrites the
value the control displays it).

Unlike [`reseed`](Self::reseed) this **preserves the caret** at its current
byte offset, clamped into the new value and floored to a char boundary
(standard controlled-input behaviour: an `on-input` that normalizes — trim
/ uppercase — should not fling the caret to the end mid-edit). The undo/redo
history is cleared and any composition dropped (the value is no longer the
one being composed), scroll is reset (re-derived on the next caret move),
and the retained-value [`mirror`](Self::mirror) watermark is advanced to the
new value.

The `change` baseline [`committed`](Self::committed) is deliberately **left
untouched**: an external rewrite is not a user commit, so a later blur still
compares against the value the field had at focus.
