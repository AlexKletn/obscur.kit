Open a layer surface described by these settings.

The namespace is the string the compositor shows in its own tooling
(`hyprctl layers` and friends) and the one a compositor rule matches on. It is
not part of the settings because it is the *application's* identity, which the
entry point already knows.

Returns [`LayerShellError::Unsupported`](crate::error::LayerShellError::Unsupported)
where the compositor has no `wlr-layer-shell` — the expected answer on GNOME,
and a capability result rather than a crash.

# Errors

[`Unsupported`](crate::error::LayerShellError::Unsupported) where the compositor
does not implement the protocol, `NoDisplay` where there is no Wayland session
to connect to, `MissingGlobal` where a required global is absent,
`InvalidSettings` when the selected display is unavailable, and `Dispatch`
where the connection itself failed.
