The brush alpha that makes compositing visible.

At α = ½ one composite reads 128 and two read 1-(1-½)² = ¾ → 192. Those are 64
apart, which no antialiasing difference between the two rasterizers approaches
at a fully covered pixel.
