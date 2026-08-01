Prints the realized geometry of every virtual list when
`OBSCUR_VLIST_DEBUG=1` is set.

A rendering artifact can be a layout fault or a paint fault, and arguing from
the picture cannot tell them apart. This says where each realized child
actually is: if the boxes overlap, layout produced them that way; if they tile
cleanly, the fault is downstream.
