A pointer went down on the hit node — `on:pointerdown="…"`.

Dispatched **before** the runtime does anything with the press, so a listener
that calls `prevent_default` suppresses the whole of the runtime's press
behaviour: the focus move, the caret placement, the press anchor and the arming
of the pan/long-press recognizers. Because the anchor is what a later release is
matched against, a refused press also means no `click` can follow it.

Delivered to the node the press landed on — capture-else-hit-test, the same
target hover and focus are computed from — and through §2.3's capture / target /
bubble like any other event. A press on empty space has no target and so
dispatches nothing.
