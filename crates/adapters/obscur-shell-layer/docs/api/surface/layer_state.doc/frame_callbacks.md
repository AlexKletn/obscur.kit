How many frame callbacks the compositor has sent for this surface.

Zero over a whole run is a finding, not an absence: it means the compositor never
once told the surface it was about to be composited, and every frame built during
that run was built for nobody.
