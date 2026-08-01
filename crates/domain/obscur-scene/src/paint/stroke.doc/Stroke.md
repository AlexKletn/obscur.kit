Describes how a path's outline is stroked (spec §5.1).

A stroke paints a band of the given `width` centered on the path, filled
with `brush`. The open ends of an unclosed path are shaped by `line_cap`,
corners are shaped by `line_join`, and sharp miter joins longer than
`miter_limit` times half the width fall back to a bevel.

A `dash` pattern (with a `dash_offset` phase) breaks the stroked band into
alternating on/off runs, measured along the path in user-space units — the
same `[on, off, on, off, …]` convention as SVG/Canvas/CSS. An **empty**
`dash` (the default from [`Stroke::new`]) is a continuous solid stroke, so
every existing stroke is byte-identical.
