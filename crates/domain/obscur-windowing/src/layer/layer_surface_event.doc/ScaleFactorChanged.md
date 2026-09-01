The display's scale changed.

Worth its own event rather than folding into a resize, because the logical size
may be identical across it while every buffer the surface owns is now the wrong
number of pixels.
