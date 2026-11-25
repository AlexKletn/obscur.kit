Draw `source` over the whole of `target`, premultiplied.

**The target is written whole**, which is what makes the attachment's load op a
free choice: nothing of what the attachment held is read, so `Load` and `Clear`
put the same pixels on the screen and differ only in cost. Three conditions make
"written whole" true, and all three are enforced here rather than assumed:

1. The draw is one oversized triangle whose clip-space vertices are `(-1, 1)`,
   `(3, 1)` and `(-1, -3)` (`premultiply_blit.wgsl`, `vertex`), which covers the
   whole of `[-1, 1]²`.
2. The pipeline blends nothing and masks nothing: `blend: None`,
   `write_mask: ColorWrites::ALL` (`new`).
3. The pass sets neither a viewport nor a scissor rectangle, so its area is the
   whole attachment, and the fragment shader samples `source` and reads the
   target never.

Break any one of them and the two ops stop agreeing — a partial cover would show
the clear colour where it once showed the previous frame. Two tests hold this
rather than the reader: `the_alpha_is_multiplied_in` reads a texel back through
the whole pass (condition 2), and
`the_two_load_ops_draw_the_same_pixels_and_here_is_what_each_costs` dirties the
target, blits under each op and compares the full readbacks (all three at once,
from the outside).

Which op is written is `LOAD_OP`, one constant with the measurement beside it.
Short version: `Load`, because it is what measured faster on the only hardware
this repository can measure, by 30-45 us a present at panel size in four runs
out of four. The tiled-GPU argument for `Clear` is real, unmeasured here, and
recorded in `LOAD_OP.md` as a tablet experiment with the number that would
confirm it.

The bind group is cached across presents; `bound` says what keys it.
