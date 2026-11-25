Reconciles the live winit window against the desired domain `props`
(spec §6.2).

Because the domain
[`UpdateWindow`](obscur_windowing::request::UpdateWindow) delta exposes no
field-level accessors, the backend instead resolves the delta against the
[`Window`](obscur_windowing::window::Window) record it owns and hands the
*resulting* full desired properties here. Pushing the full property set is
idempotent, so applying it repeatedly is harmless.

Every property winit can set is pushed: title, inner size, outer position
(only when an explicit one is requested — a platform-chosen `None`
position is left to the platform), resizability, server-side decorations,
always-on-top window level, and presentation state. Transparency is fixed
at creation time in winit and cannot be toggled afterwards, which is why
[`WinitWindowBackend`]'s `update_window` never lets a changed
`transparent` flag reach this far: it keeps the record on the live value
and reports the update `conditional()`, rather than accepting a flip that
would be silently dropped here.

[`WinitWindowBackend`]: super::WinitWindowBackend
