Every `wl_surface.commit` this process has made on the layer surface.

A `Cell` because committing is not a mutation of anything the surface state owns
— it is a request going out — and the two callers that make one hold the state by
shared reference.

Deliberately does NOT count the commit wgpu makes when it presents: that one is
the frame, and what this number exists to detect is the commits that are *not*
the frame, landing on top of one that has not been shown yet.
