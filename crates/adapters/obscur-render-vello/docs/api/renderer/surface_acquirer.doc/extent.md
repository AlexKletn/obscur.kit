The extent the swapchain is actually configured at, as last applied.

The guard that makes [`SurfaceAcquirer::resize`] cheap when nothing changed, and
it is load-bearing rather than an optimisation. A layer surface is reconfigured
by the compositor for reasons that leave its size alone — a scale event, an
ordinary `configure` ack — and the runner turns every one of them into a resize
call. Without this field each of those dropped the ready image and re-armed
[`AcquireState::resize_to`], so the next acquired image was dropped too; no
frame was built, no buffer was committed, and a compositor waiting for a buffer
answers by configuring again. The result was a livelock that produced 69205
configure events in six seconds and a surface that stayed black.

Written by the acquire thread after a real `VelloSurface::resize`, because that
is the only place that knows the reconfigure happened, and seeded at `spawn`
from the surface's own configuration so the first no-op is recognised as one.
