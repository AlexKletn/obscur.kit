The host-installed platform accessibility port (spec §7.1). Absent by
default, in which case no frame builds a semantic tree at all — the same
honest-capability default an absent vector decoder has for an SVG.

Shared rather than owned outright: an adapter's platform half lives in the
host (an `accesskit_winit`/`accesskit_unix` push loop reading what the
bridge converted), so the host must keep a handle to the very object this
runtime feeds. A `Box` here would make the frame path the bridge's sole
reachable owner and leave the host nothing to publish from.
