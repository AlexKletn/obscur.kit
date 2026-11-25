A thread-shareable [`InspectSource`] whose backing snapshot the host swaps out
each frame while a serving thread reads it (spec §7.6 host-provides-device).

The host holds one handle and calls [`set`](Self::set) after every frame with
a freshly built [`SnapshotInspectSource`] (e.g. from [`inspect_source_from`](super::inspect_source_from)
plus [`perf_payload`]); a [`clone`](Clone::clone) handed to a
[`DevToolsEndpoint`](obscur_devtools_server::endpoint::DevToolsEndpoint) on a
serving thread answers every inspect from the latest snapshot. Cloning shares
the same inner state (an `Arc`), so updates on one handle are visible on all.

# Pay only while someone is watching

Building a live source walks the whole tree and formats every node
([`inspect_source_from`](super::inspect_source_from)), which is far too expensive to do on a frame nobody
is inspecting. The serving side brackets each session with
[`client_connected`](Self::client_connected)/[`client_disconnected`](Self::client_disconnected),
and the host gates its publish on [`has_client`](Self::has_client) — so an
unobserved window pays nothing.
