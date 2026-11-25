A single hit record: a node struck by a hit-test query at a point
(spec §4.7).

Hit testing maps a point in visual space back to the logical event tree.
Each record captures the node hit, the query point expressed in that node's
local coordinate space, and the node's `z_order` so that overlapping hits
can be ordered topmost-first.
