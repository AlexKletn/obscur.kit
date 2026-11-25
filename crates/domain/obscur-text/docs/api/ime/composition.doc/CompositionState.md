The lifecycle state of a [`Composition`](super::Composition) (spec §5.12).

A composition begins [`Composing`](Self::Composing) and ends exactly once, in
either [`Committed`](Self::Committed) (the preedit was finalized into the
buffer) or [`Cancelled`](Self::Cancelled) (the composition was abandoned and
the anchored range left untouched).
