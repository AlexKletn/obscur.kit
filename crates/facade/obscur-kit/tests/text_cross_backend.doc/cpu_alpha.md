Renders through `CpuRenderer` and returns the alpha channel, one byte a pixel.

Alpha only, because that is the channel the two backends carry identically: the
CPU pixmap is premultiplied and Vello's read-back is straight, so their colour
bytes are not comparable, while alpha is alpha on both.
