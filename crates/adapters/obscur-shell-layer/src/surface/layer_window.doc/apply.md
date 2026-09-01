Push a whole settings object onto a live layer surface.

The whole object rather than a diff: the protocol's setters are idempotent, and
one description of the surface cannot disagree with itself the way two can.

Most of it is the layer shell's own vocabulary; the pointer policy is not. That
one is `wl_surface`'s input region, which is why the compositor global is a
parameter here.
