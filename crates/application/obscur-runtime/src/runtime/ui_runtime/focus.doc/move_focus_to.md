Moves focus to `next` — or clears it, when `next` is `None` — and performs
everything that follows from the move. Returns whether focus actually changed.

`cause` is the event that moved it, and it becomes the envelope both `blur` and
`focus` are dispatched with, so a handler reads the real trust, timestamp and
payload of the thing that moved focus rather than a synthetic stand-in. Today
that is always the `pointerdown` that landed on the new node.

Re-focusing the node that already has focus is a no-op in the strongest sense:
it returns early before the editing session is touched, so a caret, a selection
and a dirty baseline all survive a press that did not actually move focus.

Everything this seats is listed on the [module](self); the reason it is one
function is listed there too.
