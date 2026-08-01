A windowless stand-in for a window (spec §6.1).

On the headless and WASM backends there is no OS window and no native
surface, so every window concept maps to a `VirtualWindow`: a plain record of
the [`WindowId`], the logical `size`, the display `scale` (DPI), and whether
the window is `visible`. It owns no OS resource and never touches the
platform — it exists only so the rest of the stack can reason about "a
window" uniformly across backends.

The type is a small plain value (`Copy`), so a host can hold it by value and
hand copies out freely.
