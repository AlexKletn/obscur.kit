The main-axis size item `index` reports when offered `main_offer` of main-axis
space and the container's cross extent.

The one measuring primitive behind every intrinsic answer flex needs: `0` is
the `min-content` offer (nothing wraps that can be broken), [`f32::INFINITY`]
the `max-content` one (nothing wraps that does not have to). The offer is
**loose** — `min: ZERO` — so an item that cannot fit reports the size it needs
rather than the space it was given, which is the whole point of the probe (see
[`BoxConstraints::constrain_allowing_overflow`]).

The answer is a **border**-box extent, like every measured size, so no
`box-sizing` mapping applies to it.
