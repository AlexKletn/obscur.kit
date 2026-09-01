The second of the three places the pixel could have gone: the GPU rasterizer, at
the same extent, off-screen.

Ignored by default because it needs an adapter and this workspace has no GPU
test target; run it with `--ignored` on a machine that has one. Passing narrows
the finding to the swapchain and the compositor, both of which are below this
crate and unreachable from any test here.
