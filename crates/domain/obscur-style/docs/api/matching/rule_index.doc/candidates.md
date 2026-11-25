Offers `visit` every rule that could possibly match `element`, in ascending
global rule order, and returns whether the index was able to decide.

`false` means the element could not enumerate its classes
(see [`MatchElement::enumerate_classes`](crate::selector::MatchElement::enumerate_classes))
or carries more distinct bucket keys than the merge has room for. Nothing has
been visited in that case beyond what the caller already saw, and the caller
must fall back to walking every rule — the index refuses to guess.

Each visit receives the rule's global index, its `(sheet, rule)` position and
the global declaration index its block starts at.
