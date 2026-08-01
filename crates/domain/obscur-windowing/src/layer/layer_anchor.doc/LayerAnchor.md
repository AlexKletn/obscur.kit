The set of edges a layer surface is anchored to.

Anchoring does two things at once, and conflating them is the usual source of
surprise:

1. it decides **where** the surface sits, and
2. anchoring to two opposite edges makes the surface **stretch** between
   them, overriding the size requested on that axis.

So a bar across the top is `top | left | right` — anchored to three edges, it
spans the full width and takes its height from the requested size. Anchored
to `top` alone it would be centred horizontally at whatever width it asked
for.

Anchored to nothing, the compositor centres the surface.
