The replaceable port for accessing documents behind opaque handles
(spec §6.10 read/write/create/enumerate).

The pure domain models document access as this object-safe trait: an adapter
implements it once per host (a real filesystem, a portal, a cloud provider, a
virtual filesystem) and the domain drives it as `Box<dyn DocumentService>`.
The trait is synchronous and returns [`Result`]; asynchronous and real I/O
concerns belong to the adapter, not here (spec §5.15).

Read-only operations take `&self`; mutating ones take `&mut self`. A write to
a handle that is not writable must fail with [`DocumentError::PermissionDenied`]
rather than silently succeeding.
