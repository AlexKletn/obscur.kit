The leading offset an `auto` margin pair puts between the start of the grid
area and the item: all of the free space to a leading `auto`, half of it to
each of two, none of it to a trailing one alone.

`free` is what is left of the area on that axis once the item's used size and
its *length* margins are taken off. A negative one — an item wider than the
area it was placed in — yields `0` rather than a negative offset, so an
overflowing item spills past the end of its area and never before its start.

Only meaningful when [`absorbs_free_space`] said `true`; with two length edges
it is `0`, which is the same answer as not calling it.
