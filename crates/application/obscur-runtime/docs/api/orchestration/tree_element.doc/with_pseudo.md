Creates a cursor over `id` within `tree` that reads interaction state
from `pseudo`.

[`pseudo_state`](MatchElement::pseudo_state) then reports the state the
runtime recorded for the node, so interaction-driven pseudo-classes
(`:hover`, `:focus`, …) match. Navigation
([`parent`](MatchElement::parent),
[`previous_sibling_element`](MatchElement::previous_sibling_element))
preserves the source.
