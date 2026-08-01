The two transforms that sit between a window's pixels and the runtime's logical
coordinates, and the rule they both obey: the render transform and the pointer
mapping must be exact inverses.

Eleven tests, one subject. A HiDPI surface multiplies the display list by
`scale(s, s)` and therefore *divides* every incoming pointer coordinate by `s`;
a scrolled content offset translates the root by `−offset` and therefore *adds*
the offset back to every incoming coordinate. Composed, the root becomes
`scale·translate` and the pointer mapping becomes `physical / s + offset` — and
each of those four claims is asserted against the other, so a change to one
that forgets the other fails here rather than at a user's fingertip.

Two of the eleven are the identity guard, and they are the reason the rest can
be trusted: at scale 1.0 and offset (0, 0) *no* root transform is emitted and no
hit-test moves, so the display list is byte-identical to the one produced before
either feature existed. The remaining pair are the clamps — a non-positive or
non-finite scale becomes 0.1, a non-finite offset component keeps the previous
one — and `set_surface`, which derives the logical viewport from the **clamped**
scale so that the viewport and the scale can never disagree.
