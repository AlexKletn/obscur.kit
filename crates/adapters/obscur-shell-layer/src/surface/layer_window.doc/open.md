Connects, binds `wlr-layer-shell`, and creates a surface described by
`config`.

# Errors

[`LayerShellError::NoDisplay`] when there is no Wayland session,
[`LayerShellError::Unsupported`] when the compositor does not implement
the protocol — the expected answer on GNOME, and a capability result
rather than a failure — and [`LayerShellError::MissingGlobal`] when a
required global is absent.
