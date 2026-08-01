Resolves the computed `box-sizing` keyword to a [`BoxSizing`], defaulting to
[`BoxSizing::ContentBox`] (the CSS initial).

Compares the interned symbol against the reserved [`layout_keyword`]
`BOX_SIZING_*` handles; an absent or unrecognized value maps to
[`BoxSizing::ContentBox`] (which also covers `BOX_SIZING_CONTENT_BOX`).
