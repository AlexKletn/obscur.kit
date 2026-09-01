One committed frame at a given physical extent and scale, as the layer runner
produces it: `set_surface` divides the physical extent back into a logical
viewport, layout arranges in those logical pixels, and compose brackets the
whole display list in a root device-scale transform. Everything the on-screen
path does to place a shape on the physical grid happens here.
