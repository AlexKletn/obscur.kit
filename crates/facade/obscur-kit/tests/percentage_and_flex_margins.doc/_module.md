End-to-end: **percentage margins** resolve against the containing block's
width on every edge (CSS 2.1 §8.3), and a flex item's **`auto` margins** take
the line's free space before `justify-content` sees it (CSS Flexbox §8.1).

Both were silently inert: a `%` margin was folded to `0` by the CSS bridge, and
a flex `auto` margin was read as `0` by the algorithm.

The container here is deliberately **not square** (300×100). A square one
cannot tell the §8.3 rule from the obvious-but-wrong reading — `margin-top:
10%` is 30 either way — so every block-axis assertion below would have passed
against an implementation that resolved each edge against its own axis.
