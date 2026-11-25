The `DrawImage` commands in a display list, read as an answer rather than as a
scene.

An image test almost always asks one of two questions — *did it paint at all?*
and *at what size?* — and both are one linear scan over the commands. Written out
at the call site they are five lines of `matches!` and `find_map` that bury the
question in the traversal.
