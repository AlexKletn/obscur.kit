Ask the compositor to say when this surface is about to be composited again.

A `wl_surface.frame` request, bound like a presentation feedback to the next
commit. This is the signal the loop was missing entirely: it paced on a hardcoded
16 ms timer, which is a guess about a screen it never asked, and which keeps
building frames at full speed for a surface the compositor is not drawing.
