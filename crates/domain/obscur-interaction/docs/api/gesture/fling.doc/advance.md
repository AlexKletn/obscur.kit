Advances the coast by `seconds` and returns how far the surface travelled
during that step (signed, in the velocity's units).

Non-finite time steps are ignored.

The distance is the integral of the decaying velocity over the step —
`tau × (v_before − v_after)` — not `velocity × seconds`. Multiplying the
start-of-step velocity by the step would overshoot on every frame, and by
more the longer the frame took, so a stuttering frame would fling further
than a smooth one.
