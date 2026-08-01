An owned, `'static` pair of Wayland handles, so a host can build a
`wgpu::Surface<'static>` from a layer surface exactly as it does from a
window.

# Why this type exists at all

wgpu takes anything implementing [`HasWindowHandle`] + [`HasDisplayHandle`],
and winit's window implements both — which is why the ordinary windowed path
needs no unsafe anywhere. A raw `wl_surface` implements neither, so something
has to assert the handles are valid. That assertion is the *only* unsafe in
this crate, and it is here rather than in the host because the host forbids
unsafe outright.

# Safety contract

The pointers are borrowed from the connection and the `wl_surface` owned by
[`LayerWindow`](super::LayerWindow). This type must not outlive it. It is
constructed only by `LayerWindow::target`, which hands out a fresh one from
live objects, and it is inert data — dropping it releases nothing.
