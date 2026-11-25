Sets the viewport scroll offset in logical pixels. The offset is the
document-coordinate point at the top-left of the viewport (spec §4.1).
Each non-finite component preserves the previous value on its axis.

Subsequent frames use this offset for viewport-relative layout, including
`position: sticky`, and to determine which resources are visible.
