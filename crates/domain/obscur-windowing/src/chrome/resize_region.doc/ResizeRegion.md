A semantic resize area on one edge or corner of a window (spec §6.4).

A compositor treats a drag that begins within [`Self::bounds`] as a resize
in the direction named by [`Self::edge`]. Coordinates are in window-local
logical pixels.
