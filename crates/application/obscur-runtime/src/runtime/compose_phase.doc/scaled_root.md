Wraps a freshly composed `display_list` in the scene's single root
transform `scale(s) · translate(-ox, -oy)` — the composition of
[`device_scale`](crate::UiRuntime::device_scale) `s` and the content scroll
[`content_offset`](crate::UiRuntime::content_offset) `(ox, oy)` — so the
whole scene rasterizes at physical resolution and scrolled to the current
offset while layout stays in logical px.

At scale exactly `1.0` **and** offset `(0, 0)` the list is returned
unchanged — no `PushTransform` / `PopTransform` is emitted, so the output is
byte-identical to the pre-scroll/pre-device-scale path (the regression
guard). Otherwise the list is re-wrapped as `PushTransform(root)` + original
commands + `PopTransform`, which keeps it balanced (one push, one matching
pop) since the composed list is itself already balanced. When the offset is
`(0, 0)` the root matrix reduces exactly to `scale(s, s)`.
