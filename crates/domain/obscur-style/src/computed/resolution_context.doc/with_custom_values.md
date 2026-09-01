Attaches the CSS grammar, so a `var()` whose custom property was declared as
plain text resolves to a value of the property it lands on.

A custom property keeps the text it was written with, because it matches no
grammar until it is substituted (CSS Variables 1 §2). Without this port
there is no grammar to apply at that point, so
`font-family: var(--font-body)` computes to `unset` however well-formed the
declaration was. Any caller that has a CSS compiler should supply one; the
resolver is usable without it for a caller whose custom properties are
already typed values.
