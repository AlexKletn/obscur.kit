Open the surface described by these settings.

The surface is not drawable the moment this returns — a platform settles the
geometry asynchronously, and
[`LayerSurfaceEvent::Configured`](crate::layer::LayerSurfaceEvent::Configured)
is the signal that it has. Check [`is_drawable`](Self::is_drawable) rather than
assuming.

# Errors

[`Unsupported`](LayerError::Unsupported) where the platform has no desktop
surfaces at all, [`InvalidSettings`](LayerError::InvalidSettings) where it has
them but cannot express these ones, and
[`BackendFailure`](LayerError::BackendFailure) where it tried and failed.
