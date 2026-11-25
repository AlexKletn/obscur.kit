How long a turn with nothing to wait for is allowed to sleep before it advances
motion and builds the next frame of an animation.

A bound on an idle, and nothing more. It was doing a second job it was never
suited to — standing in for a display clock — and the surface got 62.5 frames a
second on a 120 Hz panel because of it. The display clock is now the compositor`s
own `wl_surface.frame` callback, which is the only thing that knows.
