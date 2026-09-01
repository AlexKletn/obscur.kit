Every display the compositor announces, with the id settings take.

Opens its own short-lived connection, does the one roundtrip that turns the
registry's globals into named outputs, and drops the connection. The ids match
what [`LayerWindow::open`](crate::surface::LayerWindow::open) will resolve,
because both come from the same enumeration.

A display list is a snapshot: a monitor unplugged between this call and `open`
leaves an id that resolves to nothing, and the surface then opens wherever the
compositor puts it — the same outcome as not asking.

# Errors

[`Unsupported`](crate::error::LayerShellError::Unsupported) where the compositor
has no `wlr-layer-shell`, checked before the outputs are read so a caller gets
the same answer whether they probe or open first; `NoDisplay` where there is no
Wayland session; `Dispatch` where the connection failed.
