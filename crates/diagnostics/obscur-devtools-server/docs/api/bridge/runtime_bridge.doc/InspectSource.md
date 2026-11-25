A source of live inspectable state the bridge queries on the host's behalf.

The bridge never reaches into the runtime directly: the host (a
`UiRuntime`, in future) implements this trait and hands it to the bridge,
mirroring the *host-provides-device* pattern used elsewhere in the kit. Each
call maps an [`Inspectable`] target to a typed [`SnapshotPayload`].
