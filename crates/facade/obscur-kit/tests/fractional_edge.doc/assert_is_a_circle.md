That the rasterized extent is still the circle the scene asked for. A pixel
counts as reached when the shape touches any part of it, which is where the one
pixel of tolerated overshoot comes from; nothing tolerates undershoot beyond a
quarter pixel, because a flattened side is undershoot.
