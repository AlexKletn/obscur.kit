Appends a synthetic DevTools node for each scrollbar part (track + thumb) of
every currently-overflowing scroll container to `out` (spec §4.16).

These are **not** retained-tree nodes and appear in no node's `children()`,
so the component tree never sees them; they exist only in this DevTools
projection, carrying the real part rectangles the coordinator paints. A tree
with no overflowing scroll container appends nothing.
