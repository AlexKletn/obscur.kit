Composites an already-post-processed source **texture** onto `target` at its
absolute pixel rect (top-left `source_origin`, extent `source_size`), with no
rounded mask — the composite step shared by the blur path and the color-only
`filter` path (a color matrix with no blur).

`premultiply` selects whether the (straight-alpha Vello) source is
premultiplied before the premultiplied-over blend: the blur path passes
`false` (byte-identical to the shipped opaque composite), the color-only path
passes `true` so a translucent color result (e.g. `opacity()`) composites
correctly.
