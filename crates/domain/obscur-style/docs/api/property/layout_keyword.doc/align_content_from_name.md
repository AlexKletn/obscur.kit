Maps an `align-content` keyword string to its reserved [`InternedSymbol`].

`align-content` (the grid container's block-axis track distribution) shares
the exact `justify-content` value grammar (`start` / `end` / `center` /
`space-between` / `space-around` / `space-evenly`, plus the flexbox
`flex-start` / `flex-end` spellings), so this simply delegates to
[`justify_content_from_name`]. Returns [`None`] for any keyword outside that
closed set.
