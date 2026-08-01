A surface-local Wayland position as a kit point, in **physical** pixels.

Wayland reports pointer and touch positions in *surface-local logical*
coordinates — already divided by the buffer scale. The kit's runtime expects
the opposite: `handle_pointer` divides what it is given by the device scale,
because the windowed path feeds it winit's physical positions.

Handing over logical coordinates therefore halves every click on a 2× display
— a press near the right edge lands in the middle of the surface, and the
controls that live there are never hit. Multiplying here keeps both windowing
paths in one coordinate space, which is the only way the runtime can stay
unaware of which one it is on.
