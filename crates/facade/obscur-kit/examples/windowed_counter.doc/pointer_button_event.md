Assembles a domain button [`PointerEvent`] at the tracked cursor.

[`PointerConverter`] only exposes public builders for `Move`/`Enter`/
`Leave` (a button transition *at a position* is the backend's job), so the
button event is constructed directly — reusing [`PointerConverter::kind`]
to map the winit [`ElementState`] to the domain event kind. The
`CursorMoved` path below uses the converter's `moved` builder end-to-end.
