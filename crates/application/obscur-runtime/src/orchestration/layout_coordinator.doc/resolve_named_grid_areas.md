Resolves every grid item's `grid-area: <name>` marker against its
container's `grid-template-areas`, patching the item's [`LayoutStyle`]
placement in `styles` (spec §4.2).

Named-area placement is the one grid feature that needs the tree: a child's
`grid-area: header` names a rectangle defined on its **container**'s
`grid-template-areas`. This runs after
[`layout_styles_from`](Self::layout_styles_from) has built `styles`; it only
overwrites the `grid_placement` of items carrying a marker (looked up
against the parent's parsed template), so every non-named layout is left
byte-for-byte identical.
