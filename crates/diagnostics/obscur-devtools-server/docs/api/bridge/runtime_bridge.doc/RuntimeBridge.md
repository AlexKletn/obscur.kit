Serves `Inspect` commands by turning a target into a [`SnapshotMessage`].

The bridge holds an [`InspectSource`] `S` and does nothing but correlate the
command id with the payload the source produces — it can never execute
arbitrary code against the runtime.
