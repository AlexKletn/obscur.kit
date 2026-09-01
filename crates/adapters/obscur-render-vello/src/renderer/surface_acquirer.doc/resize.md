Reconfigure the surface, discarding anything acquired at the old size.

An image fetched before the resize belongs to the old extent and presenting it
would stretch the frame, so it is dropped rather than used.

Dropping it is also a correctness requirement rather than a nicety: a live
`SurfaceOutput` at `Surface::configure` is a wgpu validation error, which wgpu
raises as a panic on the acquire thread. On a fractionally scaled desktop
surface the compositor's scale arrives while the first image is in flight, and
that panic killed the process before its first frame about half the times it
started. The caller cannot close that window alone — the image lands after this
call returns — so the acquire thread reclaims the slot itself, immediately
before it configures.
