In-window inspect overlay: highlight the hovered node's box and describe its
identity, so a developer can debug the running UI directly in the window
rather than from a separate tool (spec §7.6 — inspect mode uses the same
hit-testing engine as the runtime and can highlight bounds).

[`overlay_commands`] turns a node's absolute border-box rectangle into the
two leaf paint commands the runtime appends over the composed scene when
inspect mode is on: a translucent fill and a solid outline. [`InspectInfo`]
is the host-facing readout the runtime builds from the hovered node — its
[`Display`](std::fmt::Display) line names the element (CSS-style
`type#id.class`), its geometry, and its component id.
