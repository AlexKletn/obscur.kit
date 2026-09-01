The same question across the fractional scales a compositor actually sends,
including the ones where `round` goes *down* and the buffer ends up half a
physical pixel shorter than the surface it stands for (1.2, 1.4 and 1.75 here).

Even there the shape still reaches both extreme columns: the truncation is
sub-pixel, so it costs no row. This is what rules the rounding in
`LayerBackend::physical_size` out as the cause, rather than merely leaving it
unexamined.
