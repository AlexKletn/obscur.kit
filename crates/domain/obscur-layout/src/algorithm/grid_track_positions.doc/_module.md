Where a grid's tracks sit on one axis, once their sizes are known.

The track-sizing algorithm ([`super::grid_track_sizing`]) answers how LONG
each track is; this answers WHERE each one starts, and how far a span of them
reaches. Three facts settle that, and they are one entity because each is an
input to the next:

* which tracks **collapse** — an `auto-fit` repeat generates tracks no item
  need land in, and an empty one occupies no position at all and merges the
  gaps on either side of it (css-grid-2 §7.2.3.2);
* how the **leftover** space is distributed — `justify-content` on the
  columns, `align-content` on the rows, which move the tracks without
  resizing them (css-grid-2 §11.5 *Aligning the Grid*);
* the resulting running **offsets**, and the **extent** a multi-track span
  covers through them.

Nothing here measures or sizes anything: it is pure arithmetic over a
resolved `&[f32]` of track sizes, which is why the whole axis-positioning
question can be answered without the grid container, its children or a
measurer.
