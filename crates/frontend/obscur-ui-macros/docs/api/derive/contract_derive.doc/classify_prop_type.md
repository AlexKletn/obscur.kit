Classifies a prop field's normalized type string into the
`PropTypeSurface` variant name it should carry (spec §2.8).

The check is on whole identifier tokens, not substrings, so a user type named
`MyString` is `Other` rather than a string. An `Option<T>` classifies as its
inner `T` — the `Option` token is simply not one of the ones looked for. Every
unrecognised type is `Other`, which the compiler leaves untyped.
