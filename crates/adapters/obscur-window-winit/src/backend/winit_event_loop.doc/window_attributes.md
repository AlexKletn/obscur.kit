Builds winit [`WindowAttributes`] from a domain [`CreateWindow`] request
(spec §6.1, §6.2).

The whole request is mapped, not just its [`WindowProperties`]: the §6.2
min/max [`WindowConstraints`](obscur_windowing::window::WindowConstraints)
are creation-time attributes in winit too, so they are pushed here rather
than dropped. Mapped: title, inner size, min/max inner size, resizability,
decorations, transparency, always-on-top level, and an explicit position when
one is requested. Post-creation-only concerns (state toggles) are applied
afterwards via [`WinitWindow::apply_properties`].

The request's [`WindowRelation`](obscur_windowing::window::WindowRelation) is
deliberately **not** mapped. winit's only ownership primitive is
`WindowAttributes::with_parent_window`, which builds a *child* window clipped
into the parent's client area (`WS_CHILD` on Windows, a child on X11) — not
the floating owned dialog §6.1 means — and winit has no modality primitive at
all. Rather than materialize a window that lies about its relation,
[`WinitWindowBackend`]'s `create_window` rejects any request carrying an owner
or a modality before it ever reaches this function.

[`WinitWindowBackend`]: super::WinitWindowBackend
