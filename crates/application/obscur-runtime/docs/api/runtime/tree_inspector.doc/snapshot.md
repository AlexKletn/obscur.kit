Returns a full structured snapshot of the live tree — one
[`NodeInspection`] per node in pre-order — plus the synthetic parts §4.16
projects.

An associated function for the same reason as
[`info_at`](Self::info_at): the snapshot is the agent-facing read half of
§7.6 and is available whether or not the in-window overlay is on.
