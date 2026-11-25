One swapchain image, carrying what a blit into it needs to know.

The extent and format travel **with** the image rather than being read from the
surface, because once a [`SurfaceAcquirer`](super::SurfaceAcquirer) owns the
surface the caller cannot ask it. That is not merely plumbing: an image acquired
before a resize and blitted against the new extent is a stretched frame, and
carrying the numbers alongside makes the mismatch visible instead of silent.
