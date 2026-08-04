Pixels somebody else produced — a video frame, a camera image, a screen
capture, the output of a software rasterizer — described well enough to be
drawn without being guessed at.

It BORROWS. Nothing is copied until [`to_scene_image`](ExternalBuffer::to_scene_image)
is called, and that call is where the repacking and the channel swizzle happen —
once, into the layout the scene speaks.

If the buffer is ALREADY tightly-packed RGBA8 and already lives in an `Arc`,
skip this type entirely and build a [`SceneImage`](super::SceneImage) directly:
that path copies nothing at all.

Spec §5.5 (foreign surfaces) also allows a texture-backed surface handed to the
GPU without ever touching the CPU. That is a different road and it is not this
one — this is the road for pixels that are in memory.
