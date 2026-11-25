Resolves the computed `position` keyword to a [`Position`], defaulting to
[`Position::Static`].

Compares the interned symbol against the reserved [`layout_keyword`]
`POSITION_*` handles; an absent or unrecognized value maps to
[`Position::Static`] (which also covers `POSITION_STATIC`).
