Clamping a node's own content-axis size into its authored min/max (spec §4.2).

A node's resolved content width/height must be clamped into its
`min-`/`max-` bounds for that axis before its border box is formed. Every
formatting context (flow, flex, grid) needs the same rule for the node's
*own* box, so it lives here rather than being duplicated (and, historically,
dropped) per algorithm.
