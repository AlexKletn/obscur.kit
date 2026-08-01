Flips between reserving screen space and floating over it, and returns
the surface configuration that now applies.

This is a live reconfiguration: no surface is destroyed and none is
created. Layer-shell's placement properties — band, anchor, exclusive
zone, margin, size — are all requests on the existing surface.
