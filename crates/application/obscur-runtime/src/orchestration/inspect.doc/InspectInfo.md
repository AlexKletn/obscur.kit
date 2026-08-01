A host-facing readout of the node currently under the pointer (spec §7.6).

Built by the runtime from the hovered node's retained record and its absolute
layout box; its [`Display`](fmt::Display) renders a one-line CSS-style summary
a host can print next to the in-window highlight.
