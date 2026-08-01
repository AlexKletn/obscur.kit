Tells the compositor how the buffer relates to logical pixels, by
whichever mechanism this surface has.

Without this a HiDPI surface is **twice the size it should be**: Wayland
measures a surface in logical units as `buffer_size / buffer_scale`, so a
buffer rendered at 2× and never declared as such is read as a surface
twice as wide and twice as tall.

Two mechanisms, and the caller should not have to know which is live:

* **Fractional** — the buffer stays at scale `1` and a viewport maps it
  onto the logical size. This is the only way to express 1.5, and it is
  what avoids rendering a 1.6× display at 2× and letting the compositor
  soften it back down.
* **Integer** — `set_buffer_scale`, where fractional scaling is absent.
  A fractional output then rounds up, as it always did.
