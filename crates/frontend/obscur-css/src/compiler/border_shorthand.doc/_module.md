The `border` / `border-<side>` / `outline` shorthands, whose components are
order-free (CSS Backgrounds and Borders 3 §4, CSS UI 4 §2).

These are the shorthands that cannot be expanded positionally: a
`<line-width>`, a `<line-style>` and a `<color>` may appear in any order and
any may be omitted, so expansion starts by *classifying* each component
([`classify_border`]) instead of zipping it to a longhand. That
classification, and the per-edge width/style/color triples it fans out over
([`BorderSide`]), are used by nothing else in the crate — which is the seam
separating this file from the positional shorthands in
[`super::shorthand_expansion`].
