§11.8 — *Stretch auto Tracks*. Splits any remaining positive free space
equally among tracks whose *max* sizing function is `auto`. The caller only
invokes this when the container axis is definite and its content alignment is
`stretch`/`normal` (the default), matching the spec's preconditions.

`auto` only: a `min-content` / `max-content` track is sized by its content and
nothing else, which is the one place the three visibly part company. §11.8 is
written against `auto` for exactly that reason.
