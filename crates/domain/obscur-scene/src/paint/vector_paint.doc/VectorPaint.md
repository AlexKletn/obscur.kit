How one vector shape is coloured — either a fixed colour, or the CSS
`currentColor` keyword left **unresolved** (spec §5.17).

# Why `currentColor` is not a colour

Spec §5.17 requires `currentColor` to resolve from the computed `color`
"allowing theme/state changes without reparsing". Baking it into a colour at
parse time would defeat exactly that: a theme switch changes the computed
`color`, and the vector document would have to be parsed again to follow it.
So it is carried as its own variant all the way to composition, where the
coordinator substitutes the node's computed `color` — the same treatment
[`BoxShadow`](https://drafts.csswg.org/css-backgrounds/#box-shadow)'s absent
colour already gets in this codebase.

A parsed vector document is therefore *theme-independent*: one parse serves
every colour it will ever be drawn in.
