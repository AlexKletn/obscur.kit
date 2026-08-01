The block-axis mirror of the defect, and **the only one of the three below
that was ever red**: a `height: 100%` item is its **row** (100), and its child
is half of *that* — not half of the grid container's 192.

Measured by reverting the one-line fix: the kid came back **96**, which is 50%
of 192, the grid container's content height — the item's used height (100) and
the height its child saw (192) disagreed, exactly as the inline axis did before
2-4 (a) was closed. The two controls under this one pass either way and say so.
