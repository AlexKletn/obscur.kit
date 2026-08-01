Step 4 of the frame (spec §3.4, §4.3): map this frame's computed styles into
layout styles, arrange the tree under the viewport, and retain the absolute
boxes plus the hit-test index built from them.

It is a **borrow bundle**, not a holder: every field is a borrow of state the
runtime owns across frames, taken for the duration of one phase and released
with it. The retained boxes, the two layout caches and the hit-test index all
outlive the frame, and none of them is *owned* here.

# Why the phase is one entity rather than a run of statements

Its interface is one call returning one `bool` — did anything arrange? — and
its internals are the assembly of a layout input set that no other phase
speaks about: three separate patches applied to the freshly built layout
styles (named grid areas, named grid lines, intrinsically-sized replaced
elements and virtual spacers), a per-node grid-template side table, two units
(`rem`, `ch`) that can only be folded here, and the pairing of the measure and
arrange caches with the invalidation set that says which of their entries this
frame may keep.

The last of those is the reason it is worth naming. §3.4 step 8's incremental
layout is only sound while the caches, the invalidation set and the boxes move
together: a frame that dropped an entry without re-arranging its subtree, or
retained boxes without recording the inputs they were arranged under, would
serve the next frame geometry from a state that never existed. Those three
writes are adjacent here, and nowhere else.
