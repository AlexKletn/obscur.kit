`wlr-layer-shell` windowing: desktop surfaces rather than application windows.

A layer surface is what a panel, a dock, a wallpaper, a notification and an
on-screen keyboard are. It is not an application window: it has no title, no
decorations and no place in the window stack — it is anchored to screen edges,
sits in one of four depth bands, and can reserve screen space that maximised
windows then keep clear of.

# Why this is not the winit adapter

winit models application windows through `xdg-shell` and does not implement
`wlr-layer-shell` at all, so there is nothing to configure and no seam to
extend. This adapter therefore owns its own Wayland connection, its own
surface and its own dispatch loop, built on `smithay-client-toolkit` — which
is already in the dependency graph, because winit uses it too.

The consequence for a host is that a layer surface and an ordinary window are
**separate event loops**. One process can have both, but they do not share a
connection.

# What is here and what is not

`OBSCUR_LAYER_DEBUG=1` prints every converted pointer event and what it
resolved to, which is how the coordinate-space defect below was found.

Pointer and touch are converted into the kit's own
[`PointerEvent`](obscur_interaction::pointer::PointerEvent), so hit-testing,
hover, gestures and capture work exactly as in a window. Keyboard input uses the seat keymap and compositor focus, with key repeat
driven by the event loop. On-demand keyboard interactivity requires layer-shell
v4.
