Build the runtime and install everything that does not need a surface.

The shared half of both runners. `viewport` is the size the runtime starts at:
a window's requested size, or — for a desktop surface — the size the platform
already settled on, since a layer surface is configured before its first frame
and there is no guessing to do.
