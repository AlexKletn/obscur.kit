The total `scroll_top` of `node`'s scrolling ancestors — how far down its
box has been carried out of view.

Strict ancestors only: a scroll container's own border box does not move
when it scrolls (CSS Overflow 3 §scrollport — the container's background
and border stay put while its content slides), and compose agrees, pushing
the scroll transform AFTER the container's own paint.

The walk is bounded by the node count so a malformed (cyclic) parent chain
terminates, and returns immediately when nothing scrolls.
