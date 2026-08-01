Assembles a mouse pointer event of `kind` at `position` with no held
buttons — the shared body of the mouse builders above, and
[`event_of`](Self::event_of) fixed to [`PointerDevice::Mouse`].

Takes `&self` for symmetry with the public builders (spec: converter
methods take `&self`); it needs no instance state.
