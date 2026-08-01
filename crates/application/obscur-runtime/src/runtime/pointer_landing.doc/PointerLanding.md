Where one pointer event landed: the two content-space points it is read in,
and the node it addresses.

# Why the two points are one answer

Because a scroll container clips its content to the scrollport **on screen**
while the boxes and the hit-test index are keyed by each node's **unscrolled**
layout box, so the two questions a hit-test asks — "is this point inside the
box?" and "is this point inside the clip?" — are asked in *different spaces*
about the same event. Carrying only one of them is how a click inside a
scrolled list picks the row that was there before the scroll, or picks a row
that has been scrolled out of sight. They are produced together, by the same
walk, and every downstream consumer takes the one its own space calls for.
