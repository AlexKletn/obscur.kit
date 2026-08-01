A reactive editable field: a `panel` (100×100) holding a single `b` (100×50, at
y ∈ [0, 50)) that binds `on-input → CMD_INPUT` (per keystroke) and contains a
text node showing the current value.

Pressing inside `b` — at `(50, 25)`, which is what
[`focused`](Self::focused) does — focuses the editable and makes the runtime
seed an editing session from that text child. The text node is what the reactive
assertions read back, through
[`SemanticSelector::first_text`](crate::selector::SemanticSelector::first_text).

The classes are `InteractionPanel`'s, so [`sheet`](Self::sheet) is that panel's
sheet: the geometry — and therefore every press coordinate written against this
fixture — is stated in exactly one place.
