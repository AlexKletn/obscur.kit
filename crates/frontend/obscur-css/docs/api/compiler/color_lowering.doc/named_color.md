Maps a CSS named color to a [`StyleColor`], case-insensitively; [`None`] for
any keyword that is not one of the modelled names.

This is the small named-color subset the domain recognises — `transparent`,
`black`, and the common basic colors — enough for authored `color` /
`background-color` / `border-*-color` values. The full CSS named-color table
is a later extension; an unrecognised name falls through to an interned
keyword rather than being lowered to a color.
