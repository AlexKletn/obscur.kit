How many segments one rounded-rectangle contour emits: a move, four edge lines,
four corner cubics and a close.

Written down so the buffer is reserved for exactly what will be pushed — twice
over for [`Path::rounded_rect_ring`], which writes two contours into one.
`the_reserved_lengths_are_the_lengths_actually_emitted` is the test that keeps
the number honest across the radii cases that could plausibly change it.
