Resolves a computed grid-item `justify-self` / `align-self` keyword (named by
`property`) to an [`AlignSelf`], defaulting to [`AlignSelf::Auto`].

The reserved `ALIGN_AUTO` handle maps to [`AlignSelf::Auto`] (inherit the
container's items default); the start/end/center/stretch handles map to the
matching override. An absent or unrecognized value is [`AlignSelf::Auto`].
