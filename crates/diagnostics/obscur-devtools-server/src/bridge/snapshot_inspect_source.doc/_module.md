`SnapshotInspectSource`: a real, non-`Null` [`InspectSource`] backed by
host-provided snapshots (spec §7.6/§7.8).

Where [`NullInspectSource`](crate::bridge::NullInspectSource) answers every
target with an empty placeholder, this source returns *live-ish* state: the
host (a `UiRuntime`, a headless test harness, or any inspectable owner)
registers a [`SnapshotPayload`] per [`Inspectable`] target it can describe,
and the bridge serves those back verbatim. Targets the host has not described
fall back to a typed default carrying a zero-count summary — never arbitrary
runtime access (spec §7.7).

It is deliberately snapshot-backed rather than holding a live runtime
reference: the host refreshes the registered payloads whenever its state
changes and hands the source to the bridge, mirroring the
host-provides-device pattern used across the kit. This keeps the source
`Send`-able onto the socket serving thread without borrowing the runtime.
