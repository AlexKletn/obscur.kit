A single `button` (CSS *type*, not class) carrying a `Save` text child, and the
type rules that style it.

The type selector is the whole reason this is not a [`ClassRule`](crate::fixture::ClassRule)
fixture: the cascade questions it answers are about a UA default sheet styling
`button { … }` at `SystemDefault` while an author sheet styles the same
`button { … }` at `GlobalApp`, and origin precedence — not specificity — decides
(§4.14). A class would let specificity confound the answer.

[`state_view`](Self::state_view) carries the reserved `:disabled` structural
state, which is what `Button::view` emits, so `button:disabled` can be shown to
bite only when the element actually reports the state and to re-resolve when it
stops.
