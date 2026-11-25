A pixel inside the left stem of the `H`, where both backends read 255 when the
glyph is drawn once at full alpha.

Found by measurement, not by eye: it is the pixel maximising the smaller of the
two backends' alphas over the opaque render. Every alpha claim in this file is
read here.
