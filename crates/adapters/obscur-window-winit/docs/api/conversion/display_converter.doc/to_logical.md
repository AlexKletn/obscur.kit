Divides physical `bounds` by `scale_factor` into the logical space
[`Display::logical_bounds`] is documented in.

winit reports both a monitor's size and its position in physical pixels, so
the two bounds differ by exactly the scale factor. Publishing the physical
rectangle as the logical one would place a 2× monitor's logical origin at
twice its true logical coordinate.
