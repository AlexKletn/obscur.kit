The same 40 px glyph twice, one pixel apart: folding replaces the source-over
composite with a non-zero fill of the union, and the union saturates where the
composite was still climbing.

Measured here: 76 pixels, up to 58/255 in alpha. The threshold asserted is 16,
far below the measurement and far above rounding, so the test states the
mechanism rather than one driver's arithmetic.
