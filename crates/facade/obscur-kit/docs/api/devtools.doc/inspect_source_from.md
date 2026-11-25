Builds a [`SnapshotInspectSource`] populated from `runtime`'s current live
state: the full annotated tree (Tree), per-node geometry (Layout), per-node
computed styles (Styles), and the focus/hover summary (Focus). Targets the
runtime does not describe fall back to the source's typed default.

When the host enables `UiRuntime::set_command_trace_enabled`, EventFlow
shows the latest 128 command dispatches with their source, target, trust,
handler status, and default-prevention flag. It contains no input payloads.
