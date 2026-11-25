Throws `active`'s container into a coast, if the contact that just lifted
at logical time `micros` was still moving (spec §6.16).

A drag that ends while the finger is travelling has *thrown* the list; one
that ends after the finger has come to rest has placed it. The recognizer
answers which, and reports nothing for a finger that stopped before
lifting — see [`PanRecognizer::velocity_at`].

The velocity is negated for the same reason the drag deltas are: the
content follows the finger, so a finger travelling up the screen leaves
`scroll_top` increasing.
