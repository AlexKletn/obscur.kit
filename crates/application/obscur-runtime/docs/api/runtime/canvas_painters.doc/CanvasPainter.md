What an application registers: a closure handed its own bounded region, what the node says, and the frame's clock.

The properties are there because ONE painter serves EVERY node that names it —
without them two canvases sharing a painter would draw the same picture.

**It returns whether it is still animating.** A canvas that draws a moving thing
must keep asking for frames or it draws once and freezes; a canvas that draws a
static thing must not, or an idle application renders forever. Neither answer
can be guessed from outside, so the painter states it.
