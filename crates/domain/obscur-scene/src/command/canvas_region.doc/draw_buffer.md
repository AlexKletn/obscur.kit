Draws pixels somebody else produced — a video frame, a camera image, a screen
capture — fitted to this region.

The buffer is repacked into the scene's RGBA8 here, once; see
[`ExternalBuffer`](crate::paint::ExternalBuffer) for what that costs and how to
avoid it.

Answers whether anything was drawn. An empty buffer draws nothing rather than
putting a nothing-sized leaf in the list for every frame it takes a camera to
wake up.
