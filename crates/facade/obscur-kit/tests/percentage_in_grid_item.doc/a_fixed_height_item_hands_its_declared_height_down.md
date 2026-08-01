The other control, and the declared-size row: an item with a height of its own
keeps it (the `stretch` fallback of 2-4 (b), on this axis), and hands *that*
down — 30 is half of 60, not half of the 100 row nor half of the 192 container.
**Green before the fix too**, for the same reason as the `auto` row: a declared
length needs no basis to resolve.
