Where the extreme pixel goes when a surface's logical size is not a whole number
of physical ones (audit 2-J).

A 56x56 layer surface on a 1.6-scaled output is 89.6 physical pixels wide. A
circle drawn to fill it exactly is therefore tangent to the surface's edge on
all four sides, with its widest point landing where the last column of an
integer buffer is not. On screen that circle came out with its left and right
extremes flattened, and the consumer could not see from outside whether the
pixel was lost in the toolkit's viewport mapping, in the swapchain, or in the
compositor.

This file answers the first of those three. The CPU backend rasterizes the
identical scene with no swapchain and no compositor anywhere in the path: the
same `round(56 * 1.6) = 90` buffer `LayerBackend::physical_size` asks for,
reached through the same `set_surface` -> logical viewport -> root device-scale
transform the on-screen path uses. If the mapping or the rasterizer dropped the
row, it would be missing here too.

It is not, at 1.6 or at any other fractional scale a compositor sends, so the
loss is not in either of them — which leaves the swapchain and the compositor,
both below this crate.

# What this does NOT cover

The GPU rasterizer is checked by the `#[ignore]`d test at the bottom (it needs
an adapter, and this workspace has no GPU test target). Nothing here reaches
the swapchain or the compositor at all, and no test in this workspace can: the
remaining two candidates are only reachable from a real Wayland session.
