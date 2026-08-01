A grid item's children resolve their percentages against the **item's** box —
because a grid item's containing block is its grid area, not the grid
container's content box.

`specs/w3c/css-grid-2.txt:712`, vendored here: "A grid item's grid area forms
the containing block into which it is laid out." §6.2 (`:1042`) says it again
from the sizing side: "A grid item is sized within the containing block defined
by its grid area."

The defect this pins: the arrange walk handed every child of a node the same
percentage basis — that node's content box. That *is* the containing block for
a block child and a flex item, so block flow and flex were both right and only
grid was wrong. A `width: 100%` grid item came out at its track (correct) while
everything below it resolved against the whole grid container's content box, a
containing block the item never had: the item's used size and the size its
children saw disagreed, and the child's number tracked the shell's *padding*
rather than its parent's width. The grid algorithm now states each item's grid
area alongside the item's box, and the runtime prefers it
(`LayoutOutput::child_percent_basis`).

The geometry below is the external-consumer audit's, measured through this
facade (`docs/external-consumer-audit.md`, 2-4).
