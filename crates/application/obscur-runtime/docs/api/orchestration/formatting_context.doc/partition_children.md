Partitions a node's children into `(in_flow, out_of_flow)`, preserving
document order within each list.

The in-flow list drives the formatting-context algorithm (its indices map to
the algorithm's `measure_child`), and the out-of-flow list is positioned
separately against the containing block.
