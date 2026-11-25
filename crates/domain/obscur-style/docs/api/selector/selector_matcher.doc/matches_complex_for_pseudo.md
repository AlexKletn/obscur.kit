Returns `true` if `selector`'s subject compound names the `pseudo`
pseudo-element (e.g. `before`) and `element` — the pseudo-element's
*originating* element — satisfies the rest of the compound and the
combinator prefix to its left (§4.16).

The pseudo-element simple itself is a *target*, not an element
constraint, so it is skipped when testing the subject; the remaining
simples (`.badge` in `.badge::before`) plus any ancestor/sibling prefix
must still match the originating element. Declarations selected this way
belong to the generated box, never to the originating element's own box —
that separation is the caller's, which is why the ordinary
[`matches_complex`](Self::matches_complex) still fails closed on a
pseudo-element (see [`matches_simple`]).
