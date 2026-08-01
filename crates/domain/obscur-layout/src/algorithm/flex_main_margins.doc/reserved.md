The main-axis space a margin edge **reserves** — what must be subtracted
before there is any free space to speak of.

`auto` reserves nothing: CSS Flexbox §8.1 gives it the opposite job, absorbing
free space *after* the items are sized. So it is `0` here and re-enters the
arithmetic in [`place_line`](super::flex_line_placement::place_line), the only
place that knows how much there is to
absorb.

The basis is irrelevant: percentages were resolved against the container's
inline size when the edges were extracted, so only `Px` and `Auto` reach here.
